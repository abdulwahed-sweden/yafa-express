# The sounds are not here yet

Every file the game asks for is listed below and none of them is in the
repository. This is deliberate: the code, the paths and the triggers are in
place, and a sound whose file is missing is simply not played, so the game
runs and the tests pass with these folders empty.

The contract is [`docs/audio/sound-catalogue.md`](../../../../docs/audio/sound-catalogue.md).
A file name here is not changed without changing that page, and a sound not
on that page is not added.

Expect six `path not found` lines from the asset server at launch, one for
each clip below. `audio::load_sfx` logs a line of its own beside them saying
as much.

## Wanted, and already wired to a message

These six are played by `audio::play_sounds` the moment their files appear.
Nothing else has to change.

| File | Played on |
| --- | --- |
| `train/train_whistle_depart.ogg` | `TrainDeparted` |
| `train/station_bell_arrive.ogg` | `TrainReached` |
| `ui/ui_ticket_tear.ogg` | `TicketTorn` - DEPART pressed |
| `ui/ui_gate_open.ogg` | `GateAnswered(Passed)` |
| `ui/ui_gate_closed.ogg` | `GateAnswered(Refused)` |
| `ui/ui_passport_stamp.ogg` | `PassportStamped`, new stamps only |

## Wanted, with nothing to drive them yet

The paths exist as constants in `audio::paths`; the loops and the crossfades
they need are not written.

| File | Waiting on |
| --- | --- |
| `train/train_wheels_loop.ogg` | a looping player whose pitch follows the timetable |
| `train/train_brake_steam.ogg` | the last tenth of a segment |
| `ui/ui_station_select.ogg` | FROM / TO written |
| `ui/ui_coins_revenue.ogg` | fares collected, once an arrival |
| `ui/ui_error_soft.ogg` | a rejected ticket away from the gates |
| `ambience/amb_coast_waves.ogg` | the line of the current segment |
| `ambience/amb_mountain_wind.ogg` | the same |
| `ambience/amb_city_market.ogg` | the great stations: Yafa, Al-Quds, Haifa |
| `music/music_menu_oud_bayati.ogg` | a main menu |
| `music/music_journey_low.ogg` | a journey under way |

## Adding one

OGG only - it is the single decoder the build asks Bevy for, and the files
stay small. Drop it at the path above and it plays; there is no registry to
update.

Sources with licences that allow a shipped game: Kenney (CC0), Freesound
filtered to CC0, the Free Music Archive for the oud. Record each file's
licence line in `LICENSES.md` beside it as it is added.
