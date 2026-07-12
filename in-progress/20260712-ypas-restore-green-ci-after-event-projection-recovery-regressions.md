---
title: Restore green CI after event projection recovery regressions
blocked_by: []
blocks: []
tags: [ci, stop-the-line, event-projection]
pr_mr_url: 
pr_mr_status: 
claim:
  host: unknown
  session: unknown
---

## Summary

Repair the stop-the-line main-branch failures exposed by the full CI suite after event-log projection recovery changes.

## Context / Why

GitHub Actions run 29195216187 passed lint, release versioning, and Nix flake checks but failed tests::check_rebuilds_clean_review_records_from_exported_events because `emc check` returned `No such file or directory (os error 2)` after generated artifacts and reviews were removed. Preserve the explicit legacy artifact-only upgrade error while restoring recovery from a populated event store.

## Acceptance criteria

- [x] Artifact-only projects without a populated event store still report the pre-release upgrade error before generated-artifact drift errors.
- [ ] The latest commit on `origin/main` has a successful GitHub Actions CI run.
- [x] `emc sync` rebuilds missing project, generated-model, and review artifacts from a populated event store, after which `emc check` succeeds without writing.

## Subtasks

## Notes / Log

- 2026-07-12: Root cause confirmed by local nextest RED and file-level syscall trace: stale `event_log_export` recovery tests still invoke validation-only `emc check` after deleting projections. Commit a8fb178 intentionally moved recovery to explicit `emc sync`; the repair must update behavioral recovery scenarios without making `check` write again.
- 2026-07-12: Final local repair now updates stale recovery scenarios to invoke explicit `emc sync`, proves `emc check` succeeds without rewriting a read-only generated slice, and checks a comprehensive manifest/model/review rebuild after sync. Focused event-log suite passes 47/47; full `just ci` passes 546/546 with XDG state/cache redirected to writable `/tmp` paths for sandbox-local Quint semaphore state.
- 2026-07-12: Exact final local gate on diff 5968c09cf6276595db922e808d47162a967a6b4f passed: `env XDG_STATE_HOME=/tmp/emc-xdg-state XDG_CACHE_HOME=/tmp/emc-xdg-cache nix develop --command just ci` completed with 546/546 tests passing in 456.109s. Focused event-log suite previously passed 47/47, and the three no-write/upgrade-precedence scenarios passed individually.
- 2026-07-12: Signed commit ae99fc71984ea812f649e3425186cfb8d193ed63 pushed directly to `origin/main` at 2026-07-12 10:46:02 PDT. Per the established ~21-minute full CI duration and rate-limit preference, do not query the run before 11:07:02 PDT. The remote-CI acceptance criterion remains pending and the task is intentionally `in-progress`.
- 2026-07-12: First permitted GitHub query at 2026-07-12 11:07 PDT found CI run 29202612564 for ae99fc71984ea812f649e3425186cfb8d193ed63 still queued. To allow a full late-start runtime without polling, do not query it again before 11:28:38 PDT.
- 2026-07-12: Successful run 29202612564 generated signed release commit 09291e2401ef305b68ddff552236937e4ef06902 (`chore(release): v0.2.0`). GitHub did not auto-trigger CI for that GITHUB_TOKEN-authored commit, so workflow_dispatch recovery run 29204000444 was started against current `main` at 11:30:31 PDT. Do not query it before 11:51:31 PDT.
