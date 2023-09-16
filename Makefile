check:
	cargo clippy
run:
	RUST_LOG=trace RUST_BACKTRACE=1 cargo test
