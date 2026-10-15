.PHONY: all build clean test

all: build

build:
	cargo build --release
	cp target/release/zenachain ./zenachain
	cp target/release/zenachaind ./zenachaind
	chmod +x ./zenachain ./zenachaind

test:
	cargo test

clean:
	cargo clean
	rm -f ./zenachain ./zenachaind
