# Python contract oracle

`python-contracts.json` records every engine call from the 77 behavioural pytest
cases at base `8526839cba47207becba39f2e7d8bdbec1b9809d`. The original tests ran
unchanged against the frozen Python engine and again through the native command.
Each row names its test and caller precision and contains the full expected
result or exact refusal message. The original Python packaging identity case is
replaced by the repository's Rust packaging, independence and CI boundary checks.

Adaptations: Python Decimal values become JSON strings and dates become canonical
ISO strings. Former Python string-date type failures are tagged objects so they
remain invalid under the new JSON date representation. Engine version `0.1.0`
becomes `1.0.0`. Dataclasses become objects and keyword arguments become fields.
The Rust tests verify every row, counts and label ordering and restoration of
the caller's Decimal precision. Data is entirely fabricated.

Frozen source SHA256:

| File | SHA256 |
| --- | --- |
| `gstreview/coding.py` | `c7fe99cf428da51c732b63a706d0aa74086daad4c2dcad926b505691c80d1830` |
| `gstreview/bas.py` | `51597261e47adb837af1afacadc0d93ae9e70a5ce34fcb8a2f77d3ce3c610297` |
| `tests/test_coding.py` | `45e83b0c9cb4c38b4974cd3d6fb22dc3385dc6ce6b79b5be682fd585cd437289` |
| `tests/test_bas.py` | `bdc35a8d87ebf6e713950b67e39b1da62d88bbcf90370bb0bc042857cb7e0da2` |

Decimal arithmetic and Unicode tables reuse the qualified Payday Super Checker
implementation at the same base. Only unused OS-string helpers were removed
from the local text support copy. No sibling engine is a runtime dependency.
