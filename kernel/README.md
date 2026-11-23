# Kernel adapter

This directory is reserved for the Kconfig/Kbuild adapter and the minimal C
helpers required by the pinned Linux tree. The architecture-independent core
is in [`../core/`](../core/).

Kernel code is intentionally not stubbed here. `kernel-feasibility` in the
[implementation plan](../docs/plan.md#kernel-feasibility-proving-kernel-and-rust-integration)
must first establish a pinned Linux commit, supported Rust APIs, socket and
packet dispatch ownership, and the complete C/Rust safety boundary.
