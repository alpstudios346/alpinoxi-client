// The offline UUID must match Minecraft's, or a cracked player is a stranger
// on a server that already knows them.
const fs = require('fs');
const src = fs.readFileSync('ui/index.html', 'utf8');
const start = src.indexOf('function offlineUuid(');
const end = src.indexOf('function planHeap(');
const code = src.slice(start, end) + '\nmodule.exports = { offlineUuid, md5 };';
const mod = { exports: {} };
new Function('module', 'TextEncoder', code)(mod, TextEncoder);
const { offlineUuid } = mod.exports;

const expected = {
  Alpinoxi: '7cce3372-aec0-3d25-9588-23579d5fcd8a',
  Notch: 'b50ad385-829d-3141-a216-7e7d7539ba7f',
  Steve: '5627dd98-e6be-3c21-b8a8-e92344183641',
  Player: 'a01e3843-e521-3998-958a-f459800e4d11'
};

let failed = 0;
for (const [name, want] of Object.entries(expected)) {
  const got = offlineUuid(name);
  if (got !== want) {
    console.log(`FAIL ${name}: got ${got}, want ${want}`);
    failed += 1;
  }
}
console.log(failed ? `${failed} uuid(s) wrong` : 'OK: offline UUIDs match Minecraft');
process.exit(failed ? 1 : 0);
