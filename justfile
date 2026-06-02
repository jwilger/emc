set shell := ["sh", "-eu", "-c"]
set quiet := true

fmt:
	RUSTFLAGS='-Dwarnings' cargo fmt --all --check

clippy:
	RUSTFLAGS='-Dwarnings' cargo clippy --all-targets --all-features -- -D warnings

test:
	RUSTFLAGS='-Dwarnings' cargo test

build:
	RUSTFLAGS='-Dwarnings' cargo build

ci: fmt clippy test build
