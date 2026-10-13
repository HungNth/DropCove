# Availability-aware whole-batch drag

Status: resolved
Type: task
Version: 0.0.2
Parent: `../spec.md`
Blocked by: 02

## Required behavior

Complete whole-batch drag-out without risking incorrect reference loss:

- Classify paths at restore and immediately before whole-batch drag as Available, Missing, or Unavailable using the canonical filesystem rules.
- Separate drag preparation from completion and record the exact Available participants prepared for that operation.
- Durably remove confirmed Missing references before drag begins and prune an empty batch.
- Retain Unavailable references and exclude them from the data package.
- Ask before dragging only Available Items when Available and Unavailable references coexist.
- Cancel with status feedback when no Available participants remain.
- Offer a read-only Windows storage-item data package with `Copy` only.
- After accepted `Copy`, durably remove only the prepared Temporary participants. Rejection, cancellation, and pre-acceptance failure retain participants.
- Missing cleanup remains committed if the later confirmation or drag is canceled.
- State changes after preparation cannot cause unrelated items to be consumed during completion.

## Acceptance criteria

1. Available paths participate in a copy-only whole-batch drag.
2. Missing paths are removed durably before the drag and stay removed after later cancellation.
3. Unavailable paths remain visible and persisted but do not participate.
4. Mixed Available/Unavailable input asks for confirmation; canceling retains both classes except prior Missing cleanup.
5. No-Available preparation starts no Windows drag and reports status.
6. Accepted `Copy` removes only the prepared Temporary participants; canceled or rejected drag removes none of them.
7. A mutation between preparation and completion cannot consume a newly added or otherwise unrelated item.
8. Restore refreshes availability without continuously watching every path.

## Testing seam

- Availability classification is behind an adapter that can deterministically produce Available, Missing, and Unavailable outcomes.
- Core preparation/completion checks cover participant snapshots, Missing cleanup, mixed confirmation, no-Available cancellation, and stale-completion safety.
- The real Windows smoke uses a reachable local file, a confirmed Missing path, and a temporarily unavailable root or controlled access failure.

## Demo path

1. Create a three-item batch with one Available path, one path that becomes Missing, and one path whose root becomes Unavailable.
2. Start whole-batch drag and observe durable Missing cleanup plus mixed-availability confirmation.
3. Cancel once and observe retained Available and Unavailable references.
4. Retry, accept `Copy`, and observe removal of only the prepared Available Temporary Item.
5. Restore the unavailable root and observe the retained reference become eligible later.

## Comments

- Native core suite: 11 passed, including Missing cleanup/cancel retention, no-Available preparation, and Copy consumption limited to recorded participants despite an intervening add.
- Real embedded frontend (`http://tauri.localhost/`) displayed the mixed-availability modal after a three-path batch lost one local file and its temporary `Z:` root was disconnected.
- Cancel retained Available and Unavailable references; Missing remained durably removed.
- Dragging the modal's available-items button into the native destination produced `target-enter` and `target:Copy` for only `release-available.txt`. The returned shelf retained exactly the Unavailable reference.
- A no-Available batch showed status without starting a native drag and retained its references.
- Reconnecting the owned drive alias made the retained path usable; direct item Copy consumed it and reset the empty shelf to 180 × 180.
- Earlier rejected/canceled smoke attempts were caused by incorrect target coordinates, stale dev assets, and a latched Escape key after tool interruption; no Copy outcome was fabricated.
- The modal supports native dragging while the button is held, or keyboard/click confirmation followed by a second card drag. Native self-drop is rejected to prevent consuming an existing reference through duplicate intake.
- Ticket 04 may proceed.
