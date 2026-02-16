#!/bin/bash
# ==========================================
# OrdoProject - Nautilus Actions Uninstaller
# ==========================================

echo "=========================================="
echo "OrdoProject Nautilus Actions Uninstaller"
echo "=========================================="
echo ""
echo "This will remove OrdoProject scripts from Nautilus."
echo ""
read -p "Press Enter to continue or Ctrl+C to cancel..."

SCRIPTS_DIR="$HOME/.local/share/nautilus/scripts"

echo ""
echo "Removing Nautilus scripts..."

rm -f "$SCRIPTS_DIR/Sort with Ordo"
rm -f "$SCRIPTS_DIR/Sort All with Ordo"

echo ""
echo "=========================================="
echo "Nautilus scripts removed."
echo "=========================================="
echo ""
echo "You may need to restart Nautilus:"
echo "  nautilus -q"
echo ""
