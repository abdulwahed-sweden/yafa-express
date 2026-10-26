//! The gameplay systems: ticketing, running the train, and stamping the
//! passport.

use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use yafa_core::{CityId, Journey, JourneyEvent, StampOutcome};

use crate::components::*;
use crate::strings::{Language, Phrase, fill};

/// The last year any line actually opened. Anything dated later than this was
/// laid by the player and has to be written back into the save.
const LAST_HISTORICAL_YEAR: u16 = 1917;

// --- Input ----------------------------------------------------------------

/// Turns a button activation into an [`ActionRequested`].
///
/// One observer serves every control in the game: the button carries a
/// [`UiAction`], and this just forwards it.
pub fn on_control_activated(
    activate: On<Activate>,
    controls: Query<&UiAction>,
    mut actions: MessageWriter<ActionRequested>,
) {
    if let Ok(&action) = controls.get(activate.entity) {
        actions.write(ActionRequested(action));
    }
}

/// Clicking a station on the map writes it onto the ticket, exactly as if the
/// matching row in the station list had been clicked.
pub fn on_station_clicked(
    click: On<PointerClick>,
    stations: Query<&StationDot>,
    mut actions: MessageWriter<ActionRequested>,
) {
    if let Ok(&StationDot(city)) = stations.get(click.entity) {
        actions.write(ActionRequested(UiAction::PickCity(city)));
    }
}

/// Keyboard equivalents for the controls that get used most.
pub fn keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    phase: Res<State<GamePhase>>,
    mut actions: MessageWriter<ActionRequested>,
) {
    if keys.just_pressed(KeyCode::Space) {
        actions.write(ActionRequested(match phase.get() {
            GamePhase::Arrived => UiAction::BackToCounter,
            _ => UiAction::Depart,
        }));
    }
    if keys.just_pressed(KeyCode::Escape) {
        actions.write(ActionRequested(UiAction::ClearTicket));
    }
    if keys.just_pressed(KeyCode::Tab) {
        actions.write(ActionRequested(UiAction::CycleClock));
    }
}

// --- Ticket counter -------------------------------------------------------

