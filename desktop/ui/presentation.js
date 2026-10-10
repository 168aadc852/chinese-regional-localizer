// Only UI preference resolution; system never resolves to the manual mono mode.
(() => {
  const appearances = ['system', 'light', 'dark', 'eink_mono'];
  function resolveAppearance(appearance, systemDark) {
    if (!appearances.includes(appearance)) throw new Error(`Unsupported appearance: ${appearance}`);
    return appearance === 'system' ? (systemDark ? 'dark' : 'light') : appearance;
  }
  function applyAppearance(appearance, systemDark, root = document.documentElement) {
    root.dataset.theme = resolveAppearance(appearance, systemDark);
  }
  globalThis.HanContextPresentation = { resolveAppearance, applyAppearance, appearances };
})();
