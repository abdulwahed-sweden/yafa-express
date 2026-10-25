# Yafa Express

A 2D top-down train simulation across the historical railways of Palestine,
written in Rust with the [Bevy](https://bevy.org) engine.

Buy a ticket at the counter, watch the train work its way along the coastal and
mountain lines, and collect a passport stamp at every station it calls at on the
way. Each station carries a sentence of its own history, so the passport fills
up with places rather than points.

```
cargo run --release
```

---

## The loop

1. **Ticket Counter** - click a departure station and a destination, on the map
   or in the station list. The route is solved for you across the network.
2. **Depart** - the train pulls out and works its way along the track, stopping
   at every station on the route.
3. **Passport** - each station it calls at is stamped. A station is only ever
   stamped once; the trick is to find routes that pass through places you have
   not been.
4. The passport is written to `~/.yafa-express/passport.json`, so progress
   survives the process.

### Controls

| Input | Effect |
| --- | --- |
| Click a station (map or list) | Write it onto the ticket - first `FROM`, then `TO` |
| Click a written station again | Rub it out |
| `Space` | Depart, or dismiss an arrived train |
| `DEPART` once arrived | Books the next trip, so the counter is never a dead end |
| `Esc` | Tear up the ticket |
| `Tab` | Cycle the timetable speed (Local / Express / Mail) |
| Click a passport card | Read that station's history |

---

## The network

Eleven stations on three corridors, with routing solved by Dijkstra over real
track distances.

| Corridor | Stations |
| --- | --- |
| **Coastal Line** | Akka, Haifa, Yafa, Ghazzah |
| **Mountain Line** | Safad, Beisan, Nablus, Al-Quds, Al-Khalil |
| **Junction Links** | Al-Ladd, Tulkarm - where the coast meets the highlands |

Station positions are real latitudes and longitudes, projected onto the map;
segment lengths are track distances, not straight lines. Routing therefore
produces the same answers a timetable would: Yafa to Al-Quds runs through the
junction at Al-Ladd, because that is how the 1892 line was built.

### Track as built, and track as imagined

The map distinguishes the two. Segments that carried traffic are drawn as rails
with sleepers between them and dated on the ticket; segments that were surveyed
or begun but never completed - Beisan to Safad, Nablus to Al-Quds, Al-Quds to
Al-Khalil - are drawn as a broken line. The train runs them anyway. The game is
partly about the network that existed and partly about the one that did not.

---

## Layout

```
crates/
  yafa_core/     the rules, with no engine attached
    geo.rs       coordinates and the map projection
    city.rs      the station table and the three corridors
    network.rs   the graph, and Dijkstra over it
    journey.rs   the train's trip, as a state machine
    ticket.rs    the ticket counter's two boxes
    passport.rs  stamps, progress and persistence
  yafa_game/     the Bevy application
    main.rs      the launcher: window and plugins
    lib.rs       plugin wiring and system ordering
    components.rs  components, messages, resources, states
    map.rs       projection, track drawing, the locomotive
    systems.rs   ticketing, movement, stamping, saving
    ui.rs        header bar; the Ticket, Now, city and Passport cards
    theme.rs     palette, typefaces and shared chrome
    assets/fonts/  Plex Sans, Plex Sans Arabic and Plex Mono, compiled in
    tests/
      journey.rs   whole journeys driven through the real ECS, headless
      layout.rs    the real interface spawned and refreshed, headless
```

The application is split into a `SimulationPlugin` - the rules, which need no
window - and a `PresentationPlugin` - everything that draws. That split is what
lets the integration tests drive a complete journey with no renderer, and it is
how both of the runtime-only traps Bevy sets (a bundle carrying the same
component twice, two queries in one system that can alias) are caught by
`cargo test` rather than by a crash on launch.

`yafa_core` has no dependency on Bevy. That is what makes the interesting parts
testable: the journey state machine, the routing and the passport are all
exercised by `cargo test` without opening a window.

```
cargo test
```

### On the borrow checker

Stations and track are addressed by `CityId` and `SegmentId` - small `Copy`
handles - everywhere in the codebase. Nothing stores a reference into the
network. A system is therefore free to read the map immutably, hold the passport
mutably, and move the train, all in one body:

```rust
pub fn passport_stamp_system(
    mut arrivals: MessageReader<TrainReached>,   // carries a CityId, not a &City
    mut book: ResMut<PassportBook>,
    mut stamped: MessageWriter<PassportStamped>,
) { /* ... */ }
```

The same discipline runs through the UI: each widget carries the `CityId` it
belongs to, so a refresh system finds its target by matching an identifier
rather than by holding an index into a list that might move.

### Events

Arrivals travel as buffered **messages**, not observer events. When the
timetable is wound up to Mail speed a single frame can carry the train through
several stations, and every one of them has to reach the stamping system in
order - which is exactly what a message queue gives and a per-entity trigger
does not.

---

## Design notes

The map is the game, so the map gets the screen. A navy header bar runs across
the top with the title and the passport count; under it the map fills
everything, and one column of four cards stands down the right-hand side:

```
┌────────────────────────────────────────────────────────┐
│ Yafa Express · يافا إكسبرس              Visited 3 / 11 │
├──────────────────────────────────┬─────────────────────┤
│                                  │ Ticket              │
│                                  │   From · To · Depart│
│                                  ├─────────────────────┤
│          the map                 │ Now                 │
│   (the map is the station list)  │   one line + bar    │
│                                  ├─────────────────────┤
│                                  │ the station in focus│
│                                  ├─────────────────────┤
│                                  │ Passport            │
│                                  │   stamp chips       │
└──────────────────────────────────┴─────────────────────┘
```

There is no second list of stations: you choose by clicking the map.

One light page, one type family, flat colour. The only colour that carries
meaning is the colour of a line or the state of a station.

| Role | |
| --- | --- |
| Page / map ground | `#f7f5ef` |
| Cards | `#ffffff` on a `#e3ded2` hairline, 12 px radius |
| Header bar | `#16243a` |
| Accent (Depart button, route in play) | `#1c8a84` |
| Headings / primary text | `#16243a` / `#141d29` |
| Coastal Line / Mountain Line / Junction Links | `#1c8a84` / `#b8651f` / `#6b7482` |
| Sea | `#d8efed` flat |
| Visited / not yet visited | `#1f8a4c` / `#9aa4b3` |

### Type

IBM Plex Sans with IBM Plex Sans Arabic behind it in every fallback chain;
IBM Plex Mono for figures. Five sizes: 15 caption, 17 body, 20 station name,
22 heading, 30 display. Every station is named twice on the map - Arabic
script first and large, the romanised name under it - and once on its stamp.

Both families are compiled into the binary rather than loaded from a folder,
so the executable runs from any working directory. They are under the SIL Open
Font License; the licence sits beside them in `crates/yafa_game/assets/fonts`.

### The map

- **Track** is one solid line, 5 px wide, in its corridor's colour. The route
  the ticket buys, or the train is running, gets a wider accent line over it.
  Surveyed-but-unbuilt track is the same line, broken.
- **Stations** are round discs - a navy ring with the corridor colour inside -
  with a 36 px click target. A green ring means stamped; a green or amber
  ring means it is on the ticket.
- **The sea, the lake and the Dead Sea** are flat fills with a thin shoreline.
  No graticule, no compass; a small scale bar is the only furniture.
- Gizmo lines come in three widths (water, track, route) through three gizmo
  config groups, set once at startup in `lib.rs`.

## Requirements

Rust 1.97 or newer (Bevy 0.20's minimum). The 3D renderer, audio, glTF and
animation are switched off in `Cargo.toml`, since none of them are used - which
cuts the dependency tree to the part this game actually runs on.

The first build pulls in and compiles the whole engine, so expect it to take a
while; after that, incremental builds are a few seconds. Debug builds compile
dependencies at `opt-level = 3` (see `[profile.dev.package."*"]`), because a
fully unoptimised Bevy is too slow to play.

## Licence

MIT. See [LICENSE](LICENSE).
