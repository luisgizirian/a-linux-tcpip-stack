# Rust TCP/IP stack for Linux: implementation plan

## Objective and confirmed decisions

Build a complete dual-stack IPv4/IPv6 TCP/IP stack from scratch in Rust, running
inside a custom Linux kernel with endpoint and router functionality, network
services, and Linux-compatible socket behavior.
Use Linux device infrastructure and socket/VFS plumbing, support existing NIC
drivers, and implement Rust NIC drivers for the documented hardware matrix;
do not delegate protocol processing to Linux's native TCP/IP implementation.
Minimal C glue is allowed where required by kernel APIs.

## Design commitments

- A maintained kernel patch set is acceptable.
- Deliver the full scope below as one release program. Milestones organize
  implementation dependencies, not reduced-scope releases.
- Target x86_64, ARM64 (AArch64, little-endian), and RISC-V (riscv64) using a
  pinned Rust-enabled upstream kernel in architecture-specific QEMU/Linux VMs.
- Deliver Linux socket compatibility, including nonblocking I/O and poll/epoll.
- Prioritize correctness; establish baselines before setting numeric targets.

Absolute absence of bugs cannot be guaranteed. Release readiness means passing
explicit conformance, robustness, lifecycle, and performance gates, with known
limitations disclosed. This is initially experimental, not production-ready.

## Scope

### Included

- Ethernet framing, ARP resolution/cache expiry, bounded unresolved-packet queues.
- IPv4 validation, local delivery, static routes, loopback, TTL, checksums,
  MTU enforcement, bounded fragmentation/reassembly, and path-MTU handling.
- ICMP echo and relevant errors, with rate limiting and quoted-packet validation.
- IPv6 validation, local delivery, static routes, loopback, hop limits, bounded
  extension-header processing, source fragmentation/reassembly, and path-MTU
  discovery.
- ICMPv6 echo/errors, Neighbor Discovery and neighbor reachability, link-local
  addressing, duplicate address detection, router discovery, and SLAAC with
  validated advertisements and address/route lifetimes.
- IPv6 multicast delivery and MLDv2 membership needed for Neighbor Discovery
  and IPv6 socket multicast operation.
- UDP datagrams, connected and unconnected sockets, and checksum semantics.
- TCP connection establishment, reliable byte streams, retransmission,
  receive-window flow control, congestion control, and complete connection teardown.
- Static address/route/MTU administration, resource budgets, and diagnostics.
- Standard syscall entry points without application relinking or LD_PRELOAD.
- Namespace isolation, multi-core race testing, and clean namespace/socket teardown.
- IPv4/IPv6 forwarding, policy routing, dynamic routing protocols, NAT, and
  netfilter/nftables integration.
- DHCP/DHCPv6 clients, servers, and relays; DNS resolution, caching/forwarding,
  authoritative service, and DNSSEC validation.
- Raw IP and AF_PACKET sockets, IPv4/IPv6 multicast membership and routing,
  IGMP/MLD interoperability, and VLAN support.
- Rust NIC drivers and hardware offloads, including checksum, segmentation,
  receive aggregation, RSS, and multiqueue operation.
- TCP SACK, timestamps, window scaling, ECN, modern loss recovery, and congestion
  algorithms exposed by the pinned Linux compatibility target.
- Complete Linux socket flags/options and networking I/O compatibility for
  TCP, UDP, raw IP, and AF_PACKET on the pinned kernel.
- IPsec, Mobile IPv6, IPv6 jumbograms, and standards-compliant extension handling.

Maintain explicit protocol, API, hardware, and standards inventories for this
scope. Every listed capability requires implementation and acceptance evidence;
an unimplemented entry blocks release rather than becoming a roadmap item.
Invalid inputs and operations forbidden by standards or configuration still
return the appropriate errors. Never claim application or hardware compatibility
without testing the relevant contract.

## Architecture and feasibility gate

1. **Protocol core:** portable `no_std` Rust, compiled into the kernel through
   Kbuild and separately into a host test harness. Keep codecs and state machines
   independent of kernel objects. Use injected clocks, packet I/O, and explicit
   resource limits; do not create an elaborate generic framework.
2. **Kernel adapter:** reviewed Rust abstractions for kernel allocation, skb
   ownership, devices, timers, synchronization, wait queues, and user I/O.
   Isolate raw bindings and any C helpers here, with documented safety contracts.
3. **Socket adapter:** reuse syscall/VFS infrastructure and provide the required
   socket operations backed by the new transport, raw-IP, and packet paths.
