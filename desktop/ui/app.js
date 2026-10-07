const sourceLocale = document.getElementById('sourceLocale');
const targetLocale = document.getElementById('targetLocale');
const inputText = document.getElementById('inputText');
const outputText = document.getElementById('outputText');
const localizeButton = document.getElementById('localizeButton');
const inputCount = document.getElementById('inputCount');
const changeCount = document.getElementById('changeCount');
const changes = document.getElementById('changes');
const reviewBadge = document.getElementById('reviewBadge');
const errorMessage = document.getElementById('errorMessage');
const runtimeBadge = document.getElementById('runtimeBadge');
const dbReadyBadge = document.getElementById('dbReadyBadge');
const sharedDbPath = document.getElementById('sharedDbPath');
const userDbPath = document.getElementById('userDbPath');
const dbValidationMessage = document.getElementById('dbValidationMessage');
const chooseSharedDb = document.getElementById('chooseSharedDb');
const chooseUserDb = document.getElementById('chooseUserDb');
const clearUserDb = document.getElementById('clearUserDb');

let runtimeReady = false;
let localizationBusy = false;
let settingsBusy = false;

function invoke() {
  const fn = window.__TAURI__?.core?.invoke;
  if (!fn) {
    throw new Error('Tauri runtime 未載入。請用 Tauri desktop app 開啟此介面。');
  }
  return fn;
}

function updateCount() {
  inputCount.textContent = `${[...inputText.value].length} 字`;
}

function syncButtons() {
  localizeButton.disabled = localizationBusy || settingsBusy || !runtimeReady;
  localizeButton.textContent = localizationBusy ? '處理中…' : '地區化';
  for (const button of [chooseSharedDb, chooseUserDb, clearUserDb]) {
    button.disabled = settingsBusy || localizationBusy;
  }
}

function setLocalizationBusy(busy) {
  localizationBusy = busy;
  syncButtons();
}

function setSettingsBusy(busy) {
  settingsBusy = busy;
  syncButtons();
}

function showError(message) {
  errorMessage.textContent = message;
  errorMessage.classList.toggle('hidden', !message);
}

function text(value) {
  return value == null ? '' : String(value);
}

function renderRuntimeStatus(status) {
  runtimeReady = Boolean(status?.runtime_ready);
  sharedDbPath.textContent = status?.shared_db || '未設定';
  userDbPath.textContent = status?.user_dictionary_enabled
    ? (status?.user_db || '已啟用，但路徑不可用')
    : '未啟用私人詞庫';

  dbReadyBadge.classList.toggle('ready', runtimeReady);
  dbReadyBadge.classList.toggle('problem', !runtimeReady);
  dbReadyBadge.textContent = runtimeReady ? '資料庫可用' : '資料庫有問題';
  runtimeBadge.textContent = status?.user_dictionary_enabled
    ? 'Rust Runtime API v1 + private dictionary'
    : 'Rust Runtime API v1';

  const validationMessage = status?.validation_error || '';
  dbValidationMessage.textContent = validationMessage;
  dbValidationMessage.classList.toggle('hidden', !validationMessage);
  clearUserDb.classList.toggle('hidden', !status?.user_dictionary_enabled);
  syncButtons();
}

async function refreshRuntimeStatus() {
  try {
    const status = await invoke()('runtime_status');
    renderRuntimeStatus(status);
  } catch (error) {
    runtimeReady = false;
    syncButtons();
    showError(typeof error === 'string' ? error : error?.message || String(error));
  }
}

async function runSettingsCommand(command) {
  showError('');
  setSettingsBusy(true);
  try {
    const status = await invoke()(command);
    renderRuntimeStatus(status);
  } catch (error) {
    showError(typeof error === 'string' ? error : error?.message || String(error));
    await refreshRuntimeStatus();
  } finally {
    setSettingsBusy(false);
  }
}

function renderChanges(items) {
  changes.replaceChildren();
  changeCount.textContent = `${items.length} 項變更`;
  if (!items.length) {
    changes.className = 'changes empty-state';
    changes.textContent = '沒有需要顯示的變更。';
    return;
  }

  changes.className = 'changes';
  for (const item of items) {
    const card = document.createElement('article');
    card.className = 'change-card';

    const top = document.createElement('div');
    top.className = 'change-top';
    const type = document.createElement('span');
    type.className = 'change-type';
    type.textContent = text(item.type || item.reason || 'change');
    const status = document.createElement('span');
    status.className = item.review_needed ? 'review-badge' : 'muted';
    status.textContent = item.review_needed ? '需要檢查' : '已套用';
    top.append(type, status);

    const body = document.createElement('p');
    body.className = 'change-text';
    body.textContent = `${text(item.original)} → ${text(item.replacement)}`;

    const meta = document.createElement('div');
    meta.className = 'change-meta';
    const parts = [
      item.reason && `原因：${item.reason}`,
      item.qid && `Wikidata：${item.qid}`,
      item.source_id && `來源：${item.source_id}`,
      item.provenance && `資料層：${item.provenance}`,
      item.user_term_id && `User rule #${item.user_term_id}`,
      Array.isArray(item.original_input_span) && `原文位置：${item.original_input_span.join('–')}`,
      Array.isArray(item.final_output_span) && `結果位置：${item.final_output_span.join('–')}`,
    ].filter(Boolean);
    for (const part of parts) {
      const span = document.createElement('span');
      span.textContent = part;
      meta.appendChild(span);
    }

    card.append(top, body, meta);
    changes.appendChild(card);
  }
}

async function localize() {
  showError('');
  setLocalizationBusy(true);
  try {
    const response = await invoke()('localize_text', {
      request: {
        api_version: '1',
        text: inputText.value,
        source_locale: sourceLocale.value,
        target_locale: targetLocale.value,
        context: null,
      },
    });
    outputText.value = response.output || '';
    reviewBadge.classList.toggle('hidden', !response.review_needed);
    renderChanges(Array.isArray(response.changes) ? response.changes : []);
  } catch (error) {
    outputText.value = '';
    reviewBadge.classList.add('hidden');
    renderChanges([]);
    showError(typeof error === 'string' ? error : error?.message || String(error));
    await refreshRuntimeStatus();
  } finally {
    setLocalizationBusy(false);
  }
}

inputText.addEventListener('input', updateCount);
localizeButton.addEventListener('click', localize);
chooseSharedDb.addEventListener('click', () => runSettingsCommand('choose_shared_database'));
chooseUserDb.addEventListener('click', () => runSettingsCommand('choose_user_database'));
clearUserDb.addEventListener('click', () => runSettingsCommand('clear_user_database'));

updateCount();
syncButtons();
refreshRuntimeStatus();
