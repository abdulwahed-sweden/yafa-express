# The SL concept — 26 October 2026

Stockholm's metro (SL) solves, on a moving train, exactly the interface problem
Yafa Express has: a passenger who glances at a screen must know in one second
where they are, what comes next and where the train ends. Four pieces of that
system are worth copying. A working browser demo of all four is in
[`demo/sl-concept.html`](demo/sl-concept.html); open the file directly.

This note supersedes D2 and D3 of the
[design review](2026-10-25-design-review.md). It changes nothing in
`yafa_core`: distances, routing and the calendar stay geographic. Only the
drawing becomes schematic.

## 1. The in-car progress screen → the "Now" panel

```
Sätra ─── Skärholmen ─── Vårby gård ══▶ Vårberg ─── Masmo ─── … ─── Mörby
  ●            ●            ●               ○          ○              [ box ]
  passed: coloured, dimmed labels           next: larger        terminus boxed
```

Rules, as the demo implements them:

- One horizontal strip, stations of the **current route only**, evenly spaced.
- The line behind the train is drawn in the route colour; the line ahead is
  white. The train is a small marker on the line.
- Labels of passed stations are dimmed; the **next** station is set one size
  larger and heavier; the **terminus** sits in a white box at the end.
- A single status line above: "At station: X" / "Next: X" / "Arrived: X".

This replaces the "heading for Al-Ladd" text in the Now panel of the review.

## 2. The schematic network map → `map.rs`

SL's map is not geographic; it is a diagram with strict rules, and those rules
are what make it legible at a glance.

- Track runs at **0°, 45° or 90° only**.
- **One colour per line**: Coastal `#1c8a84`, Mountain `#b8651f`, Junction
  Links `#8b94a3`.
- Ordinary stations are a dot; **interchanges are a white capsule** (Ghazzah,
  Yafa, Al-Ladd, Al-Quds; add Tulkarm, Haifa and Beisan when their links are
  drawn).
- The **line number sits in a coloured square at each terminus** ("1" coast,
  "2" mountain).
- Labels are set at −40° when stations are dense, Arabic first, Latin
  beneath in a smaller mono.
- Dark, flat ground (`#2b2f36`); nothing else drawn on it: no sea, no
  graticule, no sleepers.
- The selected route is overdrawn in white, thicker than the line; the hit
  target of every station is 36 px.

The demo's schematic coordinates (1000 × 400 viewBox) can be lifted straight
into a `schematic: (i32, i32)` field on the station table — a second
coordinate alongside the geographic one, not a replacement for it, so distances
and the calendar are untouched. Palestine is north–south; the schematic lays it
west-to-east like SL does, Rafah left, Akka and Safad right, so the two lines
read as two horizontal bands with the links between them.

```
coast     Rafah · Khan Yunis · Ghazzah · Al-Majdal · Isdud · Yibna · Yafa · Qaisariya · Haifa · Akka
                                  │                                      │
                                  │                                   Al-Ladd
                                  │                                      ╲ 45°
mountain               Bir al-Sabi' · Al-Khalil · Bayt Lahm · Al-Quds · Ramallah · Nablus · Jenin · Al-Nasira · Tabariya · Safad
```

## 3. The departure board → a station screen

The platform board at a station entrance: navy band, yellow stripe, amber
dot-matrix type, a real clock in the middle, two directions side by side.

```
●  1  Akka   6 min        ⌚        ●  2  Safad   7 min
   1  Akka  25 min                    2  Safad  27 min
```

In the game this is the screen the player sees **while a ticket is being
written**: next departures in each direction from the FROM station, counting
down in game minutes (the demo uses 5 s per minute). It is flavour with a use:
it tells the player the timetable speed they have chosen (Local / Express /
Mail) in a way the `Tab` hint never did.

Type for this board is Plex Mono, amber `#ffb000` with a faint glow, on
`#0b0f1c`. The band is `#1d2a6b` with a `#f2c200` stripe. These two colours are
the board's own and appear nowhere else.

## 4. The ticket gates → ticket validation

Gates with a green arrow and a glowing strip are open; gates with a red cross
are shut. In the game this is the step between "ticket written" and "train
departs":

```
ticket written?
   ├─ no  → tap gate → red flash, soft error tone, hint "pick two stations"
   └─ yes → tap green gate → green flash, chime → departure (whistle)
             tap red gate  → "this gate is closed, use a green one"
```

It gives the departure a physical beat (tap, flash, chime, whistle) and it is
the natural home for the error state D4 asked for, without a modal.

## Tasks

Each is one branch, the narrowest change that does it, `cargo test --workspace`
before done. None touches `yafa_core` except S1.

- **S1 — schematic coordinates.** Add `schematic: (i32, i32)` to the station
  table in `city.rs`, taken from the demo. Geographic coordinates stay; nothing
  that computes distance changes. Test: every station has one, no two
  coincide.
- **S2 — `map.rs` draws the schematic.** 0/45/90° polylines in line colour,
  dots and interchange capsules, terminus badges, −40° labels, white route
  overdraw. Remove water meshes, shorelines and the graticule from the draw
  path (keep the code behind a flag only if the geographic view is wanted
  later).
- **S3 — the Now panel as the progress strip.** Current route only, passed /
  next / terminus states as above, status line.
- **S4 — the departure board.** Shown in the ticket panel while FROM is set;
  counts down at the chosen timetable speed.
- **S5 — gates before departure.** `Space` / DEPART opens the gate step; green
  and red gates with the behaviour above; wires the first three sounds from
  [`../audio/sound-catalogue.md`](../audio/sound-catalogue.md).
- **S6 — audio plugin.** `bevy_audio` on, `assets/audio/` laid out per the
  catalogue, a `Sfx` resource of handles, played from messages
  (`TrainDeparted`, `TrainReached`, `PassportStamped`), never from the draw
  systems.

Order: S1 → S2 → S3, then S4, S5, S6 in any order.
