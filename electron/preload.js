const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electronAPI', {
  readRules: () => ipcRenderer.invoke('read-rules'),
  writeRules: (rules) => ipcRenderer.invoke('write-rules', rules),
  openFolderDialog: () => ipcRenderer.invoke('open-folder-dialog'),
  getHistory: () => ipcRenderer.invoke('get-history'),
  showInFolder: (path) => ipcRenderer.invoke('show-in-folder', path),
});
