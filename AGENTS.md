Tests must exercise behavior, not source-file text.

Do not add tautological tests that assert a file does or does not contain a
specific option, rule, command-line flag, or guidance string. Project rules
belong in documentation for agents and maintainers, not in tests that inspect
that documentation or unrelated source files.

It is acceptable for tests to inspect generated Lean4/Quint artifacts, command
output, logs, and other products of executing EMC behavior. It is not acceptable
for tests to enforce architecture, packaging, CI, or maintainer guidance by
reading repository source/config files and matching strings.
