# Index separate entity reviews

`python examples/group_review.py /approved/local/path/manifest.json` prints a JSON
index containing each source report in full. It reads local files only and does
not call an engine or a service. Keep real workpapers outside a repository.

Use schema `group-review-input.v1` with a non-empty `reports` list. Each report
has exactly these text fields: `id`, `entity`, `period`, `engine`,
`engine_version`, `scope`, `path` and `sha256`. The path names a UTF-8 `.txt` or
`.json` report under the manifest directory. SHA-256 binds the exact bytes.
Missing files, duplicate IDs, changed digests and paths outside that directory
stop the index. The example accepts files up to 2 MB each.

Run company, trust and Division 7A checks separately using their own commands
and supported facts. Save stdout, retain exit codes in the accompanying work
record, and supply the manifest only after reviewing the entity and period
labels. These labels remain assertions; the index does not validate them.

A company calculation cannot clear a trust's missing facts or a loan engine's
refusal. Every warning, engine status, convention and boundary statement remains
in `source_output`. There is no aggregate pass, consolidated tax calculation,
elimination journal or cross-entity legal conclusion. Inspect the source files
with each engine's own verification facilities where provided.
