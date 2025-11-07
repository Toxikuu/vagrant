all: build

build:
	cargo build --release

check: lint test

clean:
	cargo clean

fmt: format

format:
	rustup component add --toolchain nightly-x86_64-unknown-linux-gnu rustfmt
	cargo +nightly fmt

lint:
	cargo clippy
	typos

purge: clean
	rm -f .vagrant-cache

run: build
	target/release/vagrant | tee vagrant.log

test: build
	cargo test --no-fail-fast --future-incompat-report --all-features --locked
	target/release/vagrant -pg | tee vagrant.log
	! grep -E 'ERROR|WARN' vagrant.log

.PHONY: all build check clean fmt format lint purge run test
