# Native Residency Recovery Specification

Status: ready-for-agent

## Problem Statement

DropCove's current self-contained x64 Release candidate fails the existing native residency release gate at the canonical 100 Shelf Batch / 1,000 Shelf Item scale. Fresh visible residency is `153.2461 MB`, first post-dismissal residency is approximately `156.27 MB`, the 30-cycle peak is `182.5195 MB`, and the post-30-cycle settled result is `178.8672 MB`. Total process `WorkingSet64` must remain below `160 MB`; passing CPU and latency results do not waive the residency failure.

The current failure has two observable components: a fresh-process baseline regression and additional resident pages accumulated across repeated show/dismiss cycles. Historical attribution cannot identify the owner of the current delta. Selecting a cache, presentation lifetime, XAML, composition, icon or runtime change before current attribution would repeat earlier ineffective experiments.

## Solution

Recover the existing `<160 MB` gate in three evidence-gated vertical steps:

1. Attribute current fresh-process and cycle-dependent resident pages without changing production behavior.
2. Apply the smallest measured ownership/lifetime change that returns every fresh visible and first post-dismissal sample below `160 MB`.
3. Eliminate measured show/dismiss accumulation and qualify the complete 30-cycle Release gate while preserving DropCove interactions and lifecycle.

The diagnosis may prove that one implementation change solves both residency components. The tickets remain separate so fresh-process improvement cannot hide cycle accumulation, and cycle work cannot proceed from an already failing baseline.

## Implementation Decisions

- The acceptance metric remains total process `WorkingSet64`, including shared native pages. It is never replaced by private memory, committed memory, installer size, managed heap size or an average that hides an individual sample over the limit.
- The target remains strictly below `160 MB`. No threshold change, tolerance band, forced GC, `EmptyWorkingSet`, process trimming, working-set API, periodic cleanup, background polling or measurement-only behavior is allowed.
- Profiling precedes production architecture changes. Current snapshots must distinguish native images/modules, shared pages, private heap/runtime pages, mapped files, visual/icon resources and scale-dependent presentation state strongly enough to predict the proposed reduction.
- The canonical acceptance fixture is exactly 100 Shelf Batches / 1,000 Shelf Items in an explicit isolated test profile. A one-batch control may be used only for attribution, never as acceptance evidence.
- Fresh-process acceptance requires every sampled visible and first post-dismissal value below `160 MB`, not only the average.
- Cycle acceptance requires every recorded cycle sample, the peak settled sample and the post-30-cycle settled sample below `160 MB`.
- Idle CPU remains at or below `0.1%`; shelf-show p95 remains at or below `150 ms`.
- The production hotkey/show path and real owned-foreground dismissal path remain the measurement lifecycle. UI Automation may prove interactions in a separate run but must not contaminate the canonical 30-cycle residency loop.
- `DropShelfManager` remains the durable state owner. Presentations own only projections and native resources required by current visible surfaces. Do not add a public abstraction until profiling proves behavior that genuinely varies.
- Use dependencies already present and built-in Windows diagnostics first. Any new development tool or dependency requires necessity and license evidence before addition; no runtime package is added for profiling.
- Make a clean cutover. Remove obsolete ownership/caching paths identified by the measured fix; no compatibility switch, fallback implementation or feature flag.
- High-contrast visual qualification remains a separate blocker in the Bulk Pinning qualification ticket. This effort neither waives nor silently absorbs it.

## Testing Decisions

- Diagnosis records executable and application DLL fingerprints, OS/build, GPU/driver, DPI, fixture counts, lifecycle checkpoints and exact measurement method.
- Current page attribution reconciles page totals against `WorkingSet64` at fresh visible, first dismissal and cycles 5/10/20/30. If page categories are insufficient, use built-in Windows profiling to identify allocation/retention owners; do not infer ownership from module size alone.
- Implementation verification uses the existing automated suite for state/lifecycle contracts and installed Release smoke for real shelf, Edge Rail, popup, drag, Pin/Unpin, Remove, Clear Temporary Items, shake, hotkey, tray, fullscreen and sizing behavior.
- The canonical performance run starts a fresh owned test-profile process, verifies the exact 100/1,000 fixture, performs 30 production show/dismiss cycles, and records all samples. It does not touch the live profile or unrelated process.
- Cleanup records graceful shutdown separately from residency results. A forced stop of the owned test process is a cleanup warning, never evidence that the gate passed.
- Failed experiments are reverted and recorded with their measured result. One failed run does not authorize threshold changes or speculative follow-up code.

## Out of Scope

- Changing Bulk Pinning semantics, drag acceptance, Path Reference lifecycle, availability classification, shelf sizing tiers, Edge Rail product behavior or source filesystem ownership.
- Replacing WinUI, Windows App SDK, .NET, the self-contained unpackaged deployment model or the GPU stack without profiling that proves the platform choice itself makes the approved gate impossible and a separate planning decision approves replacement.
- Forced memory reclamation, process restart, periodic cleanup, background workers, telemetry or user-facing memory controls.
- Treating high-contrast verification, Store submission, installer upgrade or unrelated release gates as residency fixes.

## Further Notes

- Current diagnosis: [diagnosis.md](diagnosis.md).
- Current failing measurement: [`artifacts/bulk-pinning-scale.json`](../../artifacts/bulk-pinning-scale.json).
- Historical passing baseline: [`artifacts/ticket19-final-post-collapse-release.json`](../../artifacts/ticket19-final-post-collapse-release.json).
- Historical page attribution: [`artifacts/ticket19-profiler-attribution.json`](../../artifacts/ticket19-profiler-attribution.json).
- Prior ineffective/reverted experiments: [`artifacts/automatic-growth-scale.json`](../../artifacts/automatic-growth-scale.json).
- No ADR is warranted yet. The root cause and architecture decision are not established; diagnosis must precede any hard-to-reverse choice.
- Repository changes remain uncommitted until separately authorized.
