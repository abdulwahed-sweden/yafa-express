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
`default-features = false`, with `bevy_audio` and `vorbis` among the few
features it does ask for. Linux needs `libasound2-dev` to build that.

## Backlog

Design background: `docs/design/`. Sounds: `docs/audio/sound-catalogue.md`.
A browser demo of the target interface: `docs/design/demo/sl-concept.html`.

### Done

- D1 one light theme (`theme.rs`), D4 semantic colour, D5 the full station
  table in both scripts — see `docs/design/2026-10-25-design-review.md`.
- Calendar, passengers and money, arrival card, locomotive, browser build.
- The SL interface, S1 to S6 (`docs/design/2026-10-26-sl-concept.md`):
  - **S1** every station carries a `schematic: (i32, i32)` in `city.rs`. The
    geographic coordinates and every distance are untouched.
  - **S2** `map.rs` draws the schematic — 0/45/90° elbow-routed polylines,
    dots, interchange capsules, terminus badges, −40° labels, white route
    overdraw. The water, the shorelines and the graticule are still in
    `map.rs` but no longer in the draw path.
  - **S3** the Now panel is the in-car progress strip, and the station list
    is gone: the schematic map is the list. That was the D3 remainder.
  - **S4** a platform board in the ticket panel counts the next departures
    down at the chosen timetable speed.
  - **S5** the ticket passes a gate before the train moves — green and red
    gates, the error states, and the messages the first three sounds play on.
  - **S6** `bevy_audio` is on with the OGG decoder, `assets/audio/` is laid
    out per the catalogue, and one system plays from the messages. The clips
    themselves are not in the repository yet; a file that is missing is
    simply not played.

### Next

- The sounds. Every path, trigger and folder is in place, and
  `assets/audio/README.md` lists what is wanted: six one-shots are already
  wired and play the moment their files appear, while the loops and the
  ambiences still need a player written for them.
- Settings: language toggle in the UI (the string table already exists),
  volume.
- A main menu with the oud.

### Later

- The geographic view, if it is ever wanted back. The water meshes, the
  shorelines, the graticule and the scale bar are all still in `map.rs`,
  unregistered, held together by `spawn_geographic_markers`.

## Handing a task to Claude Code

```
Read CLAUDE.md, README.md and docs/design/2026-10-26-sl-concept.md.
Do task S1 only. Branch feat/s1-schematic-coords. Smallest change.
Run cargo test --workspace, show me the diff, do not push to main.
```
