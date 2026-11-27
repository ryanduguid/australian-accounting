"""Print what CPython 3.15's `decimal` answers for the cases in `tests/cov_b_decimal.rs`.

Run with CPython 3.15: `python -I tests/data/cov_b_decimal.py`. The default context traps
InvalidOperation, DivisionByZero and Overflow; a trapped signal prints as the exception's name.
"""

from decimal import Decimal as D, ROUND_DOWN, ROUND_HALF_EVEN

CASES = [
    ("add NaN 1", lambda: D("NaN") + D("1")),
    ("add 1 NaN7", lambda: D("1") + D("-NaN7")),
    ("add sNaN 1", lambda: D("sNaN") + D("1")),
    ("add 1 sNaN", lambda: D("1") + D("sNaN")),
    ("add Inf -Inf", lambda: D("Infinity") + D("-Infinity")),
    ("add Inf Inf", lambda: D("Infinity") + D("Infinity")),
    ("add -Inf 5", lambda: D("-Infinity") + D("5")),
    ("add 5 -Inf", lambda: D("5") + D("-Infinity")),
    ("add 2.50 -2.5", lambda: D("2.50") + D("-2.5")),
    ("mul NaN 2", lambda: D("NaN") * D("2")),
    ("mul Inf 0", lambda: D("Infinity") * D("0")),
    ("mul 0 -Inf", lambda: D("0") * D("-Infinity")),
    ("mul -Inf 2", lambda: D("-Infinity") * D("2")),
    ("mul 3 -Inf", lambda: D("3") * D("-Infinity")),
    ("div NaN 2", lambda: D("NaN") / D("2")),
    ("div Inf Inf", lambda: D("Infinity") / D("-Infinity")),
    ("div -Inf 2", lambda: D("-Infinity") / D("2")),
    ("div 2 -Inf", lambda: D("2") / D("-Infinity")),
    ("normalize NaN12", lambda: D("NaN12").normalize()),
    ("normalize sNaN", lambda: D("sNaN").normalize()),
    ("normalize -Inf", lambda: D("-Infinity").normalize()),
    ("normalize -0.000", lambda: D("-0.000").normalize()),
    ("integral NaN", lambda: D("NaN").to_integral_value()),
    ("integral Inf", lambda: D("Infinity").to_integral_value()),
    ("integral down -Inf", lambda: D("-Infinity").to_integral_value(rounding=ROUND_DOWN)),
    ("long NaN payload", lambda: D("NaN" + "1" + "0" * 5 + "9" * 25) + D("1")),
    ("adjusted Inf", lambda: D("Infinity").adjusted()),
    ("float nan", lambda: D(float("nan"))),
    ("float -nan", lambda: D(-float("nan"))),
    ("float inf", lambda: D(float("inf"))),
    ("float -inf", lambda: D(float("-inf"))),
    ("float -0.0", lambda: D(-0.0)),
    ("float 0.1", lambda: D(0.1)),
    ("float 3.0", lambda: D(3.0)),
    ("float 2**60", lambda: D(2.0**60)),
    ("float 5e-324", lambda: D(5e-324)),
    ("fixed Inf", lambda: f"{D('-Infinity'):f}"),
    ("fixed NaN", lambda: f"{D('NaN'):f}"),
]

for name, case in CASES:
    try:
        print(f"{name}: {case()}")
    except ArithmeticError as error:
        print(f"{name}: {type(error).__name__}")
print("round", ROUND_HALF_EVEN)
