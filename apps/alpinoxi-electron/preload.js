// The one bridge between the page and the launcher.
//
// The page gets a single function, `send`, and nothing else: no Node, no
// filesystem, no shell. Everything it asks for is checked in the main process.
const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('alpinoxiIpc', {
  send: (command) => ipcRenderer.invoke('alpinoxi:command', command),
  boot: () => ipcRenderer.invoke('alpinoxi:boot'),
  onNotice: (handler) => ipcRenderer.on('alpinoxi:notice', (_event, text) => handler(text)),
  // Events with data, such as the account a finished sign-in produced.
  onEvent: (handler) => ipcRenderer.on('alpinoxi:event', (_event, message) => handler(message)),
  // Tests only: hands an event to the page as if the launcher had sent it.
  __testEvent: (handler) => {
    contextBridge.exposeInMainWorld('__fireEvent', (message) => handler(message));
  }
});
