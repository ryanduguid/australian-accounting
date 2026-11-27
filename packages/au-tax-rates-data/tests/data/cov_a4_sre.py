"""CPython oracle for sre.rs: prints the `CASES` table of `src/cov_a4_sre.rs`
(run with CPython 3.15.0, `python3.15 -I cov_a4_sre.py`). Patterns of 200
characters or more are printed as comments; the test builds them.

Each case is `(pattern, flags, subject)`. The answer is `str(re.error)`, or
`Class: message` for another exception from `re.compile`; then, with no
subject, `groups N`; otherwise `re.search`'s span of every group (`a-b`, `-`
for a group that took no part) or `no match`."""

import re
import sys

assert sys.version.split()[0] == "3.15.0", sys.version

IC, M, A, U = re.IGNORECASE, re.MULTILINE, re.ASCII, re.UNICODE
CASES = [
    # Error positions on the first of several lines.
    ["a(\nb", 0, None],
    ["(\nb", 0, None],
    # Negated categories inside a negated set.
    ["[^\\D]", 0, "a1"],
    ["[^\\s]", 0, " a"],
    ["[^\\S]", 0, "a b"],
    ["[^\\w]", 0, "a-"],
    ["[^\\W]", 0, "-a"],
    # Alternatives that share a first item.
    [".a|.b", 0, "xb"],
    ["[xy]a|[xy]b", 0, "yb"],
    ["\\ba|\\bb", 0, " b"],
    ["(a)(?:\\1b|\\1c)", 0, "aac"],
    ["(?!)a|(?!)b", 0, "b"],
    # Group names.
    ["(?P<", 0, None],
    ["(?P<ab", 0, None],
    ["(?P=", 0, None],
    ["(?P=ab", 0, None],
    ["(?P<a-b>x)", 0, None],
    ["(?P<x>(?P=x))", 0, None],
    ["(?<=(?P<x>a)(?P=x))b", 0, None],
    ["(?Px)", 0, None],
    ["(a\\1)", 0, None],
    # Conditional groups.
    ["(a)?(?<=(?(1)b|c))d", 0, "cd"],
    ["(a)?(?<=(?(1)b))d", 0, "bd"],
    ["(?P<n>a)?(?(n)b|c)", 0, "ab"],
    ["(?P<n>a)?(?(n)b|c)", 0, "c"],
    ["(?(5)a)", 0, None],
    ["(?(01)a)", 0, None],
    ["(a)(?(1)b", 0, None],
    # Escapes.
    ["\\a\\f\\n\\r\\t\\v\\\\", 0, "\a\f\n\r\t\v\\"],
    ["[\\a\\b\\f\\n\\r\\t\\v\\\\]+", 0, "\b\a\\"],
    ["\\D+", 0, "12ab"],
    ["\\U0001F600", 0, "x\U0001f600"],
    ["\\U0011FFFF", 0, None],
    ["\\U1234", 0, None],
    ["[\\x41]", 0, "A"],
    ["[\\101]", 0, "A"],
    ["[\\0]", 0, "\0"],
    ["[\\8]", 0, None],
    ["[\\q]", 0, None],
    ["[\\U0001F600]", 0, "\U0001f600"],
    ["[\\u0041]", 0, "A"],
    ["[\\N{LATIN SMALL LETTER A}]", 0, "a"],
    ["[\\N{NO SUCH NAME}]", 0, None],
    ["\\N{NO SUCH NAME}", 0, None],
    ["\\q", 0, None],
    ["\\0", 0, "\0"],
    ["\\01", 0, "\x01"],
    ["\\x41", 0, "A"],
    ["(a)\\10", 0, None],
    ["\\u0041", 0, "A"],
    ["a{" + "1" * 5000 + "}", 0, None],
    # Nesting.
    ["(" * 500 + ")" * 500, 0, None],
    ["(?:" * 500 + ")" * 500, 0, None],
    ["[a]" + "(?:" * 479 + "a" + ")" * 479, 0, "aa"],
    # Alternatives of single characters become a set.
    ["a|b|[cd]", 0, "d"],
    ["[^a]|b", 0, "a"],
    ["x|[^a]", 0, "a"],
    # Ranges and literals.
    ["[a-]", 0, "-"],
    ["[\\w-]", 0, "-"],
    ["[a-\\x7a]", 0, "q"],
    ["a]", 0, "a]"],
    ["(?x)a\\ b", 0, "a b"],
    ["a{", 0, "a{"],
    ["a{,", 0, "a{,"],
    ["a{x}", 0, "a{x}"],
    ["a{1,99999999999}", 0, None],
    ["^*", 0, None],
    ["(?:)*", 0, "a"],
    ["(?!)", 0, "a"],
    ["a(?!)", 0, "a"],
    # Inline flags.
    ["(?i", 0, None],
    ["(?z)", 0, None],
    ["(?i!", 0, None],
    ["(?-)", 0, None],
    ["(?-:a)", 0, None],
    ["(?-a:x)", 0, None],
    ["(?-u:x)", 0, None],
    ["(?-i", 0, None],
    ["(?-i!)", 0, None],
    ["(?-iz:a)", 0, None],
    ["(?i-:a)", 0, None],
    ["(?a-i:x)", 0, None],
    ["(?-x:a)", 0, "a"],
    ["(?i-x)", 0, None],
    ["(?x-i)", 0, None],
    ["a(?a:b)", 0, None],
    # ASCII categories and case.
    ["\\d+", A, "\u0661\u06621"],
    ["\\s+", A, "\u2003 \t"],
    ["\\D+", A, "\u0661a1"],
    ["\\S+", A, "\u2003a "],
    ["[^\\d]+", A, "a1"],
    ["k", IC | A, "K\u212a"],
    ["\u212a", IC | A, "k\u212a"],
    ["[^k]+", IC | A, "Kx"],
    ["[k-m]+", IC | A, "KLM"],
    ["\\w+", IC | A, "\u00e9a"],
    ["K", IC, "\u212a"],
    ["(?u)K", IC, "\u212a"],
    ["a", A | U, None],
    # Case-insensitive sets.
    ["[k]", IC, "\u212a"],
    ["[a-c]x", IC, "BX"],
    ["[\u017f-\u017f]", IC, "S"],
    ["[k-k]", IC, "\u212a"],
    ["[\u0130]", IC, "i"],
    ["[\u0130-\u0131]", IC, "I"],
    ["[^\u017f]", IC, "s"],
    ["[a\u212a-\u212b]", IC, "\u00e5"],
    ["[0-9k]", IC, "K"],
    # Look-behind width.
    ["(?<=(?:a{100000}){100000})b", 0, None],
    # Line anchors.
    ["a$", M, "a\nb"],
    ["a$", M, "ab"],
    ["^b", M, "a\nb"],
    ["\\Aa\\Z", 0, "a"],
    # Repeats.
    ["a{2,3}+a", 0, "aaaa"],
    ["(?:ab)*+", 0, "ababa"],
    ["a{2}+", 0, "aaa"],
    ["a{3}+", 0, "aa"],
    ["(a|ab){1,2}+c", 0, "abc"],
    ["(?:a|b)*+b", 0, "ab"],
    ["(a)*+", 0, "aa"],
    ["(?:)*+x", 0, "x"],
    ["a{2,3}?", 0, "aaa"],
    ["(?:a|b){2,}?c", 0, "abac"],
    ["(?:)*?x", 0, "x"],
    ["(a?){2,3}?b", 0, "ab"],
    ["(?:a{0,2}?){3}b", 0, "aab"],
    ["a{1,2}?b", 0, "ab"],
    ["a{2,}?", 0, "a"],
    ["(?:a*)*?b", 0, "aab"],
]

