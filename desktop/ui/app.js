const { t, setLocale, apply: applyTranslations } = globalThis.HanContextI18n;
const { applyAppearance } = globalThis.HanContextPresentation;
const key = value => value; // Register dynamic semantic keys for release validation.
const elements = Object.fromEntries([
  'appShell', 'uiLocale', 'appearance', 'presentationMessage',
  'sourceLocale', 'targetLocale', 'inputText', 'outputText', 'localizeButton',
  'inputCount', 'changeCount', 'changes', 'reviewBadge', 'errorMessage', 'runtimeBadge',
  'sharedDatabaseStatus', 'userDatabaseStatus', 'chooseSharedDatabase', 'chooseUserDatabase',
  'clearUserDatabase', 'enableUserDatabase', 'refreshDatabaseStatus', 'databaseMessage',
  'updateBadge', 'updateStatusText', 'checkDataUpdate', 'installDataUpdate', 'updateMessage',
].map(id => [id, document.getElementById(id)]));
const systemAppearance = window.matchMedia('(prefers-color-scheme: dark)');
let preferences = { ui_locale: 'zh-HK', appearance: 'system' };
let lastDatabaseStatus = null;
let lastUpdateStatus = null;
let lastResponse = null;
let hasRun = false;
let localizationBusy = false;
let settingsBusy = false;
let updateBusy = false;
const messages = new Map();

function invokeTauri() {
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) throw new Error('runtime_unavailable');
  return invoke;
}

function operationError(error, fallback) {
  return error?.message === 'runtime_unavailable' ? key('error.runtime_unavailable') : fallback;
}

function setMessage(id, messageKey = null, isError = false) {
  messages.set(id, { messageKey, isError });
  renderMessage(id);
}

function renderMessage(id) {
  const message = messages.get(id);
  const element = elements[id];
  element.textContent = message?.messageKey ? t(message.messageKey) : '';
  element.classList.toggle('hidden', !message?.messageKey);
  element.classList.toggle('settings-error', Boolean(message?.isError));
}

function updateCount() {
  elements.inputCount.textContent = t('editor.count', { count: [...elements.inputText.value].length });
}

function renderBusy() {
  elements.localizeButton.disabled = localizationBusy;
  elements.localizeButton.textContent = localizationBusy ? t('action.processing') : t('action.localize');
  elements.localizeButton.setAttribute('aria-busy', String(localizationBusy));
  for (const id of ['chooseSharedDatabase', 'chooseUserDatabase', 'clearUserDatabase', 'enableUserDatabase', 'refreshDatabaseStatus']) {
    elements[id].disabled = settingsBusy;
  }
  if (!settingsBusy && lastDatabaseStatus) {
    elements.clearUserDatabase.disabled = !lastDatabaseStatus.user_name || !lastDatabaseStatus.user_enabled;
    elements.enableUserDatabase.disabled = !lastDatabaseStatus.user_name || lastDatabaseStatus.user_enabled || !lastDatabaseStatus.user_ready;
  }
  elements.checkDataUpdate.disabled = updateBusy || !lastUpdateStatus?.configured;
  elements.installDataUpdate.disabled = updateBusy || !lastUpdateStatus?.update_available;
  elements.checkDataUpdate.textContent = updateBusy ? t('action.processing') : t('action.check_update');
}

function renderDatabaseStatus(status) {
  const name = status.shared_name || t('status.unset');
  elements.sharedDatabaseStatus.textContent = status.shared_ready
    ? t('database.ready', { name }) : t('database.unavailable', { name });
  elements.sharedDatabaseStatus.classList.toggle('status-ok', Boolean(status.shared_ready));
  elements.sharedDatabaseStatus.classList.toggle('status-error', !status.shared_ready);
  if (!status.user_name) {
    elements.userDatabaseStatus.textContent = t('database.private_unused');
  } else {
    const userKey = !status.user_ready ? key('database.unavailable')
      : status.user_enabled ? key('database.private_enabled') : key('database.private_disabled');
    elements.userDatabaseStatus.textContent = t(userKey, { name: status.user_name });
  }
  elements.userDatabaseStatus.classList.toggle('status-ok', Boolean(status.user_ready));
  elements.userDatabaseStatus.classList.toggle('status-error', !status.user_ready);
}

