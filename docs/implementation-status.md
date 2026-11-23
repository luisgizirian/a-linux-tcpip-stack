# Implementation status

The repository implements the first portable portion of
`core-foundation`. It does **not** yet implement a bootable kernel module or
any complete protocol, socket, router, driver, or network service.

| Plan milestone | Status | Evidence and blocker |
| --- | --- | --- |
| `kernel-feasibility` | Blocked | No Linux commit, architecture configurations, boot images, or Rust kernel abstractions have been selected and audited. This gates protocol integration. |
| `core-foundation` | In progress | A dependency-free `no_std` core provides checked wire cursors, bounded owned packets, S/G Internet checksums, resource budgets, injected clock/I/O traits, and wrap-safe deadlines. Host tests and `x86_64`, `aarch64`, and `riscv64` bare-metal checks pass. Kernel dual-build and adapter context/locking checks are still absent. |
| `ipv4-endpoint` | Blocked | Preliminary Ethernet and IPv4 header codecs exist; packet dispatch, IPv4 options, ARP, ICMP, routing, fragmentation, loopback, and administration are not implemented. |
| `ipv6-endpoint` | Blocked | Not implemented. |
| `udp-sockets` | Blocked | Not implemented. |
| `tcp-engine` | Blocked | Not implemented. |
| `tcp-sockets` | Blocked | Not implemented. |
| `routing-policy` | Blocked | Not implemented. |
| `network-services` | Blocked | Not implemented. |
| `ip-extensions` | Blocked | Not implemented. |
| `device-offloads` | Blocked | Not implemented. |
| `socket-compatibility` | Blocked | Not implemented. |
| `robustness-validation` | Blocked | Core unit tests and CI exist; kernel instrumentation, fuzzing, VM, lifecycle, and conformance suites do not. |
| `performance-baseline` | Blocked | Not implemented. |
| `optimization-release` | Blocked | Not eligible until all release gates pass. |

## Core foundation requirement matrix

| Requirement | Implementation | Acceptance evidence |
| --- | --- | --- |
| Checked byte parsing | `core/src/buffer.rs` `Reader`; `core/src/wire.rs` codecs | Truncated-prefix and invalid IPv4-header tests |
| Endian-safe serialization | `Writer`, `Reader` big-endian integer methods | Host unit tests and bare-metal target checks |
| Scatter/gather checksums | `core/src/checksum.rs` | RFC 1071, odd-length, chunk-boundary, and verification tests |
| Bounded packet ownership | `OwnedPacket<CAPACITY>` | Capacity and partial-write tests |
| Explicit resource limits | `ResourceBudget` | Reserve failure and release-underflow tests |
| Monotonic clock injection | `Clock` trait; feature-gated `ManualClock` | `--all-features` compile and wraparound deadline tests |
| Timer wrap and ordering | `Deadline` with half-range bound | Exact-expiry, wraparound, and ambiguous-range tests |
| Deterministic packet I/O | `PacketSource` and `PacketSink` traits | API present; end-to-end simulation is not implemented |
| Kernel build parity | Core is `no_std` and forbids unsafe code | Three bare-metal target checks pass; Kbuild parity is blocked on feasibility |
| Panic-free malformed input | Checked slices and explicit errors | Current codecs' truncation/invalid-input tests |

## Reproduce the implemented checks

The repository pins Rust 1.80.0 in `rust-toolchain.toml`. Once its three bare
targets are installed, run:

```sh
make
```

To reproduce the setup manually:

```sh
rustup target add \
  x86_64-unknown-none \
  aarch64-unknown-none \
  riscv64gc-unknown-none-elf
make
```

Passing `make` is evidence for the host-testable foundation only. It is not
evidence of Linux kernel compatibility, protocol conformance, hardware
support, or release readiness.
