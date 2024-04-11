.PHONY: build clean run

build:
	cargo build

release:
	cargo build --release

clean:
	cargo clean

run:
	cargo run

doc:
	cargo doc

# Add more custom commands as needed
