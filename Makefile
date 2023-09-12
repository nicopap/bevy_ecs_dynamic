check:
	cargo clippy
run:
	RUST_BACKTRACE=1 RUST_LOG=trace cargo test
