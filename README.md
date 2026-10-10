# HanContext

**懂情境的中文地區化**  
**Chinese, localized with context.**

> **同一種中文，不只換字，更要換對語境。**

HanContext 是一個 offline-first、可控制、可解釋、可重現的中文地區化工具。

它不只是處理簡體與繁體字形，還會考慮中國內地、香港、台灣之間不同的地區用詞、專業術語、人名及專有名稱，以及不同使用情境下的用詞習慣。

目標是：

> **盡量保留原句，只處理真正需要地區化的內容。**

## 為甚麼核心不依賴 LLM？

HanContext 的核心使用 **詞庫 + 規則 + 使用情境 + deterministic engine**，而不是每次把全文交給生成式 AI 重新判斷。

這種設計特別適合需要穩定術語的工作：

- **一致**：相同文字、設定及資料版本，得到相同結果；
- **可解釋**：重要修改可以說明原因；
- **可控制**：有多個合理答案時，由使用者決定；
- **不亂猜**：沒有足夠依據時保留原文；
- **可離線**：核心功能不需要帳戶、API 或雲端 LLM；
- **較穩定**：不直接依賴 token 費用、模型版本或外部服務可用性。

**Same rules. Same result. Every time.**

HanContext 並不排斥 AI。未來 AI 可以作為選擇性輔助，但 deterministic core 可以獨立運作。

## 情境化用詞與替代詞

同一個詞，在不同情境可以有不同答案。HanContext 以明確的 **Usage Context / 使用情境** 協助選詞，而不是暗中猜測文章內容。

目前 Alpha 的基本情境包括：

- General / 一般
- Technology / Software / 科技・軟件
- Banking / Finance / 銀行・金融
- Business / Marketing / 商業・市場推廣
- Legal / 法律
- Education / 教育
- Government / Public Administration / 政府・公共行政

核心現已支援 occurrence-specific 替代詞及一次性選擇：有明確答案時標示 **Recommended**；其他合理選擇列為 **Also valid**；如果無法安全判斷，則標示 **Needs your decision**。同一詞在不同位置可以獨立選擇，並支援安全的單次修改、位置追蹤及 undo。

> **寧願保留原文，也不要自行猜測。**

## 目前支援

```text
zh-CN   → zh-HK
zh-CN   → zh-TW
zh-Hant → zh-HK
zh-Hant → zh-TW
```

目前尚未支援所有反向轉換。

## Desktop Alpha 狀態

**目前進度：** 核心功能已完成至 **My Terms / My Dictionaries**。Desktop Alpha 的主要 Pre-UX 設計基線亦已確立；下一階段將集中於把使用情境、地區化模式、替代詞、解釋、記憶選擇、私人詞庫及資料來源透明度整合成一般使用者可以直接操作的流程。

已完成主要基礎：

- deterministic 中文地區化核心；
- Usage Context 與情境化術語選擇；
- Alternative Terms、精確位置追蹤、單次選擇及 undo 核心；
- My Terms、多個私人詞庫、情境／全情境記憶偏好及安全兼容遷移核心；
- 人名、專有名稱及 protected terms 處理；
- Python reference engine 與 Rust shared core；
- Runtime API v1；
- Tauri 2 Desktop 基礎；
- 本機資料庫、版本化資料包及經驗證的更新機制；
- offline fixture-backed CI。

Desktop Alpha 仍在完成：

- Desktop 內的使用情境、地區化模式、可點擊替代詞、解釋及記憶選擇介面；
- Desktop My Dictionaries 管理、匯入及啟用／停用介面；
- 四語介面（`zh-HK` / `zh-TW` / `zh-CN` / `en`）、System / Light / Dark / E-ink / Mono 外觀模式，以及 WCAG 2.2 AA 無障礙介面基礎；
- Official Dictionary 版本、已收錄來源及來源詳情的漸進式透明度介面；
- 簡化 Desktop Alpha 操作流程；
- Alpha 安裝及新手使用體驗。

詳情請參閱 `PROJECT_STATE.md`、`docs/HANCONTEXT_ALPHA_MVP.md` 及 `docs/ALTERNATIVE_TERMS.md`。
私人詞庫核心與兼容遷移詳見 [My Dictionaries](docs/MY_DICTIONARIES.md)。

Issue #60 的 Stage A 實作提供四語切換、四種外觀選項及安全保存介面顯示設定，待 PR 審核；這不是完整 Desktop Alpha 流程，也不代表已取得 WCAG 認證。使用情境、模式及私人詞庫管理介面仍屬後續工作。詳見 [Desktop 介面基礎](docs/DESKTOP_UI_FOUNDATION.md)。

## 開源與資料授權

HanContext 軟件採用 **Apache License 2.0**。

第三方詞庫、術語表及其他資料來源會獨立審核授權。**公開可以查閱，不代表一定可以重新發佈。** 未確認授權的資料不會因格式轉換而自動變成可重新分發的官方資料。

使用者自行匯入的私人詞庫，也不會自動成為 HanContext 官方資料包的一部分。

詳情請參閱：

- `docs/DATA_POLICY.md`
- `docs/DATA_SOURCES.md`
- `docs/LICENSE_PACKAGING.md`

## Developer information

以下內容主要供開發者及貢獻者參考。

目前技術基礎包括：

- Rust shared localization core
- Python reference implementation
- SQLite terminology database
- versioned Usage Context profiles
- deterministic context-aware terminology selection
- occurrence-specific alternatives and one-time review sessions
- versioned private My Dictionaries / My Terms core
- Tauri 2 desktop shell
- authenticated shared-data package updates
- protected `main` with required tests

主要文件：

- `PROJECT_STATE.md`
- `docs/PRODUCT_REQUIREMENTS.md`
- `docs/HANCONTEXT_ALPHA_MVP.md`
- `docs/ROADMAP.md`
- `docs/CONTEXT_PROFILES.md`
- `docs/CONTEXT_SELECTION.md`
- `docs/ALTERNATIVE_TERMS.md`

### Quick development demo

```bash
python -m pip install -r requirements-dev.txt
python scripts/build_demo_database.py
python scripts/localize_text.py \
  --db build/regional-demo.sqlite \
  --from zh-CN \
  --to zh-TW \
  --text "布拉德·皮特研究人工智能"
```

The fixture database is for tests/development only and is not an authoritative terminology release.

## English summary

**HanContext — Chinese, localized with context.**

HanContext is an offline-first, open-source Chinese regional localization project focused on deterministic, explainable and controllable terminology adaptation across the Chinese Mainland, Hong Kong and Taiwan.

Its core does not require an LLM, allowing reproducible, auditable and offline localization. Context-aware terminology selection, occurrence-specific alternatives and core private remembered preferences are implemented in the shared core/reference; the project is currently in **Desktop Alpha development**.

---

The repository retains the historical name `chinese-regional-localizer`; the working product identity is **HanContext**.
