# OrdoProject - Installation Scripts

This folder contains platform-specific scripts for installing context menu integration and file sorting actions.

## 📁 Directory Structure

```
scripts/
├── windows/
│   ├── install-context-menu.bat      # Install Windows context menu
│   └── uninstall-context-menu.bat    # Remove Windows context menu
├── macos/
│   ├── install-context-menu.sh       # Install macOS Quick Actions
│   └── uninstall-context-menu.sh     # Remove macOS Quick Actions
├── linux/
│   ├── ordo-sort.sh                  # Generic sorting script
│   ├── install-nautilus-actions.sh   # Install Nautilus (GNOME) scripts
│   └── uninstall-nautilus-actions.sh # Remove Nautilus scripts
└── README.md                          # This file
```

---

## 🪟 Windows

### Installation

1. **Build the application:**
   ```bash
   npm run dist
   ```

2. **Run the installer script:**
   - Double-click `scripts/windows/install-context-menu.bat`
   - Or run from command line:
     ```cmd
     cd scripts\windows
     install-context-menu.bat
     ```

3. **The script will:**
   - Automatically detect the OrdoProject executable
   - Create a temporary registry file with correct paths
   - Install context menu entries

### Features

After installation, you can right-click on:
- **Files**: "Sort this with Ordo"
- **Folders**: "Sort this folder with Ordo"
- **Folder background**: "Sort all with Ordo"

### Uninstallation

Run `scripts/windows/uninstall-context-menu.bat` to remove all context menu entries.

### Manual Installation

If the automatic script doesn't work, you can manually edit `AddOrdoToContextMenu.reg` in the root directory and replace all instances of the hardcoded path with your actual installation path, then double-click the `.reg` file.

---

## 🍎 macOS

### Installation

1. **Build the application:**
   ```bash
   npm run dist
   ```

2. **Make the script executable:**
   ```bash
   chmod +x scripts/macos/install-context-menu.sh
   ```

3. **Run the installer:**
   ```bash
   ./scripts/macos/install-context-menu.sh
   ```

4. **Enable the Quick Actions:**
   - Open **System Settings** > **Keyboard** > **Shortcuts** > **Services**
   - Find "Sort with Ordo" and "Sort All with Ordo"
   - Check the boxes to enable them

5. **Restart Finder (if needed):**
   - Option+Right-click the Finder icon in the Dock
   - Select "Relaunch"

### Features

After installation, you can:
- Right-click files/folders: "Sort with Ordo"
- Right-click folders: "Sort All with Ordo"

### Uninstallation

```bash
chmod +x scripts/macos/uninstall-context-menu.sh
./scripts/macos/uninstall-context-menu.sh
```

---

## 🐧 Linux

### Installation (Nautilus/GNOME Files)

1. **Build the application:**
   ```bash
   npm run dist
   ```

2. **Make scripts executable:**
   ```bash
   chmod +x scripts/linux/*.sh
   ```

3. **Run the installer:**
   ```bash
   ./scripts/linux/install-nautilus-actions.sh
   ```

4. **Restart Nautilus:**
   ```bash
   nautilus -q
   ```

### Features

After installation, you can:
- Right-click files/folders > **Scripts** > "Sort with Ordo"
- Right-click folders > **Scripts** > "Sort All with Ordo"

### Uninstallation

```bash
./scripts/linux/uninstall-nautilus-actions.sh
```

### Other File Managers

For other Linux file managers (Dolphin, Thunar, etc.), you can use the generic `ordo-sort.sh` script and configure custom actions manually in your file manager's settings.

**Example usage:**
```bash
./scripts/linux/ordo-sort.sh /path/to/file
./scripts/linux/ordo-sort.sh /path/to/folder ALL
```

---

## 🔧 Troubleshooting

### Windows

**Problem:** Script can't find the executable
- Ensure you've built the app with `npm run dist`
- Manually enter the path when prompted
- Check that the executable exists in `dist/win-unpacked/ordoproject.exe`

**Problem:** Context menu doesn't appear
- Make sure you ran the script successfully
- Try logging out and back in
- Check Windows Registry Editor for the entries under `HKEY_CURRENT_USER\Software\Classes`

### macOS

**Problem:** Quick Actions don't appear
- Enable them in System Settings > Keyboard > Shortcuts > Services
- Restart Finder
- Check that the workflow files exist in `~/Library/Services/`

**Problem:** Permission denied
- Make sure you ran `chmod +x` on the script
- Run the script from Terminal, not by double-clicking

### Linux

**Problem:** Scripts don't appear in Nautilus
- Restart Nautilus with `nautilus -q`
- Check that scripts exist in `~/.local/share/nautilus/scripts/`
- Ensure scripts are executable (`chmod +x`)

**Problem:** Executable not found
- Update the path in the script to point to your OrdoProject installation
- Make sure the app is built and the executable exists

---

## 📝 Notes

- All scripts automatically detect the OrdoProject executable location
- Scripts create platform-appropriate context menu entries
- Uninstall scripts are provided for clean removal
- No hardcoded paths - scripts adapt to your installation location

---

## 🤝 Contributing

If you create scripts for other file managers or platforms, please submit a pull request!

Supported file managers we'd like to add:
- Dolphin (KDE)
- Thunar (XFCE)
- Nemo (Cinnamon)
- PCManFM (LXDE)