/// Applies every requested action. The one place that decides what a control
/// means.
pub fn apply_actions(
    mut commands: Commands,
    lang: Res<Language>,
    mut requests: MessageReader<ActionRequested>,
    mut desk: ResMut<Desk>,
    mut clock: ResMut<Clock>,
    mut focus: ResMut<Focus>,
    mut announcer: ResMut<Announcer>,
    mut phase: ResMut<NextState<GamePhase>>,
    mut reached: MessageWriter<TrainReached>,
    mut rail: ResMut<Rail>,
    mut counting: ResMut<Counting>,
    almanac: Res<Almanac>,
    trains: Query<(Entity, Option<&TrainRun>), With<Train>>,
) {
    let Ok((train, run)) = trains.single() else { return };

    // Whether a train is out is read from the train itself, not from the game
    // phase: `NextState` only lands on the next frame, and a click arriving in
    // that gap must not rewrite a ticket the train is already working.
    // Spawning the run is deferred too, so these flags are carried through the
    // loop by hand.
    let mut riding = run.is_some();
    let mut arrived = run.is_some_and(|run| run.is_finished());

    for &ActionRequested(requested) in requests.read() {
        // Once the train is in, "depart" means the next departure - so the
        // same control sends it back to the counter rather than going dead.
        let action = match requested {
            UiAction::Depart if arrived => UiAction::BackToCounter,
            other => other,
        };

        match action {
            UiAction::PickCity(city) => {
                // Always show the history, even while the train is running
                // and even for a station the network has not reached yet.
                focus.0 = Some(city);
                if riding {
                    continue;
                }
                if !rail.station_is_open(city, almanac.year()) {
                    announcer.say(fill(
                        lang.say(Phrase::AnnNoLineTo),
                        &[name(*lang, rail.city(city)), &almanac.year().to_string()],
                    ));
                    continue;
                }
                let _ = desk.select(city);
            }
            UiAction::ShowCity(city) => focus.0 = Some(city),
            // Handled by `ui::apply_language_toggle`, which owns the resource.
            UiAction::ToggleLanguage => {}
            UiAction::OpenLine(segment) => {
                let (line_name, cost) = {
                    let track = rail.segment(segment);
                    (
                        format!(
                            "{}-{}",
                            name(*lang, rail.city(track.from)),
                            name(*lang, rail.city(track.to))
                        ),
                        counting.tariff.track(track.track_m),
                    )
                };
                if !counting.treasury.debit(cost) {
                    announcer.say(fill(
                        lang.say(Phrase::AnnCannotAfford),
                        &[&line_name, &cost.to_string(), &counting.treasury.balance().to_string()],
                    ));
                    continue;
                }
                let year = almanac.year();
                if rail.0.open_line(segment, year) {
                    announcer.say(fill(
                        lang.say(Phrase::AnnLineLaid),
                        &[&year.to_string(), &line_name, &cost.to_string()],
                    ));
                } else {
                    // Already built: hand the money straight back.
                    counting.treasury.credit(cost);
                }
            }
            UiAction::ClearTicket => {
                if !riding {
                    desk.clear();
                }
            }
            UiAction::CycleClock => {
                clock.cycle_clock();
                let speed = lang.say(match clock.clock_label() {
                    "Local" => Phrase::Local,
                    "Mail" => Phrase::Mail,
                    _ => Phrase::Express,
                });
                announcer.say(fill(lang.say(Phrase::AnnTimetable), &[speed]));
            }
            UiAction::Depart => {
                if riding {
                    continue;
                }
                let Some(route) = desk.issue(&rail, almanac.year()) else { continue };
                let origin = route.origin();
                let destination = route.destination();
                announcer.say(fill(
                    lang.say(Phrase::AnnDeparting),
                    &[
                        name(*lang, rail.city(origin)),
                        name(*lang, rail.city(destination)),
                        &format!("{:.0}", route.total_km()),
                        &route.legs.len().saturating_sub(1).to_string(),
                    ],
                ));

                // The passport is stamped on arrival at each station, and the
                // origin is never "arrived at" - so book it here.
                reached.write(TrainReached { city: origin, final_stop: false });
                focus.0 = Some(origin);

                let coal = counting.tariff.coal(route.total_m);
                counting.treasury.credit(-coal);
                announcer.say(fill(lang.say(Phrase::AnnCoal), &[&coal.to_string()]));

                commands.entity(train).insert((
                    TrainRun(Journey::board(route, &clock.0)),
                    Riders::default(),
                    Visibility::Visible,
                ));
                phase.set(GamePhase::OnTheRails);
                riding = true;
                arrived = false;
            }
            UiAction::BackToCounter => {
                commands
                    .entity(train)
                    .remove::<TrainRun>()
                    .remove::<Riders>()
                    .insert(Visibility::Hidden);
                desk.clear();
                phase.set(GamePhase::AtTheCounter);
                announcer.say(lang.say(Phrase::AnnCounterOpen));
                riding = false;
                arrived = false;
            }
        }
    }
}

// --- Running the train ----------------------------------------------------

/// Advances the locomotive along its route and reports what it passed.
///
/// All the movement rules live in `yafa_core::Journey`; this system's job is to
/// hand it a delta time, translate its events into messages, and put the sprite
/// where the journey says it is.
pub fn train_movement_system(
    time: Res<Time>,
    clock: Res<Clock>,
    rail: Res<Rail>,
    view: Res<MapView>,
    mut scratch: ResMut<JourneyScratch>,
    mut trains: Query<(&mut TrainRun, &mut Transform), With<Train>>,
    mut reached: MessageWriter<TrainReached>,
    mut departed: MessageWriter<TrainDeparted>,
    mut finished: MessageWriter<JourneyFinished>,
) {
    let Ok((mut run, mut transform)) = trains.single_mut() else { return };

    let (origin, destination, track_m) = {
        let route = run.route();
        (route.origin(), route.destination(), route.total_m)
    };

    scratch.0.clear();
    run.advance(time.delta_secs(), &clock.0, &mut scratch.0);

    for event in scratch.0.drain(..) {
        match event {
            JourneyEvent::Departed(city) => {
                departed.write(TrainDeparted(city));
            }
            JourneyEvent::Reached { city, final_stop } => {
                reached.write(TrainReached { city, final_stop });
            }
            JourneyEvent::Completed => {
                finished.write(JourneyFinished { origin, destination, track_m });
            }
        }
    }

    let at = view.world(run.position(&rail));
    transform.translation = at.extend(20.0);
    let (hx, hy) = run.heading(&rail);
    transform.rotation = Quat::from_rotation_z(hy.atan2(hx));
}

/// Moves to the "arrived" phase once the route is complete.
pub fn settle_on_arrival(
    mut finished: MessageReader<JourneyFinished>,
    mut phase: ResMut<NextState<GamePhase>>,
) {
    if finished.read().next().is_some() {
        phase.set(GamePhase::Arrived);
    }
}

// --- The passport ---------------------------------------------------------

