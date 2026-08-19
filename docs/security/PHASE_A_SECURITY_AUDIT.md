# Phase A LAN Activation Audit

Completed 2026-08-18 for LS-031.

| Gate                                       | Evidence                                                                         | Result                           |
| ------------------------------------------ | -------------------------------------------------------------------------------- | -------------------------------- |
| Stable protected node root                 | Keyring boundary, corruption/reset tests                                         | PASS                             |
| Short-lived server leaf and verified names | issuance, renewal, SAN-change, wrong-root/name tests                             | PASS                             |
| TLS-only transport                         | Rustls lifecycle and plaintext downgrade tests                                   | PASS                             |
| Explicit browser trust                     | trusted-local export, full-fingerprint acknowledgement, no remote CA route       | PASS                             |
| Native pin and credential protection       | isolated keyring boundary, mismatch/replacement/corruption/deletion tests        | PASS                             |
| Explicit local pairing                     | approval, expiry, replay, rate-limit tests                                       | PASS                             |
| Protected API/media routes                 | bearer/session/capability/revocation tests                                       | PASS                             |
| Same-origin browser policy                 | Host, Origin, Fetch Metadata, cookie misuse tests                                | PASS                             |
| Static UI containment                      | traversal, malformed path, size, API precedence tests                            | PASS                             |
| Resource limits                            | body, pairing, stream, TLS connection and handshake limits                       | PASS                             |
| Explicit endpoint only                     | disabled default and loopback/wildcard/public-address rejection                  | PASS                             |
| Fail-closed orchestration                  | missing config/assets/identity/TLS and activation-permit design                  | PASS                             |
| Isolated local-network lifecycle           | exact private-interface bind, trusted TLS, unauthorized API, downgrade rejection | PASS on development Windows host |
| Physical second-device workflow            | requires another trusted device and interactive certificate installation         | NOT VERIFIED                     |

Runtime activation requires a typed permit produced only after every compiled security evidence gate passes. Configuration alone cannot bind. Failure leaves the trusted-local desktop server running and reports a safe status code without plaintext fallback.
