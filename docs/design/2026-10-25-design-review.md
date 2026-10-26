# Design review — 25 October 2026

**Status of the tasks below, as of 26 October:** D1, D4 and D5 are done
(`theme.rs` is a single light theme, red is reserved for errors, the network is
back to 23 stations with both scripts). D2 is partly done (flat water meshes;
the graticule and geographic projection remain) and D3 is not started. The
successor to D2 and D3 is the schematic approach in
[`2026-10-26-sl-concept.md`](2026-10-26-sl-concept.md).

## Verdict

**Below the interface: keep as it is.** `yafa_core` is separate from the engine,
routing is Dijkstra over real track distances, the journey is a state machine,
the passport persists, and all of it is covered by headless tests. None of this
needed touching.

**Above the interface: the art direction was working against the game.** The
version reviewed had chosen "old railway documents on a dark desk". It reads
well as a sentence and badly as a screen: three colour worlds, 10–13 px type,
a textured map and five panels for a game whose only action is "pick two
stations and go".

The visual decisions were almost entirely confined to `theme.rs`, with layout
in `ui.rs` and the map in `map.rs`, which is why the fix could be scoped as
tasks rather than a rewrite.

## What made the screen unclear

1. **Three palettes on one screen.** A blue-black map ground, cream paper
   panels and brass gold. The eye never settles.
2. **Type too small.** The scale started at 10 px and body text was 13 px, in
   three families (Oswald, Plex Serif, Plex Mono) with tracking and forced
   capitals on every heading: noise, not hierarchy.
3. **A textured map.** Seven wave lines, a graticule, hatched lakes and rails
   drawn as two lines with sleepers, all at ~2 px. Coast, mountain and water
   could not be told apart, and the map had a third of the width.
4. **Five panels for one task.** Ticket counter, station list, city panel,
   passport and the station board, each with title, subtitle, double rule and
   tear-off perforation.
5. **Red for a neutral state.** "Not yet visited" was red. Red means failure.
6. **Invented codes louder than names.** Three-letter station codes (AKA, HFA,
   QDS) sat larger than the names; passport cards were uniformly grey, so
   progress was invisible.
7. **Lost content.** 11 stations instead of 22, no Arabic names, and the
   bundled fonts did not cover Arabic.

## The direction chosen

One light visual world, the same one used across the project's other pages:
one warm ground for map and panels, white cards with a hairline border, a navy
header, one colour per line, everything roughly one and a half times larger.

```
┌──────────────────────────────────────────────────────────────┐
│  Yafa Express                                  Stamps 7 / 23 │  navy bar, 28 px
├──────────────────────────────────────┬───────────────────────┤
│                                      │  Ticket               │
│          Map  (~65 % width)          │  FROM Yafa  TO Al-Quds│
│   flat sea, no waves, no graticule   │  [ Depart ]           │
│   6 px lines, 14 px stations,        ├───────────────────────┤
│   18 px names                        │  Now                  │
│                                      │  heading for Al-Ladd  │
│                                      ├───────────────────────┤
│                                      │  Passport             │
│                                      │  ○ ● ● ○ ○ ● ○ ○ …    │
└──────────────────────────────────────┴───────────────────────┘
```

### Tokens

| Role | Value | Replaced |
| --- | --- | --- |
| Page and map ground | `#f7f5ef` | dark DESK + PAPER |
| Cards | `#ffffff` on `#e3ded2` | PAPER / PAPER_DEEP / PAPER_PRESSED |
| Header / primary text | `#16243a` / `#141d29` | three INK shades |
| Muted | `#4b5563` | — |
| Coastal Line | `#1c8a84` | (kept) |
| Mountain Line | `#b8651f` | (kept) |
| Junction Links | `#6b7482` | BRASS |
| Sea | `#d8efed`, flat | SEA_INK + wave lines |
| Visited / not yet | `#1f8a4c` / `#9aa4b3` | STAMP_RED for the neutral state |

### Type

One family: IBM Plex Sans for Latin, IBM Plex Sans Arabic for Arabic (they mix
well), Plex Mono for distances only.

| Role | Was | Now |
| --- | --- | --- |
| Caption | 10 | 14 |
| Body | 13 | 17 |
| Station name on the map | 15 | 18 |
| Card heading | 12, uppercase | 20 |
| Game title / state line | 27–30 | 28 |

## Tasks as handed to Claude Code

Each task is one branch, the narrowest change that does it, tests before done.
D1–D4 are visual only and never touch `yafa_core`; D5 is content.

- **D1 — `theme.rs`: one palette, one family, the new scale.** Delete SHADOW,
  perforation, double rules and TRACKED. Keep every public signature so
  `ui.rs` and `map.rs` compile unchanged. *(done)*
- **D2 — `map.rs`: a flat, legible map.** Sea and lakes as solid shapes, no
  graticule or wave lines, 6 px single-colour track, 14 px stations with a
  32 px hit target, scale so the network fills ~65 % of the width.
  *(partly done; superseded by the schematic map in the SL concept)*
- **D3 — `ui.rs`: three panels instead of five.** One right column: Ticket /
  Now / Passport. Drop the station list (the map is the list) and the city
  panel (becomes an arrival card). Drop the three-letter codes. Green stamps
  for visited, grey before. *(not started; the arrival card exists)*
- **D4 — semantic colour.** Red only for a real error (same station twice, no
  route). *(done)*
- **D5 — `city.rs`: the 22 stations back, with Arabic.** *(done: 23 stations,
  both scripts, Al-Ladd corrected)*