/// Stamps the passport for every station the train called at.
///
/// Stations arrive as [`CityId`]s on a message, so this system can hold the
/// passport mutably while reading the network immutably - no aliasing, and no
/// need for interior mutability anywhere.
pub fn passport_stamp_system(
    mut arrivals: MessageReader<TrainReached>,
    mut book: ResMut<PassportBook>,
    mut stamped: MessageWriter<PassportStamped>,
) {
    for arrival in arrivals.read() {
        let outcome = book.passport.stamp(arrival.city);
        let stamp = outcome.stamp();
        if outcome.is_fresh() {
            book.unsaved = true;
        }
        stamped.write(PassportStamped {
            city: arrival.city,
            fresh: matches!(outcome, StampOutcome::Fresh(_)),
            order: stamp.order,
        });
    }
}

/// Books the finished trip in the passport's mileage record, and winds the
/// calendar on by the distance run.
///
/// Time passes because track passes: 120 km to the year. A long enough run can
/// cross several years at once, and each may bring a line into service, so
/// every one of them is reported.
pub fn record_completed_journeys(
    mut finished: MessageReader<JourneyFinished>,
    mut book: ResMut<PassportBook>,
    mut almanac: ResMut<Almanac>,
    rail: Res<Rail>,
    mut opened: MessageWriter<LineOpened>,
) {
    for trip in finished.read() {
        book.passport.record_journey(trip.track_m);
        book.unsaved = true;

        for year in almanac.advance(trip.track_m) {
            for segment in rail.segments_opening_in(year) {
                opened.write(LineOpened { year, segment: segment.id });
            }
        }
    }
}

/// Reads out each line as it comes into service.
pub fn announce_openings(
    rail: Res<Rail>,
    lang: Res<Language>,
    mut opened: MessageReader<LineOpened>,
    mut announcer: ResMut<Announcer>,
) {
    // Several stretches can open in one year - six did in 1917 - so they are
    // gathered into one line rather than flooding the board.
    let mut by_year: Vec<(u16, Vec<String>)> = Vec::new();
    for notice in opened.read() {
        let segment = rail.segment(notice.segment);
        let name = format!("{}-{}", rail.city(segment.from).name, rail.city(segment.to).name);
        match by_year.iter_mut().find(|(year, _)| *year == notice.year) {
            Some((_, names)) => names.push(name),
            None => by_year.push((notice.year, vec![name])),
        }
    }

    for (year, names) in by_year {
        announcer.say(match names.as_slice() {
            [] => continue,
            [one] => fill(lang.say(Phrase::AnnLineOpens), &[&year.to_string(), one]),
            [first, rest @ ..] => fill(
                lang.say(Phrase::AnnLinesOpen),
                &[&year.to_string(), first, &rest.len().to_string()],
            ),
        });
    }
}

/// Writes the passport back to disk once the dust has settled.
///
/// A failed write is reported and the dirty flag cleared, so a read-only home
/// directory degrades to an unsaved game rather than an error every frame.
pub fn save_passport(
    mut book: ResMut<PassportBook>,
    almanac: Res<Almanac>,
    counting: Res<Counting>,
    rail: Res<Rail>,
) {
    if !book.unsaved {
        return;
    }
    // The year, the money and the lines that were paid for are all part of
    // the traveller's progress, and share its file.
    book.passport.calendar = almanac.0;
    book.passport.treasury = counting.treasury;
    book.passport.bought_lines = rail
        .segments()
        .iter()
        .filter_map(|segment| match segment.opening {
            // Anything open later than the table says was bought.
            yafa_core::Opening::Opened(year) if year > LAST_HISTORICAL_YEAR => {
                Some(yafa_core::passport::BoughtLine { from: segment.from, to: segment.to, year })
            }
            _ => None,
        })
        .collect();
    let path = book.path.clone();
    match book.passport.save(&path) {
        Ok(()) => info!("passport saved to {}", path.display()),
        Err(error) => warn!("could not save passport to {}: {error}", path.display()),
    }
    book.unsaved = false;
}

// --- The announcer --------------------------------------------------------

