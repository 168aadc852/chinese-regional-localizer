// Zero-dependency presentation regression tests, not a browser/accessibility audit.
const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const ui = path.join(__dirname, '../ui');
const locales = ['zh-HK', 'zh-TW', 'zh-CN', 'en'];

test('desktop enables accessible zoom with only the required webview capability', () => {
  const config = JSON.parse(fs.readFileSync(path.join(__dirname, '../src-tauri/tauri.conf.json'), 'utf8'));
  assert.equal(config.app.windows[0].zoomHotkeysEnabled, true);
  assert.deepEqual(config.app.security.capabilities, [{
    identifier: 'presentation-zoom', windows: ['main'], permissions: ['core:webview:allow-set-webview-zoom'],
  }]);
});

class Element {
  constructor(tag = 'div', attributes = {}) {
    this.tag = tag;
    this.attributes = attributes;
    this.dataset = {};
    for (const [key, value] of Object.entries(attributes)) {
      if (key.startsWith('data-')) this.dataset[key.slice(5).replace(/-([a-z])/g, (_, c) => c.toUpperCase())] = value;
    }
    this.value = '';
    this.textContent = '';
    this.children = [];
    this.listeners = new Map();
    this.classes = new Set((attributes.class || '').split(' '));
    this.classList = {
      toggle: (name, force) => force ? this.classes.add(name) : this.classes.delete(name),
      remove: name => this.classes.delete(name),
      contains: name => this.classes.has(name),
    };
  }
  setAttribute(name, value) { this.attributes[name] = value; }
  append(...children) { this.children.push(...children); }
  appendChild(child) { this.children.push(child); }
  replaceChildren(...children) { this.children = children; this.textContent = ''; }
  addEventListener(name, handler) { this.listeners.set(name, handler); }
  async fire(name) { await this.listeners.get(name)?.(); }
}

