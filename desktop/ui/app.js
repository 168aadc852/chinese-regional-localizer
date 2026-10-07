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
const sharedDatabaseStatus = document.getElementById('sharedDatabaseStatus');
const userDatabaseStatus = document.getElementById('userDatabaseStatus');
const chooseSharedDatabase = document.getElementById('chooseSharedDatabase');
const chooseUserDatabase = document.getElementById('chooseUserDatabase');
const clearUserDatabase = document.getElementById('clearUserDatabase');
const refreshDatabaseStatus = document.getElementById('refreshDatabaseStatus');
const databaseMessage = document.getElementById('databaseMessage');

function invokeTauri() {
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    throw new Error('Tauri runtime 未載入。請用 Tauri desktop app 開啟此介面。');
  }
  return invoke;
}

function updateCount() {
  inputCount.textContent = `${[...inputText.value].length} 字`;
}

function setBusy(busy) {
  localizeButton.disabled = busy;
  localizeButton.textContent = busy ? '處理中…' : '地區化';
}

function setSettingsBusy(busy) {
  for (const button of [chooseSharedDatabase, chooseUserDatabase, clearUserDatabase, refreshDatabaseStatus]) {
    button.disabled = busy;
  }
}

function showError(message) {
  errorMessage.textContent = message;
  errorMessage.classList.toggle('hidden', !message);
}

function showDatabaseMessage(message, isError = false) {
  databaseMessage.textContent = message || '';
  databaseMessage.classList.toggle('hidden', !message);
  databaseMessage.classList.toggle('settings-error', Boolean(message) && isError);
}

function text(value) {
  return value == null ? '' : String(value);
}

function renderDatabaseStatus(status) {
  const sharedName = status?.shared_name || '未設定';
  sharedDatabaseStatus.textContent = status?.shared_ready
    ? `可用：${sharedName}`
    : `不可用：${sharedName}${status?.shared_message ? ` · ${status.shared_message}` : ''}`;
  sharedDatabaseStatus.classList.toggle('status-ok', Boolean(status?.shared_ready));
  sharedDatabaseStatus.classList.toggle('status-error', !status?.shared_ready);

  if (!status?.user_name) {
    userDatabaseStatus.textContent = '未啟用；會只使用 shared regional database。';
    userDatabaseStatus.classList.remove('status-error');
    userDatabaseStatus.classList.add('status-ok');
    clearUserDatabase.disabled = true;
  } else {
    userDatabaseStatus.textContent = status?.user_ready
      ? `已啟用：${status.user_name}`
      : `不可用：${status.user_name}${status?.user_message ? ` · ${status.user_message}` : ''}`;
    userDatabaseStatus.classList.toggle('status-ok', Boolean(status?.user_ready));
    userDatabaseStatus.classList.toggle('status-error', !status?.user_ready);
    clearUserDatabase.disabled = false;
  }
}

async function loadDatabaseStatus() {
  showDatabaseMessage('');
  setSettingsBusy(true);
  try {
    const status = await invokeTauri()('database_status');
    renderDatabaseStatus(status);
  } catch (error) {
    showDatabaseMessage(typeof error === 'string' ? error : error?.message || String(error), true);
  } finally {
    setSettingsBusy(false);
  }
}

async function runDatabaseCommand(command, successMessage) {
  showDatabaseMessage('');
  setSettingsBusy(true);
  try {
    const status = await invokeTauri()(command);
    renderDatabaseStatus(status);
    showDatabaseMessage(successMessage);
  } catch (error) {
    showDatabaseMessage(typeof error === 'string' ? error : error?.message || String(error), true);
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
  setBusy(true);
  try {
    const response = await invokeTauri()('localize_text', {
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
    runtimeBadge.textContent = response.user_dictionary_applied ? 'Rust + user dictionary' : 'Rust runtime';
    renderChanges(Array.isArray(response.changes) ? response.changes : []);
  } catch (error) {
    outputText.value = '';
    reviewBadge.classList.add('hidden');
    renderChanges([]);
    showError(typeof error === 'string' ? error : error?.message || String(error));
  } finally {
    setBusy(false);
  }
}

inputText.addEventListener('input', updateCount);
localizeButton.addEventListener('click', localize);
refreshDatabaseStatus.addEventListener('click', loadDatabaseStatus);
chooseSharedDatabase.addEventListener('click', () => runDatabaseCommand('choose_shared_database', 'Shared database 設定已更新。'));
chooseUserDatabase.addEventListener('click', () => runDatabaseCommand('choose_user_database', 'User dictionary 設定已更新。'));
clearUserDatabase.addEventListener('click', () => runDatabaseCommand('clear_user_database', 'User dictionary 已停用。'));

updateCount();
loadDatabaseStatus();
