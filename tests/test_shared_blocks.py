# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Pin the code that several engines each keep a copy of.

Engines are released on their own and never import a sibling, and the root may
not hold a shared runtime library, so these helpers stay copied. A copy nobody
checks drifts: a fix made in one engine's `atomic_io.py` and not the other's is
invisible in review, because neither diff shows the sibling. This is where that
drift fails instead.

`IDENTICAL` names definitions that are the same bytes in every copy. `SAME_LOGIC`
names definitions whose docstrings or comments have diverged, so only the
executable code is compared. The first file in each group is the reference.

`RUST_COPIES` pins the Rust workbook tooling the div7a and ato-benchmark
builders copied from the payday builder: whole files that differ only by
declared substitutions, the model's code outside its tests, and the staging
build's functions. It also pins the functions the payday and div7a engines
each keep of `verify` and the WebAssembly `call`. Those checks read rustfmt's
layout, in which a top-level item and its closing brace start at column 0; each
package's CI runs `cargo fmt --check`, which refuses a file laid out otherwise.

`CSVREAD_COPIES` pins the div7a and ato-benchmark copies of payday's CSV reader
to declared substitutions, so payday's fuzz target covers the code they share.

`PYTHON_COPIES` pins the Python-compatibility modules every Rust crate copies:
each file belongs to a family of byte-identical files, and a new copy has to
join one. The Unicode tables are pinned harder: a family with fewer tables keeps
each one as `packages/au-tax-rates-data/src/pyunicode.rs` has it, and that file
is what `tools/gen_pyunicode.py` beside it generates.

