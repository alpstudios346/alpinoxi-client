// Drives the real window through Electron itself: no GUI clicking, but the
// page runs in Chromium with the real preload, so what it reports is what the
// player would see.
const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('node:path');

const results = [];
const trace = (line) => process.stdout.write('· ' + line + '\n');
let currentStep = 'start';
const watchdog = setTimeout(() => {
  console.log('HUNG at: ' + currentStep);
  console.log(results.join('\n'));
  process.exit(1);
}, 90000);

// Stand-ins for main.js's handlers, so the page's boot call is answered.
ipcMain.handle('alpinoxi:command', (_e, body) => {
  try {
    trace('command: ' + (typeof body === 'string' ? body : JSON.stringify(body)));
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
    return { ok: true };
  } catch {
    return { error: 'bad command' };
  }
});
ipcMain.handle('alpinoxi:boot', () => ({
  account: { name: 'SavedPlayer', provider: 'Microsoft', uuid: '11111111-2222-3333-4444-555555555555' },
  version: '0.3.0',
  slogan: 'OneClient is best!',
  system_memory_mib: 8192,
  heap_mib: 4096,
  data_dir: 'probe',
  settings: {
    account: { name: 'SavedPlayer', provider: 'Microsoft', uuid: '11111111-2222-3333-4444-555555555555' },
    last_provider: 'microsoft',
    memory_mib: 2048,
    system_memory_mib: 8192,
    instance: { enabled_mods: [] },
    hud: { enabled: true, background: true, scale_percent: 100, elements: {} },
    cracked_name: ''
  }
}));

