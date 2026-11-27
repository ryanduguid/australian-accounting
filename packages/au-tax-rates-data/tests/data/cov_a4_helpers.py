"""CPython oracle for rates.rs's Python-semantics helpers. Writes the case file
`tests/data/cov_a4_helpers.json` to stdout: run with CPython 3.15.0,
`python3.15 -I cov_a4_helpers.py > cov_a4_helpers.json`."""

import decimal
import json
import sys

assert sys.version.split()[0] == "3.15.0", sys.version


def answer(fn):
    try:
        return fn()
    except Exception as e:  # the case records what Python raised
        return f"!{type(e).__name__}: {e}"


COMPARE = [
    "[1, 2.5]",
    "[3, 3.0]",
    "[3, 2.5]",
    "[3, 3.5]",
    "[-3, -3.5]",
    "[-4, -3.5]",
    "[2, NaN]",
    "[NaN, 2]",
    "[NaN, 1.0]",
    "[2, Infinity]",
    "[2, -Infinity]",
    "[Infinity, 2]",
    "[-Infinity, 2]",
    "[1.5, 2.5]",
    "[2.5, 2.5]",
    "[100000000000000000000, 1e20]",
    "[100000000000000000001, 1e20]",
    "[true, 2]",
    "[false, 0.0]",
    '["a", "b"]',
    '["b", "a"]',
    '["a", "a"]',
    "[[1, 2], [1, 3]]",
    "[[1, 3], [1, 2]]",
    "[[1], [1, 2]]",
    "[[1, 2], [1]]",
    "[[1], [1]]",
    '[[1, "a"], [1, 2]]',
    "[null, 1]",
    "[{}, 1]",
    '["a", 1]',
    "[[1], 1]",
]
SETS = [
    "[3, 1, 2, 1]",
    '["b", "a", "c", "a"]',
    "[1, 1.0, true, 0, false, 2.5]",
    "[2.5, -1, 100000000000000000000, 0.5]",
    "[[1], 2]",
    "[{}]",
]
# Set orders differ between CPython and the port, so only the class is checked.
SETS_CLASS = ['[1, "a"]', "[null, 1]", '["a", null, "b"]']
TUPLES = ['["x", "y"]', '["x", 1]', '["x", null]', '["x", [1]]', '["x", {}]', '[[1], "x"]']
DECIMALS = [
    "[0, [1, 2], -1]",
    "[1, [1, 2], -1]",
    "[0, [], 0]",
    "[1, [], 5]",
    "[true, [7], 0]",
    "[2, [1], 0]",
    '["0", [1], 0]',
    "[-1, [1], 0]",
    "[99999999999999999999, [1], 0]",
    '[0, [1], "F"]',
    '[1, [], "F"]',
    '[0, [1, 2], "n"]',
    '[0, [3], "N"]',
    '[1, [], "n"]',
    '[0, [1], "x"]',
    "[0, [1], 1.5]",
    "[0, [1], null]",
    "[0, [1], 99999999999999999999]",
    "[0, [1], 1000000000000000000]",
    "[0, [10], 0]",
    '[0, ["1"], 0]',
    "[0, [-1], 0]",
    "[0, [99999999999999999999], 0]",
    '[0, "12", 0]',
    "[0, [1]]",
    "[]",
    "[0, [1], 0, 0]",
    "null",
    "{}",
    '"abc"',
    '"1.50"',
    "true",
    "false",
    "1.5",
    "0.1",
    "12345678901234567890",
]
BYTES = [
    "22ff22",
    "efbbbf22ff22",
    "22eda08022",
    "22eda080ff22",
    "22c322",
    "2261e282",
    "efbbbf22eda080c022",
    "22eda0",
    "22ed9f",
]
WIDE = ["2200610022", "fffe220061", "feff002200", "0000feff000000", "00000022000000"]


def dec(v):
    if isinstance(v, list):
        v = tuple(tuple(x) if isinstance(x, list) else x for x in v)
    return decimal.Decimal(v)


def loads_bytes(h):
    try:
        json.loads(bytes.fromhex(h))
    except UnicodeDecodeError as e:
        return str(e)
    return None


cases = {
    "lt": [[c, answer(lambda c=c: repr(json.loads(c)[0] < json.loads(c)[1]))] for c in COMPARE],
    "le": [[c, answer(lambda c=c: repr(json.loads(c)[0] <= json.loads(c)[1]))] for c in COMPARE],
    "sorted_set": [[c, answer(lambda c=c: repr(sorted(set(json.loads(c)))))] for c in SETS],
    "sorted_set_class": [
        [c, answer(lambda c=c: repr(sorted(set(json.loads(c))))).split(":")[0]] for c in SETS_CLASS
    ],
    "tuple_key": [[c, answer(lambda c=c: repr(len({tuple(json.loads(c))})))] for c in TUPLES],
    "decimal": [[c, answer(lambda c=c: str(dec(json.loads(c))))] for c in DECIMALS],
    "bytes": [[h, loads_bytes(h)] for h in BYTES if loads_bytes(h) is not None],
    "decodes": [h for h in BYTES if loads_bytes(h) is None],
    "wide": [[h, loads_bytes(h)] for h in WIDE],
}
json.dump(
    {"python": sys.version.split()[0], "cases": cases}, sys.stdout, indent=1, ensure_ascii=True
)
print()
