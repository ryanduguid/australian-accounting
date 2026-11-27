"""CPython's answers for the replies `https::tests` feeds `exchange`.

Each reply is read as `rates ato-tables` reads one: `HTTPResponse.begin()`,
then `read()` for a 200. Prints one line per case: the status (not 200), the
body as Latin-1, or the `str()` of the exception. Run with `python -I`.
"""

import http.client
import io


class Sock:
    def __init__(self, data):
        self.data = data

    def makefile(self, mode):
        return io.BytesIO(self.data)


def answer(reply):
    response = http.client.HTTPResponse(Sock(reply), method="GET")
    try:
        response.begin()
        if response.status != 200:
            return f"status {response.status}"
        return "body " + response.read().decode("latin-1")
    except Exception as e:
        return f"error {e}"


OK = b"HTTP/1.1 200 OK\r\n"
CHUNKED = OK + b"Transfer-Encoding: chunked\r\n\r\n"
LONG = 65537

CASES = {
    "short_body": OK + b"Content-Length: 5\r\n\r\nab",
    "blank_status": b"\r\n",
    "version_only": b"HTTP/1.1\r\n\r\n",
    "no_reason": b"HTTP/1.1 200\r\nContent-Length: 2\r\n\r\n{}",
    "long_status": b"HTTP/1.1 200 " + b"x" * LONG,
    "not_http": b"ICY 200 OK\r\n\r\n",
    "long_header": OK + b"X: " + b"y" * LONG,
    "many_headers": OK + b"X: y\r\n" * 101 + b"\r\n{}",
    "continue": b"HTTP/1.1 100 Continue\r\nX: y\r\n\r\n" + OK + b"Content-Length: 2\r\n\r\n{}",
    "endless_continue": b"HTTP/1.1 100 Continue\r\n\r\n" * 101,
    "http2": b"HTTP/2.0 200 OK\r\n\r\n",
    "no_content": b"HTTP/1.1 204 No Content\r\nContent-Length: 9\r\n\r\n",
    "bare_lf": b"HTTP/1.1 200 OK\nContent-Length: 2\n\n{}JUNK",
    "lone_cr": OK + b"Content-Length: 2\rX: y\r\n\r\n{}JUNK",
    "header_at_eof": OK + b"Content-Length: 2",
    "folded": OK + b"Content-Length:\r\n 2\r\n\r\n{}JUNK",
    "leading_fold": OK + b" X: y\r\nContent-Length: 2\r\n\r\n{}JUNK",
    "from_line": OK + b"From nobody\r\n:x\r\nContent-Length: 2\r\n\r\n{}JUNK",
    "chunks": CHUNKED + b"2\r\n{}\r\n1;ext=1\r\n \r\n0\r\nX: y\r\n\r\n",
    "long_chunk_size": CHUNKED + b"1" * LONG,
    "bad_chunk_size": CHUNKED + b"2\r\n{}\r\nzz\r\n",
    "non_ascii_chunk_size": CHUNKED + b"\xe9\r\n",
    "separator_chunk_size": CHUNKED + b"2\x1c\r\n{}\r\n0\r\n\r\n",
    "negative_chunk_size": CHUNKED + b"-2\r\n{}\r\n0\r\n\r\n",
    "short_chunk_end": CHUNKED + b"2\r\n{}\r",
    "short_chunk": CHUNKED + b"2\r\n{}\r\n5\r\nab",
    "huge_chunk": CHUNKED + b"f" * 20 + b"\r\nab",
    "long_trailer": CHUNKED + b"0\r\nX: " + b"y" * LONG,
    "many_trailers": CHUNKED + b"0\r\n" + b"X: y\r\n" * 101 + b"\r\n",
}

for name, reply in CASES.items():
    print(f"{name}: {answer(reply)!r}")
