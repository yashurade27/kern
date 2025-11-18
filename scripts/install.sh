#!/bin/bash

set -e

echo "🔧 Installing KERN Resource Manager..."
echo ""

# Check if running as root (user service should not be root)
if [ "$EUID" -eq 0 ]; then 
   echo "❌ ERROR: Please don't run as root. User services are per-user."
   exit 1
fi

# Determine where to install the binary
BINARY_PATH="$HOME/.local/bin/kern"
if [ -d "$HOME/.cargo/bin" ]; then
    BINARY_PATH="$HOME/.cargo/bin/kern"
fi

echo "📁 Creating directories..."
mkdir -p "$HOME/.config/kern/profiles"
mkdir -p "$HOME/.local/bin"

# Check if binary exists
if [ ! -f "target/release/kern" ]; then
    echo "❌ ERROR: Binary not found at target/release/kern"
    echo "   Please run: cargo build --release"
    exit 1
fi

# Install binary to user directory (no sudo needed for user bin)
echo "📦 Installing binary to $BINARY_PATH..."
cp target/release/kern "$BINARY_PATH"
chmod +x "$BINARY_PATH"
echo "   ✓ Binary installed"

# Copy configuration files
echo "⚙️  Setting up configuration..."
cp -r config/* "$HOME/.config/kern/" 2>/dev/null || true
if [ -d "$HOME/.config/kern/profiles" ]; then
    echo "   ✓ Config directory created at ~/.config/kern"
else
    echo "   ✓ ~/.config/kern exists"
fi

# Install systemd user service
echo "🔧 Installing systemd user service..."
mkdir -p "$HOME/.config/systemd/user"
cp systemd/kern.service "$HOME/.config/systemd/user/"
echo "   ✓ Service file installed"

# Enable the service
echo "🔄 Enabling systemd service..."
systemctl --user daemon-reload
systemctl --user enable kern.service
echo "   ✓ Service enabled for auto-start"

# Start the service
echo "▶️  Starting service..."
if systemctl --user start kern.service; then
    echo "   ✓ Service started successfully"
else
    echo "   ⚠️  Could not start service. Check with: systemctl --user status kern"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ KERN installed successfully!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📍 Installation Summary:"
echo "   Binary:     $BINARY_PATH"
echo "   Config:     ~/.config/kern/"
echo "   Service:    ~/.config/systemd/user/kern.service"
echo ""
echo "🎯 Next Steps:"
echo "   • Check status:   systemctl --user status kern"
echo "   • View logs:      journalctl --user -u kern -f"
echo "   • Stop service:   systemctl --user stop kern"
echo "   • Restart:        systemctl --user restart kern"
echo ""
echo "⚙️  Configuration:"
echo "   • Edit config:    $HOME/.config/kern/kern.yaml"
echo "   • Add profiles:   $HOME/.config/kern/profiles/*.yaml"
echo ""
echo "✨ The enforcer will start automatically on login."
echo ""
