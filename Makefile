PROGRAM = eye
TARGET = armv7-unknown-linux-musleabihf

client:
	cargo run --bin client

server:
	cargo run --bin server

deploy:
	cargo build --bin server --target $(TARGET)
	scp target/$(TARGET)/debug/server $(PI_HOST):$(PROGRAM)
	scp -r config $(PI_HOST):.config/$(PROGRAM)
