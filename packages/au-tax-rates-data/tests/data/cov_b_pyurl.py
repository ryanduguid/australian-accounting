"""Print what CPython 3.15's `urllib.parse.urlsplit` returns for the URLs in `tests/cov_b_pyurl.rs`.

Run with CPython 3.15: `python -I tests/data/cov_b_pyurl.py`. Each line is the URL's `repr`, then
`netloc=<repr>` or `ValueError: <message>`; the Rust test holds the same answers as literals.
"""

import urllib.parse

URLS = [
    "https://bücher.de/x",
    "https://ａ.com/x",
    "https://evil℀.com/x",
    "http://user@[::1]:80/x",
    "http://x[::1]/x",
    "http://[::1]x/x",
    "http://[v1.x]@host]/x",
    "http://[v1.fe]/x",
    "http://[V1.fe]/x",
    "http://[vz.x]/x",
    "http://[v1.]/x",
    "http://[1.2.3.4]/x",
    "http://[zz]/x",
    "http://[" + "1" * 46 + "]/x",
    "http://[1::2:]/x",
    "http://[1:2:3:4::5:6:7:8]/x",
    "http://[1:2:3:4:5:6:1.2.3.4]/x",
]

for url in URLS:
    try:
        print(repr(url), "netloc=" + repr(urllib.parse.urlsplit(url).netloc))
    except ValueError as error:
        print(repr(url), "ValueError:", error)
