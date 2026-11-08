# Real-world usage scenario

This document describes a concrete scenario for how the stack described in
[`plan.md`](plan.md) is intended to be deployed and used once it is
implemented. It is meant to ground the architecture in a tangible example,
not to describe anything that exists in the repository today — nothing here
is implemented yet; see `plan.md` for current status and milestones.

## Scenario: an edge router with an experimental data plane

A network operator runs a small edge router (x86_64, with a few physical
NICs) that handles internet-facing traffic for a lab network: endpoint
sockets for services running on the box itself, plus IPv4/IPv6 forwarding
and NAT for hosts behind it. The operator wants to run this stack's TCP/IP
implementation on that router without putting the rest of the machine, or
the rest of the network, at risk.

### 1. Isolating the experiment to one namespace and one device

The operator creates a dedicated network namespace (for example `ns-rust`)
and moves a single physical NIC (or a VLAN sub-interface) into it. Inside
that namespace, the operator opts a device into this stack via the kernel
patch set's per-namespace selector. Every other namespace on the machine,
including the default one, keeps using Linux's native TCP/IP stack
untouched — this is a hard design requirement, not a convenience.

```
default namespace            ns-rust namespace
┌─────────────────────┐      ┌─────────────────────────┐
│ native Linux stack   │      │ this stack (Rust core)  │
│ eth0 (native)        │      │ eth1 (opted in)         │
└─────────────────────┘      └─────────────────────────┘
```

### 2. Running ordinary applications unmodified

Inside `ns-rust`, the operator starts ordinary, unmodified binaries: an SSH
daemon, a small HTTP service, `curl`, `ping`, `iperf3`. These programs call
the standard `socket()`/`bind()`/`connect()`/`accept()`/`sendmsg()` syscalls
exactly as they would against Linux's native stack. The socket adapter
layer routes that traffic through this stack's protocol core instead of
Linux's native TCP/IP, with no relinking, no `LD_PRELOAD`, and no
application-visible API differences — including nonblocking I/O and
poll/epoll readiness.

### 3. Talking to the rest of the network as a normal peer

On the wire, `ns-rust` looks like an ordinary IPv4/IPv6 host or router:

- It resolves neighbors with ARP (IPv4) and Neighbor Discovery (IPv6),
  including SLAAC and duplicate address detection.
- It answers `ping`/`ping6`, negotiates TCP connections (including SACK,
  timestamps, window scaling, and congestion control), and exchanges UDP
  datagrams with native Linux hosts and third-party devices on the lab
  network.
- It forwards IPv4/IPv6 traffic between its interfaces, applies NAT and
  firewall rules through the same netfilter/nftables tooling the operator
  already uses, and advertises/consumes routes via the routing control
  planes (for example OSPF or BGP) running as userspace services on top of
  the stack's own sockets.
- Administration stays familiar: routes, addresses, and link state are
  configured and inspected through the normal Linux netlink tools (`ip
  addr`, `ip route`, `ip link`), because the routing/policy adapter exposes
  this stack's state through the same interfaces.

From the perspective of every other device on the network — switches,
other routers, DNS/DHCP servers, remote peers — `ns-rust` is indistinguishable
from a normal Linux host, except that every packet it sends or receives was
produced and parsed by this project's Rust implementation.

### 4. Validating before trusting it with real traffic

Before the operator relies on `ns-rust` for anything other than lab
traffic, they run the conformance and robustness suites described in
`plan.md`'s `robustness-validation` and `performance-baseline` milestones:
interoperability tests against native Linux peers, fuzzing/property tests on
the parsers, stress tests under packet loss/reordering/duplication, and
baseline throughput/latency measurements compared against the native stack
on matched hardware and configuration. Only capabilities with passing gates
and documented limitations are exposed to production traffic; everything
else stays confined to the lab namespace.

### 5. Falling back safely

Because the integration is strictly per-namespace and per-device, the
operator can always move the NIC back out of `ns-rust` (or simply stop
routing traffic to it) and fall back to Linux's native stack without
touching any other namespace, without rebooting, and without any traffic
having silently fallen back to native processing mid-stream — the stack's
resource and error policy explicitly forbids silent fallback.

## Why this scenario matters

This walk-through exists to make two design commitments concrete for
readers who are new to the project:

- **Compatibility first.** Applications, administration tools, and remote
  peers should never need to know or care that this stack is in use instead
  of Linux's native one.
- **Safety through isolation.** The blast radius of an experimental,
  from-scratch protocol implementation is bounded to an opted-in namespace
  and device, so it can be evaluated on real hardware and real traffic
  without risking the rest of the system or network.

See `plan.md` for the exact milestones and acceptance gates that must pass
before each piece of this scenario becomes real.
