// Presentation only. Never use these resources to localize document content.
(() => {
  const messages = globalThis.HanContextMessages;
  const locales = ['zh-HK', 'zh-TW', 'zh-CN', 'en'];
  let locale = 'zh-HK';
  function t(key, parameters = {}, requestedLocale = locale) {
    const message = messages[requestedLocale]?.[key];
    if (typeof message !== 'string' || !message.trim()) throw new Error(`Missing UI translation: ${requestedLocale}/${key}`);
    return message.replace(/\{([a-z][a-z0-9_]*)\}/g, (_, name) => {
      if (!Object.hasOwn(parameters, name)) throw new Error(`Missing UI parameter: ${key}/${name}`);
      return String(parameters[name]);
    });
  }
  function setLocale(value) {
    if (!locales.includes(value)) throw new Error(`Unsupported UI locale: ${value}`);
    locale = value;
  }
  function apply(root = document) {
    root.documentElement.lang = locale;
    for (const element of root.querySelectorAll('[data-i18n]')) element.textContent = t(element.dataset.i18n);
    for (const element of root.querySelectorAll('[data-i18n-label]')) element.setAttribute('aria-label', t(element.dataset.i18nLabel));
    for (const element of root.querySelectorAll('[data-i18n-placeholder]')) element.setAttribute('placeholder', t(element.dataset.i18nPlaceholder));
    for (const element of root.querySelectorAll('[data-i18n-self]')) element.textContent = t('locale.self', {}, element.dataset.i18nSelf);
  }
  globalThis.HanContextI18n = { t, setLocale, apply, locales };
})();
