#!/bin/bash

set -e

echo "🔧 Installing Kern..."

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
   echo "❌ Please don't run as root"
   exit 1
fi

# Check if binary exists
if [ ! -f "target/release/kern" ]; then
    echo "❌ Binary not found. Run 'cargo build --release' first"
    exit 1
fi

# Install binary
echo "📦 Installing binary..."
sudo cp target/release/kern /usr/local/bin/
sudo chmod +x /usr/local/bin/kern

# Create config directory
echo "⚙️  Setting up configuration..."
mkdir -p ~/.config/kern/profiles
cp -r config/* ~/.config/kern/

# Install systemd service
echo "🔧 Installing systemd service..."
sudo cp systemd/kern.service /etc/systemd/system/
sudo systemctl daemon-reload

echo ""
echo "✅ Kern installed successfully!"
echo ""
echo "Next steps:"
echo "  1. Start service:  systemctl --user start kern"
echo "  2. Enable on boot: systemctl --user enable kern"
echo "  3. Check status:   kern status"
echo ""
