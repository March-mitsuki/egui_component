"""Release preparation tests use a temporary checkout; no upload is performed."""
import importlib.util
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("release", ROOT / "publish.py")
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class PublishTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for filename in ["Cargo.toml", "style_macros/Cargo.toml", "router/Cargo.toml", "README.md", "README.zh.md"]:
            target = self.root / filename
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / filename, target)
        for name, value in {
            "CWD": self.root,
            "CARGO_TOML_EGUI_COMPONENT": self.root / "Cargo.toml",
            "CARGO_TOML_STYLE_MACROS": self.root / "style_macros/Cargo.toml",
            "CARGO_TOML_ROUTER": self.root / "router/Cargo.toml",
        }.items():
            patcher = patch.object(release, name, value)
            patcher.start()
            self.addCleanup(patcher.stop)

    def test_bump_keeps_all_crates_dependencies_and_readmes_in_sync(self):
        version = "0.2.0-beta.1"
        with patch.object(release, "run_cmd") as run:
            release.Cli().bump(version)
        run.assert_called_once_with(["cargo", "update", "--workspace"], self.root)
        self.assertEqual(release.validate_release(), version)
        main = release.load_toml(self.root / "Cargo.toml")
        self.assertEqual(main["dependencies"]["egui_component_style_macros"]["path"], "style_macros")
        self.assertEqual(main["dev-dependencies"]["egui_component_router"]["path"], "router")
        self.assertEqual(main["dependencies"]["egui"]["version"], "0.34")
        self.assertIn('branch = "dev"', (self.root / "README.zh.md").read_text())

    def test_mismatch_prevents_any_publish(self):
        release.bump_cargo_toml_version(self.root / "router/Cargo.toml", "9.0.0")
        with patch.object(release, "run_cmd") as run, self.assertRaises(ValueError):
            release.Cli().publish("router", publish=True)
        run.assert_not_called()

    def test_missing_registry_version_prevents_publish(self):
        release.replace_toml_value(self.root / "Cargo.toml", "dev-dependencies",
                                   "egui_component_router", '{ path = "router" }')
        with self.assertRaises(ValueError):
            release.validate_release()

    def test_router_aliases_dry_run_and_real_publish(self):
        with patch.object(release, "run_cmd") as run:
            release.Cli().publish("router")
            run.assert_called_once_with(["cargo", "publish", "--dry-run", "--allow-dirty"], self.root / "router")
        with patch.object(release, "run_cmd") as run:
            release.Cli().publish("egui_component_router", publish=True)
            run.assert_called_once_with(["cargo", "publish"], self.root / "router")

    def test_invalid_versions_do_not_change_files(self):
        before = (self.root / "Cargo.toml").read_bytes()
        for version in ["", "v1.0.0", "1.2", "01.0.0", "1.0.0-alpha.01", '1.0.0"\nfoo = "bar']:
            with self.subTest(version=version), self.assertRaises(ValueError):
                release.bump_version(version)
        self.assertEqual(before, (self.root / "Cargo.toml").read_bytes())

    def test_check_does_not_modify_manifests(self):
        before = {path: path.read_bytes() for path in self.root.rglob("*") if path.is_file()}
        release.Cli().check()
        self.assertEqual(before, {path: path.read_bytes() for path in before})

    def test_run_cmd_uses_argv_and_propagates_failure(self):
        release.run_cmd([sys.executable, "-c", "import sys; assert sys.argv[1] == 'with spaces'", "with spaces"], self.root)
        with self.assertRaises(subprocess.CalledProcessError):
            release.run_cmd([sys.executable, "-c", "raise SystemExit(7)"], self.root)


if __name__ == "__main__":
    unittest.main()
