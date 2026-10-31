"""Rust owns its checks, command and release format independently of Python."""
import runpy
import tempfile
import tomllib
import unittest
from pathlib import Path

from test_boundaries import DATASETS, ROOT, RUST_APPLICATIONS, RUST_ENGINES, select_package

rust_crates = runpy.run_path(str(ROOT / ".github/ci/rust_crates.py"))

# The Python import package each port removed.
REMOVED_IMPORTS = {
    "the-exchequer-tally": "edwinnixon",
    "solomons-sword": "louisgoldberg",
    "the-wip-tally": "wiptally",
    "australian-tax-calculators": "austaxcalc",
    "div7a-loan-review": "div7aloan",
    "ato-benchmark-compare": "atobenchmark",
    "payday-super-checker": "paydaysuper",
    "gst-review": "gstreview",
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
        # ci-rust.yml packages <crate>-<version>-<target>/<bin> as .tgz, or .zip on
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
        self.assertIn('tar -C dist -czf "dist/$base.tgz" "$base"', workflow)
        for component in [*RUST_ENGINES, "apps/au-accounting"]:
            with self.subTest(component=component):
                manifest = tomllib.loads((ROOT / component / "Cargo.toml").read_text())
                package = manifest["package"]
                self.assertEqual(package["repository"], "https://github.com/ryanduguid/australian-accounting")
                self.assertEqual(package["metadata"]["binstall"], expected)
                binary = manifest.get("bin", [{"name": package["name"]}])[0]["name"]
                for target in ("x86_64-unknown-linux-gnu", "aarch64-apple-darwin", "x86_64-pc-windows-msvc"):
                    metadata = package["metadata"]["binstall"]
                    fmt = metadata.get("overrides", {}).get(target, {}).get("pkg-fmt", metadata["pkg-fmt"])
                    suffix = {"tgz": ".tgz", "zip": ".zip"}[fmt]
                    extension = ".exe" if target.endswith("windows-msvc") else ""
                    url, binary_path = metadata["pkg-url"], metadata["bin-dir"]
                    values = {"repo": package["repository"], "name": package["name"],
                              "version": package["version"], "target": target,
                              "archive-suffix": suffix, "bin": binary, "binary-ext": extension}
                    for key, value in values.items():
                        url = url.replace("{ " + key + " }", value)
                        binary_path = binary_path.replace("{ " + key + " }", value)
                    base = f"{package['name']}-{package['version']}-{target}"
                    self.assertEqual(url, f"{package['repository']}/releases/download/{package['name']}/v{package['version']}/{base}{suffix}")
                    self.assertEqual(binary_path, f"{base}/{binary}{extension}")

    def test_umbrella_links_every_engine_by_path(self):
        cargo = tomllib.loads((ROOT / "apps/au-accounting/Cargo.toml").read_text())
        engines = {name: dep for name, dep in cargo["dependencies"].items()
                   if name not in {"serde", "serde_json"}}
        self.assertEqual(engines, {name: {"path": f"../../{component}"} for component, name in RUST_ENGINES.items()})
        source = (ROOT / "apps/au-accounting/src/main.rs").read_text()
        for name in RUST_ENGINES.values():
            self.assertIn(name.replace("-", "_") + "::", source)
        # CI compares au-accounting with exactly the engines it links.
        self.assertEqual(rust_crates["linked"](), sorted(RUST_ENGINES.values()))

    def test_cargo_ci_security_and_publisher_boundary(self):
        workflows = ROOT / ".github/workflows"
        caller = (workflows / "ci.yml").read_text()
        self.assertIn("needs: [changes, packages, crates, rust, umbrella-parity, test, dependency-audit, lint]", caller)
        workflow = (workflows / "ci-rust.yml").read_text()
        for gate in ("cargo fmt --check", "cargo clippy --locked --all-targets -- -D warnings", "cargo test --locked", "cargo build --locked --release", "cargo audit --deny warnings", "cargo llvm-cov --locked --fail-under-lines"):
            self.assertIn(gate, workflow)
        self.assertIn("os: [ubuntu-latest, windows-latest, macos-latest]", workflow)
        self.assertNotIn("contents: write", workflow)
        self.assertNotIn("id-token: write", workflow)
        self.assertIn("language: rust", (workflows / "codeql.yml").read_text())
        snyk = (workflows / "snyk.yml").read_text()
        self.assertIn("snyk sbom test --experimental --file=snyk-sbom.json", snyk)
        self.assertNotIn("--package-manager=cargo", snyk)
        for name in RUST_ENGINES.values():
            with self.subTest(engine=name):
                self.assertFalse((workflows / f"release-{name}.yml").exists())

    def test_crate_lists_follow_the_manifests(self):
        # The Cargo.toml files are the one list of Rust crates. CI and the justfile
        # derive theirs; the lists that cannot be derived must match them.
        crates = rust_crates["crates"]()
        directories = {f"{crate['root']}/{crate['package']}" for crate in crates}
        self.assertEqual(directories, set(RUST_ENGINES) | set(DATASETS) | set(RUST_APPLICATIONS) | {"apps/au-accounting"})
        root = tomllib.loads((ROOT / "pyproject.toml").read_text())
        exclude = root["tool"]["uv"]["workspace"]["exclude"]
        self.assertEqual(len(exclude), len(set(exclude)))
        self.assertEqual(set(exclude), directories)
        # Dependabot's directories key takes globs, but GitHub does not document
        # what a glob matching a directory without a Cargo.toml does, so the list
        # stays explicit: every crate, every crate's fuzz harness, every crate's
        # WebAssembly wrapper and every tool crate.
        fuzz = {manifest.parent.relative_to(ROOT).as_posix()
                for manifest in [*ROOT.glob("packages/*/fuzz/Cargo.toml"), *ROOT.glob("apps/*/fuzz/Cargo.toml")]}
        wasm = {manifest.parent.relative_to(ROOT).as_posix() for manifest in ROOT.glob("packages/*/wasm/Cargo.toml")}
        tools = {manifest.parent.relative_to(ROOT).as_posix() for manifest in ROOT.glob("tools/*/Cargo.toml")}
        cargo = dependabot_directories("cargo", (ROOT / ".github/dependabot.yml").read_text())
        self.assertEqual(len(cargo), len(set(cargo)))
        self.assertEqual(set(cargo), {f"/{d}" for d in directories | fuzz | wasm | tools})
        workflows = ROOT / ".github/workflows"
        matrix = "matrix: ${{ fromJSON(needs.crates.outputs.matrix) }}"
        caller = (workflows / "ci.yml").read_text()
        snyk = (workflows / "snyk.yml").read_text()
        self.assertIn("matrix: ${{ fromJSON(needs.crates.outputs.ci-matrix) }}", caller)
        self.assertIn(matrix, snyk)
        # A crate with its own caller workflow is checked there, not again in ci.yml.
        own = {crate["package"] for crate in crates} - {crate["package"] for crate in rust_crates["crates"](ci_yml=True)}
        self.assertEqual(own, {"lodgeit-calculator-adapter"})
        self.assertIn("package: lodgeit-calculator-adapter\n", (workflows / "ci-lodgeit-adapter.yml").read_text())
        self.assertIn('pattern: "${{ needs.crates.outputs.parity }}-${{ matrix.os }}"', caller)
        for crate in crates:
            with self.subTest(crate=crate["package"]):
                self.assertNotIn(f"package: {crate['package']}\n", caller)
                self.assertNotIn(crate["package"], snyk)
        by_name = {crate["package"]: crate for crate in crates}
        self.assertEqual(by_name["au-tax-rates-data"], {"package": "au-tax-rates-data", "root": "packages", "binary": "rates", "smoke": "validate"})
        self.assertEqual(by_name["au-accounting"], {"package": "au-accounting", "root": "apps", "binary": "au-accounting", "smoke": "--version"})
        self.assertEqual(by_name["the-wip-tally"]["binary"], "wip-tally")
        self.assertEqual(by_name["gst-review"]["smoke"], "--api-help")
        self.assertEqual(by_name["lodgeit-calculator-adapter"], {"package": "lodgeit-calculator-adapter", "root": "apps", "binary": "lodgeit-adapter", "smoke": "contract"})

    def test_crate_lister_rejects_what_ci_rust_cannot_check(self):
        engine = '[package]\nname = "engine"\n'
        cases = {
            "renamed": ('[package]\nname = "other"\n', ["src/main.rs"]),
            "two commands": (engine + '[[bin]]\nname = "a"\n[[bin]]\nname = "b"\n', ["src/main.rs"]),
            "an implicit second command": (engine, ["src/main.rs", "src/bin/extra.rs"]),
            "no command": (engine, ["src/lib.rs"]),
            "a chosen command it does not build": (engine + '[package.metadata.ci]\nbinary = "c"\n[[bin]]\nname = "a"\n[[bin]]\nname = "b"\n', ["src/main.rs"]),
        }
        for case, (manifest, sources) in cases.items():
            with self.subTest(case=case), tempfile.TemporaryDirectory() as scratch:
                crate = Path(scratch) / "packages/engine"
                for source in sources:
                    (crate / source).parent.mkdir(parents=True, exist_ok=True)
                    (crate / source).write_text("", encoding="utf-8")
                (crate / "Cargo.toml").write_text(manifest, encoding="utf-8")
                with self.assertRaises(SystemExit):
                    rust_crates["crates"](Path(scratch))
        with tempfile.TemporaryDirectory() as scratch, self.assertRaises(SystemExit):
            rust_crates["crates"](Path(scratch))

    def test_crate_lister_takes_the_chosen_command_of_several(self):
        with tempfile.TemporaryDirectory() as scratch:
            crate = Path(scratch) / "packages/engine"
            (crate / "src/bin").mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "engine"\n[package.metadata.ci]\nbinary = "a"\n'
                '[[bin]]\nname = "a"\npath = "src/main.rs"\n[[bin]]\nname = "b"\npath = "src/bin/b.rs"\n',
                encoding="utf-8")
            self.assertEqual(rust_crates["crates"](Path(scratch))[0]["binary"], "a")

    def test_parity_names_a_renamed_dependency_by_its_crate(self):
        with tempfile.TemporaryDirectory() as scratch:
            umbrella = Path(scratch) / "apps/au-accounting"
            umbrella.mkdir(parents=True)
            (umbrella / "Cargo.toml").write_text(
                '[package]\nname = "au-accounting"\n[dependencies]\n'
                'serde_json = "1"\ngst = { path = "../../packages/gst-review", package = "gst-review" }\n'
                'wip = { path = "../../packages/the-wip-tally" }\n', encoding="utf-8")
            self.assertEqual(rust_crates["linked"](Path(scratch)), ["gst-review", "wip"])

    def test_dependabot_reader_sees_every_directory(self):
        base = ("version: 2\nupdates:\n  - package-ecosystem: cargo\n    directories:\n"
                "      - /packages/a\n      - /packages/b\n    schedule:\n      interval: weekly\n")
        self.assertEqual(dependabot_directories("cargo", base), ["/packages/a", "/packages/b"])
        hidden = {
            "after a comment": base.replace("      - /packages/b\n", "      - /packages/b\n      # more\n      - /packages/c\n"),
            "with a trailing comment": base.replace("/packages/b\n", "/packages/b  # note\n      - /packages/c\n"),
            "in a second entry": base + "  - package-ecosystem: cargo\n    directory: /packages/c\n",
        }
        for case, text in hidden.items():
            with self.subTest(case=case):
                self.assertIn("/packages/c", dependabot_directories("cargo", text))
                self.assertNotIn("/packages/b  # note", dependabot_directories("cargo", text))
        unsupported = {
            "flow style": base.replace('    directories:\n      - /packages/a\n      - /packages/b\n', '    directories: ["/packages/a"]\n'),
            "nested": base.replace("      - /packages/b\n", "      - /packages/b\n        - /packages/c\n"),
        }
        for case, text in unsupported.items():
            with self.subTest(case=case), self.assertRaises(ValueError):
                dependabot_directories("cargo", text)


def dependabot_directories(ecosystem: str, text: str) -> list[str]:
    """Every directory the dependabot.yml entries for ecosystem name, without a YAML parser.

    Blank and comment lines are skipped; a directories list in any other shape fails.
    """
    directories = []
    for entry in text.split("\n  - ")[1:]:
        lines = entry.splitlines()
        if lines[0].strip() != f"package-ecosystem: {ecosystem}":
            continue
        listing = False
        for line in lines[1:]:
            value = "" if line.strip().startswith("#") else line.split(" #", 1)[0].strip()
            if not value:
                continue
            if value.startswith("directory:"):
                directories.append(value.removeprefix("directory:").strip())
                listing = False
            elif value.startswith("directories:"):
                if value != "directories:":
                    raise ValueError(f"unsupported directories value: {line!r}")
                listing = True
            elif listing and line.startswith("      - "):
                directories.append(value.removeprefix("- ").strip())
            elif listing and line.startswith("      "):
                raise ValueError(f"unsupported directories line: {line!r}")
            else:
                listing = False
    return directories
