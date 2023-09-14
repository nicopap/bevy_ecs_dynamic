check:
	cargo clippy
run:
	RUST_BACKTRACE=1 cargo run -p query_interpreter
