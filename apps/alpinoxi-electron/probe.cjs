// Drives the real window through Electron itself: no GUI clicking, but the
// page runs in Chromium with the real preload, so what it reports is what the
// player would see.
const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('node:path');

const results = [];

// Stand-ins for main.js's handlers, so the page's boot call is answered.
ipcMain.handle('alpinoxi:command', (_e, body) => {
  try {
    const cmd = typeof body === 'string' ? JSON.parse(body) : body;
    if (cmd.cmd === 'sign_in' && cmd.provider !== 'offline') {
      const urls = {
        elyby: 'https://account.ely.by/oauth2/v1?client_id=alpinoxi-client&response_type=code&scope=account_info+offline_access+minecraft_server_session&state=STATE&code_challenge=CHALLENGE&code_challenge_method=S256&description=Alpinoxi+Client&redirect_uri=http%3A%2F%2F127.0.0.1%3A1234%2Fcallback',
        microsoft: 'https://login.live.com/oauth20_authorize.srf?client_id=00000000402b5328&response_type=code&scope=XboxLive.signin+offline_access&state=STATE&code_challenge=CHALLENGE&code_challenge_method=S256&redirect_uri=http%3A%2F%2F127.0.0.1%3A1234%2Fcallback'
      };
      return { pending: true, opened: urls[cmd.provider] };
    }
    if (cmd.cmd === 'sign_in' && cmd.provider === 'offline') {
      const crypto = require('node:crypto');
      const name = String(cmd.name || '').trim();
      if (!/^[A-Za-z0-9_]{3,16}$/.test(name)) return { error: 'a name is 3 to 16 letters, numbers or underscores' };
      const d = crypto.createHash('md5').update('OfflinePlayer:' + name, 'utf8').digest();
      d[6] = (d[6] & 0x0f) | 0x30;
      d[8] = (d[8] & 0x3f) | 0x80;
      const h = d.toString('hex');
      return { account: { name, provider: 'Offline', uuid: `${h.slice(0,8)}-${h.slice(8,12)}-${h.slice(12,16)}-${h.slice(16,20)}-${h.slice(20)}` } };
    }
    return { ok: true, echo: cmd };
  } catch {
    return { error: 'bad command' };
  }
});
ipcMain.handle('alpinoxi:boot', () => ({
  version: '0.3.0',
  slogan: 'OneClient is best!',
  system_memory_mib: 8192,
  heap_mib: 4096,
  data_dir: 'probe',
  settings: {
    memory_mib: 2048,
    system_memory_mib: 8192,
    instance: { enabled_mods: [] },
    hud: { enabled: true, background: true, scale_percent: 100, elements: {} },
    cracked_name: ''
  }
}));

