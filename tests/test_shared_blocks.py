"""Pin the code that several engines each keep a copy of.

Engines are released on their own and never import a sibling, and the root may
not hold a shared runtime library, so these helpers stay copied. A copy nobody
checks drifts: a fix made in one engine's `atomic_io.py` and not the other's is
invisible in review, because neither diff shows the sibling. This is where that
drift fails instead.

`IDENTICAL` names definitions that are the same bytes in every copy. `SAME_LOGIC`
names definitions whose docstrings or comments have diverged, so only the
executable code is compared. The first file in each group is the reference.

`RUST_COPIES` pins the Rust workbook tooling the div7a builder copied from the
payday builder: whole files that differ only by declared substitutions, the
model's code outside its tests, and the staging build's functions.

Standard library only, and no component is imported: the files are read and
parsed.
"""

from __future__ import annotations

import ast
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGES = ROOT / "packages"

# No Python definition is copied across components at present: the workbook
# builders' helpers left with the payday and div7a Python builders.
GROUPS: dict[str, dict] = {}

PAYDAY_TOOLS = PACKAGES / "payday-super-checker" / "examples" / "build_workbook"
RUST_COPIES = {
    "copy": PACKAGES / "div7a-loan-review" / "examples" / "build_workbook",
    # file: (payday text, div7a text) pairs; every other byte is the same.
    "substituted": {
        "excel.rs": (("PAYDAY_WORKBOOK", "DIV7A_WORKBOOK"),),
        "recalc.ps1": (
            ("PAYDAY_WORKBOOK", "DIV7A_WORKBOOK"),
            ("PaydayBuild", "Div7aBuild"),
            ("Review Checks B16", "Review Checks B10"),
            ("Range('B16')", "Range('B10')"),
            # The Python div7a builder ran Excel with macros force-disabled.
            ("$xl.AutomationSecurity = 1", "$xl.AutomationSecurity = 3"),
        ),
    },
    # The model's tests exercise each builder's own features.
    "code_before_tests": ("model.rs",),
    "functions": {"main.rs": ("build", "fail", "replace_part")},
}



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


def _rust_function(text: str, name: str) -> str:
    """A top-level `fn` item, from its signature to its closing brace."""
    match = re.search(rf"^(?:pub )?fn {name}\b.*?^}}\n", text, re.M | re.S)
    assert match is not None, name
    return match.group(0)


class RustCopyTests(unittest.TestCase):
    def read(self, directory: Path, name: str) -> str:
        return (directory / name).read_text(encoding="utf-8")

    def test_substituted_files_differ_only_by_their_declared_substitutions(self) -> None:
        for name, pairs in RUST_COPIES["substituted"].items():
            reference = self.read(PAYDAY_TOOLS, name)
            for old, new in pairs:
                with self.subTest(file=name, text=old):
                    # A renamed literal must fail here, not quietly stop matching.
                    self.assertIn(old, reference)
                reference = reference.replace(old, new)
            with self.subTest(file=name):
                self.assertEqual(reference, self.read(RUST_COPIES["copy"], name))

    def test_the_model_code_is_identical_outside_its_tests(self) -> None:
        for name in RUST_COPIES["code_before_tests"]:
            reference, copy = (
                self.read(d, name).split("#[cfg(test)]")
                for d in (PAYDAY_TOOLS, RUST_COPIES["copy"])
            )
            with self.subTest(file=name):
                self.assertEqual(len(reference), 2)
                self.assertEqual(reference[0], copy[0])
            for part, path in ((reference, PAYDAY_TOOLS), (copy, RUST_COPIES["copy"])):
                with self.subTest(file=str((path / name).relative_to(ROOT))):
                    # The test module comes last, so no code escapes the comparison.
                    top_level = [line for line in part[1].splitlines() if line[:1].strip()]
                    self.assertEqual(top_level, ["mod tests {", "}"])

    def test_the_staging_build_functions_are_identical(self) -> None:
        for name, functions in RUST_COPIES["functions"].items():
            reference = self.read(PAYDAY_TOOLS, name)
            copy = self.read(RUST_COPIES["copy"], name)
            for function in functions:
                with self.subTest(file=name, function=function):
                    self.assertEqual(
                        _rust_function(reference, function), _rust_function(copy, function)
                    )

    def test_a_function_ends_at_its_own_closing_brace(self) -> None:
        text = "fn a() {\n    x\n}\n\npub fn b() {\n}\n"
        self.assertEqual(_rust_function(text, "a"), "fn a() {\n    x\n}\n")
        self.assertEqual(_rust_function(text, "b"), "pub fn b() {\n}\n")
        # A longer name sharing the prefix is a different function.
        self.assertEqual(_rust_function("fn ab() {\n}\nfn a() {\n}\n", "a"), "fn a() {\n}\n")


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