out = []
for pattern, flags, subject in CASES:
    try:
        p = re.compile(pattern, flags)
    except re.error as e:
        out.append([pattern, flags, subject, str(e)])
        continue
    except (OverflowError, RecursionError, ValueError) as e:
        out.append([pattern, flags, subject, f"{type(e).__name__}: {e}"])
        continue
    if subject is None:
        answer = f"groups {p.groups}"
    else:
        m = p.search(subject)
        if m is None:
            answer = "no match"
        else:
            spans = (m.span(i) for i in range(p.groups + 1))
            answer = " ".join("-" if s == (-1, -1) else f"{s[0]}-{s[1]}" for s in spans)
    out.append([pattern, flags, subject, answer])


def rust(text):
    """A Rust string literal."""
    named = {
        '"': '\\"',
        "\\": "\\\\",
        "\n": "\\n",
        "\t": "\\t",
        "\r": "\\r",
        "\0": "\\0",
    }
    body = "".join(named.get(c) or (c if " " <= c <= "~" else f"\\u{{{ord(c):x}}}") for c in text)
    return f'"{body}"'


LONG = 200
print("const CASES: &[(&str, u32, Option<&str>, &str)] = &[")
for pattern, flags, subject, answer in out:
    if len(pattern) < LONG:
        found = "None" if subject is None else f"Some({rust(subject)})"
        print(f"    ({rust(pattern)}, {flags}, {found}, {rust(answer)}),")
print("];")
for pattern, flags, subject, answer in out:
    if len(pattern) >= LONG:
        print(f"// long: {pattern[:12]!r}... {len(pattern)} chars, {subject!r}: {answer}")
