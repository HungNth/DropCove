# Separate Edge Rail preference from remembered placement

`settings.json` owns the user’s `RailEdge` and `RailMonitorId` preferences, while `shelf.db` owns only the nullable monitor actually used for the latest non-empty shelf dismissal. A connected explicit monitor takes precedence; an unavailable configured or remembered monitor temporarily falls back to the Windows primary display without rewriting settings or persisting a display-change-only fallback, which avoids competing sources of truth while preserving the user’s choice across temporary disconnection.
