all:
	build

build-debug:
	cargo build

build-release:
	# RUSTFLAGS="-C target-feature=+ssse3 -C target-feature=+sse4.2 -C target-cpu=native -C inline-threshold=1000" cargo build --release
	cargo build --release

build:
	build-debug
	build-release
