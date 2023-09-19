.PHONY: fmt
fmt:
	cargo fmt
	cargo tomlfmt
	cargo clippy
