all: build

run: build
	target/release/vagrant

build:
	cargo build --release

clean:
	cargo clean

purge: clean
	rm -f .vagrant-cache

lint:
	cargo clippy

test:
	cargo test --no-fail-fast --future-incompat-report --all-features --locked

check: lint test

format:
	rustup component add --toolchain nightly-x86_64-unknown-linux-gnu rustfmt
	cargo +nightly fmt

fmt: format

.PHONY: all build check clean fmt format lint run test