4. **Kernel patch set:** prototype opt-in per-network-namespace selection for
   AF_INET/AF_INET6 socket creation and IPv4/ARP/IPv6 ingress dispatch. This is a
   proposed design, not an existing supported Linux extension API.
5. **Test harness:** deterministic packet simulation plus real VM kernel tests
   against native Linux peers. Native Linux remains the interoperability peer
   and baseline, never a fallback implementing the new stack's protocols.
6. **Routing and policy adapter:** expose route/device administration through
   Linux netlink and connect the new packet paths to netfilter, NAT, XFRM, and
   multicast forwarding with explicit ownership and hook ordering.
7. **Network services:** run DHCP, DNS, and routing control planes as Rust
   userspace services over the new stack's sockets and administration APIs.
   Keep service lifecycle and policy separate from kernel packet processing.

Before building protocols, prove that the pinned kernel can dispatch sockets
and packets into the adapter without double delivery, native-stack leakage,
or global socket-family replacement. Socket selection alone is insufficient:
ingress, ARP/Neighbor Discovery, loopback, egress, device/configuration ownership,
IPv6 address/router lifecycle, and teardown must agree.
Use a dedicated device in the selected namespace and disallow switching stacks
while sockets or devices are active. Native namespaces must remain unchanged.

Audit available Rust abstractions on that exact kernel. Kernel documentation
states that not every internal API has a Rust abstraction; do not assume that
a complete networking API exists. If the routing/ownership design is infeasible,
stop and obtain a revised integration decision rather than silently switching
to userspace or a private socket family.

The Windows workspace is for editing. Kernel builds and boot tests run in a
dedicated Linux environment. Do not alter the shared host's routing, kernel, or
network devices. Namespace isolation is not kernel fault containment: crashes
can affect the entire guest, so all execution stays inside disposable VMs.

## Initial proposed repository layout

- [docs/](./): shared home for project documentation. Agents, developers, and
  other contributors should place design notes, specifications, guides, and
  benchmark reports here rather than scattering documents across the repository.
- `core/`: portable Rust codecs and protocol state machines; Cargo host tests.
- `kernel/`: Kbuild/Kconfig adapter sources and reviewed C helpers if needed.
- `drivers/`: Rust NIC drivers and device-specific offload integration.
- `services/`: DHCP, DNS, and routing control-plane services.
- `patches/`: minimal patch series against the pinned external Linux source.
- `tests/`: socket, packet-level, impairment, and VM lifecycle integration tests.
- `tools/`: reproducible Linux build/VM and benchmark commands.

Record the exact Linux commit/tag, per-architecture kernel configurations,
Rust/LLVM/bindgen versions, guest images, and test-tool versions. Choose a
compatible combination through `make LLVM=1 rustavailable` for all target
configurations; do not use a moving latest kernel/toolchain. Verify Rust support
on the pinned kernel for x86_64, ARM64, and riscv64 rather than assuming rolling
documentation applies to that version. Use LLVM/Clang for the riscv64 kernel
build and record the required ISA extensions and guest firmware.
Validate how the same core sources are compiled by Cargo and Kbuild before
choosing dependency packaging. Protocol implementations must be original;
dependencies may supply test/build infrastructure and audited kernel facilities.

## Todos and completion gates

### kernel-feasibility: Proving kernel and Rust integration

Pin the environment, build/boot an unmodified Rust-enabled kernel on x86_64,
ARM64, and riscv64, then implement the smallest isolated adapter proof on all
three architectures.
Confirm separate socket and packet dispatch,
ordinary AF_INET/AF_INET6 use, namespace/device ownership, and native-stack
preservation. Ensure native IPv6 autoconfiguration and Neighbor Discovery do not
also process traffic or manage addresses on devices owned by the new stack.
Inventory missing abstractions and the required patch/C glue surface.

Gate: reproducible builds, boot, and dispatch tests pass on all three architectures;
lifecycle/locking design and
the supported socket/configuration interfaces are documented. No protocol
milestone proceeds while these prerequisites are unresolved.

### core-foundation: Building the testable protocol foundation

Establish `no_std` host/kernel dual builds, checked byte parsing, endian-safe
serialization, scatter/gather checksum handling, packet ownership, monotonic
timers, explicit errors, and deterministic allocation/time/I/O test controls.
Avoid x86-specific alignment, atomic-ordering, and instruction assumptions;
parse wire fields without unaligned typed references. Require core and adapter
build checks for x86_64, ARM64, and riscv64.
Define lock ordering and execution-context rules in the adapter. No sleeping
or sleeping allocation in atomic/softirq contexts. Any bounded copy of a
nonlinear skb must be explicit and measured, not justified by unsafe casts.

