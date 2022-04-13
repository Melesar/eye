PROGRAM = eye
TARGET = armv7-unknown-linux-musleabihf
HOST = pi@192.168.1.251

client:
	cargo run --features client

server:
	cargo build --target $(TARGET)
	scp target/$(TARGET)/debug/$(PROGRAM) $(HOST):$(PROGRAM)
	ssh $(HOST) -t sudo ./$(PROGRAM)
