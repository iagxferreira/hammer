SHELL := /bin/sh

.PHONY: build check fmt test clean run

build:
	cargo build --release

check:
	cargo fmt --check
	cargo check
	cargo test

fmt:
	cargo fmt

test:
	cargo test

clean:
	cargo clean

run:
	cargo run --
