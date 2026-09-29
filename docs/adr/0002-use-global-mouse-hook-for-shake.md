# Use a global mouse hook for shake-to-open

Windows has no documented global event that exposes another application's OLE file-drag lifecycle before DropCove receives a drag-enter event. When shake-to-open is enabled, DropCove will therefore keep a low-level mouse hook installed, process only minimal input state in the callback, and detect shake as a heuristic without polling cursor coordinates.

## Considered Options

- Poll cursor state: rejected because it creates continuous background work.
- Require a modifier or disable shake by default: rejected because shake is intended to remain immediately available in V1.
- Remove shake-to-open: rejected because it is a defining access path for the product.

## Consequences

- Shake is enabled by default but can be disabled; disabling it removes the hook.
- The hook cannot prove that the user is dragging files. False positives are handled by cooldown and by restoring the previous shelf state when no supported drop is accepted.
- The hook callback must remain minimal and queue analysis elsewhere so it stays within Windows hook timing constraints.
- Idle CPU and summon latency must satisfy the product performance targets on packaged release builds.
