const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const path = require('node:path');

const source = fs.readFileSync(path.join(__dirname, '../src/main.js'), 'utf8');

function element() {
  return {
    attrs: {}, handlers: {}, children: {}, dataset: {}, disabled: false,
    hidden: true, textContent: '', showCount: 0, closeCount: 0,
    set innerHTML(value) { throw new Error('Unexpected HTML interpretation: ' + value); },
    setAttribute(key, value) { this.attrs[key] = value; },
    getAttribute(key) { return this.attrs[key]; },
    removeAttribute(key) { delete this.attrs[key]; },
    querySelector(key) { return this.children[key] ??= element(); },
    addEventListener(key, handler) { this.handlers[key] = handler; },
    showModal() { this.showCount++; }, close() { this.closeCount++; },
    getBoundingClientRect() { return {left: 100, right: 200, top: 100, bottom: 200}; }
  };
}

function boot({ invoke, reduced = false, stored = null } = {}) {
  const nodes = Object.fromEntries(['motion-toggle', 'play-btn', 'preview-dialog', 'status-msg'].map(id => [id, element()]));
  const badge = element(), classes = new Set(), preference = { matches: reduced, addEventListener(key, fn) { this[key] = fn; } };
  const document = {
    getElementById: id => nodes[id], querySelector: () => badge,
    body: { classList: { toggle(key, state) { if (state) classes.add(key); else classes.delete(key); } } }
  };
  const window = { matchMedia(query) { assert.equal(query, '(prefers-reduced-motion: reduce)'); return preference; } };
  if (invoke) window.__TAURI__ = {core: {invoke}};
  vm.runInNewContext(source, { document, window, localStorage: {getItem: () => stored, setItem() {}} });
  return { nodes, badge, classes, preference, click: () => nodes['play-btn'].handlers.click() };
}

(async () => {
  const preview = boot();
  await preview.click();
  assert.equal(preview.nodes['preview-dialog'].showCount, 1);
  assert.equal(preview.nodes['play-btn'].disabled, false);
  console.log('PASS: browser preview opens its dialog without launching the game.');

  const calls = [];
  let complete;
  const native = boot({invoke(...args) { calls.push(args); return new Promise((resolve, reject) => { complete = {resolve, reject}; }); }});
  const pendingSuccess = native.click();
  assert.equal(native.nodes['play-btn'].disabled, true);
  assert.equal(native.nodes['play-btn'].attrs['aria-busy'], 'true');
  assert.equal(native.nodes['status-msg'].dataset.state, 'loading');
  assert.equal(native.nodes['status-msg'].textContent, 'Launching the game…');
  assert.equal(native.nodes['play-btn'].querySelector('strong').textContent, 'LAUNCHING…');
  await native.click();
  assert.deepEqual(calls, [['open_game']]);
  complete.resolve();
  await pendingSuccess;
  assert.equal(native.nodes['play-btn'].disabled, false);
  assert.equal(native.nodes['play-btn'].attrs['aria-busy'], undefined);
  assert.equal(native.nodes['play-btn'].querySelector('strong').textContent, 'PLAY');
  assert.equal(native.nodes['status-msg'].dataset.state, 'success');
  assert.equal(native.nodes['status-msg'].textContent, 'The game is running in a separate window.');
  console.log('PASS: one open_game request at a time; English loading and success messages, button restored.');

  const pendingFailure = native.click();
  await native.click();
  assert.deepEqual(calls, [['open_game'], ['open_game']]);
  const errorPayload = '<img src=x onerror=alert(1)>';
  complete.reject(errorPayload);
  await pendingFailure;
  assert.equal(native.nodes['play-btn'].disabled, false);
  assert.equal(native.nodes['play-btn'].attrs['aria-busy'], undefined);
  assert.equal(native.nodes['play-btn'].querySelector('strong').textContent, 'PLAY');
  assert.equal(native.nodes['status-msg'].dataset.state, 'error');
  assert.equal(native.nodes['status-msg'].textContent, 'Unable to start the game: ' + errorPayload);
  console.log('PASS: English error message treats details as text; button is ready for another attempt.');

  for (const stored of [null, 'on', 'off']) {
    const reduced = boot({reduced: true, stored});
    const motionToggle = reduced.nodes['motion-toggle'];
    assert.equal(reduced.classes.has('motion-paused'), true);
    assert.equal(motionToggle.attrs['aria-pressed'], 'false');
    assert.equal(motionToggle.disabled, true);
    assert.equal(motionToggle.querySelector('span').textContent, 'Reduced motion');
    motionToggle.handlers.click();
    assert.equal(reduced.classes.has('motion-paused'), true);
    assert.equal(motionToggle.attrs['aria-pressed'], 'false');
    assert.equal(motionToggle.disabled, true);
    assert.equal(motionToggle.querySelector('span').textContent, 'Reduced motion');
  }
  assert.equal(boot({stored: 'off'}).classes.has('motion-paused'), true);
  assert.equal(boot().classes.has('motion-paused'), false);
  console.log('PASS: reduced motion is respected on load and click; disabled button has an English label.');
})().catch(error => { console.error(error); process.exitCode = 1; });
