---
title: Reduce CI test critical-path duration
blocked_by: []
blocks: []
tags: []
pr_mr_url: 
pr_mr_status: 
---

## Summary

Measure and reduce the CI test critical path without weakening coverage or allowing expensive checks to start before the short gate passes.

## Context / Why

The current trunk test suite takes roughly 20 minutes. Investigate safe parallelization, cache effectiveness, and test partitioning; preserve behavior coverage and the fast-first CI gate ordering.

## Acceptance criteria

- [ ] Benchmark the current critical path and identify the dominant test or setup costs.
- [ ] Implement a measured reduction through safe parallelization, caching, or partitioning without reducing behavior coverage.

## Subtasks

## Notes / Log
