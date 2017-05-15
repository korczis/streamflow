# RUSTFLAGS="-C target-feature=+ssse3 -C target-feature=+sse4.2 -C target-cpu=native -C inline-threshold=500"

all:
	build

build-debug:
	cargo build debug

build-release:
	cargo build --release

build:
	build-debug
	build-release

test-cities:
	time ./target/release/sli2dli  -d "	" --manifest tmp/manifest.json tmp/cities-big.csv -vvv

test-incidents:
	time ./target/release/sli2dli --has-header --manifest tmp/manifest.json tmp/incidents.csv -vvv

test:
	buld-debug
	build-release