async function shell(invoke) {
  const nodes = [...fs.readFileSync(path.join(ui, 'index.html'), 'utf8').matchAll(/<([a-z]+)\b([^>]*)>/g)].map(match => {
    const attributes = Object.fromEntries([...match[2].matchAll(/([a-z-]+)="([^"]*)"/g)].map(x => [x[1], x[2]]));
    return new Element(match[1], attributes);
  });
  const byId = new Map(nodes.filter(node => node.attributes.id).map(node => [node.attributes.id, node]));
  const document = {
    body: nodes.find(node => node.tag === 'body'),
    documentElement: nodes.find(node => node.tag === 'html'),
    getElementById: id => byId.get(id),
    querySelectorAll: selector => nodes.filter(node => Object.hasOwn(node.attributes, selector.slice(1, -1))),
    createElement: tag => new Element(tag),
  };
  for (const node of nodes) node.focus = () => { document.activeElement = node; };
  const media = { matches: false, addEventListener: (_, handler) => { media.handler = handler; } };
  const context = vm.createContext({ document, window: { matchMedia: () => media, __TAURI__: invoke ? { core: { invoke } } : null } });
  const e = id => byId.get(id);
  e('sourceLocale').value = 'zh-CN';
  e('targetLocale').value = 'zh-HK';
  e('inputText').value = '原文🙂';
  for (const filename of ['i18n/locales.js', 'i18n.js', 'presentation.js', 'app.js']) {
    vm.runInContext(fs.readFileSync(path.join(ui, filename), 'utf8'), context, { filename });
  }
  await new Promise(setImmediate);
  return { context, document, e, media, nodes };
}

function backend() {
  const api = {
    calls: [], preferences: { ui_locale: 'zh-HK', appearance: 'system' }, failSave: false,
    async invoke(command, payload) {
      api.calls.push({ command, payload });
      if (command === 'presentation_settings') return { ...api.preferences };
      if (command === 'save_presentation_settings') {
        if (api.failSave) throw 'settings_save_failed';
        api.preferences = { ...payload.preferences };
        return { ...api.preferences };
      }
      if (command === 'database_status') return { shared_name: 'shared.sqlite', shared_ready: true, user_name: null, user_ready: true, user_enabled: false };
      if (command === 'update_status') return { configured: false, update_available: false };
      if (command === 'localize_text') return { output: '結果🙂', changes: [{ type: 'term', original: '原文', replacement: '結果', review_needed: false }], review_needed: false };
      throw new Error('Unexpected mock command: ' + command);
    },
  };
  return api;
}

test('four independent complete catalogs interpolate strictly; no mixed-language fallback', async () => {
  const { context } = await shell();
  const i18n = context.HanContextI18n;
  const self = ['繁體中文（香港）', '繁體中文（台灣）', '简体中文（中国内地）', 'English'];
  for (const [index, locale] of locales.entries()) {
    i18n.setLocale(locale);
    assert.equal(i18n.t('locale.self'), self[index]);
    assert.ok(i18n.t('editor.count', { count: 7 }).includes('7'));
  }
  assert.throws(() => i18n.t('unknown.key'), /Missing UI translation/);
  assert.throws(() => i18n.t('editor.count'), /Missing UI parameter/);
  assert.throws(() => i18n.setLocale('fr'), /Unsupported UI locale/);
});

test('system follows light/dark only; explicit mono ignores system changes', async () => {
  const { context, document, media, e } = await shell();
  const resolve = context.HanContextPresentation.resolveAppearance;
  assert.equal(resolve('system', false), 'light');
  assert.equal(resolve('system', true), 'dark');
  assert.equal(resolve('eink_mono', true), 'eink_mono');
  assert.throws(() => resolve('sepia', false), /Unsupported appearance/);
  media.matches = true;
  media.handler();
  assert.equal(document.documentElement.dataset.theme, 'dark');
  e('appearance').value = 'eink_mono';
  await e('appearance').fire('change');
  media.matches = false;
  media.handler();
  assert.equal(document.documentElement.dataset.theme, 'eink_mono');
});

test('all four locales and all appearances switch live without document locale changes or rerun', async () => {
  const api = backend();
  const { e, document, nodes } = await shell(api.invoke);
  e('targetLocale').value = 'zh-TW';
  await e('localizeButton').fire('click');
  const originalCalls = api.calls.filter(x => x.command === 'localize_text').length;
  const selfLabels = nodes.filter(x => x.attributes['data-i18n-self']).map(x => x.textContent);
  for (const locale of locales) {
    e('uiLocale').value = locale;
    await e('uiLocale').fire('change');
    for (const appearance of ['system', 'light', 'dark', 'eink_mono']) {
      e('appearance').value = appearance;
      await e('appearance').fire('change');
      assert.equal(document.documentElement.lang, locale);
      assert.equal(document.documentElement.dataset.theme, appearance === 'system' ? 'light' : appearance);
      assert.equal(e('inputText').value, '原文🙂');
      assert.equal(e('outputText').value, '結果🙂');
      assert.equal(e('sourceLocale').value, 'zh-CN');
      assert.equal(e('targetLocale').value, 'zh-TW');
      assert.deepEqual(nodes.filter(x => x.attributes['data-i18n-self']).map(x => x.textContent), selfLabels);
      assert.equal(e('localizeButton').disabled, false);
      assert.ok(e('changes').children[0].children[0].children[1].textContent.includes('✓'));
    }
  }
  assert.equal(api.calls.filter(x => x.command === 'localize_text').length, originalCalls);
  const request = api.calls.find(x => x.command === 'localize_text').payload.request;
  assert.deepEqual(Object.keys(request).sort(), ['api_version', 'context', 'source_locale', 'target_locale', 'text']);
  assert.equal(request.api_version, '1');
  assert.equal(request.context, null);
});

test('accepted backend preference is restored on a fresh shell; path-free payload', async () => {
  const api = backend();
  const first = await shell(api.invoke);
  first.e('uiLocale').value = 'en';
  await first.e('uiLocale').fire('change');
  first.e('appearance').value = 'dark';
  await first.e('appearance').fire('change');
  const restarted = await shell(api.invoke);
  assert.equal(restarted.document.documentElement.lang, 'en');
  assert.equal(restarted.document.documentElement.dataset.theme, 'dark');
  const payload = api.calls.find(x => x.command === 'save_presentation_settings').payload;
  assert.deepEqual(Object.keys(payload.preferences).sort(), ['appearance', 'ui_locale']);
  assert.ok(!JSON.stringify(payload).includes('db'));
});

test('failed durable save rolls back theme and language and displays localized typed error', async () => {
  const api = backend();
  api.preferences = { ui_locale: 'en', appearance: 'light' };
  const { e, document } = await shell(api.invoke);
  api.failSave = true;
  e('appearance').value = 'dark';
  e('uiLocale').value = 'zh-CN';
  await e('uiLocale').fire('change');
  assert.equal(e('uiLocale').value, 'en');
  assert.equal(e('appearance').value, 'light');
  assert.equal(document.documentElement.dataset.theme, 'light');
  assert.match(e('presentationMessage').textContent, /could not be saved safely/);
  assert.equal(e('presentationMessage').classList.contains('settings-error'), true);
});

test('language switching keeps recoverable errors localized, not raw backend sentences', async () => {
  const api = backend();
  const { e } = await shell(async (command, payload) => {
    if (command === 'choose_shared_database') throw '硬編碼中文錯誤';
    return api.invoke(command, payload);
  });
  await e('chooseSharedDatabase').fire('click');
  e('uiLocale').value = 'en';
  await e('uiLocale').fire('change');
  assert.match(e('databaseMessage').textContent, /Database operation/);
  assert.ok(!e('databaseMessage').textContent.includes('硬編碼'));
  assert.equal(e('clearUserDatabase').disabled, true);
});

test('ordinary browser preview says not saved; never simulates the core', async () => {
  const { e, document } = await shell();
  e('uiLocale').value = 'en';
  await e('uiLocale').fire('change');
  assert.equal(document.documentElement.lang, 'en');
  assert.match(e('presentationMessage').textContent, /not saved/);
  await e('localizeButton').fire('click');
  assert.equal(e('outputText').value, '');
  assert.match(e('errorMessage').textContent, /Tauri runtime is not loaded/);
});

test('presentation save restores blurred keyboard focus but never steals moved focus', async () => {
  const api = backend();
  let host;
  let moveFocus = false;
  host = await shell(async (command, payload) => {
    if (command === 'save_presentation_settings') {
      host.document.activeElement = moveFocus ? host.e('inputText') : host.document.body;
    }
    return api.invoke(command, payload);
  });
  host.e('uiLocale').focus();
  host.e('uiLocale').value = 'en';
  await host.e('uiLocale').fire('change');
  assert.equal(host.document.activeElement, host.e('uiLocale'));
  moveFocus = true;
  host.e('appearance').focus();
  host.e('appearance').value = 'dark';
  await host.e('appearance').fire('change');
  assert.equal(host.document.activeElement, host.e('inputText'));
});
