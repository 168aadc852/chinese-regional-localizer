import sqlite3
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = REPO_ROOT / "scripts"
if str(SCRIPTS) not in sys.path:
    sys.path.insert(0, str(SCRIPTS))

import build_data_package  # noqa: E402
import build_release_catalog  # noqa: E402


class ReleaseCatalogBuilderTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.source_manifest = REPO_ROOT / "data-registry" / "sources.yaml"

    def tearDown(self):
        self.tmp.cleanup()

    def make_database(self) -> Path:
        path = self.root / "shared.sqlite"
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
        conn.execute("INSERT INTO build_metadata VALUES ('pack_type', 'core')")
        conn.execute(
            """
            INSERT INTO source_versions(
                source_id, resource_key, is_current, version_label, revision_id, checksum_sha256
            ) VALUES ('opencc', 'opencc:STCharacters.txt', 1, 'test', 'rev', 'abc')
            """
        )
        conn.commit()
        conn.close()
        return path

    def make_package(self) -> Path:
        package_dir = self.root / "package"
        build_data_package.build_package(
            db_path=self.make_database(),
            manifest_path=self.source_manifest,
            output_dir=package_dir,
            package_id="regional-core",
            version="1.0.0",
            created_at="2026-10-07T00:00:00Z",
        )
        return package_dir

    def test_catalog_contains_exact_package_manifest_hash(self):
        package_dir = self.make_package()
        output = self.root / "release-catalog.json"
        catalog = build_release_catalog.build_catalog(
            package_dirs=[package_dir],
            manifest_path=self.source_manifest,
            output_path=output,
            sequence=7,
            generated_at_unix=1_000,
            expires_at_unix=2_000,
        )
        raw_manifest = (package_dir / "package.json").read_bytes()
        self.assertEqual(
            catalog["packages"][0]["manifest_sha256"],
            build_release_catalog.sha256_bytes(raw_manifest),
        )
        self.assertEqual(catalog["sequence"], 7)
        self.assertTrue(output.read_bytes().endswith(b"\n"))

    def test_expiry_must_be_after_generation(self):
        package_dir = self.make_package()
        with self.assertRaises(build_release_catalog.CatalogBuildError):
            build_release_catalog.build_catalog(
                package_dirs=[package_dir],
                manifest_path=self.source_manifest,
                output_path=self.root / "catalog.json",
                sequence=1,
                generated_at_unix=2_000,
                expires_at_unix=2_000,
            )

    def test_duplicate_package_identity_is_rejected(self):
        package_dir = self.make_package()
        with self.assertRaises(build_release_catalog.CatalogBuildError):
            build_release_catalog.build_catalog(
                package_dirs=[package_dir, package_dir],
                manifest_path=self.source_manifest,
                output_path=self.root / "catalog.json",
                sequence=1,
                generated_at_unix=1_000,
                expires_at_unix=2_000,
            )


if __name__ == "__main__":
    unittest.main()
