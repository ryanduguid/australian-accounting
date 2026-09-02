<!--
Keep the change single-purpose. A pull request that fixes a bug and reformats
the file is two pull requests.
-->

## What this changes

<!-- One or two sentences. What behaviour is different after this merges? -->

## Why

<!--
The problem being fixed. Link the issue if there is one.
If this changes a computed number, a rate, a threshold or a deadline, cite the
primary source here: section of the Act, legislative instrument, ruling, or
AASB paragraph. A changed number without a citation will not be merged.
-->

## Verification

<!--
The exact commands you ran, and their result. Not "tests pass" - the commands.
A green suite on its own is not evidence when the defect being fixed was not
covered before: say which new test fails on the base commit and passes here.
-->

```
```

## Checklist

- [ ] No client, taxpayer, employee, payroll or job-cost data, and no credentials, tokens, tenant IDs or `.env` contents, appear in the diff, the commit messages or this description.
- [ ] The change is single-purpose.
- [ ] New or changed behaviour has a test that fails without the change.
- [ ] Documentation, README and any docstring that describes the changed rule were updated together with it.
- [ ] Any new dependency is justified in the description, or there is none.
