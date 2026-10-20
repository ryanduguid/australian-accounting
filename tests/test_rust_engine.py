"""Rust owns its checks, command and release format independently of Python."""
import tomllib
import unittest

from test_boundaries import ROOT, RUST_ENGINES, select_package

# The Python import package each port removed.
REMOVED_IMPORTS = {
    "the-exchequer-tally": "edwinnixon",
    "solomons-sword": "louisgoldberg",
    "the-wip-tally": "wiptally",
    "australian-tax-calculators": "austaxcalc",
    "div7a-loan-review": "div7aloan",
    "ato-benchmark-compare": "atobenchmark",
    "payday-super-checker": "paydaysuper",
}
# Runtime crates an engine may use; none is another engine or a path dependency.
ALLOWED_DEPENDENCIES = {"num-bigint", "num-integer", "num-traits", "serde_json", "chrono", "sha2"}


class RustEngineTests(unittest.TestCase):
    def test_component_and_root_routing(self):
        root = tomllib.loads((ROOT / "pyproject.toml").read_text())
        self.assertEqual(set(REMOVED_IMPORTS), set(RUST_ENGINES.values()))
        for component, name in RUST_ENGINES.items():
            with self.subTest(component=component):
                directory = ROOT / component
                cargo = tomllib.loads((directory / "Cargo.toml").read_text())
                self.assertEqual(cargo["package"]["name"], name)
                self.assertEqual(cargo["package"]["version"], "1.0.0")
                self.assertEqual(select_package.project_version(name), "1.0.0")
                self.assertFalse((directory / "pyproject.toml").exists())
                self.assertFalse((directory / REMOVED_IMPORTS[name]).exists())
                self.assertTrue((directory / "src/main.rs").is_file())
                self.assertIn(component, root["tool"]["uv"]["workspace"]["exclude"])
                # The engine is standalone: no engine/application/path dependency can sneak in.
                self.assertLessEqual(set(cargo["dependencies"]), ALLOWED_DEPENDENCIES)
                for value in cargo["dependencies"].values():
                    self.assertNotIn("path", value if isinstance(value, dict) else {})

    def test_binstall_metadata_names_the_ci_archive_layout(self):
        # ci-rust.yml packages <crate>-<version>-<target>/<bin> as .tar.gz, or .zip on
        # Windows; every crate's binstall metadata must name that layout.
        expected = {
            "pkg-url": "{ repo }/releases/download/{ name }/v{ version }/{ name }-{ version }-{ target }{ archive-suffix }",
            "bin-dir": "{ name }-{ version }-{ target }/{ bin }{ binary-ext }",
            "pkg-fmt": "tgz",
            "overrides": {"x86_64-pc-windows-msvc": {"pkg-fmt": "zip"}},
        }
        workflow = (ROOT / ".github/workflows/ci-rust.yml").read_text()
        self.assertIn('base="$CRATE-$version-$target"', workflow)
        self.assertIn('7z a -tzip "$base.zip" "$base"', workflow)
        self.assertIn('tar -C dist -czf "dist/$base.tar.gz" "$base"', workflow)
        for component in [*RUST_ENGINES, "apps/au-accounting"]:
            with self.subTest(component=component):
                package = tomllib.loads((ROOT / component / "Cargo.toml").read_text())["package"]
                self.assertEqual(package["repository"], "https://github.com/ryanduguid/australian-accounting")
                self.assertEqual(package["metadata"]["binstall"], expected)

    def test_umbrella_links_every_engine_by_path(self):
        cargo = tomllib.loads((ROOT / "apps/au-accounting/Cargo.toml").read_text())
        engines = {name: dep for name, dep in cargo["dependencies"].items()
                   if name not in {"serde", "serde_json"}}
        self.assertEqual(engines, {name: {"path": f"../../{component}"} for component, name in RUST_ENGINES.items()})
        source = (ROOT / "apps/au-accounting/src/main.rs").read_text()
        for name in RUST_ENGINES.values():
            self.assertIn(name.replace("-", "_") + "::", source)
        caller = (ROOT / ".github/workflows/ci.yml").read_text()
        self.assertIn("uses: ./.github/workflows/ci-rust.yml\n    with:\n      package: au-accounting\n      root: apps\n", caller)
        self.assertIn("/apps/au-accounting", (ROOT / ".github/dependabot.yml").read_text())

    def test_cargo_ci_security_and_publisher_boundary(self):
        workflows = ROOT / ".github/workflows"
        caller = (workflows / "ci.yml").read_text()
        self.assertIn("needs: [changes, packages, exchequer, solomons, wip, taxcalc, div7a, atobench, payday, umbrella, umbrella-parity, test, dependency-audit, lint]", caller)
        workflow = (workflows / "ci-rust.yml").read_text()
        for gate in ("cargo fmt --check", "cargo clippy --locked --all-targets -- -D warnings", "cargo test --locked", "cargo build --locked --release", "cargo audit --deny warnings"):
            self.assertIn(gate, workflow)
        self.assertIn("os: [ubuntu-latest, windows-latest, macos-latest]", workflow)
        self.assertNotIn("contents: write", workflow)
        self.assertNotIn("id-token: write", workflow)
        self.assertIn("language: rust", (workflows / "codeql.yml").read_text())
        dependabot = (ROOT / ".github/dependabot.yml").read_text()
        snyk = (workflows / "snyk.yml").read_text()
        self.assertIn("snyk sbom test --experimental --file=snyk-sbom.json", snyk)
        self.assertNotIn("--package-manager=cargo", snyk)
        for name in RUST_ENGINES.values():
            with self.subTest(engine=name):
                self.assertIn(f"uses: ./.github/workflows/ci-rust.yml\n    with:\n      package: {name}\n", caller)
                self.assertFalse((workflows / f"release-{name}.yml").exists())
                self.assertIn(f"/packages/{name}", dependabot)
                self.assertIn(name, snyk.split("matrix:")[1].split("steps:")[0])