function renderUpdateStatus(status) {
  if (!status.configured) {
    elements.updateBadge.textContent = t('status.unset');
    elements.updateStatusText.textContent = t('update.unconfigured');
    return;
  }
  const version = status.current_version || t('update.unmanaged');
  elements.updateBadge.textContent = status.update_available ? t('update.available') : t('update.ready');
  elements.updateStatusText.textContent = status.update_available
    ? t('update.versions', { version, offered: status.offered_version })
    : t('update.version', { version });
}

function renderChanges(items) {
  elements.changes.replaceChildren();
  elements.changeCount.textContent = t('changes.count', { count: items.length });
  if (!items.length) {
    elements.changes.className = 'changes empty-state';
    elements.changes.textContent = hasRun ? t('changes.empty') : t('changes.initial');
    return;
  }
  elements.changes.className = 'changes';
  // Existing read-only v1 technical list, not the future Alpha review workflow.
  // Document text and protocol identifiers are data, never UI translations.
  const fields = [
    ['reason', key('changes.reason')], ['qid', key('changes.qid')],
    ['source_id', key('changes.source')], ['provenance', key('changes.provenance')],
    ['user_term_id', key('changes.user_rule')], ['original_input_span', key('changes.input_span')],
    ['final_output_span', key('changes.output_span')],
  ];
  for (const item of items) {
    const card = document.createElement('article');
    card.className = 'change-card';
    const top = document.createElement('div');
    top.className = 'change-top';
    const type = document.createElement('span');
    type.className = 'change-type';
    type.textContent = t('changes.type', { value: item.type || item.reason || '' });
    const status = document.createElement('span');
    status.className = item.review_needed ? 'review-badge' : 'muted';
    status.textContent = item.review_needed ? t('changes.review_needed') : t('changes.applied');
    top.append(type, status);
    const body = document.createElement('p');
    body.className = 'change-text';
    body.textContent = String(item.original ?? '') + ' → ' + String(item.replacement ?? '');
    const meta = document.createElement('div');
    meta.className = 'change-meta';
    for (const [field, messageKey] of fields) {
      if (item[field] == null || item[field] === '') continue;
      const span = document.createElement('span');
      const value = Array.isArray(item[field]) ? item[field].join('–') : item[field];
      span.textContent = t(messageKey, { value });
      meta.appendChild(span);
    }
    card.append(top, body, meta);
    elements.changes.appendChild(card);
  }
}

function renderShell() {
  setLocale(preferences.ui_locale);
  applyTranslations();
  elements.uiLocale.value = preferences.ui_locale;
  elements.appearance.value = preferences.appearance;
  applyAppearance(preferences.appearance, systemAppearance.matches);
  updateCount();
  if (lastDatabaseStatus) renderDatabaseStatus(lastDatabaseStatus);
  if (lastUpdateStatus) renderUpdateStatus(lastUpdateStatus);
  elements.runtimeBadge.textContent = lastResponse?.user_dictionary_applied ? t('runtime.private') : t('runtime.core');
  elements.reviewBadge.classList.toggle('hidden', !lastResponse?.review_needed);
  renderChanges(lastResponse?.changes || []);
  for (const id of messages.keys()) renderMessage(id);
  renderBusy();
}

async function loadPresentation() {
  try {
    if (!window.__TAURI__?.core?.invoke) {
      setMessage('presentationMessage', key('presentation.preview'));
    } else {
      preferences = await invokeTauri()('presentation_settings');
    }
  } catch {
    setMessage('presentationMessage', key('error.preferences_load'), true);
  } finally {
    renderShell();
    elements.uiLocale.disabled = false;
    elements.appearance.disabled = false;
  }
}

const presentationErrors = {
  preferences_unavailable: key('error.preferences_unavailable'),
  settings_unsupported: key('error.settings_unsupported'),
  settings_save_failed: key('error.settings_save_failed'),
};

async function savePresentation() {
  const focused = document.activeElement;
  const desired = { ui_locale: elements.uiLocale.value, appearance: elements.appearance.value };
  elements.uiLocale.disabled = true;
  elements.appearance.disabled = true;
  try {
    if (!window.__TAURI__?.core?.invoke) {
      // Explicit browser-only preview, no localStorage or persistence simulation.
      preferences = desired;
      setMessage('presentationMessage', key('presentation.preview'));
    } else {
      preferences = await invokeTauri()('save_presentation_settings', { preferences: desired });
      setMessage('presentationMessage', key('presentation.saved'));
    }
  } catch (error) {
    setMessage('presentationMessage', presentationErrors[error] || key('error.settings_save_failed'), true);
  } finally {
    // Failed save restores the last accepted theme and locale, not an unsaved choice.
    renderShell();
    elements.uiLocale.disabled = false;
    elements.appearance.disabled = false;
    // Disabling a native select during the write can blur it. Restore focus only
    // if the user has not moved elsewhere while waiting; keep keyboard continuity.
    if (document.activeElement === document.body) focused?.focus();
  }
}

