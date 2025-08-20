const path = require('path');
const { app, BrowserWindow, dialog, ipcMain } = require('electron');
const fs = require('fs');
const os = require('os');
const isDev = require('electron-is-dev');

const appDataPath = path.join(os.homedir(), 'AppData', 'Roaming', 'OrdoProject');
const rulesFile = path.join(appDataPath, 'rules.json');

ipcMain.handle('read-rules', () => {
  if (!fs.existsSync(appDataPath)) fs.mkdirSync(appDataPath);
  if (fs.existsSync(rulesFile)) {
    return JSON.parse(fs.readFileSync(rulesFile));
  }
  return [];
});

ipcMain.handle('write-rules', (event, rules) => {
  fs.writeFileSync(rulesFile, JSON.stringify(rules, null, 2));
});

function createWindow() {
  const win = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'), // usa preload sicuro
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

const args = process.argv.slice(process.defaultApp ? 2 : 1);
if (args.length > 0) {
  app.whenReady().then(() => {
    dialog.showMessageBox({
      type: 'info',
      title: 'Parametri ricevuti',
      message: 'Parametri passati: ' + args.join(', ')
    }).then(() => app.quit());
  });
} else {
  app.whenReady().then(createWindow);
}

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});
