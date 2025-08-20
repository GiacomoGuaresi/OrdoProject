const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electronAPI', {
  readRules: () => ipcRenderer.invoke('read-rules'),
  writeRules: (rules) => ipcRenderer.invoke('write-rules', rules),
});