async function loadDatabaseStatus() {
  setMessage('databaseMessage');
  settingsBusy = true;
  renderBusy();
  try {
    lastDatabaseStatus = await invokeTauri()('database_status');
    if (lastDatabaseStatus.settings_message) setMessage('databaseMessage', key('error.settings_restore'), true);
  } catch (error) {
    setMessage('databaseMessage', operationError(error, key('error.database_operation')), true);
  } finally {
    settingsBusy = false;
    renderShell();
  }
}

async function loadUpdateStatus() {
  setMessage('updateMessage');
  try {
    lastUpdateStatus = await invokeTauri()('update_status');
    if (lastUpdateStatus.message && lastUpdateStatus.configured) setMessage('updateMessage', key('update.notice'));
  } catch (error) {
    setMessage('updateMessage', operationError(error, key('error.update_operation')), true);
  } finally {
    renderShell();
  }
}

async function runUpdateCommand(command) {
  setMessage('updateMessage');
  updateBusy = true;
  renderBusy();
  try {
    lastUpdateStatus = await invokeTauri()(command);
    if (lastUpdateStatus.message && lastUpdateStatus.configured) setMessage('updateMessage', key('update.notice'));
    if (command === 'install_data_update') await loadDatabaseStatus();
  } catch (error) {
    setMessage('updateMessage', operationError(error, key('error.update_operation')), true);
  } finally {
    updateBusy = false;
    renderShell();
  }
}

async function runDatabaseCommand(command, successKey) {
  setMessage('databaseMessage');
  settingsBusy = true;
  renderBusy();
  try {
    lastDatabaseStatus = await invokeTauri()(command);
    setMessage('databaseMessage', lastDatabaseStatus.settings_message ? key('error.settings_restore') : successKey, Boolean(lastDatabaseStatus.settings_message));
  } catch (error) {
    setMessage('databaseMessage', operationError(error, key('error.database_operation')), true);
  } finally {
    settingsBusy = false;
    renderShell();
  }
}

async function localize() {
  setMessage('errorMessage');
  localizationBusy = true;
  hasRun = true;
  renderBusy();
  try {
    lastResponse = await invokeTauri()('localize_text', {
      request: {
        api_version: '1',
        text: elements.inputText.value,
        source_locale: elements.sourceLocale.value,
        target_locale: elements.targetLocale.value,
        context: null,
      },
    });
    elements.outputText.value = lastResponse.output || '';
  } catch (error) {
    lastResponse = null;
    elements.outputText.value = '';
    setMessage('errorMessage', operationError(error, key('error.localization')), true);
  } finally {
    localizationBusy = false;
    renderShell();
  }
}

elements.uiLocale.addEventListener('change', savePresentation);
elements.appearance.addEventListener('change', savePresentation);
systemAppearance.addEventListener('change', () => applyAppearance(preferences.appearance, systemAppearance.matches));
elements.inputText.addEventListener('input', updateCount);
elements.localizeButton.addEventListener('click', localize);
elements.refreshDatabaseStatus.addEventListener('click', loadDatabaseStatus);
elements.chooseSharedDatabase.addEventListener('click', () => runDatabaseCommand('choose_shared_database', key('database.shared_saved')));
elements.chooseUserDatabase.addEventListener('click', () => runDatabaseCommand('choose_user_database', key('database.private_saved')));
elements.clearUserDatabase.addEventListener('click', () => runDatabaseCommand('clear_user_database', key('database.disabled')));
elements.enableUserDatabase.addEventListener('click', () => runDatabaseCommand('enable_user_database', key('database.enabled')));
elements.checkDataUpdate.addEventListener('click', () => runUpdateCommand('check_data_update'));
elements.installDataUpdate.addEventListener('click', () => runUpdateCommand('install_data_update'));
renderShell();
elements.appShell.classList.remove('hidden');
loadPresentation();
loadDatabaseStatus();
loadUpdateStatus();
