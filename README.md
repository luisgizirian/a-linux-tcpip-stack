# a-linux-tcpip-stack

A project to build a complete, dual-stack IPv4/IPv6 TCP/IP implementation
from scratch in Rust, running inside a custom Linux kernel. The goal is a
stack that behaves like a normal Linux network stack from an application's
point of view — same sockets, same syscalls, same tools — while every packet
is parsed, validated, and processed by original Rust code instead of Linux's
native networking code.

This repository is in the **foundation stage**. It contains a portable,
dependency-free `no_std` Rust core with bounded packet buffers, checked wire
parsing, scatter/gather checksums, resource budgets, and wrap-safe timers.
There is not yet a bootable Linux kernel adapter or a complete network
protocol. The full scope, architecture, milestones, and acceptance gates are
recorded in [`docs/plan.md`](docs/plan.md), which remains authoritative.
Current evidence and blockers are tracked in
[`docs/implementation-status.md`](docs/implementation-status.md).

## Why this exists

Linux's in-kernel TCP/IP stack is mature, fast, and deeply trusted — which is
exactly why it is hard to study, extend, or replace experimentally. This
project exists to:

- Provide a from-scratch, memory-safer (Rust) reimplementation of the core
  networking protocols that still plugs into real Linux device drivers,
  sockets, and administration tools.
- Give researchers and engineers a vehicle to experiment with protocol
  internals (TCP congestion control, IPv6 Neighbor Discovery, fragmentation,
  routing policy, and more) without forking or patching Linux's native stack.
- Establish, through explicit conformance and robustness gates, how far a
  clean-room implementation can go in matching Linux's socket and protocol
  behavior.

## What it is not (yet)

- Not production-ready. The plan explicitly treats the stack as experimental
  until broader deployment evidence justifies a stronger claim.
- Not a drop-in replacement for Linux's native networking on every device or
  workload — scope is delivered as one coherent release program, not as a
  series of partial, independently shippable features.
- Not a fork of Linux's TCP/IP code. Protocol logic is implemented
  independently; only kernel plumbing (sockets, devices, allocation,
  synchronization) and audited facilities (crypto, kernel entropy) are reused.

## High-level architecture

The stack is organized as a small number of cooperating layers, detailed in
[`docs/plan.md`](docs/plan.md#architecture-and-feasibility-gate):

1. **Protocol core** — portable `no_std` Rust codecs and protocol state
   machines (Ethernet/ARP, IPv4/IPv6, ICMP/ICMPv6, UDP, TCP), testable on the
   host without a kernel.
2. **Kernel adapter** — Rust glue over kernel allocation, device/skb
   ownership, timers, and synchronization, with minimal C helpers where the
   kernel API requires it.
3. **Socket adapter** — wires the protocol core into Linux's existing
   syscall/VFS socket infrastructure, so unmodified applications keep using
   standard `socket()`/`bind()`/`connect()`/etc.
4. **Kernel patch set** — an opt-in, per-network-namespace mechanism that
   routes AF_INET/AF_INET6 traffic on a dedicated device to this stack
   instead of Linux's native implementation, leaving other namespaces and
   devices untouched.
5. **Routing/policy adapter and network services** — netlink-driven routing
   and firewall/NAT integration, plus userspace DHCP/DNS control-plane
   services built on top of the stack's own sockets.

## Repository layout

The planned directories now exist:

- `core/` contains the host-testable `no_std` foundation.
- `kernel/` will contain Kbuild/Kconfig integration and reviewed C helpers.
- `drivers/`, `services/`, and `patches/` reserve the remaining implementation
  surfaces.
- `tests/` and `tools/` will hold VM/interoperability suites and reproducible
  build and benchmark commands.

## Foundation checks

The local toolchain is pinned to Rust 1.80.0. Install the three bare targets
listed in [`docs/implementation-status.md`](docs/implementation-status.md) and
run:

```sh
make
```

This runs formatting checks, host tests, and `no_std` compile checks for
x86_64, ARM64, and RISC-V. It intentionally does not claim kernel
compatibility; the `kernel-feasibility` milestone remains the release gate
before protocol integration.

## Where to go next

- [`docs/plan.md`](docs/plan.md) — full implementation plan: scope,
  milestones, acceptance gates, and standards references.
- [`docs/implementation-status.md`](docs/implementation-status.md) — current
  implementation inventory, evidence, and blockers.
- [`docs/use-case.md`](docs/use-case.md) — a concrete, real-world scenario
  describing how this stack is meant to be deployed and used once it reaches
  that milestone.

## Contributing

This project is still being scoped out loud in `docs/plan.md`. If you are
interested in contributing, read the plan first — it defines what "done"
means for each milestone and the evidence required before anything is
considered complete.
