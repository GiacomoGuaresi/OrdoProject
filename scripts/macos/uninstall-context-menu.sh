#!/bin/bash
# ==========================================
# OrdoProject - macOS Context Menu Uninstaller
# ==========================================

echo "=========================================="
echo "OrdoProject macOS Context Menu Uninstaller"
echo "=========================================="
echo ""
echo "This will remove OrdoProject Quick Actions from Finder."
echo ""
read -p "Press Enter to continue or Ctrl+C to cancel..."

WORKFLOWS_DIR="$HOME/Library/Services"

echo ""
echo "Removing Quick Actions..."

rm -rf "$WORKFLOWS_DIR/Sort with Ordo.workflow"
rm -rf "$WORKFLOWS_DIR/Sort All with Ordo.workflow"

echo ""
echo "=========================================="
echo "Quick Actions removed."
echo "=========================================="
echo ""
echo "You may need to restart Finder for changes to take effect."
echo ""
