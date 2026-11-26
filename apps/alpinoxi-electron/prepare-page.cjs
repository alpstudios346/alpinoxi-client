// Builds `ui/index.html` from the shared page.
//
// The page itself lives in `ui/hudui.html` and is filled by `ui/build_ui.py`.
// Electron needs two things on top of that: the preload bridge's name, and a
// boot call that asks the main process for the settings. Both are applied
// here, in one place, so CI and a developer's machine produce the same file
// and a fix lands in both.
//
//   node prepare-page.cjs      # from apps/alpinoxi-electron

const fs = require('node:fs');
const path = require('node:path');

const source = path.resolve(__dirname, '..', '..', 'ui', 'dist', 'alpinoxi-ui.html');
const target = path.resolve(__dirname, 'ui', 'index.html');

const SHIM = `<script>
// Electron: the page talks to the main process over a preload bridge instead
// of a webview's own IPC. This keeps the page one file for both windows.
window.alpinoxi = {
  send(command) {
    if (window.alpinoxiIpc) return window.alpinoxiIpc.send(command);
    return Promise.resolve(null);
  }
};
</script>
`;

const BOOT = `// Electron hands the page its settings and answers commands; the webview
// build has neither, so both paths are guarded here.
async function boot() {
  if (!window.alpinoxiIpc) return;
  try {
    const info = await window.alpinoxiIpc.boot();
    if (info && info.settings) {
      state.data.memory_mib = info.settings.memory_mib;
      state.data.system_memory_mib = info.settings.system_memory_mib;
      state.data.instance.enabled_mods = info.settings.instance.enabled_mods || [];
      state.data.hud.enabled = info.settings.hud.enabled;
      state.data.hud.background = info.settings.hud.background;
      state.data.hud.scale_percent = info.settings.hud.scale_percent;
      if (info.settings.hud.elements) {
        state.data.hud.elements.forEach((el) => {
          if (el.id in info.settings.hud.elements) el.visible = info.settings.hud.elements[el.id];
        });
      }
      if (info.heap_mib) state.heap = info.heap_mib;
      if (info.version) state.data.version = info.version;
      if (info.settings.cracked_name) state.data.cracked_name = info.settings.cracked_name;
    }
    renderAll();
    if (window.alpinoxiIpc.onNotice) {
      window.alpinoxiIpc.onNotice((text) => {
        toast(text);
        if ($('login-notice') && !$('login-view').hidden) $('login-notice').textContent = text;
      });
    }
    // A sign-in finished in the browser: the launcher sends the account here.
    const onLauncherEvent = (message) => {
      if (message && message.type === 'signed_in' && message.account) {
        signIn(message.account);
      }
    };
    if (window.alpinoxiIpc.onEvent) {
      window.alpinoxiIpc.onEvent(onLauncherEvent);
      if (window.alpinoxiIpc.__testEvent) window.alpinoxiIpc.__testEvent(onLauncherEvent);
    }
    window.__eventWired = !!window.alpinoxiIpc.onEvent;
  } catch (error) {
    toast('Could not reach the launcher: ' + error.message);
  }
}
boot();

`;

function main() {
  if (!fs.existsSync(source)) {
    console.error(`missing ${source}: run \`python ui/build_ui.py\` first`);
    process.exit(1);
  }

  let page = fs.readFileSync(source, 'utf8');
  const before = page.length;

  const scriptStart = /<script>\r?\n"use strict";/;
  if (!scriptStart.test(page)) {
    console.error('the page has no script block to patch');
    process.exit(1);
  }
  page = page.replace(scriptStart, (match) => SHIM + match);

  const anchor = '$("login-slogan").textContent = state.data.slogan;';
  if (!page.includes(anchor)) {
    console.error('the boot anchor is gone: did the page change shape?');
    process.exit(1);
  }
  page = page.replace(anchor, BOOT + anchor);

  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, page);

  // A page that keeps the old promises would ship a broken window.
  for (const stale of ['No Electron', 'no web view']) {
    if (page.includes(stale)) {
      console.error(`the page still says "${stale}"`);
      process.exit(1);
    }
  }

  console.log(`built ${target} (${(page.length / 1024).toFixed(1)} KB, from ${(before / 1024).toFixed(1)} KB)`);
}

main();
