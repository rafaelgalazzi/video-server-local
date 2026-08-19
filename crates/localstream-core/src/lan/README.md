# LAN Configuration and Lifecycle

## Purpose

Own explicit secure-LAN endpoint configuration and safe public status independently from desktop startup.

## Invariants

Serving is disabled by default. A configured address must be a concrete private or link-local unicast address; loopback, wildcard, multicast, broadcast, and public Internet addresses are rejected. Configuration alone never binds a socket.

## Planned Work

TLS rotation, fail-closed listener orchestration, and the final activation preflight build on this module.