Standard library only, and no component is imported: the files are read and
parsed, and the generator runs as a script.
"""

from __future__ import annotations

import ast
import re
import subprocess
import sys
import unicodedata
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGES = ROOT / "packages"

# No Python definition is copied across components at present: the workbook
# builders' helpers left with the payday and div7a Python builders.
GROUPS: dict[str, dict] = {}

PAYDAY_TOOLS = PACKAGES / "payday-super-checker" / "examples" / "build_workbook"
RUST_COPIES = {
    # Each copy's directory, then file: (payday text, copy text) pairs; every
    # other byte is the same.
    "copies": {
        PACKAGES / "div7a-loan-review" / "examples" / "build_workbook": {
            "excel.rs": (("PAYDAY_WORKBOOK", "DIV7A_WORKBOOK"),),
            "recalc.ps1": (
                ("PAYDAY_WORKBOOK", "DIV7A_WORKBOOK"),
                ("PaydayBuild", "Div7aBuild"),
                ("Review Checks B16", "Review Checks B10"),
                ("Range('B16')", "Range('B10')"),
            ),
        },
        PACKAGES / "ato-benchmark-compare" / "examples" / "build_workbook": {
            "excel.rs": (("PAYDAY_WORKBOOK", "ATO_WORKBOOK"),),
            "recalc.ps1": (
                ("PAYDAY_WORKBOOK", "ATO_WORKBOOK"),
                ("PaydayBuild", "AtoBuild"),
                ("Review Checks B16", "Review Checks B13"),
                ("Range('B16')", "Range('B13')"),
            ),
        },
    },
    # The model's tests exercise each builder's own features.
    "code_before_tests": ("model.rs",),
    "functions": {"main.rs": ("build", "fail", "replace_part")},
    # Functions the payday and div7a engines each keep, by path in the engine.
    "engine_functions": {
        "src/derivation.rs": ("first_field", "pick", "difference", "first_difference", "verify"),
        "wasm/src/lib.rs": ("call",),
    },
}

# The CSV reader payday's fuzz target runs, as the div7a and ato-benchmark
# engines copy it: (payday text, copy text) pairs, applied in order; every other
# byte is the same. Neither copy has the strict mode.
CSVREAD_WITHOUT_STRICT = (
    ("    strict: bool,\n", ""),
    ("            strict: false,\n", ""),
    (
        "    /// `csv.reader(..., strict=True)`.\n"
        "    pub fn strict(data: &'a [u8]) -> Self {\n"
        "        Reader {\n"
        "            strict: true,\n"
        "            ..Reader::new(data)\n"
        "        }\n"
        "    }\n"
        "\n",
        "",
    ),
    (
        "                Input::Char(_) if self.strict => {\n"
        "                    return Err(ReadError::Csv(\"',' expected after '\\\"'\".into()));\n"
        "                }\n",
        "",
    ),
    (
        "                    if self.strict {\n"
        '                        return Err(ReadError::Csv("unexpected end of data".into()));\n'
        "                    }\n",
        "",
    ),
)
CSVREAD_COPIES = {
    "ato-benchmark-compare": CSVREAD_WITHOUT_STRICT,
    # div7a's also reads a list of lines, which no fuzz target runs.
    "div7a-loan-review": CSVREAD_WITHOUT_STRICT
    + (
        (
            "    text: TextReader<'a>,\n",
            "    text: TextReader<'a>,\n"
            "    /// `csv.reader(list_of_lines)`: each item is one line with no terminator.\n"
            "    lines: Option<std::collections::VecDeque<Vec<char>>>,\n",
        ),
        (
            "            text: TextReader::new(data),\n",
            "            text: TextReader::new(data),\n            lines: None,\n",
        ),
        (
            "    fn save_field(&mut self) {\n",
            "    /// `csv.reader` over a list of strings, such as `text.splitlines()` gives.\n"
            "    pub fn from_lines(lines: &[&str]) -> Reader<'static> {\n"
            "        Reader {\n"
            "            text: TextReader::new(&[]),\n"
            "            lines: Some(lines.iter().map(|l| l.chars().collect()).collect()),\n"
            "            state: State::StartRecord,\n"
            "            field: Vec::new(),\n"
            "            fields: Vec::new(),\n"
            "        }\n"
            "    }\n"
            "\n"
            "    fn save_field(&mut self) {\n",
        ),
        (
            "            let Some(line) = self.text.readline()? else {\n",
            "            let next = match self.lines.as_mut() {\n"
            "                Some(lines) => lines.pop_front(),\n"
            "                None => self.text.readline()?,\n"
            "            };\n"
            "            let Some(line) = next else {\n",
        ),
    ),
}


# The Python-compatibility modules, by file: each family lists the crates whose
# copies are the same bytes. The first family of pyunicode.rs has every table.
RATES_DATA = "packages/au-tax-rates-data"
LODGEIT_ADAPTER = "apps/lodgeit-calculator-adapter"
ATO_BENCHMARK = "packages/ato-benchmark-compare"
DIV7A = "packages/div7a-loan-review"
PAYDAY = "packages/payday-super-checker"
WIP_TALLY = "packages/the-wip-tally"
PYTHON_COPIES = {
    "pyunicode.rs": (
        (RATES_DATA, LODGEIT_ADAPTER),
        (ATO_BENCHMARK, DIV7A, PAYDAY, WIP_TALLY),
    ),
    "py.rs": (
        (RATES_DATA, LODGEIT_ADAPTER),
        (ATO_BENCHMARK, DIV7A),
        (PAYDAY,),
        (WIP_TALLY,),
    ),
    "pyjson.rs": (
        (ATO_BENCHMARK, DIV7A, PAYDAY, WIP_TALLY),
        (RATES_DATA,),
        (LODGEIT_ADAPTER,),
    ),
}
UNICODE_GENERATOR = ROOT / RATES_DATA / "tools" / "gen_pyunicode.py"
EXACT_UNICODE = (
    sys.implementation.name == "cpython"
    and sys.version_info[:3] == (3, 15, 0)
    and unicodedata.unidata_version == "17.0.0"
)


def _definitions(path: Path) -> dict[str, str]:
    """Every top-level definition and simple assignment, by name, as written."""
    return _definitions_in(path.read_text(encoding="utf-8"))


def _definitions_in(text: str) -> dict[str, str]:
    lines = text.splitlines(keepends=True)
    found: dict[str, str] = {}
    for node in ast.parse(text).body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            # A decorator such as @contextmanager changes behaviour, so the
            # compared source starts at the first decorator, not at `def`.
            start = min([node.lineno, *(d.lineno for d in node.decorator_list)])
            found[node.name] = "".join(lines[start - 1 : node.end_lineno])
            continue
        if (
            isinstance(node, ast.Assign)
            and len(node.targets) == 1
            and isinstance(node.targets[0], ast.Name)
        ):
            name = node.targets[0].id
        elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
            name = node.target.id
        else:
            continue
        segment = ast.get_source_segment(text, node)
        if segment is not None:
            found[name] = segment
    return found


def _executable(source: str) -> str:
    """The code alone: comments dropped by unparse, docstrings removed."""
    tree = ast.parse(source)
    for node in ast.walk(tree):
        if not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            continue
        first = node.body[0]
        if (
            len(node.body) > 1
            and isinstance(first, ast.Expr)
            and isinstance(first.value, ast.Constant)
            and isinstance(first.value.value, str)
        ):
            node.body = node.body[1:]
    return ast.unparse(tree)


class SharedBlockTests(unittest.TestCase):
    def test_a_decorator_is_part_of_the_compared_definition(self) -> None:
        plain = "def writer():\n    yield\n"
        decorated = "@contextmanager\n" + plain
        self.assertNotEqual(
            _definitions_in(plain)["writer"], _definitions_in(decorated)["writer"]
        )
        self.assertNotEqual(
            _executable(_definitions_in(plain)["writer"]),
            _executable(_definitions_in(decorated)["writer"]),
        )
    def test_every_pinned_name_is_still_defined_in_every_copy(self) -> None:
        """A renamed or deleted helper must fail here, not quietly stop being compared."""
        for group, spec in GROUPS.items():
            pinned = set(spec["identical"]) | set(spec["same_logic"])
            for path in spec["files"]:
                with self.subTest(group=group, file=str(path.relative_to(ROOT))):
                    self.assertLessEqual(pinned, set(_definitions(path)))

    def test_the_shared_definitions_are_byte_identical(self) -> None:
        for group, spec in GROUPS.items():
            reference, *copies = [_definitions(path) for path in spec["files"]]
            for copy, path in zip(copies, spec["files"][1:]):
                for name in spec["identical"]:
                    with self.subTest(group=group, name=name, file=str(path.relative_to(ROOT))):
                        self.assertEqual(reference[name], copy[name])

    def test_the_shared_code_is_identical_where_only_the_prose_differs(self) -> None:
        for group, spec in GROUPS.items():
            reference, *copies = [_definitions(path) for path in spec["files"]]
            for copy, path in zip(copies, spec["files"][1:]):
                for name in spec["same_logic"]:
                    with self.subTest(group=group, name=name, file=str(path.relative_to(ROOT))):
                        self.assertEqual(_executable(reference[name]), _executable(copy[name]))


def _rust_outer_attributes(before: str) -> str:
    """The attributes above a top-level item. Comments and blank lines are not compared."""
    lines = before.splitlines(keepends=True)
    found: list[str] = []
    while lines:
        line = lines.pop()
        if not line.strip() or line.startswith("//"):
            continue
        if line.startswith(")]"):
            # rustfmt closes a multi-line attribute at column 0.
            found.append(line)
            while lines and not found[-1].startswith("#["):
                found.append(lines.pop())
        elif line.startswith("#["):
            found.append(line)
        else:
            break
    return "".join(reversed(found))


def _rust_function(text: str, name: str) -> str:
    """A top-level `fn` item: its outer attributes, signature and body to its closing brace."""
    match = re.search(rf"^(?:pub )?fn {name}\b.*?^}}\n", text, re.M | re.S)
    assert match is not None, name
    # A second match, such as a commented-out original, could hide a changed live definition.
    assert len(re.findall(rf"^(?:pub )?fn {name}\b", text, re.M)) == 1, name
    # An attribute such as `#[cfg(...)]` or `#[export_name = "..."]` changes behaviour.
    return _rust_outer_attributes(text[: match.start()]) + match.group(0)


class RustFunctionTests(unittest.TestCase):
    def test_a_commented_out_copy_cannot_stand_in_for_the_live_function(self) -> None:
        with self.assertRaises(AssertionError):
            _rust_function("/*\nfn verify() {\n}\n*/\nfn verify() {\n    changed();\n}\n", "verify")


class RustCopyTests(unittest.TestCase):
    def read(self, directory: Path, name: str) -> str:
        return (directory / name).read_text(encoding="utf-8")

    def test_substituted_files_differ_only_by_their_declared_substitutions(self) -> None:
        for directory, files in RUST_COPIES["copies"].items():
            copy = str(directory.relative_to(PACKAGES))
            for name, pairs in files.items():
                reference = self.read(PAYDAY_TOOLS, name)
                for old, new in pairs:
                    with self.subTest(copy=copy, file=name, text=old):
                        # A renamed literal must fail here, not quietly stop matching.
                        self.assertIn(old, reference)
                    reference = reference.replace(old, new)
                with self.subTest(copy=copy, file=name):
                    self.assertEqual(reference, self.read(directory, name))

    def test_the_model_code_is_identical_outside_its_tests(self) -> None:
        for name in RUST_COPIES["code_before_tests"]:
            reference = self.read(PAYDAY_TOOLS, name).split("#[cfg(test)]")
            with self.subTest(file=name):
                self.assertEqual(len(reference), 2)
            for directory in RUST_COPIES["copies"]:
                copy = self.read(directory, name).split("#[cfg(test)]")
                with self.subTest(file=str((directory / name).relative_to(ROOT))):
                    self.assertEqual(len(copy), 2)
                    self.assertEqual(reference[0], copy[0])
                for part, path in ((reference, PAYDAY_TOOLS), (copy, directory)):
                    with self.subTest(file=str((path / name).relative_to(ROOT))):
                        # The test module comes last, so no code escapes the comparison:
                        # nothing follows its closing brace, indented or not.
                        top_level = [line for line in part[-1].splitlines() if line[:1].strip()]
                        self.assertEqual(top_level, ["mod tests {", "}"])
                        self.assertTrue(part[-1].rstrip().endswith("\n}"))

    def test_the_staging_build_functions_are_identical(self) -> None:
        for name, functions in RUST_COPIES["functions"].items():
            reference = self.read(PAYDAY_TOOLS, name)
            for directory in RUST_COPIES["copies"]:
                copy = self.read(directory, name)
                for function in functions:
                    with self.subTest(copy=str(directory.relative_to(PACKAGES)), function=function):
                        self.assertEqual(
                            _rust_function(reference, function), _rust_function(copy, function)
                        )

    def test_a_function_includes_only_its_own_outer_attributes(self) -> None:
        for attributes in ("#[inline]\n", "#[cfg(\n    any()\n)]\n#[inline]\n"):
            with self.subTest(attributes=attributes):
                text = f"fn a() {{\n}}\n\n/// Doc.\n{attributes}/// Doc.\n\npub fn b() {{\n}}\n"
                self.assertEqual(_rust_function(text, "b"), f"{attributes}pub fn b() {{\n}}\n")
        # The previous item's attributes are not this item's.
        self.assertEqual(_rust_function("#[inline]\nfn a() {\n}\nfn b() {\n}\n", "b"), "fn b() {\n}\n")

    def test_the_engines_verify_and_wasm_call_functions_are_identical(self) -> None:
        for name, functions in RUST_COPIES["engine_functions"].items():
            reference = self.read(PACKAGES / "payday-super-checker", name)
            copy = self.read(PACKAGES / "div7a-loan-review", name)
            for function in functions:
                with self.subTest(file=name, function=function):
                    self.assertEqual(
                        _rust_function(reference, function), _rust_function(copy, function)
                    )

    def test_the_csv_reader_copies_differ_only_by_their_declared_substitutions(self) -> None:
        reference = self.read(PACKAGES / "payday-super-checker" / "src", "csvread.rs")
        for engine, pairs in CSVREAD_COPIES.items():
            expected = reference
            for old, new in pairs:
                with self.subTest(engine=engine, text=old):
                    self.assertIn(old, expected)
                expected = expected.replace(old, new)
            with self.subTest(engine=engine):
                self.assertEqual(expected, self.read(PACKAGES / engine / "src", "csvread.rs"))

    def test_the_engines_no_network_harness_is_identical(self) -> None:
        marker = "// Everything below is the same in every engine;"
        copies = {path.parent.parent.name: path.read_text(encoding="utf-8").split(marker)[1]
                  for path in PACKAGES.glob("*/tests/no_network.rs")}
        self.assertEqual(len(copies), 8)
        self.assertEqual(len(set(copies.values())), 1, sorted(copies))

    def test_a_function_ends_at_its_own_closing_brace(self) -> None:
        text = "fn a() {\n    x\n}\n\npub fn b() {\n}\n"
        self.assertEqual(_rust_function(text, "a"), "fn a() {\n    x\n}\n")
        self.assertEqual(_rust_function(text, "b"), "pub fn b() {\n}\n")
        # A longer name sharing the prefix is a different function.
        self.assertEqual(_rust_function("fn ab() {\n}\nfn a() {\n}\n", "a"), "fn a() {\n}\n")


def _rust_tables(text: str) -> dict[str, str]:
    """Each `pub static` item, by name, with comments and layout removed.

    rustfmt alone decides the layout, which the generator's output has not had.
    """
    code = re.sub(r"^\s*//.*\n", "", text, flags=re.M)
    return {
        match.group(1): "".join(match.group(0).split()).replace(",]", "]")
        for match in re.finditer(r"^pub static (\w+):.*?\];\n", code, re.M | re.S)
    }


class PythonCopyTests(unittest.TestCase):
    def test_every_copy_belongs_to_a_family(self) -> None:
        for name, families in PYTHON_COPIES.items():
            listed = [crate for family in families for crate in family]
            found = [path.parents[1].relative_to(ROOT).as_posix()
                     for path in (*PACKAGES.glob(f"*/src/{name}"), *ROOT.glob(f"apps/*/src/{name}"))]
            with self.subTest(name=name):
                self.assertEqual(len(listed), len(set(listed)))
                self.assertEqual(set(listed), set(found))

    def test_a_family_is_the_same_bytes(self) -> None:
        for name, families in PYTHON_COPIES.items():
            for family in families:
                contents = {crate: (ROOT / crate / "src" / name).read_bytes() for crate in family}
                with self.subTest(name=name, family=family):
                    self.assertEqual(len(set(contents.values())), 1, sorted(contents))

    def test_a_family_with_fewer_tables_keeps_them_as_the_full_file_has_them(self) -> None:
        full, *others = PYTHON_COPIES["pyunicode.rs"]
        tables = _rust_tables((ROOT / full[0] / "src" / "pyunicode.rs").read_text(encoding="utf-8"))
        for family in others:
            copy = _rust_tables((ROOT / family[0] / "src" / "pyunicode.rs").read_text(encoding="utf-8"))
            self.assertTrue(copy)
            for table, body in copy.items():
                with self.subTest(family=family, table=table):
                    self.assertEqual(body, tables.get(table))

    @unittest.skipUnless(EXACT_UNICODE, "the tables record CPython 3.15.0's Unicode 17.0.0")
    def test_the_generator_reproduces_the_saved_unicode_tables(self) -> None:
        generated = subprocess.run(
            [sys.executable, "-I", str(UNICODE_GENERATOR)], capture_output=True, check=True
        ).stdout.decode("utf-8")
        saved = _rust_tables((ROOT / RATES_DATA / "src" / "pyunicode.rs").read_text(encoding="utf-8"))
        tables = _rust_tables(generated)
        # WORD_DIFF also depends on the regex crate, so the generator leaves it out.
        self.assertEqual(set(saved) - set(tables), {"WORD_DIFF"})
        for table, body in tables.items():
            with self.subTest(table=table):
                self.assertEqual(body, saved[table])

    def test_a_table_name_ends_at_its_own_closing_bracket(self) -> None:
        text = "// Header.\npub static A: &[u32] = &[\n    1,\n];\n\n/// Doc.\npub static B: &[u32] = &[2];\n"
        self.assertEqual(_rust_tables(text), {"A": "pubstaticA:&[u32]=&[1];", "B": "pubstaticB:&[u32]=&[2];"})


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