Gate: malformed/truncated input tests, odd-length checksum tests, timer wrap/
ordering tests, allocation-failure tests, and host/kernel build checks pass.
No peer-controlled input may cause a panic or unchecked indexing.

### ipv4-endpoint: Implementing Ethernet, ARP, IPv4, and ICMP

Implement the local endpoint and administration boundary. Define
unicast/broadcast behavior, implement IPv4 options with standards-required
validation, and implement
bounded fragment handling with a deterministic overlap policy, byte/entry caps,
and expiry. Handle DF/MTU errors, ICMP rate limits, route failures, and quoted
packet validation without allowing arbitrary ICMP to mutate transport state.

Gate: native-peer ARP/ping and loopback pass; malformed headers, fragment gaps/
overlaps, reassembly exhaustion, expiry, MTU changes, and unreachable routes
produce the documented result and counters without leaks.

### ipv6-endpoint: Implementing IPv6, ICMPv6, and Neighbor Discovery

Reuse the Ethernet/device foundation and implement IPv6 addressing, route and
scope selection, link-local addresses, DAD, Router Solicitation/Advertisement,
SLAAC, Neighbor Solicitation/Advertisement, neighbor-cache states, and expiry.
Validate hop limits, source addresses, options, and address/router lifetimes for
Neighbor Discovery; never accept arbitrary advertisements as transport errors.
Implement required multicast filtering and MLDv2 membership/report lifecycle.

Implement IPv6 header validation and bounded extension-header traversal with
RFC-defined unknown-option actions. Implement ICMPv6 checksums, echo and errors,
including Packet Too Big validation and per-path MTU state. Handle the IPv6
minimum MTU, source-only fragmentation, bounded reassembly with overlap rejection,
and mandatory UDP/TCP IPv6 pseudo-header checksums. Do not reuse IPv4 broadcast,
header-checksum, or router-fragmentation assumptions.

Gate: native-peer IPv6 ping, link-local/global addressing, loopback, DAD conflict,
SLAAC/router expiry, neighbor reachability, multicast, and path-MTU tests pass.
Malformed/oversized extension chains, fragments, invalid advertisements, and
resource exhaustion produce documented results without leaks.

### udp-sockets: Implementing UDP and the first socket contract

Provide bind, connect, sendto/recvfrom, sendmsg/recvmsg, read/write, getsockname/
getpeername, close, and required socket flags/options through normal syscalls.
Implement ephemeral port allocation, namespace-local demultiplexing, connected
peer filtering, truncation semantics, zero-length datagrams, checksum-zero IPv4
behavior, EAGAIN/readiness, and documented ICMP/error propagation.
Support AF_INET6 datagrams, IPv6 scope IDs, ICMPv6 error propagation, and mandatory
IPv6 UDP checksums; reject zero checksums for ordinary IPv6 UDP. Specify and test
IPV6_V6ONLY, IPv4-mapped addresses, cross-family port conflicts, IPv6 multicast
options, and interface selection. Dual-stack sockets must use this stack's IPv4
path for mapped traffic, never the native stack.

Create a complete compatibility matrix for flags, options, blocking/signal
behavior, errno, and readiness on the pinned kernel. Unimplemented entries remain
failing acceptance requirements, not successful native-stack fallbacks.

Gate: unmodified IPv4/IPv6 socket test programs interoperate with native Linux
through the new stack, including nonblocking I/O, level-/edge-triggered epoll,
IPv6-only sockets, and dual-stack mapped-address behavior.

### tcp-engine: Implementing TCP wire and state-machine correctness

Implement the RFC 9293 state machine, sequence-number wraparound, randomized
initial sequence numbers using kernel entropy, MSS negotiation, cumulative ACKs,
bounded out-of-order queues, retransmission with RFC 6298 timers, Karn's rule,
and RFC 5681 baseline congestion control.
Use the same transport state machine for both IP families, with family-correct
pseudo-header checksums, address/scope identities, MSS, and path-MTU updates.

