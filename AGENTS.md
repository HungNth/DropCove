# AGENTS.md

## Engineering

Apply these principles within the assigned role and approved scope. During ticket delivery, treat the approved spec and tickets as settled requirements; report gaps for a planning decision before changing them.

- Do not preserve backward compatibility. Remove obsolete paths instead of adding compatibility layers, fallbacks, or migrations.
- Choose the simplest implementation that fully meets the current requirements. Avoid speculative abstractions, configuration, and indirection.
- Grow the system in layers. Start from the smallest version that works end to end, and add each new capability on top of a product that already works. Never trade a working product for unfinished complexity.
- Keep components modular and concerns clearly separated.
- Prefer established, well-maintained libraries when they reduce overall complexity or improve reliability. Do not reimplement common functionality without a clear reason.
- Lean on the dependencies already in the project before writing your own implementation or adding packages. Do not assume a library lacks a capability without checking its documentation and types.
- Make architectural decisions for the long term. Do not accept a stopgap that only works for now and is meant to be replaced later.
- Study how established products solve the problem before designing a solution. Adopt their proven patterns and conventions rather than inventing an approach from scratch.

## CLI WinUI Toolchain

Build DropCove through the .NET CLI with the official WinUI 3 C# templates, Windows Developer Mode, and NuGet-declared project dependencies. Do not add Visual Studio workloads, MSVC Build Tools, standalone MSBuild, or Visual Studio XAML tooling unless a concrete implementation requirement proves the CLI toolchain insufficient.

Validate the toolchain with `dotnet new winui`, `dotnet restore`, `dotnet build`, and `dotnet run`. A globally accessible `msbuild.exe` or populated `dotnet workload list` is not required when `dotnet msbuild` and project builds succeed.

Before adding any development tool, executable, SDK, or NuGet dependency, establish why it is necessary and its license. Prefer permissive dependencies; Microsoft SDKs that support commercial use without a developer-seat license are permitted.

## Agent Development Workflow

This project uses Matt Pocock Skills for planning and delivery.

### Planning Workflow

For non-trivial feature work:

```text
grill-with-docs
→ to-spec
→ to-tickets
→ user approval
```

Use `grill-me` instead of `grill-with-docs` only when persistent domain documentation is not desired.

**Do not create Git commits without explicit user approval. This applies to all repository changes, including specs, ADRs, tickets, documentation, source code, tests, configuration, generated files, and fixes. Creating, editing, implementing, reviewing, testing, or validating changes does not imply approval to commit. Keep all changes uncommitted until the user explicitly authorizes the commit.**

Keep requirement discovery, specification, and ticket decomposition in the primary OMP context.

### Ticket Design

Tickets should be self-contained vertical slices and include, where applicable:

- required behavior;
- acceptance criteria;
- testing seam;
- demo path;
- dependencies/blockers;
- parent specification.

Do not turn tickets into implementation scripts with unnecessary file paths or line numbers.

## Agent skills

### Issue tracker

Issues and specs are tracked as local markdown files under `.scratch/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Canonical five-role triage vocabulary using default label strings. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context layout (`CONTEXT.md` and `docs/adr/` at repo root). See `docs/agents/domain.md`.

## Output style

The reader has ADHD. Shape every response so it can be acted on:

1. Lead with the answer or next action: command, path, or snippet first.
2. Number multi-step work; one bounded action per step.
3. When blocked, end with one concrete user decision or action needed to resume.
4. Finish the current issue before raising a new one.
5. During multi-step work, state the current ticket or step and its evidence-backed status.
6. When giving a supported time estimate, use concrete units.
7. After a change, show what now works.
8. Errors: state location, cause, and fix. No drama.
9. Prefer lists of at most 5 items; preserve every field and evidence item required by the active workflow's report contract.
10. Omit preambles and repeated recaps; include required acceptance or blocker reports.

Exceptions: explain fully when asked to explain. Confirm before destructive actions. Use the active workflow's retry and escalation rules; outside such a workflow, stop after three failed fixes and name the doubtful assumption. Ask one focused question when a required decision cannot be resolved from repository evidence.
