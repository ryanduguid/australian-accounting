"""Print what CPython 3.15's `decimal` answers for the cases in `tests/cov_b_decimal.rs`.

Run with CPython 3.15: `python -I tests/data/cov_b_decimal.py`. The default context traps
InvalidOperation, DivisionByZero and Overflow; a trapped signal prints as the exception's name.
"""

from decimal import ROUND_DOWN
from decimal import Decimal as D

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
    ("space inside", lambda: D("1 0")),
    ("empty exponent", lambda: D("1E")),
    ("long exponent", lambda: D("1E" + "1" * 30)),
    ("zero-led exponent", lambda: D("1E" + "0" * 30 + "5")),
    ("div 1E-40 3", lambda: D("1E-40") / D("3")),
    ("div 2 3E+40", lambda: D("2") / D("3E+40")),
    ("div 1 7", lambda: D("1") / D("7")),
    ("add 1 -2.5", lambda: D("1") + D("-2.5")),
    ("add -1 2.5", lambda: D("-1") + D("2.5")),
    ("overflow", lambda: D("9E+999999") * D("10")),
    ("integral 1E+2", lambda: D("1E+2").to_integral_value()),
    ("integral 2.000", lambda: D("2.000").to_integral_value()),
    ("integral down 2.00", lambda: D("2.00").to_integral_value(rounding=ROUND_DOWN)),
    ("integral down 2.50", lambda: D("2.50").to_integral_value(rounding=ROUND_DOWN)),
    ("fixed 0E+3", lambda: f"{D('0E+3'):f}"),
    ("fixed -0E+3", lambda: f"{D('-0E+3'):f}"),
    ("compare -Inf Inf", lambda: D("-Infinity").compare(D("Infinity"))),
    ("compare Inf 5", lambda: D("Infinity").compare(D("5"))),
    ("compare -1 2", lambda: D("-1").compare(D("2"))),
    ("compare 1 -2", lambda: D("1").compare(D("-2"))),
    ("compare -1 -2", lambda: D("-1").compare(D("-2"))),
    ("compare -2 -1", lambda: D("-2").compare(D("-1"))),
    ("parse 1E+", lambda: D("1E+")),
    ("parse 1E1x", lambda: D("1E1x")),
    ("div 40 ones by 1", lambda: D("1" * 40) / D("1")),
    ("div 40 ones by 3", lambda: D("1" * 40) / D("3")),
    ("add 29 nines E+999971 0", lambda: D("9" * 29 + "E+999971") + D("0")),
    ("add 29 nines E+999970 0", lambda: D("9" * 29 + "E+999970") + D("0")),
]

for name, case in CASES:
    try:
        print(f"{name}: {case()}")
    except ArithmeticError as error:
        print(f"{name}: {type(error).__name__}")
