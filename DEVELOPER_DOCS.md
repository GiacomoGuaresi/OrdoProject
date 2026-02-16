# OrdoProject - Developer Documentation

**Last Updated:** February 2026  
**Version:** 1.0.0  
**Author:** Giacomo Guaresi

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture](#architecture)
3. [Tech Stack](#tech-stack)
4. [Project Structure](#project-structure)
5. [Setup & Development](#setup--development)
6. [Build & Distribution](#build--distribution)
7. [Core Workflows](#core-workflows)
8. [Known Issues & Technical Debt](#known-issues--technical-debt)
9. [Security Considerations](#security-considerations)
10. [Future Roadmap](#future-roadmap)
11. [Troubleshooting](#troubleshooting)
12. [Contributing Guidelines](#contributing-guidelines)

---

## Project Overview

**OrdoProject** is a lightweight Electron-based desktop application that automates file organization using wildcard-based rules. Users define patterns (e.g., `*.jpg`, `invoice_*.pdf`) and target directories, then apply these rules to files, folders, or entire directories.

### Key Features
- Rule-based file sorting with wildcard pattern matching
- Drag-and-drop rule prioritization
- Batch file processing (single file, folder, or all files in directory)
- Movement history with CSV logging
- Cross-platform support (Windows, macOS, Linux)
- Windows context menu integration

---

## Architecture

OrdoProject follows the standard Electron architecture with three main components:

### 1. Main Process (`electron/main.js`)
**Responsibilities:**
- Application lifecycle management
- File system operations (reading, moving, logging)
- Rules storage and retrieval
- IPC (Inter-Process Communication) handlers
- Window creation and management

**Key Functions:**
- `readRules()` - Loads rules from JSON or creates defaults
- `getUniqueDestination()` - Handles file name conflicts
- `logMove()` - Writes movement history to CSV
- Command-line argument processing for context menu integration

### 2. Preload Script (`electron/preload.js`)
**Responsibilities:**
- Secure bridge between main and renderer processes
- Exposes limited API via `contextBridge`

**Exposed API:**
```javascript
window.electronAPI = {
  readRules: () => Promise<Rule[]>,
  writeRules: (rules) => Promise<void>,
  openFolderDialog: () => Promise<string>,
  getHistory: () => Promise<HistoryEntry[]>,
  showInFolder: (path) => Promise<void>
}
```

### 3. Renderer Process (React App)
**Responsibilities:**
- User interface rendering
- Rule management UI
- History display
- User interactions

**Components:**
- `App.jsx` - Main application shell with tab navigation
- `Rules.jsx` - Rule editor with drag-and-drop
- `History.jsx` - Movement history viewer
- `theme.jsx` - Material-UI dark theme configuration

---

## Tech Stack

### Frontend
- **React** 19.1.1 - UI library
- **Material-UI (MUI)** 7.3.1 - Component library
- **@hello-pangea/dnd** 18.0.1 - Drag-and-drop functionality
- **date-fns** 4.1.0 - Date formatting

### Backend/Desktop
- **Electron** 37.3.1 - Desktop framework
- **Node.js** - Runtime environment
- **fs-extra** 11.3.1 - Enhanced file system operations
- **minimatch** 10.0.3 - Wildcard pattern matching

### Build Tools
- **Vite** 6.3.5 - Frontend bundler
- **Electron Builder** 26.0.12 - Desktop app packager
- **Concurrently** 9.2.0 - Parallel script execution

---

## Project Structure

```
OrdoProject/
├── electron/
│   ├── main.js              # Main process entry point
│   └── preload.js           # Preload script for IPC bridge
├── src/
│   ├── components/
│   │   ├── Rules.jsx        # Rule management component
│   │   └── History.jsx      # History viewer component
│   ├── App.jsx              # Main React component
│   ├── index.jsx            # React entry point
│   └── theme.jsx            # MUI theme configuration
├── build/
│   └── icon.{png,ico,icns}  # Application icons
├── dist/                    # Build output (generated)
├── index.html               # HTML template
├── package.json             # Dependencies and scripts
├── vite.config.js           # Vite configuration
├── AddOrdoToContextMenu.reg # Windows registry file
└── README.md                # User documentation
```

### Data Storage Locations

**Rules File:**
```
Windows: %APPDATA%\OrdoProject\rules.json
macOS: ~/Library/Application Support/OrdoProject/rules.json (intended)
Linux: ~/.config/OrdoProject/rules.json (intended)
```

**Movement Log:**
```
Windows: %APPDATA%\OrdoProject\move_log.csv
macOS: ~/Library/Application Support/OrdoProject/move_log.csv (intended)
Linux: ~/.config/OrdoProject/move_log.csv (intended)
```

⚠️ **Current Issue:** Path is hardcoded to Windows style - see [Known Issues](#known-issues--technical-debt)

---

## Setup & Development

### Prerequisites
- Node.js 18+ and npm
- Git

### Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/yourusername/OrdoProject.git
   cd OrdoProject
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Start development mode:**
   ```bash
   npm run dev
   ```
   
   This runs two processes concurrently:
   - Vite dev server on `http://localhost:5173`
   - Electron app loading the dev server

### Development Scripts

| Script | Command | Description |
|--------|---------|-------------|
| `dev:vite` | `vite` | Start Vite dev server only |
| `dev:electron` | `electron .` | Start Electron only |
| `dev` | `concurrently ...` | Start both Vite and Electron |
| `dist` | `vite build && electron-builder` | Build production app |

---

## Build & Distribution

### Production Build

```bash
npm run dist
```

This will:
1. Build the React app with Vite → `dist/` folder
2. Package the Electron app with electron-builder

### Build Outputs

**Windows:**
- Portable executable: `dist/win-unpacked/ordoproject.exe`
- Installer (if configured): `dist/OrdoProject Setup.exe`

**macOS:**
- App bundle: `dist/mac/OrdoProject.app`

**Linux:**
- AppImage/deb/rpm (depending on configuration)

### Windows Context Menu Integration

Run `AddOrdoToContextMenu.reg` to add context menu entries:
- Right-click file → "Sort this with Ordo"
- Right-click folder → "Sort this folder with Ordo"
- Right-click folder background → "Sort all with Ordo"

⚠️ **Note:** Update paths in `.reg` file to match your installation directory.

---

## Core Workflows

### 1. Rule Processing Flow

```
User opens app
    ↓
Main process reads rules.json (or creates defaults)
    ↓
Renderer displays rules in UI
    ↓
User edits/reorders rules
    ↓
User clicks "Save" → writeRules IPC call
    ↓
Main process writes to rules.json
```

### 2. File Movement Flow (CLI Mode)

```
User right-clicks file/folder (Windows context menu)
    ↓
Electron starts with command-line args
    ↓
Main process reads rules
    ↓
For each file:
    - Match against rules (first match wins)
    - Generate unique destination path
    - Move file with fs.renameSync()
    - Log movement to CSV
    ↓
App quits
```

### 3. Default Rules

When no `rules.json` exists, the app creates 8 default rules:

| Pattern | Destination |
|---------|-------------|
| `*.{txt,doc,docx,pdf,odt,rtf}` | `~/Documents` |
| `*.{xls,xlsx,csv,ods}` | `~/Documents/Spreadsheets` |
| `*.{ppt,pptx,odp}` | `~/Documents/Presentations` |
| `*.{png,jpg,jpeg,gif,bmp,webp,svg,ico,icns}` | `~/Pictures` |
| `*.{mp3,wav,flac,aac,ogg}` | `~/Music` |
| `*.{mp4,avi,mkv,wmv,mov}` | `~/Videos` |
| `*.{zip,rar,7z,tar,gz}` | `~/Downloads/Archives` |
| `*.{exe,msi}` | `~/Downloads/Installers` |

### 4. Pattern Matching

Uses `minimatch` library (glob patterns):
- `*.jpg` - All JPG files
- `invoice_*.pdf` - PDFs starting with "invoice_"
- `*.{png,jpg,jpeg}` - Multiple extensions
- `report_2024_*.xlsx` - Complex patterns

**Processing Order:** Rules are evaluated top-to-bottom; first match wins.

---

## Known Issues & Technical Debt

### Critical Issues

#### 1. **Save Button Doesn't Persist Rules** ⚠️
**Location:** `src/components/Rules.jsx:53-62`

**Problem:**
```javascript
const saveRules = async () => {
  // Validates but doesn't call writeRules!
  alert('Regole salvate!');
};
```

**Fix Required:**
```javascript
const saveRules = async () => {
  for (const rule of rules) {
    if (!rule.pattern || !rule.destination) {
      alert('Ogni regola deve avere pattern e destination.');
      return;
    }
  }
  await window.electronAPI.writeRules(rules);
  alert('Regole salvate!');
};
```

#### 2. **Cross-Platform Path Bug** ⚠️
**Location:** `electron/main.js:7`

**Problem:**
```javascript
const appDataPath = path.join(os.homedir(), 'AppData', 'Roaming', 'OrdoProject');
```

This hardcodes Windows-style paths, breaking macOS/Linux.

**Fix Required:**
```javascript
const appDataPath = app.getPath('userData');
```

#### 3. **Folder Opening Path Bug** ⚠️
**Location:** `electron/main.js:107`

**Problem:**
```javascript
const filePath = folderPath.split(/[/\\]/).slice(0, -1).join('\\');
```

Rebuilds path with Windows separators, breaking on Unix systems.

**Fix Required:**
```javascript
const filePath = path.dirname(folderPath);
```

#### 4. **Fragile CSV Parsing** ⚠️
**Location:** `electron/main.js:97-100`

**Problem:**
```javascript
const [timestamp, source, destination] = line.split(',');
```

Breaks if file paths contain commas.

**Fix Options:**
- Use proper CSV parser library (`csv-parse`)
- Switch to JSONL format (one JSON object per line)

### Minor Issues

#### 5. **Documentation Mismatch**
**Location:** `README.md:48-58` vs `package.json:29-34`

README says `npm start` / `npm run build`, but actual scripts are `npm run dev` / `npm run dist`.

#### 6. **No Tests**
No test files exist. Consider adding:
- Unit tests for rule matching logic
- Integration tests for file movement
- E2E tests for UI workflows

#### 7. **No Error Handling in UI**
IPC calls in React components lack error handling:
```javascript
window.electronAPI.readRules().then((data) => {
  setRules(data || []);
}); // No .catch()
```

---

## Security Considerations

### Current Security Posture ✅

**Good Practices:**
- `contextIsolation: true` - Renderer can't access Node.js directly
- `nodeIntegration: false` - No direct Node.js in renderer
- Limited IPC API surface via `contextBridge`

### Potential Risks ⚠️

1. **Arbitrary File System Access**
   - App can move/delete any file the user has permissions for
   - No sandboxing of destination paths
   - Mitigation: User must explicitly configure rules

2. **Command Injection Risk (Low)**
   - `show-in-folder` uses `child_process.exec` with user paths
   - Paths are quoted, but consider using `execFile` instead

3. **No Input Validation**
   - Rule patterns aren't validated before saving
   - Malformed patterns could cause crashes

### Recommendations

- Add path validation (prevent moving system files)
- Sanitize all user inputs
- Add confirmation dialogs for bulk operations
- Implement undo functionality
- Add dry-run mode to preview changes

---

## Future Roadmap

### Planned Features (from README)
- Regex-based rules (in addition to wildcards)
- Scheduled automatic sorting (background service)
- Cloud sync for rules across devices
- Undo operations for recent movements

### Suggested Enhancements
- **Testing Suite** - Unit, integration, and E2E tests
- **Rule Templates** - Predefined rule sets for common scenarios
- **Statistics Dashboard** - Show files moved, storage saved, etc.
- **Rule Conflicts Detection** - Warn when patterns overlap
- **Dry Run Mode** - Preview what would be moved without moving
- **Custom Sorting Logic** - Date-based folders, file size limits, etc.
- **Multi-language Support** - i18n for UI
- **Auto-Update** - Electron auto-updater integration
- **Backup/Restore** - Export/import rules and history

---

## Troubleshooting

### Development Issues

**Problem:** Vite dev server won't start
```bash
# Clear cache and reinstall
rm -rf node_modules dist .vite
npm install
npm run dev:vite
```

**Problem:** Electron window is blank
- Check browser console in DevTools (Ctrl+Shift+I)
- Verify Vite is running on port 5173
- Check `electron/main.js:140` for correct URL

**Problem:** IPC calls fail
- Verify `preload.js` is loaded (check `webPreferences`)
- Check `contextBridge` exposes the right methods
- Look for errors in main process console

### Build Issues

**Problem:** electron-builder fails
```bash
# Clear build cache
rm -rf dist
npm run dist
```

**Problem:** Icons missing in build
- Verify `build/` folder contains all icon formats
- Check `package.json` build configuration

### Runtime Issues

**Problem:** Rules not saving
- See [Known Issue #1](#1-save-button-doesnt-persist-rules-)
- Check file permissions on AppData folder

**Problem:** Files not moving
- Verify rules.json exists and is valid JSON
- Check file permissions (can't move system files)
- Look for errors in main process console

**Problem:** Context menu not working (Windows)
- Re-run `AddOrdoToContextMenu.reg`
- Update paths in .reg file to match installation
- Check registry entries in `HKEY_CURRENT_USER\Software\Classes`

---

## Contributing Guidelines

### Code Style

- **JavaScript/JSX:** Use ES6+ syntax, functional components
- **Indentation:** 2 spaces (no tabs)
- **Naming:** camelCase for variables/functions, PascalCase for components
- **Comments:** Explain "why", not "what"

### Git Workflow

1. Create feature branch: `git checkout -b feature/your-feature`
2. Make changes and commit: `git commit -m "feat: description"`
3. Push and create PR: `git push origin feature/your-feature`

### Commit Message Format

```
<type>: <description>

[optional body]
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `style` - Code style changes (formatting)
- `refactor` - Code refactoring
- `test` - Adding tests
- `chore` - Build/tooling changes

### Before Submitting PR

- [ ] Code follows project style
- [ ] No console.log() left in code
- [ ] Tested on target platform(s)
- [ ] Updated documentation if needed
- [ ] No new warnings/errors

### Testing Checklist

- [ ] Rules can be created, edited, deleted
- [ ] Rules can be reordered via drag-and-drop
- [ ] Rules persist after app restart
- [ ] Files are moved correctly based on rules
- [ ] History shows all movements
- [ ] "Show in folder" opens correct location
- [ ] Context menu integration works (Windows)

---

## Additional Resources

### Electron Documentation
- [Electron Docs](https://www.electronjs.org/docs)
- [IPC Communication](https://www.electronjs.org/docs/latest/tutorial/ipc)
- [Security Best Practices](https://www.electronjs.org/docs/latest/tutorial/security)

### Libraries Used
- [Material-UI](https://mui.com/)
- [minimatch](https://github.com/isaacs/minimatch)
- [fs-extra](https://github.com/jprichardson/node-fs-extra)
- [date-fns](https://date-fns.org/)

### Related Projects
- [Hazel](https://www.noodlesoft.com/) - macOS file automation
- [DropIt](http://www.dropitproject.com/) - Windows file organizer
- [File Juggler](https://www.filejuggler.com/) - Advanced file automation

---

## License

This project is licensed under the ISC License.

---

## Contact

**Author:** Giacomo Guaresi  
**Repository:** [GitHub](https://github.com/yourusername/OrdoProject)

For bugs and feature requests, please open an issue on GitHub.
