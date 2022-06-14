PROGRAM = eye
TARGET = armv7-unknown-linux-musleabihf

client:
	cargo run --bin client

server:
	cargo run --bin server

test:
	cargo run --bin test

deploy:
	cargo build --bin server --target $(TARGET) --features camera
	scp target/$(TARGET)/debug/$(PROGRAM) $(PI_HOST):$(PROGRAM)
