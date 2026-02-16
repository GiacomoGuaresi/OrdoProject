#!/bin/bash
# ==========================================
# OrdoProject - Linux Sorting Script
# ==========================================
# This script can be called from file managers or command line

# Find the OrdoProject executable
ORDO_EXE=""

# Check common installation locations
if [ -f "/usr/local/bin/ordoproject" ]; then
    ORDO_EXE="/usr/local/bin/ordoproject"
elif [ -f "/usr/bin/ordoproject" ]; then
    ORDO_EXE="/usr/bin/ordoproject"
elif [ -f "$HOME/.local/bin/ordoproject" ]; then
    ORDO_EXE="$HOME/.local/bin/ordoproject"
elif [ -f "$(dirname "$0")/../../dist/linux-unpacked/ordoproject" ]; then
    ORDO_EXE="$(cd "$(dirname "$0")/../../dist/linux-unpacked" && pwd)/ordoproject"
fi

# If not found, try to find it
if [ -z "$ORDO_EXE" ]; then
    ORDO_EXE=$(which ordoproject 2>/dev/null)
fi

# If still not found, error
if [ -z "$ORDO_EXE" ] || [ ! -f "$ORDO_EXE" ]; then
    echo "ERROR: Could not find ordoproject executable"
    echo "Please install OrdoProject or update this script with the correct path"
    exit 1
fi

# Check if we should process all files in directory
if [ "$2" = "ALL" ]; then
    "$ORDO_EXE" "$1" ALL
else
    "$ORDO_EXE" "$@"
fi
