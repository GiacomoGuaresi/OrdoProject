const path = require('path');
const { app, BrowserWindow, ipcMain } = require('electron');
const fs = require('fs');
const os = require('os');
const isDev = require('electron-is-dev');
const minimatch = require('minimatch').minimatch;

const appDataPath = path.join(os.homedir(), 'AppData', 'Roaming', 'OrdoProject');
const rulesFile = path.join(appDataPath, 'rules.json');
const logFile = path.join(appDataPath, 'move_log.csv');

function readRules() {
  if (!fs.existsSync(appDataPath)) fs.mkdirSync(appDataPath, { recursive: true });
  if (fs.existsSync(rulesFile)) {
    return JSON.parse(fs.readFileSync(rulesFile));
  }
  return [];
}

ipcMain.handle('read-rules', () => readRules());
ipcMain.handle('write-rules', (event, rules) => {
  fs.writeFileSync(rulesFile, JSON.stringify(rules, null, 2));
});

function createWindow() {
  const win = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  if (isDev) {
    win.loadURL('http://localhost:5173');
  } else {
    win.loadFile(path.join(__dirname, '../dist/index.html'));
  }
}

// Funzione per generare un nome unico se il file esiste già
function getUniqueDestination(destPath) {
  if (!fs.existsSync(destPath)) return destPath;

  const dir = path.dirname(destPath);
  const ext = path.extname(destPath);
  const baseName = path.basename(destPath, ext);

  let counter = 1;
  let newPath;
  do {
    newPath = path.join(dir, `${baseName} (${counter})${ext}`);
    counter++;
  } while (fs.existsSync(newPath));

  return newPath;
}

// Funzione per scrivere il log CSV
function logMove(source, destination) {
  const timestamp = new Date().toISOString();
  const line = `"${timestamp}","${source}","${destination}"\n`;

  // Se il file non esiste, aggiungo intestazione
  if (!fs.existsSync(logFile)) {
    fs.writeFileSync(logFile, `"Timestamp","Source","Destination"\n`, { flag: 'wx' });
  }
  fs.appendFileSync(logFile, line);
}

const args = process.argv.slice(process.defaultApp ? 2 : 1);
const targetPath = args[0];
const processAll = args.includes('ALL');

app.whenReady().then(() => {
  const rules = readRules();

  if (targetPath) {
    let filesToProcess = [];

    if (processAll) {
      const stat = fs.statSync(targetPath);
      if (stat.isDirectory()) {
        filesToProcess = fs.readdirSync(targetPath)
          .map(f => path.join(targetPath, f))
          .filter(f => fs.statSync(f).isFile());
      } else {
        filesToProcess = [targetPath];
      }
    } else {
      filesToProcess = [targetPath];
    }

    for (const filePath of filesToProcess) {
      const matchingRule = rules.find(rule =>
        minimatch(path.basename(filePath), rule.pattern)
      );

      if (matchingRule) {
        const destinationDir = matchingRule.destination;
        if (!fs.existsSync(destinationDir)) fs.mkdirSync(destinationDir, { recursive: true });

        const destPath = path.join(destinationDir, path.basename(filePath));
        const uniqueDestPath = getUniqueDestination(destPath);

        fs.renameSync(filePath, uniqueDestPath);
        logMove(filePath, uniqueDestPath);
      }
      // Se non c'è regola, ignora il file silenziosamente
    }

    app.quit();
  } else {
    createWindow();
  }
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});