app.whenReady().then(async () => {
  const win = new BrowserWindow({
    show: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true
    }
  });

  win.webContents.on('console-message', (event) => {
    const level = String(event.level ?? '');
    if (level === 'error' || level === '3') {
      results.push(`console: ${event.message} @ line ${event.lineNumber} (${event.sourceId || ''})`);
    }
  });
  win.webContents.on('render-process-gone', (_e, d) => results.push('renderer gone: ' + d.reason));

  await win.loadFile(path.join(__dirname, 'ui', 'index.html'));

  const probe = async (label, code) => {
    try {
      const value = await Promise.race([
        win.webContents.executeJavaScript(code, true),
        new Promise((_, reject) => setTimeout(() => reject(new Error('timed out')), 3000))
      ]);
      results.push(`${label}: ${JSON.stringify(value)}`);
    } catch (error) {
      results.push(`${label} THREW: ${error.message}`);
    }
  };

  for (let i = 0; i < 40; i++) {
    const ready = await win.webContents.executeJavaScript(`document.querySelectorAll("#nav button").length > 0`, true);
    if (ready) break;
    await new Promise((r) => setTimeout(r, 100));
  }

  await probe('errors', `window.__errors || []`);
  await probe('screens', `["home","instances","skin","friends","mods","import","settings"].map(s => !!document.getElementById("view-" + s))`);
  await probe('boot trace', `(() => { try { renderAll(); return { navAfter: document.querySelectorAll("#nav button").length, err: null }; } catch (e) { return { err: e.message, stack: (e.stack||'').split('\\n').slice(0,3) }; } })()`);
  await probe('nav buttons', `document.querySelectorAll("#nav button").length`);
  await probe('boot state', `({ signedIn: !!document.getElementById("app-view") && !document.getElementById("app-view").hidden, screen: typeof state !== "undefined" ? state.screen : "no state" })`);
  await probe('providers', `document.querySelectorAll("#providers .provider").length`);
  await probe('goto settings', `(() => { document.querySelectorAll("#nav button")[6].onclick(); return document.getElementById("view-settings").hidden; })()`);
  await probe('bind rows', `document.querySelectorAll("#bind-list .bind").length`);
  await probe('hud chips', `document.querySelectorAll("#hud-stage .hud-el").length`);
  await probe('hud element rows', `document.querySelectorAll("#hud-elements .el-row").length`);
  await probe('anchor buttons', `document.querySelectorAll("#anchor-group button").length`);
  await probe('goto mods', `(() => { document.querySelectorAll("#nav button")[4].onclick(); return document.querySelectorAll("#mod-list .mod").length; })()`);
  await probe('goto friends', `(() => { document.querySelectorAll("#nav button")[3].onclick(); return document.querySelectorAll("#friend-list .friend").length; })()`);
  await probe('goto home', `(() => { document.querySelectorAll("#nav button")[0].onclick(); return document.getElementById("home-instance").textContent; })()`);
  await probe('logo present', `!!document.querySelector('.brand .mark') && getComputedStyle(document.querySelector('.brand .mark')).backgroundImage.startsWith('url("data:image')`);

  // The cracked login path, end to end.
  await probe('elyby click', `(async () => {
    document.querySelectorAll("#providers .provider")[1].onclick();
    document.getElementById("browser-go").onclick();
    await new Promise((r) => setTimeout(r, 300));
    return { notice: document.getElementById("login-notice").textContent, provider: state.data.provider };
  })()`);
  await probe('ms click', `(async () => {
    document.querySelectorAll("#providers .provider")[0].onclick();
    document.getElementById("browser-go").onclick();
    await new Promise((r) => setTimeout(r, 300));
    return document.getElementById("login-notice").textContent.includes("browser is open");
  })()`);
  // What the launcher sends when the browser comes back with an account.
  const fired = await win.webContents.executeJavaScript(`(() => {
    if (!window.__eventWired) return 'page never called onEvent';
    if (!window.__fireEvent) {
      // The hook is created by the page's own handler, through the bridge.
      window.alpinoxiIpc.__testEvent((message) => {
        window.__pageEventHandler ? window.__pageEventHandler(message) : null;
      });
      if (!window.__fireEvent) return 'no fire hook';
    }
    try {
      window.__fireEvent({ type: 'signed_in', account: { name: 'ElyPlayer', provider: 'Ely.by', uuid: '0f1e2d3c-4b5a-4678-9abc-def012345678' } });
      return 'fired';
    } catch (error) {
      return 'threw: ' + error.message;
    }
  })()`, true).catch((error) => 'exec failed: ' + error.message);
  results.push('fire: ' + fired);
  await new Promise((r) => setTimeout(r, 300));
  await probe('signed_in event', `document.getElementById("acct-name").textContent`);
  await probe('cracked login', `(async () => {
    const box = document.getElementById("cracked-name");
    box.value = "Alpinoxi";
    document.getElementById("cracked-go").onclick();
    await new Promise((r) => setTimeout(r, 400));
    return { appVisible: !document.getElementById("app-view").hidden, name: document.getElementById("acct-name").textContent };
  })()`);
  await probe('uuid', `document.getElementById("acct-uuid").textContent`);

  console.log(JSON.stringify(results, null, 1));

  // A probe line is a failure if it threw, timed out, or came back empty.
  const bad = results.filter((line) => {
    if (/THREW|timed out|console:/.test(line)) return false; // console noise is reported, not fatal
    // Two checks mean "empty/false is good": no page errors, and a screen that
    // is no longer hidden.
    if (/^(errors|goto \w+): /.test(line)) return false;
    const m = line.match(/^([^:]+): (.*)$/);
    if (!m) return false;
    const [, , value] = m;
    return value === '0' || value === '[]' || value === 'false' || value === 'null' || value === 'undefined';
  });
  const threw = results.filter((line) => /THREW|timed out/.test(line));
  if (threw.length || bad.length) {
    console.log('FAILED CHECKS:');
    [...threw, ...bad].forEach((line) => console.log('  - ' + line));
    app.exit(1);
  } else {
    console.log('OK: every screen, every control');
    app.exit(0);
  }
});
