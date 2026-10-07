import json
import sqlite3
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = REPO_ROOT / "scripts"
if str(SCRIPTS) not in sys.path:
    sys.path.insert(0, str(SCRIPTS))

import build_data_package as package_builder  # noqa: E402


class DataPackageBuilderTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.manifest = REPO_ROOT / "data-registry" / "sources.yaml"

    def tearDown(self):
        self.tmp.cleanup()

    def make_db(
        self,
        *,
        pack: str = "core",
        source_id: str = "opencc",
        resource_key: str = "STCharacters.txt",
    ) -> Path:
        path = self.root / f"{source_id}-{pack}.sqlite"
        conn = sqlite3.connect(path)
        conn.executescript(
            """
            CREATE TABLE build_metadata(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            CREATE TABLE source_versions(
                source_version_id INTEGER PRIMARY KEY,
                source_id TEXT NOT NULL,
                resource_key TEXT NOT NULL,
                is_current INTEGER NOT NULL,
                version_label TEXT,
                revision_id TEXT,
                checksum_sha256 TEXT
            );
            """
        )
        conn.execute("INSERT INTO build_metadata VALUES ('pack_type', ?)", (pack,))
        conn.execute(
            """
            INSERT INTO source_versions(
                source_id, resource_key, is_current, version_label, revision_id, checksum_sha256
            ) VALUES (?, ?, 1, 'test', 'rev', 'abc')
            """,
            (source_id, resource_key),
        )
        conn.commit()
        conn.close()
        return path

    def test_build_package_emits_checksum_and_source_summary(self):
        db = self.make_db()
        output = self.root / "package"
        result = package_builder.build_package(
            db_path=db,
            manifest_path=self.manifest,
            output_dir=output,
            package_id="regional-core",
            version="1.0.0",
            created_at="2026-10-07T00:00:00Z",
        )
        self.assertEqual(result["pack_type"], "core")
        self.assertEqual(result["database"]["file"], "regional.sqlite")
        self.assertEqual(len(result["database"]["sha256"]), 64)
        self.assertEqual(result["sources"][0]["source_id"], "opencc")
        saved = json.loads((output / "package.json").read_text(encoding="utf-8"))
        self.assertEqual(saved, result)

    def test_wrong_pack_for_source_is_rejected(self):
        db = self.make_db(pack="attribution", source_id="opencc")
        with self.assertRaises(package_builder.PackageBuildError):
            package_builder.validate_database_for_release(
                db, package_builder.poc_builder.load_manifest(self.manifest)
            )

    def test_blocked_source_is_rejected(self):
        db = self.make_db(
            pack="pending",
            source_id="openhownet",
            resource_key="core-data",
        )
        with self.assertRaises(package_builder.PackageBuildError):
            package_builder.validate_database_for_release(
                db, package_builder.poc_builder.load_manifest(self.manifest)
            )

    def test_resource_outside_allowlist_is_rejected(self):
        db = self.make_db(resource_key="not-approved.txt")
        with self.assertRaises(package_builder.PackageBuildError):
            package_builder.validate_database_for_release(
                db, package_builder.poc_builder.load_manifest(self.manifest)
            )

    def test_unsafe_package_tokens_are_rejected(self):
        db = self.make_db()
        with self.assertRaises(package_builder.PackageBuildError):
            package_builder.build_package(
                db_path=db,
                manifest_path=self.manifest,
                output_dir=self.root / "package",
                package_id="../escape",
                version="1.0.0",
            )


if __name__ == "__main__":
    unittest.main()
