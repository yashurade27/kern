.PHONY: build install uninstall clean test run dev extension

# Build release binary
build:
	cargo build --release

# Build debug binary
dev:
	cargo build

# Run in development mode
run:
	cargo run -- status

# Run tests
test:
	cargo test

# Install to system
install: build
	sudo cp target/release/kern /usr/local/bin/
	mkdir -p ~/.config/kern
	cp -r config/* ~/.config/kern/
	sudo cp systemd/kern.service /etc/systemd/system/
	@echo "✅ Kern installed successfully"
	@echo "Run 'systemctl --user enable kern' to start on boot"

# Uninstall from system
uninstall:
	sudo rm -f /usr/local/bin/kern
	sudo rm -f /etc/systemd/system/kern.service
	systemctl --user stop kern 2>/dev/null || true
	@echo "✅ Kern uninstalled"

# Clean build artifacts
clean:
	cargo clean

# Build GNOME extension
extension:
	./scripts/build-extension.sh

# Format code
fmt:
	cargo fmt

# Lint code
lint:
	cargo clippy

# Check code without building
check:
	cargo check
