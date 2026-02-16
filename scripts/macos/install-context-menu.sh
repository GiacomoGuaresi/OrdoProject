#!/bin/bash
# ==========================================
# OrdoProject - macOS Context Menu Installer
# ==========================================
# This script creates Automator Quick Actions for Finder context menu

set -e

echo "=========================================="
echo "OrdoProject macOS Context Menu Installer"
echo "=========================================="
echo ""

# Find the OrdoProject.app
ORDO_APP=""

# Check in Applications
if [ -d "/Applications/OrdoProject.app" ]; then
    ORDO_APP="/Applications/OrdoProject.app"
    echo "Found OrdoProject.app in /Applications"
# Check in user Applications
elif [ -d "$HOME/Applications/OrdoProject.app" ]; then
    ORDO_APP="$HOME/Applications/OrdoProject.app"
    echo "Found OrdoProject.app in ~/Applications"
# Check in dist/mac (development build)
elif [ -d "$(dirname "$0")/../../dist/mac/OrdoProject.app" ]; then
    ORDO_APP="$(cd "$(dirname "$0")/../../dist/mac" && pwd)/OrdoProject.app"
    echo "Found OrdoProject.app in dist/mac"
fi

# If not found, ask user
if [ -z "$ORDO_APP" ]; then
    echo ""
    echo "ERROR: Could not find OrdoProject.app automatically."
    echo "Please enter the full path to OrdoProject.app:"
    read -r ORDO_APP
fi

# Verify the app exists
if [ ! -d "$ORDO_APP" ]; then
    echo ""
    echo "ERROR: Application not found: $ORDO_APP"
    echo "Please check the path and try again."
    exit 1
fi

ORDO_EXE="$ORDO_APP/Contents/MacOS/OrdoProject"

if [ ! -f "$ORDO_EXE" ]; then
    echo ""
    echo "ERROR: Executable not found: $ORDO_EXE"
    exit 1
fi

echo ""
echo "Using application: $ORDO_APP"
echo ""

# Create Quick Actions directory
WORKFLOWS_DIR="$HOME/Library/Services"
mkdir -p "$WORKFLOWS_DIR"

# Create "Sort with Ordo" Quick Action for files
echo "Creating Quick Action: Sort with Ordo (Files)..."

WORKFLOW_FILE="$WORKFLOWS_DIR/Sort with Ordo.workflow"
mkdir -p "$WORKFLOW_FILE/Contents"

cat > "$WORKFLOW_FILE/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>NSServices</key>
    <array>
        <dict>
            <key>NSMenuItem</key>
            <dict>
                <key>default</key>
                <string>Sort with Ordo</string>
            </dict>
            <key>NSMessage</key>
            <string>runWorkflowAsService</string>
            <key>NSRequiredContext</key>
            <dict>
                <key>NSApplicationIdentifier</key>
                <string>com.apple.finder</string>
            </dict>
            <key>NSSendFileTypes</key>
            <array>
                <string>public.item</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
EOF

mkdir -p "$WORKFLOW_FILE/Contents/QuickLook"
mkdir -p "$WORKFLOW_FILE/Contents/document.wflow"

cat > "$WORKFLOW_FILE/Contents/document.wflow/document.wflow" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>actions</key>
    <array>
        <dict>
            <key>action</key>
            <dict>
                <key>AMActionVersion</key>
                <string>1.0</string>
                <key>AMApplication</key>
                <array>
                    <string>Automator</string>
                </array>
                <key>AMParameterProperties</key>
                <dict/>
                <key>AMProvides</key>
                <dict>
                    <key>Container</key>
                    <string>List</string>
                    <key>Types</key>
                    <array>
                        <string>com.apple.cocoa.string</string>
                    </array>
                </dict>
                <key>ActionBundlePath</key>
                <string>/System/Library/Automator/Run Shell Script.action</string>
                <key>ActionName</key>
                <string>Run Shell Script</string>
                <key>ActionParameters</key>
                <dict>
                    <key>COMMAND_STRING</key>
                    <string>for f in "\$@"
do
    "$ORDO_EXE" "\$f"
done</string>
                    <key>CheckedForUserDefaultShell</key>
                    <true/>
                    <key>inputMethod</key>
                    <integer>1</integer>
                    <key>shell</key>
                    <string>/bin/bash</string>
                    <key>source</key>
                    <string></string>
                </dict>
            </dict>
        </dict>
    </array>
</dict>
</plist>
EOF

# Create "Sort All with Ordo" Quick Action for folders
echo "Creating Quick Action: Sort All with Ordo (Folders)..."

WORKFLOW_ALL_FILE="$WORKFLOWS_DIR/Sort All with Ordo.workflow"
mkdir -p "$WORKFLOW_ALL_FILE/Contents"

cat > "$WORKFLOW_ALL_FILE/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>NSServices</key>
    <array>
        <dict>
            <key>NSMenuItem</key>
            <dict>
                <key>default</key>
                <string>Sort All with Ordo</string>
            </dict>
            <key>NSMessage</key>
            <string>runWorkflowAsService</string>
            <key>NSRequiredContext</key>
            <dict>
                <key>NSApplicationIdentifier</key>
                <string>com.apple.finder</string>
            </dict>
            <key>NSSendFileTypes</key>
            <array>
                <string>public.folder</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
EOF

mkdir -p "$WORKFLOW_ALL_FILE/Contents/QuickLook"
mkdir -p "$WORKFLOW_ALL_FILE/Contents/document.wflow"

cat > "$WORKFLOW_ALL_FILE/Contents/document.wflow/document.wflow" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>actions</key>
    <array>
        <dict>
            <key>action</key>
            <dict>
                <key>AMActionVersion</key>
                <string>1.0</string>
                <key>AMApplication</key>
                <array>
                    <string>Automator</string>
                </array>
                <key>AMParameterProperties</key>
                <dict/>
                <key>AMProvides</key>
                <dict>
                    <key>Container</key>
                    <string>List</string>
                    <key>Types</key>
                    <array>
                        <string>com.apple.cocoa.string</string>
                    </array>
                </dict>
                <key>ActionBundlePath</key>
                <string>/System/Library/Automator/Run Shell Script.action</string>
                <key>ActionName</key>
                <string>Run Shell Script</string>
                <key>ActionParameters</key>
                <dict>
                    <key>COMMAND_STRING</key>
                    <string>for f in "\$@"
do
    "$ORDO_EXE" "\$f" ALL
done</string>
                    <key>CheckedForUserDefaultShell</key>
                    <true/>
                    <key>inputMethod</key>
                    <integer>1</integer>
                    <key>shell</key>
                    <string>/bin/bash</string>
                    <key>source</key>
                    <string></string>
                </dict>
            </dict>
        </dict>
    </array>
</dict>
</plist>
EOF

echo ""
echo "=========================================="
echo "SUCCESS! Quick Actions installed."
echo "=========================================="
echo ""
echo "The following Quick Actions are now available:"
echo "  - Right-click files/folders: 'Sort with Ordo'"
echo "  - Right-click folders: 'Sort All with Ordo'"
echo ""
echo "Note: You may need to:"
echo "  1. Restart Finder (Option+Right-click Finder icon > Relaunch)"
echo "  2. Enable the services in System Settings > Keyboard > Shortcuts > Services"
echo ""
