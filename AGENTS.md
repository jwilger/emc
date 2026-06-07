<!-- Copyright 2026 John Wilger -->

Tests must exercise behavior, not source-file text.

Do not add tautological tests that assert a file does or does not contain a
specific option, rule, command-line flag, or guidance string. Project rules
belong in documentation for agents and maintainers, not in tests that inspect
that documentation or unrelated source files.

It is acceptable for tests to inspect generated Lean4/Quint artifacts, command
output, logs, and other products of executing EMC behavior. It is not acceptable
for tests to enforce architecture, packaging, CI, or maintainer guidance by
reading repository source/config files and matching strings.

Primitive and structural types are permitted only at I/O boundaries. Within the
core domain, command, effect, runtime, projector, and verification logic, use
semantic data types instead of raw `String`, `Vec<String>`, `Option<String>`,
tuples used as records, or untyped structural payloads such as
`serde_json::Value`. Parse external input into semantic types immediately at the
boundary, keep those types through internal logic, and serialize primitives only
when writing external formats such as JSON event files, process arguments, file
contents, or command output.

Work one major change at a time. Do not start another major task while a PR for
the current task is still waiting on CI, review, approval, merge, or local
cleanup.

Before pushing to a PR branch, run focused local verification appropriate to
the files changed. Do not run local `just ci` solely as a pre-push ritual;
Forgejo CI is the full PR gate. If local `just ci` is run and returns green,
commit the current green state and push it.

The repository requires pull requests; do not push directly to `main`. Push a
feature branch, open a PR, and use conventional commits format for the PR title
and description just as for commit messages.

Do not start Forgejo PR bodies by repeating the conventional commit title. PR
bodies should start with useful sections such as Summary, Rationale, and
Verification.

After opening a PR, monitor CI and review feedback. If CI fails, inspect the
failing job logs, make the smallest appropriate fix, rerun relevant local
verification, commit, and push back to the same branch. Address all review
feedback from auto_review in the same way. All review comments must be handled
before merging, including non-blocking warnings on approved reviews. Do not
merge until CI is green, every review comment has been addressed or explicitly
resolved, and `@auto-review` has approved the PR.

Once approval is in place, merge the PR before starting any new task. After the
merge, clean up the merged local and remote feature branch, switch back to
`main`, and refresh local `main` from `origin/main`.

Keep `docs/event-model/formal-modeling-rules.md` status markers current when
changing the formal metamodel, generators, or verification workflow. Use `✅`
only for rules mechanically enforced all the way through Lean4/Quint artifacts
and verification, `🟡` for partial enforcement, and `❌` for rules that are not
currently mechanically enforced.
