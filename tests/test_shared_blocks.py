"""Pin the code that several engines each keep a copy of.

Engines are released on their own and never import a sibling, and the root may
not hold a shared runtime library, so these helpers stay copied. A copy nobody
checks drifts: a fix made in one engine's `atomic_io.py` and not the other's is
invisible in review, because neither diff shows the sibling. This is where that
drift fails instead.

`IDENTICAL` names definitions that are the same bytes in every copy. `SAME_LOGIC`
names definitions whose docstrings or comments have diverged, so only the
executable code is compared. The first file in each group is the reference.

Standard library only, and no component is imported: the files are read and
parsed.
"""

from __future__ import annotations

import ast
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGES = ROOT / "packages"

GROUPS = {
    "atomic_io.py": {
        "files": (
            PACKAGES / "ato-benchmark-compare" / "atobenchmark" / "atomic_io.py",
            PACKAGES / "the-wip-tally" / "wiptally" / "atomic_io.py",
        ),
        "identical": ("atomic_text_writer", "atomic_write_text"),
        "same_logic": (),
    },
    "money.py": {
        "files": (
            PACKAGES / "ato-benchmark-compare" / "atobenchmark" / "money.py",
            PACKAGES / "the-wip-tally" / "wiptally" / "money.py",
        ),
        "identical": ("CENTS", "_NUMBER", "_ACCOUNTING_NUMBER"),
        "same_logic": ("parse_amount",),
    },
    "tools/build_workbook.py": {
        "files": (
            PACKAGES / "div7a-loan-review" / "tools" / "build_workbook.py",
            PACKAGES / "payday-super-checker" / "tools" / "build_workbook.py",
            PACKAGES / "ato-benchmark-compare" / "tools" / "build_workbook.py",
        ),
        "identical": ("header", "style"),
        "same_logic": ("recalc",),
    },
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


if __name__ == "__main__":  # pragma: no cover
    unittest.main()
