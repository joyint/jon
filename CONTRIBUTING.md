# Contributing

## Coding Conventions

- rustfmt defaults; `cargo clippy` must be clean (crates carry
  `#![deny(clippy::all)]`).
- Shared, non-interactive logic lives in `jon-core`; `jon-cli` wraps it
  with prompts and terminal output (the same split joy-core's
  `ai_setup` uses). No domain logic in the binaries.
- English for all artifacts: code, comments, docs, item content, commit
  messages.

## Commit Messages

- Conventional commits: `type(scope): summary [JON-XXXX-XX]`, e.g.
  `feat(pda): render the session prompt [JON-0007-9F]`.
- Every commit references the Joy item it belongs to.
- AI-member commits end with the trailers required by the Joy
  integration (`Co-Authored-By:` with the tool brand and
  `Delegated-By:` with the operator).

## Testing

- `cargo test` runs unit tests beside the code and the `jon init` e2e
  coverage in `crates/jon-cli/tests/`.
- While the PDA iterates, coverage is deliberately targeted: the
  five-outputs contract of the session template, the bootstrap happy
  path, and the CLI branches (empty dir, existing Joy project, unknown
  tool). Extend tests where behavior is decided, not speculatively.

## CI/CD

- No pipeline yet. Before pushing: `cargo test` and `cargo clippy`
  locally. Release packaging follows once the MVP settles.

## Branching Strategy

- `main` is protected; work happens on feature branches
  (`feat/<topic>`, e.g. `feat/jon-pda`) merged via pull request.
- Joy items track the work; branches reference their epic or story in
  the PR description.