/// Turns the frame's messages into lines on the station board.
pub fn announce(
    rail: Res<Rail>,
    lang: Res<Language>,
    mut announcer: ResMut<Announcer>,
    mut focus: ResMut<Focus>,
    mut departures: MessageReader<TrainDeparted>,
    mut stamps: MessageReader<PassportStamped>,
    mut finished: MessageReader<JourneyFinished>,
) {
    for TrainDeparted(city) in departures.read() {
        announcer.say(fill(lang.say(Phrase::AnnRightAway), &[name(*lang, rail.city(*city))]));
    }

    for stamp in stamps.read() {
        let city = rail.city(stamp.city);
        // The panel follows the train, so the player reads about each city as
        // they reach it.
        focus.0 = Some(stamp.city);
        if stamp.fresh {
            announcer.say(fill(
                lang.say(Phrase::AnnStamped),
                &[name(*lang, city), &stamp.order.to_string(), &yafa_core::CITY_COUNT.to_string()],
            ));
        } else {
            announcer.say(fill(lang.say(Phrase::AnnStampStands), &[name(*lang, city)]));
        }
    }

    for trip in finished.read() {
        announcer.say(fill(
            lang.say(Phrase::AnnArrived),
            &[
                name(*lang, rail.city(trip.destination)),
                &format!("{:.0}", trip.track_m as f32 / 1000.0),
                name(*lang, rail.city(trip.origin)),
            ],
        ));
    }
}

/// Sets the opening announcement and focuses a station worth reading about.
pub fn opening_announcement(
    book: Res<PassportBook>,
    mut announcer: ResMut<Announcer>,
    mut focus: ResMut<Focus>,
) {
    announcer.say("Ticket counter open. Choose a departure and a destination.");
    if book.passport.visited_count() > 0 {
        announcer.say(format!(
            "Welcome back. {} already stamped.",
            book.passport.counter().to_lowercase()
        ));
    }
    // Yafa is where the first line in the country began; a fair place to start.
    focus.0 = Some(CityId::Yafa);
}

/// Says what year it is, and what that means for the map.
pub fn opening_year_announcement(
    rail: Res<Rail>,
    lang: Res<Language>,
    almanac: Res<Almanac>,
    mut announcer: ResMut<Announcer>,
) {
    let open = rail.open_stations(almanac.year()).count();
    announcer.say(fill(
        lang.say(Phrase::AnnYearStanding),
        &[&almanac.year().to_string(), &open.to_string(), &yafa_core::CITY_COUNT.to_string()],
    ));
}

// --- Passengers and money -------------------------------------------------

/// Draws fresh travellers onto the open platforms.
///
/// On a timer rather than every frame: at sixty ticks a second every platform
/// on the network would be full inside a second and the queues would mean
/// nothing.
pub fn fill_platforms(
    time: Res<Time>,
    rail: Res<Rail>,
    almanac: Res<Almanac>,
    mut timer: ResMut<TrafficTimer>,
    mut queues: ResMut<Queues>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    queues.tick(&rail, almanac.year());
}

/// Works the doors: everyone bound for this station gets off and pays, then
/// everyone going the train's way gets on.
///
/// Fares are collected here rather than at the end of the trip, because a
/// passenger pays for the distance they were carried and most of them are not
/// riding to the terminus.
pub fn work_the_doors(
    mut arrivals: MessageReader<TrainReached>,
    rail: Res<Rail>,
    mut counting: ResMut<Counting>,
    mut queues: ResMut<Queues>,
    mut trains: Query<(&TrainRun, &mut Riders)>,
    mut takings: MessageWriter<Takings>,
) {
    let Ok((run, mut riders)) = trains.single_mut() else { return };

    for arrival in arrivals.read() {
        let leaving = riders.alight_at(arrival.city);
        let mut earned = 0;
        for rider in &leaving {
            // What they actually travelled, not what the train will run.
            let carried =
                rail.route(rider.boarded_at, arrival.city, u16::MAX).map_or(0, |leg| leg.total_m);
            earned += counting.tariff.fare(carried);
        }
        if earned > 0 {
            counting.treasury.credit(earned);
            takings.write(Takings {
                piastres: earned,
                at: Some(arrival.city),
                passengers: leaving.len(),
            });
        }

        if !arrival.final_stop {
            riders.board_at(&mut queues.0, arrival.city, run.route());
        }
    }
}

/// Reads the takings out on the board.
pub fn announce_takings(
    rail: Res<Rail>,
    lang: Res<Language>,
    mut takings: MessageReader<Takings>,
    mut announcer: ResMut<Announcer>,
) {
    for take in takings.read() {
        let Some(city) = take.at else { continue };
        // Just the count: "one passenger" and "N passengers" would need a
        // plural rule per language, and the number alone reads fine in both.
        let who = take.passengers.to_string();
        announcer.say(fill(
            lang.say(Phrase::AnnSetDown),
            &[&who, name(*lang, rail.city(city)), &take.piastres.to_string()],
        ));
    }
}

/// A station's name, in the language showing.
fn name(lang: Language, city: &yafa_core::City) -> &'static str {
    match lang {
        Language::English => city.name,
        Language::Arabic => city.arabic,
    }
}
