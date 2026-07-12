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

- [ ] Artifact-only projects without a populated event store still report the pre-release upgrade error before generated-artifact drift errors.
- [ ] The latest commit on `origin/main` has a successful GitHub Actions CI run.

## Subtasks

## Notes / Log
