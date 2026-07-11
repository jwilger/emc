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

Tiber is the authoritative accepted-work backlog. GitHub Issues are public
triage inputs and release-notification records; do not begin implementation
until an issue is associated with an accepted Tiber task. Work one major Tiber
task at a time, and move it to `in-progress` before changing code. Do not begin
another task until the current task is closed and its local worktree is cleaned
up.

For long-running architectural goals, keep the main agent focused on
integration, review, CI, and cleanup. Use subagents only for bounded work that
can run in parallel without blocking the next local step: specific codebase
scouting questions, disjoint implementation slices with explicit file ownership,
or focused verification. Do not use subagents by default. First prefer
deterministic shell facts such as `rg`, `git diff`, generated artifact
inspection, and focused test output. Use a subagent only when the delegated task
is likely to consume fewer main-thread tokens than local inspection, usually no
more than one scout subagent per increment. Prefer the smallest available capable
model for scout-style subagents, and do not use subagents for overlapping
shared-type refactors, broad architecture surveys, or work that requires
immediate main-thread decisions.

For token efficiency on long-running goals, keep a short progress ledger in the
repository when the work spans multiple PRs. The ledger should record completed
increments, current PR boundaries, remaining typed-domain/string-boundary
targets, focused verification already run, and the next likely increment. Use
the ledger to avoid rediscovering repository state at the start of each
increment. When resuming such a goal, first read these instructions, read the
ledger, and check `git status`; inspect only the files needed for the next
increment unless the ledger is stale or contradicted by the worktree.

Prefer deterministic scripts, including Codex hooks, only when they replace
repeated agent inspection with cheap mechanical checks or concise state recall.
Useful Codex hooks are narrow and fast: session-start reminders that point to a
progress ledger, pre-tool guardrails that reject known-dangerous commands, or
focused validation of generated artifacts. Avoid Codex hooks that emit large
context, run broad repository scans, run tests on every prompt/tool call, update
the worktree automatically, or duplicate Forgejo CI. Do not add hooks that
enforce maintainer guidance by matching strings in documentation or source
files.

Direct signed trunk development is the normal workflow for write collaborators.
Create each increment with `just worktree-create <name>` from the primary
checkout and do all changes in that linked worktree. The primary checkout guard
blocks commits and pushes. `just worktree-remove <name>` removes the linked
checkout after the task is complete; delete its merged branch separately.

Run focused local gates before committing. Use a signed conventional commit and
include `Closes: <tiber-task-ref>` when the commit completes that task. Push a
green increment immediately from its linked worktree with `git push origin
HEAD:main`. Monitor the resulting trunk CI run. A failed trunk run is
stop-the-line: inspect it, make the smallest repair in a new linked worktree,
and do not start other work until trunk is green again.

Contributors without write access use maintainer-approved external pull requests.
Keep external PRs squash-only and delete their merged branches. The installed
Codex plugins for engineering standards, development discipline, worktrees,
Tiber, advisor, and PR babysitting are the repository's preferred workflow
tools; use the relevant plugin when its task matches.

Keep `docs/event-model/formal-modeling-rules.md` status markers current when
changing the formal metamodel, generators, or verification workflow. Use `✅`
only for rules mechanically enforced all the way through Lean4/Quint artifacts
and verification, `🟡` for partial enforcement, and `❌` for rules that are not
currently mechanically enforced.
