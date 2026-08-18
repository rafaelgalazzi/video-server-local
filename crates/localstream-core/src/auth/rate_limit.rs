use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Mutex,
    time::{Duration, Instant},
};

const WINDOW: Duration = Duration::from_secs(60);
const BEGIN_PER_SOURCE: u32 = 5;
const BEGIN_GLOBAL: u32 = 40;
const CLAIM_PER_SOURCE: u32 = 10;
const CLAIM_GLOBAL: u32 = 80;
const MAX_TRACKED_SOURCES: usize = 1_024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairingAttemptKind {
    Begin,
    Claim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitDecision {
    Allowed,
    Limited { retry_after_seconds: u64 },
}

#[derive(Debug, Clone, Copy)]
struct Policy {
    per_source: u32,
    global: u32,
    window: Duration,
}

#[derive(Debug, Clone, Copy)]
struct WindowCounter {
    started_at: Instant,
    attempts: u32,
}

#[derive(Debug)]
struct SourceState {
    last_seen: Instant,
    begin: Option<WindowCounter>,
    claim: Option<WindowCounter>,
}

#[derive(Debug, Default)]
struct LimiterState {
    sources: HashMap<IpAddr, SourceState>,
    begin_global: Option<WindowCounter>,
    claim_global: Option<WindowCounter>,
}

#[derive(Debug)]
pub(crate) struct PairingRateLimiter {
    state: Mutex<LimiterState>,
    begin_policy: Policy,
    claim_policy: Policy,
    max_sources: usize,
}

impl Default for PairingRateLimiter {
    fn default() -> Self {
        Self::new(
            Policy {
                per_source: BEGIN_PER_SOURCE,
                global: BEGIN_GLOBAL,
                window: WINDOW,
            },
            Policy {
                per_source: CLAIM_PER_SOURCE,
                global: CLAIM_GLOBAL,
                window: WINDOW,
            },
            MAX_TRACKED_SOURCES,
        )
    }
}

impl PairingRateLimiter {
    fn new(begin_policy: Policy, claim_policy: Policy, max_sources: usize) -> Self {
        Self {
            state: Mutex::new(LimiterState::default()),
            begin_policy,
            claim_policy,
            max_sources,
        }
    }

    pub(crate) fn check(&self, kind: PairingAttemptKind, remote: SocketAddr) -> RateLimitDecision {
        self.check_at(kind, normalize_ip(remote.ip()), Instant::now())
    }

    fn check_at(
        &self,
        kind: PairingAttemptKind,
        source: IpAddr,
        now: Instant,
    ) -> RateLimitDecision {
        let policy = match kind {
            PairingAttemptKind::Begin => self.begin_policy,
            PairingAttemptKind::Claim => self.claim_policy,
        };
        let mut state = match self.state.lock() {
            Ok(state) => state,
            Err(_) => {
                return RateLimitDecision::Limited {
                    retry_after_seconds: duration_seconds_ceil(policy.window),
                };
            }
        };
        let stale_after = self.begin_policy.window.max(self.claim_policy.window);
        state
            .sources
            .retain(|_, entry| now.saturating_duration_since(entry.last_seen) < stale_after);
        if !state.sources.contains_key(&source) && state.sources.len() >= self.max_sources {
            return RateLimitDecision::Limited {
                retry_after_seconds: duration_seconds_ceil(stale_after),
            };
        }

        {
            let global_counter = match kind {
                PairingAttemptKind::Begin => &mut state.begin_global,
                PairingAttemptKind::Claim => &mut state.claim_global,
            };
            reset_expired(global_counter, now, policy.window);
            if let Some(retry_after_seconds) =
                limited_retry(*global_counter, policy.global, now, policy.window)
            {
                return RateLimitDecision::Limited {
                    retry_after_seconds,
                };
            }
        }

        {
            let source_state = state.sources.entry(source).or_insert(SourceState {
                last_seen: now,
                begin: None,
                claim: None,
            });
            source_state.last_seen = now;
            let source_counter = match kind {
                PairingAttemptKind::Begin => &mut source_state.begin,
                PairingAttemptKind::Claim => &mut source_state.claim,
            };
            reset_expired(source_counter, now, policy.window);
            if let Some(retry_after_seconds) =
                limited_retry(*source_counter, policy.per_source, now, policy.window)
            {
                return RateLimitDecision::Limited {
                    retry_after_seconds,
                };
            }
            increment(source_counter, now);
        }

        let global_counter = match kind {
            PairingAttemptKind::Begin => &mut state.begin_global,
            PairingAttemptKind::Claim => &mut state.claim_global,
        };
        increment(global_counter, now);
        RateLimitDecision::Allowed
    }
}

fn normalize_ip(address: IpAddr) -> IpAddr {
    match address {
        IpAddr::V6(address) => address
            .to_ipv4_mapped()
            .map_or(IpAddr::V6(address), IpAddr::V4),
        address => address,
    }
}

fn reset_expired(counter: &mut Option<WindowCounter>, now: Instant, window: Duration) {
    if counter.is_some_and(|counter| now.saturating_duration_since(counter.started_at) >= window) {
        *counter = None;
    }
}

fn limited_retry(
    counter: Option<WindowCounter>,
    limit: u32,
    now: Instant,
    window: Duration,
) -> Option<u64> {
    let counter = counter.filter(|counter| counter.attempts >= limit)?;
    Some(duration_seconds_ceil(window.saturating_sub(
        now.saturating_duration_since(counter.started_at),
    )))
}

fn increment(counter: &mut Option<WindowCounter>, now: Instant) {
    match counter {
        Some(counter) => counter.attempts = counter.attempts.saturating_add(1),
        None => {
            *counter = Some(WindowCounter {
                started_at: now,
                attempts: 1,
            });
        }
    }
}

fn duration_seconds_ceil(duration: Duration) -> u64 {
    duration
        .as_secs()
        .saturating_add(u64::from(duration.subsec_nanos() > 0))
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::{normalize_ip, PairingAttemptKind, PairingRateLimiter, Policy, RateLimitDecision};
    use std::{
        net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
        time::{Duration, Instant},
    };

    fn limiter(per_source: u32, global: u32, max_sources: usize) -> PairingRateLimiter {
        let policy = Policy {
            per_source,
            global,
            window: Duration::from_secs(60),
        };
        PairingRateLimiter::new(policy, policy, max_sources)
    }

    fn ip(last: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(192, 0, 2, last))
    }

    #[test]
    fn enforces_the_per_source_boundary_and_resets_after_the_window() {
        let limiter = limiter(2, 10, 10);
        let now = Instant::now();
        assert_eq!(
            limiter.check_at(PairingAttemptKind::Begin, ip(1), now),
            RateLimitDecision::Allowed
        );
        assert_eq!(
            limiter.check_at(PairingAttemptKind::Begin, ip(1), now),
            RateLimitDecision::Allowed
        );
        assert_eq!(
            limiter.check_at(
                PairingAttemptKind::Begin,
                ip(1),
                now + Duration::from_millis(1)
            ),
            RateLimitDecision::Limited {
                retry_after_seconds: 60
            }
        );
        assert_eq!(
            limiter.check_at(
                PairingAttemptKind::Begin,
                ip(1),
                now + Duration::from_secs(60)
            ),
            RateLimitDecision::Allowed
        );
    }

    #[test]
    fn isolates_sources_but_enforces_the_global_boundary() {
        let limiter = limiter(3, 3, 10);
        let now = Instant::now();
        assert_eq!(
            limiter.check_at(PairingAttemptKind::Begin, ip(1), now),
            RateLimitDecision::Allowed
        );
        assert_eq!(
            limiter.check_at(PairingAttemptKind::Begin, ip(2), now),
            RateLimitDecision::Allowed
        );
        assert_eq!(
            limiter.check_at(PairingAttemptKind::Begin, ip(3), now),
            RateLimitDecision::Allowed
        );
        assert_eq!(
            limiter.check_at(PairingAttemptKind::Begin, ip(4), now),
            RateLimitDecision::Limited {
                retry_after_seconds: 60
            }
        );
    }

    #[test]
    fn begin_and_claim_policies_have_independent_counters() {
        let limiter = PairingRateLimiter::new(
            Policy {
                per_source: 1,
                global: 1,
                window: Duration::from_secs(60),
            },
            Policy {
                per_source: 2,
                global: 2,
                window: Duration::from_secs(60),
            },
            10,
        );
        let now = Instant::now();
        assert_eq!(
            limiter.check_at(PairingAttemptKind::Begin, ip(1), now),
            RateLimitDecision::Allowed
        );
        assert!(matches!(
            limiter.check_at(PairingAttemptKind::Begin, ip(1), now),
            RateLimitDecision::Limited { .. }
        ));
        assert_eq!(
            limiter.check_at(PairingAttemptKind::Claim, ip(1), now),
            RateLimitDecision::Allowed
        );
        assert_eq!(
            limiter.check_at(PairingAttemptKind::Claim, ip(1), now),
            RateLimitDecision::Allowed
        );
        assert!(matches!(
            limiter.check_at(PairingAttemptKind::Claim, ip(1), now),
            RateLimitDecision::Limited { .. }
        ));
    }

    #[test]
    fn bounds_sources_and_reclaims_stale_entries() {
        let limiter = limiter(5, 20, 2);
        let now = Instant::now();
        assert_eq!(
            limiter.check_at(PairingAttemptKind::Begin, ip(1), now),
            RateLimitDecision::Allowed
        );
        assert_eq!(
            limiter.check_at(PairingAttemptKind::Begin, ip(2), now),
            RateLimitDecision::Allowed
        );
        assert!(matches!(
            limiter.check_at(PairingAttemptKind::Begin, ip(3), now),
            RateLimitDecision::Limited { .. }
        ));
        assert_eq!(
            limiter
                .state
                .lock()
                .expect("state should lock")
                .sources
                .len(),
            2
        );

        assert_eq!(
            limiter.check_at(
                PairingAttemptKind::Begin,
                ip(3),
                now + Duration::from_secs(60)
            ),
            RateLimitDecision::Allowed
        );
        assert_eq!(
            limiter
                .state
                .lock()
                .expect("state should lock")
                .sources
                .len(),
            1
        );
    }

    #[test]
    fn normalizes_source_ip_and_ignores_socket_ports() {
        let limiter = limiter(1, 10, 10);
        let ipv4 = Ipv4Addr::new(192, 0, 2, 1);
        let mapped = IpAddr::V6(ipv4.to_ipv6_mapped());
        assert_eq!(normalize_ip(mapped), IpAddr::V4(ipv4));

        assert_eq!(
            limiter.check(
                PairingAttemptKind::Begin,
                SocketAddr::new(IpAddr::V4(ipv4), 1000)
            ),
            RateLimitDecision::Allowed
        );
        assert!(matches!(
            limiter.check(PairingAttemptKind::Begin, SocketAddr::new(mapped, 2000)),
            RateLimitDecision::Limited { .. }
        ));
        assert_eq!(
            normalize_ip(IpAddr::V6(Ipv6Addr::LOCALHOST)),
            IpAddr::V6(Ipv6Addr::LOCALHOST)
        );
    }

    #[test]
    fn poisoned_state_fails_closed_with_safe_retry_metadata() {
        let limiter = std::sync::Arc::new(limiter(2, 10, 10));
        let poison = std::sync::Arc::clone(&limiter);
        let _ = std::thread::spawn(move || {
            let _guard = poison.state.lock().expect("state should initially lock");
            panic!("poison limiter for test");
        })
        .join();

        assert_eq!(
            limiter.check(
                PairingAttemptKind::Begin,
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1234),
            ),
            RateLimitDecision::Limited {
                retry_after_seconds: 60
            }
        );
    }
}
