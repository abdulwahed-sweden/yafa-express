# CLAUDE.md — working rules and backlog

Read at the start of every session. `README.md` explains the game and the
architecture; this file says how to work on it and what is next.

## Rules that do not bend

1. **Historical naming only.** Station names are the Arabic names of the period
   and their Latin transliteration, as in `crates/yafa_core/src/city.rs`.
   Never introduce modern Hebrew names.
2. **`yafa_core` has no Bevy.** Rules, routing, the journey state machine, the
   passport and the calendar live there and are tested headless. Presentation
   never leaks in.
3. **Indices, not references.** `CityId` and `SegmentId` everywhere. Nothing
   stores a reference into the network.
4. **One writer per resource.** Only the economy system writes the till; only
   the stamping system writes the passport. Everyone else sends a message.
5. **Semantic colour only.** Green = visited / open, grey = not yet, red = a
   real error. Line colours are the only other colour that carries meaning.
6. **Smallest correct change.** Understand the current code path first,
   preserve unrelated behaviour, no drive-by refactors, no whole-tree
   `cargo fmt`.
7. **Branch per task, never push to `main`.** Open a PR; the owner merges.
8. **Tests before done.** `cargo test -p yafa_core`, then
   `cargo test --workspace`. Never report green without running them. Inspect
   the diff before finishing.
9. **Docs are dated and append-only.** A superseded note gets a pointer at the
   top, never a rewrite. See `docs/README.md`.

## Build

```bash
cargo run --release           # desktop
cargo test --workspace        # core, then the headless ECS tests
```

Browser build: `web/README.md`. The game crate builds Bevy with
`default-features = false`; audio is off until task S6.

## Backlog

Design background: `docs/design/`. Sounds: `docs/audio/sound-catalogue.md`.
A browser demo of the target interface: `docs/design/demo/sl-concept.html`.

### Done

- D1 one light theme (`theme.rs`), D4 semantic colour, D5 the full station
  table in both scripts — see `docs/design/2026-10-25-design-review.md`.
- Calendar, passengers and money, arrival card, locomotive, browser build.

### Next — the SL interface (`docs/design/2026-10-26-sl-concept.md`)

- **S1** `city.rs`: add `schematic: (i32, i32)` per station from the demo.
  Geographic coordinates and every distance stay as they are.
- **S2** `map.rs`: draw the schematic — 0/45/90° polylines, dots, interchange
  capsules, terminus badges, −40° labels, white route overdraw; no water,
  shorelines or graticule in the draw path.
- **S3** Now panel as the progress strip (current route; passed / next /
  terminus states; one status line).
- **S4** Departure board in the ticket panel while FROM is set, counting down
  at the chosen timetable speed.
- **S5** Gate step before departure: green / red gates, the error states, the
  first three sounds.
- **S6** Audio plugin: `bevy_audio` on, `assets/audio/` per the catalogue, a
  `Sfx` resource, one system that plays from messages.

Order: S1 → S2 → S3, then S4, S5, S6 in any order.

### Later

- D3 remainder: drop the station list once the schematic map is the list.
- Settings: language toggle in the UI (the string table already exists),
  volume.
- A main menu with the oud.

## Handing a task to Claude Code

```
Read CLAUDE.md, README.md and docs/design/2026-10-26-sl-concept.md.
Do task S1 only. Branch feat/s1-schematic-coords. Smallest change.
Run cargo test --workspace, show me the diff, do not push to main.
```
