PROGRAM = eye
TARGET = armv7-unknown-linux-musleabihf
HOST = pi@192.168.1.251

client:
	cargo run --bin client

server:
	cargo run --bin server

test:
	cargo run --bin test

deploy:
	cargo build --bin server --target $(TARGET)
	scp target/$(TARGET)/debug/$(PROGRAM) $(HOST):$(PROGRAM)
