# Batch intake and responsive Shelf

Status: resolved
Type: task
Version: 0.0.2
Parent: `../spec.md`
Blocked by: 01

## Required behavior

Extend the working single-item path to complete DropCove batch intake and basic presentation:

- Accept every supported path from one storage payload as at most one Shelf Batch.
- Remove repeated path strings within the payload using ordinal case-insensitive comparison, preserving the first occurrence and source order.
- Do not canonicalize path strings or rewrite separators.
- Suppress an incoming batch when its complete path set matches any retained batch under unordered ordinal case-insensitive comparison.
- Accept partial overlap as a new newest-first batch and allow the same path in different batches.
- Accept supported entries from mixed payloads and report the skipped count; unsupported-only payloads create no state.
- Render newest-first single-item and multi-item batch cards in the canonical responsive grid.
- Apply basic Automatic Shelf Growth tiers when opening or accepting content; duplicate-only and unsupported drops do not resize, reorder, scroll, or change status.
- Persist the first sizing intent transactionally with the first accepted batch and restore the applicable automatic size.

## Acceptance criteria

1. A multi-path payload creates one batch with source-order items.
2. Repeated paths differing only by ordinal case are reduced to the first occurrence.
3. An exact retained path set in another order is a silent no-op across memory, database, layout, scrolling, sizing, and status.
4. Partial overlap creates a new newest-first batch.
5. Mixed input reports skipped entries; unsupported-only input changes nothing.
6. Grid column count and automatic height match the canonical formulas and tiers at boundary widths and row counts.
7. Restart preserves batch order, item order, paths, metadata, and the sizing intent owned by this version.

## Testing seam

- Core intake tests use explicit path strings to cover case-insensitive repetition, unordered exact-set duplicates, partial overlap, and source-order preservation.
- Layout logic is checked at documented width and row boundaries without asserting incidental component markup.
- A real mixed Windows storage payload proves adapter filtering and skipped-count feedback.

## Demo path

1. Drop three paths including one repeated path and observe one ordered batch with the first occurrence retained.
2. Drop the same set in another order and observe no change.
3. Drop a partially overlapping set and observe a new newest-first batch.
4. Resize across one-, two-, and three-column boundaries and add batches through each automatic-height tier.
5. Restart and observe retained order and applicable sizing.

## Comments

- Batch intake uses Windows ordinal case-insensitive comparison, first-occurrence/source-order retention, exact unordered path-set suppression, and mixed-entry skipped feedback.
- Native Windows regression suite: 8 passed, including duplicate/partial-overlap behavior, order/filtering, live growth, restart recalculation, and final-item sizing reset.
- Real native drops verified both CF_HDROP file/folder lists and a Shell payload containing a filesystem file plus virtual Recycle Bin; the latter accepted the file and reported one skipped entry.
- Real duplicate batch retry retained the existing batch IDs, order, height, status, and scroll position.
- Actual WebView grid columns were 1, 2, 2, and 3 at widths 347, 348, 515, and 516 respectively.
- Process restart restored source-order file/folder metadata and preferred width 516; automatic height recalculated from 236 to 180.
- Empty-shelf width intent remains transient and is inserted with the first accepted batch, per ARCHITECTURE section 6; no empty-shelf settings row is created.
- Ticket 03 may proceed.
