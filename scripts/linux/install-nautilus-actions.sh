#!/bin/bash
# ==========================================
# OrdoProject - Nautilus (GNOME Files) Actions Installer
# ==========================================
# This script creates Nautilus actions for the context menu

set -e

echo "=========================================="
echo "OrdoProject Nautilus Actions Installer"
echo "=========================================="
echo ""

# Check if nautilus-actions is available
if ! command -v nautilus-actions-config-tool &> /dev/null; then
    echo "WARNING: nautilus-actions is not installed."
    echo "Install it with: sudo apt install nautilus-actions"
    echo ""
    echo "Alternatively, this script will create .desktop files for Nautilus scripts."
    echo ""
fi

# Find the OrdoProject executable
ORDO_EXE=""

if [ -f "/usr/local/bin/ordoproject" ]; then
    ORDO_EXE="/usr/local/bin/ordoproject"
elif [ -f "/usr/bin/ordoproject" ]; then
    ORDO_EXE="/usr/bin/ordoproject"
elif [ -f "$HOME/.local/bin/ordoproject" ]; then
    ORDO_EXE="$HOME/.local/bin/ordoproject"
elif [ -f "$(dirname "$0")/../../dist/linux-unpacked/ordoproject" ]; then
    ORDO_EXE="$(cd "$(dirname "$0")/../../dist/linux-unpacked" && pwd)/ordoproject"
fi

if [ -z "$ORDO_EXE" ]; then
    echo "ERROR: Could not find ordoproject executable"
    echo "Please enter the full path to ordoproject:"
    read -r ORDO_EXE
fi

if [ ! -f "$ORDO_EXE" ]; then
    echo "ERROR: Executable not found: $ORDO_EXE"
    exit 1
fi

echo "Using executable: $ORDO_EXE"
echo ""

# Create Nautilus scripts directory
SCRIPTS_DIR="$HOME/.local/share/nautilus/scripts"
mkdir -p "$SCRIPTS_DIR"

# Create "Sort with Ordo" script
echo "Creating Nautilus script: Sort with Ordo..."

cat > "$SCRIPTS_DIR/Sort with Ordo" << EOF
#!/bin/bash
# Sort selected files/folders with OrdoProject

for file in "\$NAUTILUS_SCRIPT_SELECTED_FILE_PATHS"; do
    "$ORDO_EXE" "\$file"
done
EOF

chmod +x "$SCRIPTS_DIR/Sort with Ordo"

# Create "Sort All with Ordo" script
echo "Creating Nautilus script: Sort All with Ordo..."

cat > "$SCRIPTS_DIR/Sort All with Ordo" << EOF
#!/bin/bash
# Sort all files in selected folder with OrdoProject

for folder in "\$NAUTILUS_SCRIPT_SELECTED_FILE_PATHS"; do
    if [ -d "\$folder" ]; then
        "$ORDO_EXE" "\$folder" ALL
    fi
done
EOF

chmod +x "$SCRIPTS_DIR/Sort All with Ordo"

echo ""
echo "=========================================="
echo "SUCCESS! Nautilus scripts installed."
echo "=========================================="
echo ""
echo "The following scripts are now available in Nautilus:"
echo "  Right-click files/folders > Scripts > Sort with Ordo"
echo "  Right-click folders > Scripts > Sort All with Ordo"
echo ""
echo "You may need to restart Nautilus:"
echo "  nautilus -q"
echo ""
