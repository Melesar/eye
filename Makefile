PROGRAM = eye
TARGET = armv7-unknown-linux-musleabihf

client:
	cargo run --bin client

server:
	cargo run --bin server

deploy:
	cargo build --release --features servo --bin server --target $(TARGET)
	scp target/$(TARGET)/release/server $(PI_HOST):$(PROGRAM)
	scp -r config $(PI_HOST):.config/$(PROGRAM)
