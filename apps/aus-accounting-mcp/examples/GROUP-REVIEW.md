# Index separate entity reviews

`python examples/group_review.py /approved/local/path/manifest.json` prints a JSON
index containing each source report in full. It reads local files only and does
not call an engine or a service. Keep real workpapers outside a repository.

The unpublished native candidate provides the same local workflow:

```sh
cargo run --locked --example group_review -- /approved/local/path/manifest.json
```

Build a portable example executable with `cargo build --locked --release --example
group_review`. The executable under `target/release/examples/` needs neither
Python nor the checkout. It takes the same manifest argument and prints the
complete index on stdout, or exits 1 with a diagnostic and no index when input
validation fails. JSON formatting and parser diagnostics may differ. The Python
example remains supported.

In an extracted native CI candidate, run `./examples/group_review
/approved/local/path/manifest.json` from the archive root, or
`.\examples\group_review.exe C:\approved\local\path\manifest.json` on Windows.
The executable can also be addressed by its absolute path from another working
directory. Its manifest and reports stay outside the archive.

Use schema `group-review-input.v1` with a non-empty `reports` list. Each report
has exactly these text fields: `id`, `entity`, `period`, `engine`,
`engine_version`, `scope`, `path` and `sha256`. The path names a UTF-8 `.txt` or
`.json` report under the manifest directory. SHA-256 binds the exact bytes.
Missing files, duplicate IDs, changed digests and paths outside that directory
stop the index. The example accepts files up to 2 MB each.

Both examples accept UTF-8, UTF-16 and UTF-32 manifests with the byte encoding
detection used by Python's JSON loader. Reports remain strict UTF-8 with one
optional leading BOM; the digest includes that BOM. Duplicate decoded JSON keys
are refused. The native candidate also checks the bytes actually read against
the 2,000,000 byte limit, bounding reads if a file grows after its size check.
Lone surrogate JSON strings are refused by the native parser. Paths use the
host platform's rules; resolved report targets must remain under the resolved
manifest's directory and end in lowercase `.txt` or `.json`. On Windows the
native example refuses a report path naming a drive other than the manifest's,
such as `D:report.txt` beside a manifest on `C:`, before resolving it. The
Python example resolves such a path against that drive's current directory and
accepts it only if the result lies under the manifest's directory. As in the
Python example, this assumes local files stay stable during the operation. It
does not isolate concurrent filesystem changes by another process.

Run company, trust and Division 7A checks separately using their own commands
and supported facts. Save stdout, retain exit codes in the accompanying work
record, and supply the manifest only after reviewing the entity and period
labels. These labels remain assertions; the index does not validate them.

A company calculation cannot clear a trust's missing facts or a loan engine's
refusal. Every warning, engine status, convention and boundary statement remains
in `source_output`. There is no aggregate pass, consolidated tax calculation,
elimination journal or cross-entity legal conclusion. Inspect the source files
with each engine's own verification facilities where provided.