Cover passive/active/simultaneous open, backlog limits, duplicate segments,
partial ACKs, fast retransmit/recovery, zero-window probes, FIN retransmission,
half-close, simultaneous close, resets, and TIME_WAIT. Define memory-pressure
behavior and connection/backlog quotas; do not label TCP complete after an echo
test. Implement SACK, timestamps, window scaling, ECN, RACK/TLP loss recovery,
and congestion-control algorithms matching the pinned Linux API inventory,
including Reno and CUBIC. Validate negotiation, fallback permitted by protocol
standards, recovery interactions, and algorithm switching.

Gate: deterministic state/packet trace tests and native-peer IPv4/IPv6
interoperability pass under loss, duplication, corruption, reorder, delay, and
sequence wrap.
Tests verify payload integrity, congestion/window limits, retry bounds, and
eventual cleanup, not merely connection success.

### tcp-sockets: Completing the initial stream socket contract

Connect TCP to socket, bind, listen, accept/accept4, connect, send/recv,
sendmsg/recvmsg, read/write, shutdown, close, and peer/local address queries.
Implement partial I/O, nonblocking connect/SO_ERROR, EOF/reset, SIGPIPE behavior,
signal interruption, backlog/readiness, fd duplication, and final-reference
cleanup. Implement socket options including buffer sizes, SO_REUSEADDR,
SO_REUSEPORT, keepalive, linger, TCP_NODELAY, TCP_INFO, and congestion selection,
along with the remaining options in the pinned compatibility inventory.
Apply the contract to both AF_INET and AF_INET6, including IPV6_V6ONLY,
IPv4-mapped peers, scope IDs, and IPv6 error/MSS/path-MTU handling.

Gate: the documented socket compatibility matrix passes for IPv4, IPv6-only,
and dual-stack sockets with ordinary compiled applications and poll/epoll,
including cancellation/close/readiness races.

### routing-policy: Implementing forwarding, routing, multicast, and policy

Implement IPv4/IPv6 forwarding, policy rules, multipath routes, route lifetimes,
and Linux route/device netlink administration. Implement IGMP/MLD membership and
multicast forwarding, VLAN-tagged traffic, and IPv4/IPv6 NAT/netfilter integration
where supported by the pinned kernel. Document conntrack, fragment, checksum,
MTU, and hook-order interactions so packets traverse each required hook once.

Provide Rust routing control planes for BGP, OSPFv2/v3, RIP/RIPng, and multicast
routing, with explicit route selection, withdrawal, convergence, authentication,
and administration contracts.

Gate: isolated multi-router topologies pass forwarding, convergence, multicast,
VLAN, policy, firewall, and NAT tests with native Linux and protocol peers.
Loop prevention, hop-limit expiry, route churn, conntrack exhaustion, and
namespace isolation remain correct under concurrent updates.

### network-services: Implementing DHCP and DNS services

Implement DHCPv4/DHCPv6 client, server, and relay behavior, including lease
renewal/rebinding, expiry, conflict detection, prefix delegation, and interaction
with static configuration and SLAAC. Implement DNS resolver, cache/forwarder,
authoritative service, and DNSSEC validation using the stack's UDP/TCP sockets.
Use reviewed cryptographic primitives rather than writing new cryptography.

Gate: standards-based peer tests pass acquisition, renewal, relay, failover/
restart recovery, DNS TCP fallback, negative caching, truncation, and DNSSEC
valid/invalid responses. Apply resource caps, persistence rules, and protections
against amplification and unauthorized configuration changes.

### ip-extensions: Implementing IPsec, mobility, and jumbograms

Implement IPsec AH/ESP packet handling and reviewed kernel crypto/XFRM integration,
with policy, key management through an interoperable IKEv2 service, anti-replay,
rekeying, and transport/tunnel modes. Keep keys out of logs and test artifacts.
Implement Mobile IPv6 home/mobile-node behavior, binding lifecycle, authentication,
and route optimization, plus IPv6 jumbogram parsing, I/O, and checksum semantics.

Gate: independent peers pass encrypted transport/tunnel, replay/rekey, mobility/
handover, and jumbogram tests. Invalid extension chains, authentication failures,
MTU changes, and resource exhaustion cannot bypass policy or corrupt state.

### device-offloads: Implementing Rust drivers and accelerated packet paths

Pin a hardware matrix spanning x86_64, ARM64, and riscv64, with virtio-net in QEMU
and representative physical NICs selected for dedicated hardware testing.
Implement Rust drivers for the selected devices while maintaining compatibility
with existing Linux drivers. Inventory DMA, IRQ/NAPI, queue, reset, and teardown
contracts; device-specific capabilities must be reported accurately.