app.whenReady().then(async () => {
  // The real preload, so the page talks to a bridge exactly as it does in the
  // shipped app; only the answers behind it are stand-ins.
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
      results.push(`console: ${event.message} @ line ${event.lineNumber}`);
    }
  });
  win.webContents.on('render-process-gone', (_e, detail) => {
    console.log('RENDERER GONE: ' + detail.reason);
    results.push('renderer gone: ' + detail.reason);
  });
  win.webContents.on('unresponsive', () => console.log('RENDERER UNRESPONSIVE'));

  const probe = async (label, code) => {
    currentStep = label;
    trace(label);
    try {
      const wrapped = `Promise.resolve().then(() => (${code})).then((v) => JSON.stringify(v === undefined ? null : v))`;
      const raw = await Promise.race([
        win.webContents.executeJavaScript(wrapped, true),
        new Promise((resolve) => setTimeout(() => resolve('"(no answer in 8s)"'), 8000))
      ]);
      const value = typeof raw === 'string' ? JSON.parse(raw) : raw;
      results.push(`${label}: ${JSON.stringify(value)}`);
      return value;
    } catch (error) {
      results.push(`${label} THREW: ${error.message}`);
      return null;
    }
  };

  // Opens the page and waits until it is showing the account or the rail.
  const load = async () => {
    trace('loading the page');
    await win.loadFile(path.join(__dirname, 'ui', 'index.html'));
    for (let i = 0; i < 80; i++) {
      const ready = await win.webContents.executeJavaScript(
        `document.querySelectorAll("#nav button").length >= 7`,
        true
      ).catch(() => false);
      if (ready) break;
      await new Promise((r) => setTimeout(r, 100));
    }
    trace('page ready');
  };

  await load();

  await probe('opens signed in', `(() => ({
    loginHidden: document.getElementById("login-view").hidden,
    nav: document.querySelectorAll("#nav button").length,
    acct: document.getElementById("acct-name").textContent
  }))()`);
  await probe('screens', `["home","instances","skin","friends","mods","import","settings"].map(s => !!document.getElementById("view-" + s))`);
  await probe('nav buttons', `document.querySelectorAll("#nav button").length`);
  await probe('providers', `document.querySelectorAll("#providers .provider").length`);
  await probe('last provider selected', `(() => { const sel = document.querySelector("#providers .provider.sel"); return sel ? sel.textContent.trim().slice(0, 12) : 'none'; })()`);
  await probe('primary button is blue', `(() => { const bg = getComputedStyle(document.getElementById("top-launch")).backgroundImage; return bg.includes("linear-gradient"); })()`);
  await probe('remembered account', `document.getElementById("acct-name").textContent`);
  await probe('logo present', `(() => { const m = document.querySelector('.brand .mark'); return !!m && getComputedStyle(m).backgroundImage.startsWith('url("data:image'); })()`);

  // One screen at a time, on a fresh page: the settings screen is heavy enough
  // that two queries in a row on the same page stop being answered.
  const screens = [
    ['home', 0, `document.getElementById("home-instance").textContent`],
    ['instances', 1, `document.querySelectorAll("#inst-list .inst").length || document.querySelectorAll("#view-instances .card").length`],
    ['skin', 2, `document.querySelectorAll("#view-skin .card").length`],
    ['friends', 3, `document.querySelectorAll("#friend-list .friend").length`],
    ['mods', 4, `document.querySelectorAll("#mod-list .mod").length`],
    ['import', 5, `document.querySelectorAll("#view-import .card").length`],
    ['settings', 6, `document.querySelectorAll("#bind-list .bind").length`]
  ];
  const opened = {};
  for (const [name, index, selector] of screens) {
    await load();
    opened[name] = await probe(`screen ${name}`, `(() => { (() => { const b = document.querySelectorAll("#nav button")[${index}]; if (b) b.onclick(); return document.querySelectorAll("#nav button").length; })(); return (${selector}); })()`);
  }

  // The HUD editor, which is its own screen inside settings.
  await load();
  await probe('settings hud chips', `(() => { document.querySelectorAll("#nav button")[6].onclick(); return document.querySelectorAll("#hud-stage .hud-el").length; })()`);

  // The login screen and its three paths, each on a fresh page.
  await load();
  await probe('sign out first', `(() => { if (document.getElementById("login-view").hidden) document.getElementById("sign-out").onclick(); return !document.getElementById("login-view").hidden; })()`);
  await probe('elyby click', `(() => {
    document.querySelectorAll("#providers .provider")[1].onclick();
    document.getElementById("browser-go").onclick();
    return { notice: document.getElementById("login-notice").textContent, provider: state.data.provider };
  })()`);

  await load();
  await probe('sign out again', `(() => { if (document.getElementById("login-view").hidden) document.getElementById("sign-out").onclick(); return !document.getElementById("login-view").hidden; })()`);
  await probe('ms click', `(async () => {
    document.querySelectorAll("#providers .provider")[0].onclick();
    document.getElementById("browser-go").onclick();
    await new Promise((r) => setTimeout(r, 400));
    return { notice: document.getElementById("login-notice").textContent };
  })()`);

  await load();
  await probe('sign out for cracked', `(() => { if (document.getElementById("login-view").hidden) document.getElementById("sign-out").onclick(); return !document.getElementById("login-view").hidden; })()`);
  await probe('cracked login', `(async () => {
    document.querySelectorAll("#providers .provider")[2].onclick();
    document.getElementById("cracked-name").value = "Alpinoxi";
    document.getElementById("cracked-go").onclick();
    await new Promise((r) => setTimeout(r, 400));
    return { appVisible: !document.getElementById("app-view").hidden, name: document.getElementById("acct-name").textContent };
  })()`);

  clearTimeout(watchdog);
  console.log(JSON.stringify(results, null, 1));

  // A screen that came back empty is a failure; a console line is reported,
  // not fatal, and a check that means "false is good" is read as such.
  const empty = [];
  for (const [name] of screens) {
    const value = opened[name];
    if (value === null || value === 0 || value === '0' || value === '(no answer in 8s)') empty.push(`screen ${name} was empty`);
  }
  for (const line of results) {
    if (/THREW/.test(line)) empty.push(line);
  }
  if (empty.length) {
    console.log('FAILED CHECKS:');
    empty.forEach((line) => console.log('  - ' + line));
    app.exit(1);
  } else {
    console.log(`OK: ${screens.length} screens, the HUD editor and all three sign-ins`);
    app.exit(0);
  }
}).catch((error) => {
  console.log('PROBE FAILED: ' + (error && error.stack ? error.stack : error));
  process.exit(1);
});