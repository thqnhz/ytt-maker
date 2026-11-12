.PHONY: all lint dev rel clean

all: dev

lint:
	@cargo fmt
	@cargo clippy

dev:
	@cargo run

rel:
	@cargo build --release --workspace

clean:
	@cargo clean
