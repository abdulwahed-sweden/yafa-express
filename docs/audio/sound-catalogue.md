# Sound catalogue

Every sound the game will play, grouped by the moment it belongs to, with the
file name it is stored under and the message that triggers it. The catalogue
is the contract: a sound not listed here is not added, and a file name here is
not changed without changing this page.

Naming: `group_event_description.ogg`, lower case, OGG only (Bevy decodes it by
default and the files stay small). The constant in code carries the same name
in upper case, so file, constant and trigger always match.

```
assets/audio/

train/
  train_whistle_depart.ogg      steam whistle on departure
  train_wheels_loop.ogg         wheels on rail; looped, pitch follows speed
  train_brake_steam.ogg         brakes and steam on approach
  station_bell_arrive.ogg       station bell on arrival

ui/
  ui_station_select.ogg         soft wooden click, picking a station
  ui_ticket_tear.ogg            tearing the ticket on Depart
  ui_gate_open.ogg              gate chime, valid ticket
  ui_gate_closed.ogg            short low buzz, closed gate or no ticket
  ui_passport_stamp.ogg         heavy rubber stamp, the reward moment
  ui_coins_revenue.ogg          coins when fares are paid
  ui_error_soft.ogg             quiet tone for a real error (same station twice, no route)

ambience/
  amb_coast_waves.ogg           waves and a gull, Coastal Line
  amb_mountain_wind.ogg         wind and distant sheep bells, Mountain Line
  amb_city_market.ogg           market murmur at the great stations (Yafa, Al-Quds, Haifa)

music/
  music_menu_oud_bayati.ogg     quiet oud, maqam Bayati, main menu
  music_journey_low.ogg         the same, lowered, during a journey
```

## Triggers

| Sound | Played on | Notes |
| --- | --- | --- |
| `train_whistle_depart` | `TrainDeparted` | once |
| `train_wheels_loop` | while `Journey::Moving` | loop; volume and pitch from timetable speed |
| `train_brake_steam` | last 10 % of a segment | fade the loop under it |
| `station_bell_arrive` | `TrainReached` | once per station |
| `ui_station_select` | FROM / TO written | — |
| `ui_ticket_tear` | Depart pressed | before the gate step |
| `ui_gate_open` / `ui_gate_closed` | gate tapped | see the SL concept, S5 |
| `ui_passport_stamp` | `PassportStamped` | only for a new stamp |
| `ui_coins_revenue` | fares collected | once per arrival, not per passenger |
| `ui_error_soft` | ticket rejected | — |
| `amb_*` | line of the current segment | crossfade at interchanges |
| `music_*` | app state | menu ↔ journey |

## In code

```rust
pub const TRAIN_WHISTLE_DEPART: &str = "audio/train/train_whistle_depart.ogg";
pub const UI_PASSPORT_STAMP:    &str = "audio/ui/ui_passport_stamp.ogg";
pub const AMB_COAST_WAVES:      &str = "audio/ambience/amb_coast_waves.ogg";
```

Sounds are played by one system that reads the messages above. Draw systems
never play audio.

## Sources

Free, with licences that allow inclusion in a shipped game: Kenney (UI and
train packs, CC0), Freesound (filter to CC0), Free Music Archive for the oud.
Keep each file's licence line in `assets/audio/LICENSES.md` when it is added.

## Build note

`bevy_audio` is currently off (`default-features = false` in `Cargo.toml`). It
has to be enabled for the game crate, and Linux builds will then need
`libasound2-dev`. The browser build plays audio only after the first user
gesture; the gate tap in S5 is that gesture.
