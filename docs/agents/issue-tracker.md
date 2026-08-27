# Issue tracker

GitHub Issues in `ryanduguid/aus-accounting-mcp` are the repository's work-request
surface. Use the `gh` CLI for issue discovery and, when explicitly authorised,
for GitHub changes.

## Read work

Use `gh issue list`, `gh issue view <number>`, and `gh pr view <number>` to
establish the current request, its discussion and linked pull requests. Read
the repository instructions and relevant domain terms before proposing work.

## Pull requests as a triage surface

**PRs as a request surface: no.** Pull requests are review and integration
artefacts, not a source of new requirements. Treat an issue, an explicitly
supplied specification, or the maintainer's current request as the authority
for scope.

## Change work

Before creating an issue, adding or removing labels, posting a comment,
opening a pull request, or closing an issue, show the proposed remote change
and obtain explicit approval. Keep each issue to one independently reviewable
outcome, with acceptance criteria and any safety boundaries it needs.

Never publish a suspected vulnerability in an issue or pull request; follow
[`SECURITY.md`](../../SECURITY.md). Never include private tax records, client
files, identifiers, bank details or other sensitive personal information;
follow [`DISCLAIMER.md`](../../DISCLAIMER.md).

When a skill says to publish to the issue tracker, create a GitHub issue. When
it says to fetch the relevant ticket, use `gh issue view <number> --comments`
and include its labels.

## Wayfinding operations

The map is one issue labelled `wayfinder:map`, with child issues as tickets.

- Create a map with `gh issue create --label wayfinder:map`.
- Link child tickets through GitHub sub-issues. If sub-issues are unavailable,
  add them to a task list in the map and put `Part of #<map>` at the top of each
  child. Use `wayfinder:research`, `wayfinder:prototype`, `wayfinder:grilling`
  or `wayfinder:task` as the child type.
- Represent blocking through GitHub issue dependencies. If dependencies are
  unavailable, put `Blocked by: #<number>` at the top of the child.
- The frontier is the first open, unassigned child in map order with no open
  blocker. Claim it with `gh issue edit <number> --add-assignee @me`.
- Resolve a child by commenting with the answer, closing it, and adding its
  durable context pointer to the map's decisions.
