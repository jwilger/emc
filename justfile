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

mutants-diff:
	changed="$(git diff --name-only HEAD -- src | grep -E '\.rs$' || true)"; if [ -n "$changed" ]; then tmp="$(mktemp)"; trap 'rm -f "$tmp"' EXIT; git diff HEAD -- src > "$tmp"; cargo mutants --in-diff "$tmp" --cap-lints true; else printf '%s\n' "no Rust source diff for mutation testing"; fi

mutants-core:
	cargo mutants --cap-lints true --file src/core/workflow.rs --file src/core/slice.rs --file src/core/connection.rs

mutants-full:
	cargo mutants --cap-lints true
