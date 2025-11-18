#!/bin/bash

set -e

echo "🗑️  Uninstalling KERN Resource Manager..."
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
   echo "❌ ERROR: Please don't run as root. User services are per-user."
   exit 1
fi

# Determine binary location
BINARY_PATH="$HOME/.local/bin/kern"
if [ -d "$HOME/.cargo/bin" ]; then
    BINARY_PATH="$HOME/.cargo/bin/kern"
fi

# Stop the service
echo "⏹️  Stopping KERN service..."
if systemctl --user is-active --quiet kern.service; then
    systemctl --user stop kern.service
    echo "   ✓ Service stopped"
else
    echo "   ℹ️  Service is not running"
fi

# Disable the service
echo "🔄 Disabling systemd service..."
if systemctl --user is-enabled --quiet kern.service; then
    systemctl --user disable kern.service
    echo "   ✓ Service disabled"
else
    echo "   ℹ️  Service not enabled"
fi

# Remove service file
echo "📁 Removing service files..."
rm -f "$HOME/.config/systemd/user/kern.service"
echo "   ✓ Service file removed"

# Reload systemd
systemctl --user daemon-reload

# Remove binary
echo "🗑️  Removing binary..."
if [ -f "$BINARY_PATH" ]; then
    rm -f "$BINARY_PATH"
    echo "   ✓ Binary removed from $BINARY_PATH"
else
    echo "   ℹ️  Binary not found at $BINARY_PATH"
fi

# Ask about config removal
echo ""
echo "⚠️  Configuration files:"
echo "   Config directory: ~/.config/kern/"
read -p "   Remove config directory? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -rf "$HOME/.config/kern"
    echo "   ✓ Config directory removed"
else
    echo "   ✓ Config directory preserved"
fi

# Ask about log removal
echo ""
echo "📝 Log files:"
echo "   Log location: ~/.config/kern/kern.log"
if [ -f "$HOME/.config/kern/kern.log" ]; then
    read -p "   Remove log file? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -f "$HOME/.config/kern/kern.log"
        echo "   ✓ Log file removed"
    else
        echo "   ✓ Log file preserved"
    fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ KERN uninstalled successfully!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📍 Uninstall Summary:"
echo "   Binary:     Removed from $BINARY_PATH"
echo "   Service:    Removed"
echo "   Config:     $([ -d "$HOME/.config/kern" ] && echo 'Preserved' || echo 'Removed')"
echo ""
echo "✨ Thank you for using KERN!"
echo ""
