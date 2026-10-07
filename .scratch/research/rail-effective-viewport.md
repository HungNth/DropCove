# Edge Rail effective viewport investigation

## Observed runtime facts

The isolated eight-batch rail is physically and logically `280 × 612`; its ScrollViewer viewport is `540` high and reports no vertical overflow. Only six rows render. Trace reports root EffectiveViewport `180 × 348` and repeater EffectiveViewport `176 × 312`; `180 × 348` matches the primary Drop Shelf dimensions. Evidence: `artifacts/adaptive-rail-rowfit-diagnosis.json`, `artifacts/test-right-8batch.png`, and `artifacts/adaptive-rail-high-contrast.png`.

Native corner-region dimensions and HWND dimensions agree. The mismatch is not explained by incorrect tier arithmetic or an eight-versus-six data count: the projection contains eight summaries.

## Verified platform APIs

- [FrameworkElement.EffectiveViewportChanged](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.frameworkelement.effectiveviewportchanged): notification of effective viewport changes.
- [UIElement.RegisterAsScrollPort](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.uielement.registerasscrollport): public static API declaring a clip as a scrolling viewport. Microsoft intends it for custom scrolling controls, not as a blanket replacement for native ScrollViewer behavior.
- [FrameworkElement.InvalidateViewport](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.frameworkelement.invalidateviewport): protected API; its element must previously be registered as a scroll port. It cannot be called directly on an arbitrary Grid from window code.
- Installed WinUI metadata documents ItemsRepeaterScrollHost primarily for Windows versions before 1809. Wrapping the rail in that stock host did not fix this repro.

## Rejected or inconclusive experiments

Layout invalidation, disabling scale motion, managed no-activate Show, and the stock scroll host still rendered six rows. Constructor Window.Activate also rendered six rows and produced a rail foreground event; it was rejected because the focus contract must remain intact. All were reverted.

Explicit root/scroller sizing and scroll-port registration were also removed. Their later no-expansion probes are inconclusive: the final clean input check identified LockApp PID 8356 instead of the owned test process. No layout conclusion can be drawn from the zero-row locked-desktop runs.

## Uncertainty

A thread-cached primary-window viewport is a plausible **[INFERENCE]**, not a source-proven root cause. The original research draft overstated that explanation and scroll-port registration as an established fix; this note supersedes those claims. No verified framework fix is recorded. Future diagnosis must preserve no-activate behavior, virtualization, the exact tiers, and the eight-complete-row criterion rather than inflating cache or suppressing the failing check.
