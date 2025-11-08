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
	rm -rf .vagrant-cache

run: build
	target/release/vagrant | tee vagrant.log
	sed -i 's,\x1b\[[0-9;]*m,,g' vagrant.log

test: build
	cargo test --no-fail-fast --future-incompat-report --all-features --locked --release
	target/release/vagrant -pg | tee vagrant.log
	sed -i 's,\x1b\[[0-9;]*m,,g' vagrant.log
	! grep -E 'ERROR|WARN' vagrant.log

.PHONY: all build check clean fmt format lint purge run test
