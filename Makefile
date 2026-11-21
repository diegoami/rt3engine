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
evidence-regen:
	@if [ -z "$(RT3_HOME)" ]; then echo "RT3_HOME unset; skipping evidence-regen"; else \
		$(CARGO) run -q -p rt3ctl -- inventory --json > docs/evidence/install-inventory-v1.json; \
		$(CARGO) run -q -p rt3ctl -- pk4 list --json > docs/evidence/pk4-archives-v1.json; \
		echo "regenerated docs/evidence artifacts"; \
	fi

evidence-check:
	@if [ -z "$(RT3_HOME)" ]; then echo "RT3_HOME unset; skipping evidence-check"; else \
		$(CARGO) run -q -p rt3ctl -- inventory --json | diff -q - docs/evidence/install-inventory-v1.json \
		|| (echo "evidence drift: install-inventory-v1.json"; exit 1); \
		$(CARGO) run -q -p rt3ctl -- pk4 list --json | diff -q - docs/evidence/pk4-archives-v1.json \
		|| (echo "evidence drift: pk4-archives-v1.json"; exit 1); \
	fi

# Full local suite: Rust workspace + evidence verification.
test: rust evidence-check
