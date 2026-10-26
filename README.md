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
4. The passport is written to `~/.yafa-express/passport.json`, so progress -
   and the year you had reached - survives the process.

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
| `Lay` on a surveyed line | Pay to build it, in the city card |

---

## The network

Twenty-three stations on three corridors, with routing solved by Dijkstra over
real track distances.

| Corridor | Stations |
| --- | --- |
| **Coastal Line** | Akka, Haifa, Qaisariya, Yafa, Yibna, Isdud, Al-Majdal, Ghazzah, Khan Yunis, Rafah |
| **Mountain Line** | Safad, Tabariya, Al-Nasira, Beisan, Jenin, Nablus, Ramallah, Al-Quds, Bayt Lahm, Al-Khalil, Bir al-Sabi' |
| **Junction Links** | Al-Ladd, Tulkarm - where the coast meets the highlands |

Where a real line ran through a town, the town is a stop on it rather than a
detour around it: the Jezreel valley railway is Haifa to Al-Nasira to Beisan,
and the 1917 military line south calls at Yibna, Isdud, Al-Majdal, Ghazzah and
Khan Yunis on its way to Rafah. Haifa to Nablus therefore routes down the
valley and its Jenin branch - 90 km - rather than round by Tulkarm at 100.

Station names are carried in both scripts, and where a station stands too close
to its neighbour for any rule to separate them - Al-Quds and Bayt Lahm are nine
kilometres apart - the label placement is data on the station rather than a
nudge to its coordinates. A coordinate is a claim about where a place is.

Station positions are real latitudes and longitudes, projected onto the map;
segment lengths are track distances, not straight lines. Routing therefore
produces the same answers a timetable would: Yafa to Al-Quds runs through the
junction at Al-Ladd, because that is how the 1892 line was built.

### The year

The game opens in 1892, the year the line from Yafa up to Al-Quds was cut
through. Nothing else exists yet: three stations on the whole map, and one
route between them.

Time passes because track passes. Every 120 km run moves the calendar on by a
year, and the network opens underneath you as it does - the Jezreel valley line
in 1904, the branch up to Akka in 1913, the military line south to Rafah in
1917. A station the network has not reached is still drawn and its history can
still be read, but it is washed out and the counter will not sell a ticket to
it. The board says so as each line comes into service.

Projected track never opens on its own account, however late it gets. It has to
be paid for.

### Passengers and money

The railway opens with 500 piastres. Travellers gather on every platform the
network reaches, bound for somewhere else on it - how often a station produces
one, and how often it is where somebody is going, follow its importance, so the
great ports and Al-Quds carry the traffic and the halts on the southern line
barely register.

A train takes on everyone whose destination is still ahead of it, up to forty
seats, and sets them down as it goes. Each pays five piastres plus fifteen
hundredths a kilometre for the distance they were actually carried - measured
along the train's own line, not the shortest way between the two. Coal costs a
piastre every two kilometres and is paid on departure, whether anybody rides or
not.

That is what buys the rest of the map. A surveyed line costs ten piastres a
kilometre to lay; pay for it and it opens in the current year, and the stations
at its ends join the network for good. Safad, Tabariya, Ramallah, Qaisariya and
Al-Khalil can be reached no other way.

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
    ui.rs        ticket counter, station board, city panel, passport
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

One light page, one type family, flat colour. The only colour that carries
meaning is the colour of a line or the state of a station.

| Role | |
| --- | --- |
| Page / map ground | `#f7f5ef` |
| Cards | `#ffffff` on a `#e3ded2` hairline |
| Headings / primary text | `#16243a` / `#141d29` |
| Muted | `#4b5563` |
| Coastal Line / Mountain Line / Junction Links | `#1c8a84` / `#b8651f` / `#6b7482` |
| Sea | `#d8efed` |
| Visited / not yet visited | `#1f8a4c` / `#9aa4b3` |

### One family

IBM Plex Sans does all the lettering, with IBM Plex Sans Arabic behind it in
every fallback chain so Arabic script is set in a face from the same design
rather than in whatever the system offers. IBM Plex Mono is kept for figures -
distances, times and station codes - where a fixed advance keeps columns
aligned.

The scale is five steps: 14 caption, 17 body, 18 station name, 20 heading, 28
display. Nothing is tracked, and nothing is set in capitals except the
three-letter station codes, which are codes rather than styling.

Both families are compiled into the binary rather than loaded from a folder,
so the executable runs from any working directory and can be copied anywhere
without losing its lettering. They are under the SIL Open Font License; the
licence sits beside them in `crates/yafa_game/assets/fonts`.

### The map

Every mark on the map is a gizmo line, redrawn each frame:

- **Track** is two running rails with sleepers between them, so it reads as
  track rather than as a stroke. Surveyed-but-unbuilt track is a broken line.
- **The sea, the lake and the Dead Sea** are flat fills. Gizmos draw lines and
  not polygons, so a fill is an even-odd scanline a pixel apart - a few hundred
  lines a frame, which costs nothing and saves carrying a mesh and a material
  just to colour the sea. The sea's polygon is the shoreline closed off well
  outside the frame, so the fill runs off every edge instead of stopping on a
  visible seam.
- **A graticule, a scale bar and a north arrow** give the sheet its structure.

### Everything else

- **Everything is in English.** Station names are romanised (Akka, Yafa,
  Al-Quds), with the name used on European timetables alongside where it
  differs.
- **No art assets beyond the fonts.** The locomotive and every panel are built
  from coloured rectangles and text. Junctions are diamonds and through
  stations are squares, which is the convention a printed railway map uses.
- **Stamps land crooked.** Each passport card carries a small fixed tilt, and
  presses in and settles when the station is first collected.

## In a browser

The game also targets `wasm32-unknown-unknown`. Everything it needs is
compiled in - the typefaces included - so there is nothing to serve but the
page and the two files `wasm-bindgen` produces. See [web/README.md](web/) for
the four commands.

The web build cannot save: there is no filesystem to write the passport to, so
the year, the money and the stamps last as long as the tab. The read and the
write are compiled out there rather than left to fail on every stamp.

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
