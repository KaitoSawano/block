.PHONY: all build clean test

all: build

build:
	cargo build --release
	cp target/release/zenachain ./zenachain
	cp target/release/zenachaind ./zenachaind
	cp target/release/zenachain-miner ./zenachain-miner
	chmod +x ./zenachain ./zenachaind ./zenachain-miner

test:
	cargo test

clean:
	cargo clean
	rm -f ./zenachain ./zenachaind ./zenachain-miner