Implement checksum/segmentation offloads, GRO/GSO integration, RSS and multiqueue,
with software-equivalent paths for devices lacking a capability. Such paths must
use the new stack, not native transport processing. Test IPv4/IPv6, VLAN, NAT,
IPsec, and nonlinear skb interactions with offloads enabled and disabled.

Gate: all devices in the hardware matrix pass functional, DMA/lifecycle,
concurrency, reset, offload-equivalence, and sustained-load tests.

### socket-compatibility: Completing Linux networking API parity

Inventory and implement the pinned kernel's AF_INET/AF_INET6 TCP, UDP, and raw-IP
socket flags/options, ancillary data, error queues, and AF_PACKET interfaces.
Include multicast options, capabilities, network namespaces,
timeouts, ioctl/netlink administration, and BPF/socket-filter integration.
Cover sendfile/splice, io_uring networking, and zero-copy APIs exposed by the
target, with correct lifetime and completion semantics.

Gate: differential syscall and networking selftests match native Linux's
documented behavior across all inventory entries, including privileged/unprivileged
access, mapped/scoped addresses, readiness, signals, and concurrent teardown.
Every discrepancy must be resolved or demonstrated to be permitted by the
reference contract; narrowing the inventory is not a substitute for completion.

### robustness-validation: Validating the complete kernel stack

Run host property tests/fuzzing on parsers and state transitions; retain minimized
failure cases. Use Miri for supported portable code, not as proof of kernel FFI
soundness. Add kernel KUnit tests where supported and syscall kselftests.

Run VM tests with KASAN/KCSAN/lockdep and allocation fault injection where supported
by the pinned configuration; record coverage gaps, including Rust instrumentation
limitations. Stress concurrent RX/TX/close, device down, and namespace destruction.
Review every unsafe boundary and timer/work cancellation ownership contract.
Run the applicable conformance, socket, and lifecycle suites on all three
architectures for both IP families, including IPv6 discovery, address/route
expiry, multicast membership, and cross-family socket behavior.
Include routed and encrypted traffic, service/control-plane lifecycle, raw/packet
sockets, drivers, offloads, mobility, and all compatibility-inventory entries.
Exercise ARM64 and RISC-V weak-memory-ordering behavior on
dedicated hardware for each architecture before release; passing x86_64 tests
or QEMU alone is not sufficient evidence.

Gate: all declared conformance/lifecycle tests pass, no unresolved data-corruption,
memory-safety, panic, leak, or critical race findings, and every known limitation
is documented. Set and record fuzz/stress budgets before release testing; do not
claim "bug-free" or universal RFC conformance from finite test runs.

### performance-baseline: Establishing reproducible baseline measurements

After UDP socket integration, establish UDP/native comparison and infrastructure;
add TCP comparison only after stream integration. Compare custom/native stacks
using matched kernel config, CPU allocation, MTU, queue sizes, topology, offload
settings, and equivalent socket semantics. Disable offloads for the first fair
baseline and keep instrumented correctness builds separate from benchmark builds.

Measure goodput and packets/s, p50/p95/p99 latency with stated load methodology,
CPU cycles/byte or cycles/packet, allocation/copy counts, connection-establishment
rate, and memory per idle/active connection. Include small/MTU-sized packets,
single/many flows, saturation/backpressure, and loss/reordering conditions.
Measure IPv4 and IPv6 separately, including their different header overhead and
path-MTU behavior; compare steady-state neighbor caches separately from discovery.
Benchmark forwarding, NAT/firewall, multicast, encryption, service workloads,
and offload-enabled/disabled paths alongside endpoint transport.
Record repeated-run variation and workload parameters in machine-readable output.
Maintain separate x86_64, ARM64, and riscv64 native/custom baselines. Report CPU model,
architecture, and compiler settings; do not extrapolate one architecture's
performance to another.

Gate: measurements are repeatable; explicitly agree numeric acceptance targets
for named workloads/hardware before declaring the stack performant. QEMU is a
functional/relative baseline, not evidence of physical NIC line-rate performance.
Select dedicated Linux hardware for any hardware-level performance claim.
Apply the agreed performance gates independently on x86_64, ARM64, and riscv64
hardware.

### optimization-release: Optimizing measured bottlenecks and documenting release

Profile first, then consider copy reduction, batching, cache locality, per-flow/
per-CPU sharding, and timer efficiency only where measurements justify them.
Preserve bounded resource use and the correctness oracle. Each optimization must
pass the relevant regressions and demonstrate measurable benefit.

