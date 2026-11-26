// Tests the sign-in machinery without a browser or a real account: PKCE, the
// loopback listener, the state check, cancellation, and the token exchange
// against a stand-in provider. If this passes, a real sign-in has a road.
const assert = require('node:assert');
const http = require('node:http');
const crypto = require('node:crypto');
const fs = require('node:fs');

// The functions are taken out of main.js itself, so the test cannot pass on a
// copy of the logic that has drifted from the launcher.
const source = fs.readFileSync('main.js', 'utf8');

function grab(name) {
  const start = source.search(new RegExp(`^(async )?function ${name}\\(`, 'm'));
  if (start < 0) throw new Error(`${name} not found in main.js`);
  const open = source.indexOf('{', start);
  let depth = 0;
  for (let i = open; i < source.length; i++) {
    if (source[i] === '{') depth++;
    else if (source[i] === '}') {
      depth--;
      if (depth === 0) return source.slice(start, i + 1);
    }
  }
  throw new Error(`${name} has no end`);
}

const timeoutLine = source.match(/const SIGN_IN_TIMEOUT_MS = [^;]+;/)[0];
const authStart = source.indexOf('const AUTH = {');
const authTable = source.slice(authStart, source.indexOf('};', authStart) + 2);

const notices = [];
const opened = [];
const table = {};

// `waitForCode` opens the page; the test answers as the browser instead.
new Function(
  'require', 'module', 'exports', 'shell', 'notice', 'send', 'saveSettings', 'state', 'http', 'crypto', 'URLSearchParams',
  `${timeoutLine}
   ${authTable}
   ${grab('base64url')}
   ${grab('listenLoopback')}
   ${grab('waitForCode')}
   ${grab('exchangeCode')}
   module.exports.base64url = base64url;
   module.exports.listenLoopback = listenLoopback;
   module.exports.waitForCode = waitForCode;
   module.exports.exchangeCode = exchangeCode;`
)(
  require, { exports: table }, table,
  { openExternal: (url) => opened.push(url) },
  (text) => notices.push(text),
  () => {},
  () => {},
  {},
  http,
  crypto,
  URLSearchParams
);

const { base64url, listenLoopback, waitForCode, exchangeCode } = table;

(async () => {
  // PKCE: url-safe, no padding, 43-character S256 challenge.
  const verifier = base64url(crypto.randomBytes(48));
  assert.match(verifier, /^[A-Za-z0-9_-]+$/, 'verifier must be url-safe');
  assert.strictEqual(verifier.length, 64);
  const challenge = base64url(crypto.createHash('sha256').update(verifier).digest());
  assert.strictEqual(challenge.length, 43);

  // The listener answers on loopback and hands the code back.
  const { server, redirectUri } = await listenLoopback();
  assert.match(redirectUri, /^http:\/\/127\.0\.0\.1:\d+\/callback$/);
  const waiting = waitForCode(server, 'abc123', 'Test', 'about:blank');
  const response = await fetch(`${redirectUri}?state=abc123&code=CODE`);
  assert.strictEqual(response.status, 200);
  assert.match(await response.text(), /Signed in/);
  assert.strictEqual(await waiting, 'CODE');
  assert.deepStrictEqual(opened, ['about:blank'], 'the page must actually be opened');

  // A wrong state is refused.
  const second = await listenLoopback();
  const rejected = assert.rejects(waitForCode(second.server, 'expected', 'Test', 'about:blank'), /unexpected state/);
  const bad = await fetch(`${second.redirectUri}?state=wrong&code=CODE`);
  assert.strictEqual(bad.status, 400);
  await rejected;
  second.server.close();

  // A provider that says no is reported as no.
  const cancelling = await listenLoopback();
  const cancelled = assert.rejects(waitForCode(cancelling.server, 's', 'Test', 'about:blank'), /access_denied/);
  await fetch(`${cancelling.redirectUri}?state=s&error=access_denied`);
  await cancelled;
  cancelling.server.close();

  // A token exchange against a stand-in provider: the body carries the code
  // and the verifier, exactly as Ely.by and Microsoft expect.
  let seen = null;
  const stub = http.createServer((request, response) => {
    let body = '';
    request.on('data', (chunk) => (body += chunk));
    request.on('end', () => {
      seen = new URLSearchParams(body);
      response.writeHead(200, { 'content-type': 'application/json' });
      response.end(JSON.stringify({ access_token: 'AT', refresh_token: 'RT' }));
    });
  });
  await new Promise((resolve) => stub.listen(0, '127.0.0.1', resolve));
  const port = stub.address().port;

  const realFetch = global.fetch;
  global.fetch = (url, options) =>
    realFetch(String(url).replace('https://account.ely.by/api/oauth2/v1/token', `http://127.0.0.1:${port}/token`), options);
  const tokens = await exchangeCode('elyby', 'THE_CODE', verifier, 'http://127.0.0.1:1/callback');
  global.fetch = realFetch;
  stub.close();

  assert.strictEqual(tokens.access_token, 'AT');
  assert.strictEqual(seen.get('grant_type'), 'authorization_code');
  assert.strictEqual(seen.get('code'), 'THE_CODE');
  assert.strictEqual(seen.get('code_verifier'), verifier);
  assert.strictEqual(seen.get('client_id'), 'alpinoxi-client');

  console.log('OK: PKCE, loopback listener, state check, cancellation, token exchange');
  process.exit(0);
})().catch((error) => {
  console.log('FAIL: ' + error.message);
  process.exit(1);
});
