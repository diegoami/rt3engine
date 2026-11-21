.PHONY: test rust fmt clippy evidence-regen evidence-check

# prefer cargo on PATH, fall back to the rustup default location
CARGO := $(shell command -v cargo 2>/dev/null || echo $$HOME/.cargo/bin/cargo)

RT3_HOME ?= $(shell printenv RT3_HOME)

rust:
	$(CARGO) test --workspace

fmt:
	$(CARGO) fmt --all -- --check

clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

# Install-backed evidence: regenerate/check docs/evidence artifacts from the
# original installation behind RT3_HOME. Self-skips without RT3_HOME (CI).
# The script checks every artifact and fails on any drift or producer error.
evidence-regen:
	RT3_HOME="$(RT3_HOME)" CARGO="$(CARGO)" sh tools/evidence.sh regen

evidence-check:
	RT3_HOME="$(RT3_HOME)" CARGO="$(CARGO)" sh tools/evidence.sh check

# Full local suite: Rust workspace + evidence verification.
test: rust evidence-check
