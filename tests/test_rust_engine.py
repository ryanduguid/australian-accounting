"""Rust owns its checks, command and release format independently of Python."""
import tomllib
import unittest

from test_boundaries import ROOT, select_package


class RustEngineTests(unittest.TestCase):
    def test_component_and_root_routing(self):
        directory = ROOT / "packages/the-exchequer-tally"
        cargo = tomllib.loads((directory / "Cargo.toml").read_text())
        self.assertEqual(cargo["package"]["name"], "the-exchequer-tally")
        self.assertEqual(cargo["package"]["version"], "1.0.0")
        self.assertEqual(select_package.project_version("the-exchequer-tally"), "1.0.0")
        self.assertFalse((directory / "pyproject.toml").exists())
        self.assertFalse((directory / "edwinnixon").exists())
        self.assertTrue((directory / "src/main.rs").is_file())
        root = tomllib.loads((ROOT / "pyproject.toml").read_text())
        self.assertIn("packages/the-exchequer-tally", root["tool"]["uv"]["workspace"]["exclude"])
        # The engine is standalone: no engine/application/path dependency can sneak in.
        self.assertEqual(set(cargo["dependencies"]), {"num-bigint", "num-integer", "num-traits", "serde_json"})
        for value in cargo["dependencies"].values():
            self.assertIsInstance(value, str)

    def test_cargo_ci_security_and_publisher_boundary(self):
        workflows = ROOT / ".github/workflows"
        caller = (workflows / "ci.yml").read_text()
        self.assertIn("uses: ./.github/workflows/ci-exchequer.yml", caller)
        self.assertIn("needs: [changes, packages, exchequer, test, dependency-audit, lint]", caller)
        workflow = (workflows / "ci-exchequer.yml").read_text()
        for gate in ("cargo fmt --check", "cargo clippy --locked --all-targets -- -D warnings", "cargo test --locked", "cargo build --locked --release", "cargo audit --deny warnings"):
            self.assertIn(gate, workflow)
        self.assertIn("os: [ubuntu-latest, windows-latest]", workflow)
        self.assertNotIn("contents: write", workflow)
        self.assertNotIn("id-token: write", workflow)
        self.assertFalse((workflows / "release-the-exchequer-tally.yml").exists())
        self.assertIn("language: rust", (workflows / "codeql.yml").read_text())
        self.assertIn("/packages/the-exchequer-tally", (ROOT / ".github/dependabot.yml").read_text())
        snyk = (workflows / "snyk.yml").read_text()
        self.assertIn("snyk sbom test --experimental --file=snyk-sbom.json", snyk)
        self.assertNotIn("--package-manager=cargo", snyk)
