.PHONY: all check fmt test

all: fmt check test

fmt:
	cargo fmt --all -- --check

check:
	cargo check -p netstack-core --target x86_64-unknown-none
	cargo check -p netstack-core --target aarch64-unknown-none
	cargo check -p netstack-core --target riscv64gc-unknown-none-elf

test:
	cargo test --workspace --all-features
