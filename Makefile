
.PHONY: build-dev
build-dev:
	maturin develop

.PHONY: build-prod
build-prod:
	maturin develop --release

.PHONY: test-rs
test-rs:
	cargo clippy
	cargo test -- --no-capture

.PHONY: test-py
test-py: build-dev
	pytest
	-rm -rf .pytest_cache
	-ruff check .
	ruff clean

.PHONY: format
format:
	ruff format --line-length=100 .
	ruff clean
	cargo fmt

.PHONY: clean
clean:
	-rm -rf `find . -name __pycache__`
	-rm -rf python/xmarkup/*.so
	-rm -rf target/release
	-rm -rf .pytest_cache
	-ruff clean