Gate: every scope and compatibility-inventory entry, all implementation
milestones, and the complete robustness suite pass on final code; agreed numeric
targets pass for both families without hiding loss, corruption, or resource
growth. Publish build/run
instructions, socket/protocol compatibility, benchmark conditions/results, limits,
and recovery/debugging guidance. Release remains experimental until broader
deployment evidence supports a stronger claim.

## Resource and error policy

Before each subsystem is enabled, record explicit byte/count/time limits for its
queues, caches, fragments, socket buffers, backlog, timers, and connections.
Limits must be testable, with expiry and cleanup independent of peer cooperation.
Use kernel-approved fallible allocation and propagate errno/structured failures.
Expected malformed-packet drops use reason counters and rate-limited diagnostics;
configuration and syscall failures remain visible to callers. Never silently
fall back to native TCP/IP or claim success after dropping user data.

## Dependency structure

Feasibility precedes foundation, then IPv4 and IPv6 endpoint work. IPv6 reuses
the Ethernet/device integration established in the IPv4 milestone.
Both endpoints are prerequisites for completing UDP sockets. The TCP engine can
begin after the foundation, but its wire integration gate requires both endpoints.
TCP socket integration needs both the TCP engine and the shared socket adapter
established with UDP. Robustness validation needs both socket implementations.
Routing/policy, network services, IP extensions, and device/offload integration
build on the endpoints and transport/socket foundation. Complete socket API
parity requires the raw/packet, policy, extension, and device paths.
Robustness validation requires all of these milestones. Each feature's benchmark
baseline requires its functional integration; complete baselines cover the full
scope before release optimization is accepted.
Benchmark infrastructure can begin after UDP; its complete TCP baseline needs TCP
sockets. Final optimization/release needs robustness and complete baselines.
Tests are developed with each milestone, not postponed until final validation.

## Standards and primary references

- [Kernel Rust documentation](https://docs.kernel.org/rust/index.html)
- [Kernel Rust quick start](https://docs.kernel.org/rust/quick-start.html)
- [Kernel Rust architecture support](https://docs.kernel.org/rust/arch-support.html)
- [Kernel Rust abstractions and no_std](https://docs.kernel.org/rust/general-information.html)
- [Kernel Rust testing](https://docs.kernel.org/rust/testing.html)
- [Linux networking APIs](https://docs.kernel.org/networking/kapi.html)
- [ARP: RFC 826](https://www.rfc-editor.org/rfc/rfc826.html)
- [IPv4: RFC 791](https://www.rfc-editor.org/rfc/rfc791.html)
- [ICMP: RFC 792](https://www.rfc-editor.org/rfc/rfc792.html)
- [IPv6: RFC 8200](https://www.rfc-editor.org/rfc/rfc8200.html)
- [IPv6 addressing: RFC 4291](https://www.rfc-editor.org/rfc/rfc4291.html)
- [IPv6 node requirements: RFC 8504](https://www.rfc-editor.org/rfc/rfc8504.html)
- [ICMPv6: RFC 4443](https://www.rfc-editor.org/rfc/rfc4443.html)
- [Neighbor Discovery: RFC 4861](https://www.rfc-editor.org/rfc/rfc4861.html)
- [SLAAC and DAD: RFC 4862](https://www.rfc-editor.org/rfc/rfc4862.html)
- [IPv6 path MTU discovery: RFC 8201](https://www.rfc-editor.org/rfc/rfc8201.html)
- [IPv6 fragment overlap rejection: RFC 5722](https://www.rfc-editor.org/rfc/rfc5722.html)
- [MLDv2: RFC 9777](https://www.rfc-editor.org/rfc/rfc9777.html)
- [UDP: RFC 768](https://www.rfc-editor.org/rfc/rfc768.html)
- [Host requirements: RFC 1122](https://www.rfc-editor.org/rfc/rfc1122.html)
- [Path MTU discovery: RFC 1191](https://www.rfc-editor.org/rfc/rfc1191.html)
- [TCP: RFC 9293](https://www.rfc-editor.org/rfc/rfc9293.html)
- [TCP congestion control: RFC 5681](https://www.rfc-editor.org/rfc/rfc5681.html)
- [TCP retransmission timer: RFC 6298](https://www.rfc-editor.org/rfc/rfc6298.html)

Maintain a requirement-to-test matrix and check applicable updates/errata when
implementing each protocol. Kernel documentation above is rolling; the selected
kernel source and its generated Rust API documentation are the build authority.
