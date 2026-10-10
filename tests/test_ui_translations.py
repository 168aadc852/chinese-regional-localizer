import csv
import io
import re
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import check_ui_translations as ui


class TranslationTests(unittest.TestCase):
    def table(self, rows):
        stream = io.StringIO(newline="")
        writer = csv.writer(stream)
        writer.writerow(ui.COLUMNS)
        writer.writerows(rows)
        return stream.getvalue()

    def row(self, key="editor.count"):
        return [key, "editor", "Unicode count", "{count} 字", "{count} 字", "{count} 字", "{count} characters"]

    def test_committed_resources_are_complete_and_deterministic(self):
        self.assertEqual(len(ui.validate()), 4)
        parsed = ui.parse_table((ui.UI / "i18n/ui-strings.csv").read_text(encoding="utf-8"))
        self.assertEqual(ui.artifacts(parsed), ui.artifacts(parsed))
        self.assertGreater(len(ui.references(ui.UI)), 50)

    def test_missing_duplicate_invalid_key_and_placeholder_fail(self):
        bad_rows = []
        missing = self.row()
        missing[4] = " "
        bad_rows.append([missing])
        mismatch = self.row()
        mismatch[3] = "{version} 字"
        bad_rows.append([mismatch])
        malformed = self.row()
        malformed[5] = "{count 字"
        bad_rows.append([malformed])
        bad_rows.extend([[self.row(), self.row()], [self.row("English sentence")]])
        for rows in bad_rows:
            with self.subTest(rows=rows), self.assertRaises(ui.TranslationError):
                ui.parse_table(self.table(rows))

    def test_malformed_csv_and_empty_or_wrong_header_fail(self):
        for source in ["", "key,en\nhello,Hello\n", self.table([]),
                       ",".join(ui.COLUMNS) + '\n"unclosed',
                       ",".join(ui.COLUMNS) + "\na,b,c\n"]:
            with self.subTest(source=source), self.assertRaises(ui.TranslationError):
                ui.parse_table(source)

    def test_quoted_comma_newline_and_bom_collaboration(self):
        row = self.row()
        row[2] = 'Note with comma, newline\nand "quotes"'
        parsed = ui.parse_table(self.table([row]))
        self.assertEqual(parsed["en"]["editor.count"], "{count} characters")
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "i18n").mkdir()
            (root / "i18n/ui-strings.csv").write_text(self.table([row]), encoding="utf-8-sig", newline="")
            self.assertEqual(len(ui.validate(root, generate=True)), 4)

    def test_unknown_code_reference_and_generated_drift_fail_read_only(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "i18n").mkdir()
            (root / "i18n/ui-strings.csv").write_text(self.table([self.row()]), encoding="utf-8", newline="")
            ui.validate(root, generate=True)
            (root / "app.js").write_text("t('unknown.key')", encoding="utf-8")
            with self.assertRaisesRegex(ui.TranslationError, "unknown keys"):
                ui.validate(root)
            (root / "app.js").write_text("key('editor.count')", encoding="utf-8")
            resource = root / "i18n/locales/en.json"
            resource.write_text('{"extra.key":"bad"}', encoding="utf-8")
            before = resource.read_bytes()
            with self.assertRaisesRegex(ui.TranslationError, "resources differ"):
                ui.validate(root)
            self.assertEqual(resource.read_bytes(), before)
            ui.validate(root, generate=True)
            (root / "i18n/locales/fr.json").write_text("{}", encoding="utf-8")
            with self.assertRaisesRegex(ui.TranslationError, "unknown locale"):
                ui.validate(root)

    def test_flag_labels_are_rejected(self):
        row = self.row()
        row[-1] = "🇭🇰 {count}"
        with self.assertRaisesRegex(ui.TranslationError, "flags"):
            ui.parse_table(self.table([row]))


def luminance(hex_colour):
    components = [int(hex_colour[index:index + 2], 16) / 255 for index in (1, 3, 5)]
    linear = [v / 12.92 if v <= .04045 else ((v + .055) / 1.055) ** 2.4 for v in components]
    return sum(a * b for a, b in zip(linear, (.2126, .7152, .0722)))


def contrast(a, b):
    values = sorted([luminance(a), luminance(b)])
    return (values[1] + .05) / (values[0] + .05)


class ThemeBaselineTests(unittest.TestCase):
    def test_tokens_meet_text_ui_and_focus_contrast_in_every_theme(self):
        css = (ui.UI / "styles.css").read_text(encoding="utf-8")
        for theme in ("light", "dark", "eink_mono"):
            block = re.search(r'data-theme="' + theme + r'"\]\s*\{([^}]+)', css).group(1)
            tokens = dict(re.findall(r"--([a-z-]+):\s*(#[0-9a-f]{6})", block))
            text_tokens = ["text-primary", "text-secondary"] + [k for k in tokens if k.startswith("status-")]
            for surface in ("surface-primary", "surface-secondary"):
                for token in text_tokens:
                    with self.subTest(theme=theme, token=token, surface=surface):
                        self.assertGreaterEqual(contrast(tokens[token], tokens[surface]), 4.5)
                for token in ("border-default", "focus-ring", "action-primary"):
                    self.assertGreaterEqual(contrast(tokens[token], tokens[surface]), 3)
            for action in ("action-primary", "action-hover"):
                self.assertGreaterEqual(contrast(tokens[action], tokens["action-text"]), 4.5)
            if theme == "eink_mono":
                for colour in tokens.values():
                    self.assertEqual(colour[1:3], colour[3:5])
                    self.assertEqual(colour[3:5], colour[5:7])

    def test_semantic_css_focus_motion_and_target_baseline(self):
        css = (ui.UI / "styles.css").read_text(encoding="utf-8")
        # No semantic colours outside the token definitions.
        css_without_tokens = re.sub(r"--[a-z-]+:\s*#[0-9a-f]{6};", "", css)
        self.assertNotRegex(css_without_tokens, r"#[0-9a-f]{3,8}\b|rgba?\(|linear-gradient|filter:")
        self.assertIn("button:focus-visible", css)
        self.assertIn("outline: 3px solid var(--focus-ring)", css)
        self.assertIn("prefers-reduced-motion: reduce", css)
        self.assertIn("min-height: 44px", css)
        self.assertIn("min-width: 44px", css)


if __name__ == "__main__":
    unittest.main()
