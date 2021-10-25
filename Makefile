.PHONY: build
build:
	cargo build
	sudo chown .input target/debug/g600-rust
	sudo chmod g+s target/debug/g600-rust

.PHONY: run
run:
	target/debug/g600-rust
