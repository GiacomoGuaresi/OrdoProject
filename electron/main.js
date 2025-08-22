const path = require('path');
const { app, BrowserWindow, ipcMain, dialog  } = require('electron');
const fs = require("fs-extra");
const os = require('os');
const minimatch = require('minimatch').minimatch;

const appDataPath = path.join(os.homedir(), 'AppData', 'Roaming', 'OrdoProject');
const rulesFile = path.join(appDataPath, 'rules.json');
const logFile = path.join(appDataPath, 'move_log.csv');

function ensureDir(dir) {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

function readRules() {
  if (!fs.existsSync(appDataPath)) {
    fs.mkdirSync(appDataPath, { recursive: true });
  }

  if (fs.existsSync(rulesFile)) {
    return JSON.parse(fs.readFileSync(rulesFile, "utf8"));
  }

  // Template base
  const defaultRules = [
    {
      "pattern": "*.{txt,doc,docx,pdf,odt,rtf}",
      "destination": path.join(os.homedir(), "Documents")
    },
    {
      "pattern": "*.{xls,xlsx,csv,ods}",
      "destination": path.join(os.homedir(), "Documents", "Spreadsheets")
    },
    {
      "pattern": "*.{ppt,pptx,odp}",
      "destination": path.join(os.homedir(), "Documents", "Presentations")
    },
    {
      "pattern": "*.{png,jpg,jpeg,gif,bmp,webp,svg,ico,icns}",
      "destination": path.join(os.homedir(), "Pictures")
    },
    {
      "pattern": "*.{mp3,wav,flac,aac,ogg}",
      "destination": path.join(os.homedir(), "Music")
    },
    {
      "pattern": "*.{mp4,avi,mkv,wmv,mov}",
      "destination": path.join(os.homedir(), "Videos")
    },
    {
      "pattern": "*.{zip,rar,7z,tar,gz}",
      "destination": path.join(os.homedir(), "Downloads", "Archives")
    },
    {
      "pattern": "*.{exe,msi}",
      "destination": path.join(os.homedir(), "Downloads", "Installers")
    }
  ];

  // 🔑 Creo tutte le cartelle necessarie
  defaultRules.forEach(rule => ensureDir(rule.destination));

  // Salvo il file rules.json
  fs.writeFileSync(rulesFile, JSON.stringify(defaultRules, null, 2), "utf8");
  
  return defaultRules;
}


// Gestione degli eventi IPC per leggere e scrivere le regole

ipcMain.handle('read-rules', () => readRules());
ipcMain.handle('write-rules', (event, rules) => {
  fs.writeFileSync(rulesFile, JSON.stringify(rules, null, 2));
});
ipcMain.handle('open-folder-dialog', async (event) => {
  const { canceled, filePaths } = await dialog.showOpenDialog({
    properties: ['openDirectory'],
  });

  if (canceled) {
    return null;
  } else {
    return filePaths[0];
  }
});
ipcMain.handle('get-history', () => {
  if (fs.existsSync(logFile)) {
    const data = fs.readFileSync(logFile, 'utf-8');
    const lines = data.split('\n').filter(line => line.trim() !== '');
    
    // Ignora la prima riga (intestazione)
    lines.shift();

    return lines.map(line => {
      const [timestamp, source, destination] = line.split(',');
      return { timestamp: timestamp.replace(/"/g, ''), source: source.replace(/"/g, ''), destination: destination.replace(/"/g, '') };
    });
  }
  return [];
});
ipcMain.handle('show-in-folder', (event, path) => {
  const folderPath = path.replace(/"/g, ''); // Rimuove eventuali virgolette
  // Rimuove il file dal path 
  const filePath = folderPath.split(/[/\\]/).slice(0, -1).join('\\');
  if (fs.existsSync(filePath)) {
    if (process.platform === 'win32') {
      require('child_process').exec(`explorer "${filePath}"`);
    }
    else if (process.platform === 'darwin') {
      require('child_process').exec(`open "${filePath}"`);
    }
    else {
      require('child_process').exec(`xdg-open "${filePath}"`);
    }
  } else {
    console.error(`La cartella ${filePath} non esiste.`);
    dialog.showErrorBox('Errore', `La cartella ${filePath} non esiste.`);
  }
});


function createWindow() {
  const win = new BrowserWindow({
    width: 800,
    height: 600,
    icon: path.join(__dirname, "../build/icon.png"),
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
    autoHideMenuBar: true,
  });

  if (!app.isPackaged) {
    console.log("Loading dev server...");
    win.loadURL('http://localhost:5173');
  } else {
    const indexPath = path.join(__dirname, "../dist/index.html");
    console.log("Loading file:", indexPath);
    win.loadFile(indexPath);
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
