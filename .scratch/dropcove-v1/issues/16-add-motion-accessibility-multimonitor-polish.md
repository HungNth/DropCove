# 16: Add motion, accessibility, and full multi-monitor polish

**What to build:** Finish the V1 interaction surfaces with responsive motion, reduced-motion behavior, accessibility basics, and full Edge Rail multi-monitor/DPI verification without weakening native workflow performance.

**Blocked by:** 15: Add Compact and Expanded shelf modes.

**Status:** ready-for-agent

- [ ] Shelf appearance, rail expansion, Compact/Expanded transition, item removal, and batch insertion use short Composition-based motion only after state changes are correct.
- [ ] Windows reduced-motion and animation settings disable or reduce non-essential motion.
- [ ] Interactive controls expose accessible names, keyboard focus, and visible focus states appropriate to their actions.
- [ ] Compact, Expanded, Settings, flyouts, and Edge Rail remain usable at supported scaling levels and per-monitor DPI changes.
- [ ] Rail placement, remembered monitor/side, fullscreen hiding, shake placement, and shelf placement remain correct when monitors are added, removed, rearranged, or use different DPI.
- [ ] Motion, thumbnail work, database work, and virtualization do not block pointer or drag interaction.
- [ ] Packaged accessibility, reduced-motion, keyboard, DPI, multi-monitor, focus, and animation smoke scenarios pass.
- [ ] Release-build CPU, latency, and working-set measurements remain within the product targets after polish.