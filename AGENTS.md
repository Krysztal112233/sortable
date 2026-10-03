This project permits LLM-assisted programming, but both LLMs and humans must follow the guidelines below.

## Coding

Code should follow idiomatic Rust style. Do not introduce non-idiomatic constructs.

For unit and integration tests, covering each case once is enough — do not introduce a large number of tests just to prove that a function is correct. Keep tests concise and effective, and always exercise restraint.

## Commits

Commit messages must be concise, and each commit should be as small as possible.

Commit message format: `<type>(scope): (msg)`

- `type` is the kind of work in the commit; it may be `refactor`, `feat`, or `fix`
- `scope` is usually the corresponding module name, such as the existing `row`, `table`, `column`

Unless additional explanation is required, do not write a description.

If a single commit seems to require a large number of changes, it usually means you have misunderstood the requirement — prompt your operator to rethink it.

Exceptions:

- Mechanical refactors
- Documentation rewrites
