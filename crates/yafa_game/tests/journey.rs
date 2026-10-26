//! Drives whole journeys through the real ECS, with no window and no renderer.
//!
//! This is what the simulation/presentation split buys: the wiring that is
//! easiest to get wrong - message ordering, when the origin gets stamped, when
//! the phase flips - is exercised here rather than by looking at the screen.

use std::time::Duration;

use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use yafa_core::{CityId, JourneyConfig, Network, Passenger, Passport};
use yafa_game::SimulationPlugin;
use yafa_game::components::*;

/// A game with no window: the rules, a clock we drive by hand, and a train.
struct Harness {
    app: App,
}

impl Harness {
    /// `name` keeps each test's save file to itself: the tests run in parallel
    /// and a shared path would have them writing over one another.
    fn new(name: &str) -> Self {
        let mut app = App::new();
        // `InputPlugin` for the keyboard shortcuts; `StatesPlugin` for the
        // game phase. Neither needs a window.
        app.add_plugins((InputPlugin, StatesPlugin)).add_plugins(SimulationPlugin {
            passport: Passport::new("Test Traveller"),
            // Nowhere real: the tests must never touch the player's own save.
            passport_path: std::env::temp_dir()
                .join("yafa-express-tests")
                .join(format!("{name}.json")),
            traffic_seed: 0xC0FFEE,
        });

        // No `TimePlugin`, so `Time` is ours to advance deterministically.
        app.init_resource::<Time>();

        // The locomotive the movement system looks for.
        app.world_mut().spawn((Train, Transform::default(), Visibility::Hidden));

        let mut harness = Self { app };
        harness.tick(0.0);
        harness
    }

    /// Runs one frame, having advanced the clock by `dt` seconds.
    fn tick(&mut self, dt: f32) {
        self.app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(dt));
        self.app.update();
    }

    fn request(&mut self, action: UiAction) {
        self.app.world_mut().write_message(ActionRequested(action));
    }

    fn passport(&self) -> &Passport {
        &self.app.world().resource::<PassportBook>().passport
    }

    /// Winds the calendar to a year, so a test can use the whole network
    /// without first running a few thousand kilometres to earn it.
    fn set_year(&mut self, year: u16) {
        *self.app.world_mut().resource_mut::<Almanac>() =
            Almanac(yafa_core::Calendar::starting_at(year));
    }

    fn year(&self) -> u16 {
        self.app.world().resource::<Almanac>().year()
    }

    /// Holds the traffic generator still, so a test can lay out exactly the
    /// passengers it means to and have nobody else wander onto the platform.
    fn stop_traffic(&mut self) {
        *self.app.world_mut().resource_mut::<TrafficTimer>() =
            TrafficTimer(Timer::from_seconds(1.0e6, TimerMode::Repeating));
    }

    fn balance(&self) -> i64 {
        self.app.world().resource::<Counting>().treasury.balance()
    }

    fn credit(&mut self, piastres: i64) {
        self.app.world_mut().resource_mut::<Counting>().treasury.credit(piastres);
    }

    fn phase(&self) -> GamePhase {
        *self.app.world().resource::<State<GamePhase>>().get()
    }

    fn desk(&self) -> TicketDeskView {
        let desk = self.app.world().resource::<Desk>();
        TicketDeskView { origin: desk.origin, destination: desk.destination }
    }

    /// Books a ticket and runs the train until it arrives, or gives up.
    fn travel(&mut self, from: CityId, to: CityId) {
        self.request(UiAction::PickCity(from));
        self.request(UiAction::PickCity(to));
        self.tick(1.0 / 60.0);
        // Out through the gates, the way the player goes.
        self.request(UiAction::ApproachGates);
        self.tick(1.0 / 60.0);
        self.request(UiAction::TapGate(GateLane::Signalled));
        self.tick(1.0 / 60.0);

        for _ in 0..4_000 {
            if self.phase() == GamePhase::Arrived {
                return;
            }
            self.tick(1.0 / 60.0);
        }
        panic!("the train never arrived at {to:?}");
    }
}

struct TicketDeskView {
    origin: Option<CityId>,
    destination: Option<CityId>,
}

/// A quick clock, so a test journey is a few hundred frames rather than tens of
/// thousands.
fn hurry_up(harness: &mut Harness) {
    *harness.app.world_mut().resource_mut::<Clock>() = Clock(JourneyConfig {
        time_scale: 4_000.0,
        dwell_seconds: 0.05,
        boarding_seconds: 0.05,
        ..default()
    });
}

#[test]
fn a_journey_stamps_both_ends_and_everything_between() {
    let mut harness = Harness::new("a_journey_stamps_both_ends_and_everything_between");
    hurry_up(&mut harness);

    // Yafa hangs off the junction at Al-Ladd, so the traveller collects three
    // stamps from a two-station ticket.
    harness.travel(CityId::Yafa, CityId::AlQuds);

    let passport = harness.passport();
    assert!(passport.is_stamped(CityId::Yafa), "the origin must be stamped too");
    assert!(passport.is_stamped(CityId::AlLadd), "the junction passed through must be stamped");
    assert!(passport.is_stamped(CityId::AlQuds), "the destination must be stamped");
    assert_eq!(passport.visited_count(), 3);
    assert_eq!(passport.journeys_completed(), 1);
}

#[test]
fn stamps_are_numbered_in_the_order_the_train_called() {
    let mut harness = Harness::new("stamps_are_numbered_in_the_order_the_train_called");
    hurry_up(&mut harness);
    harness.travel(CityId::Yafa, CityId::AlQuds);

    let passport = harness.passport();
    let order: Vec<CityId> = passport.stamps().iter().map(|s| s.city).collect();
    assert_eq!(order, vec![CityId::Yafa, CityId::AlLadd, CityId::AlQuds]);
}

#[test]
fn a_second_trip_over_the_same_ground_adds_no_new_stamps() {
    let mut harness = Harness::new("a_second_trip_over_the_same_ground_adds_no_new_stamps");
    harness.set_year(1918);
    hurry_up(&mut harness);

    harness.travel(CityId::Yafa, CityId::AlQuds);
    let after_first = harness.passport().visited_count();

    harness.request(UiAction::BackToCounter);
    harness.tick(1.0 / 60.0);
    harness.travel(CityId::AlQuds, CityId::Yafa);

    assert_eq!(harness.passport().visited_count(), after_first);
    // The trip still counts, and the mileage still accrues.
    assert_eq!(harness.passport().journeys_completed(), 2);
    assert!(harness.passport().total_track_km() > 150.0);
}

#[test]
fn the_phase_follows_the_train() {
    let mut harness = Harness::new("the_phase_follows_the_train");
    harness.set_year(1918);
    hurry_up(&mut harness);
    assert_eq!(harness.phase(), GamePhase::AtTheCounter);

    harness.travel(CityId::Haifa, CityId::Nablus);
    assert_eq!(harness.phase(), GamePhase::Arrived);

    harness.request(UiAction::BackToCounter);
    harness.tick(1.0 / 60.0);
    harness.tick(1.0 / 60.0);
    assert_eq!(harness.phase(), GamePhase::AtTheCounter);
    assert_eq!(harness.desk().origin, None, "the ticket should be cleared on return");
}

#[test]
fn the_ticket_counter_fills_from_then_to() {
    let mut harness = Harness::new("the_ticket_counter_fills_from_then_to");
    harness.set_year(1918);

    harness.request(UiAction::PickCity(CityId::Akka));
    harness.tick(1.0 / 60.0);
    assert_eq!(harness.desk().origin, Some(CityId::Akka));
    assert_eq!(harness.desk().destination, None);

    harness.request(UiAction::PickCity(CityId::Ghazzah));
    harness.tick(1.0 / 60.0);
    assert_eq!(harness.desk().destination, Some(CityId::Ghazzah));

    harness.request(UiAction::ClearTicket);
    harness.tick(1.0 / 60.0);
    assert_eq!(harness.desk().origin, None);
    assert_eq!(harness.desk().destination, None);
}

#[test]
fn the_counter_is_shut_while_the_train_is_running() {
    let mut harness = Harness::new("the_counter_is_shut_while_the_train_is_running");
    harness.set_year(1918);
    // Deliberately slow, so the train is still out on the line.
    *harness.app.world_mut().resource_mut::<Clock>() =
        Clock(JourneyConfig { time_scale: 1.0, ..default() });

    harness.request(UiAction::PickCity(CityId::Akka));
    harness.request(UiAction::PickCity(CityId::Haifa));
    harness.tick(1.0 / 60.0);
    harness.request(UiAction::Depart);

    // The very next frame, before `NextState` has even landed: the ticket must
    // already be protected. The counter closes on the train, not on the phase.
    harness.request(UiAction::PickCity(CityId::Ghazzah));
    harness.tick(1.0 / 60.0);
    assert_eq!(harness.desk().origin, Some(CityId::Akka));
    assert_eq!(harness.desk().destination, Some(CityId::Haifa));

    harness.tick(1.0 / 60.0);
    assert_eq!(harness.phase(), GamePhase::OnTheRails);

    // And it keeps holding once the phase has caught up.
    harness.request(UiAction::PickCity(CityId::Ghazzah));
    harness.tick(1.0 / 60.0);
    assert_eq!(harness.desk().origin, Some(CityId::Akka));
    assert_eq!(harness.desk().destination, Some(CityId::Haifa));
    assert_eq!(
        harness.app.world().resource::<Focus>().0,
        Some(CityId::Ghazzah),
        "the city panel should still follow the click"
    );
}

#[test]
fn a_lagged_frame_does_not_skip_a_station() {
    let mut harness = Harness::new("a_lagged_frame_does_not_skip_a_station");
    harness.set_year(1918);
    hurry_up(&mut harness);

    harness.request(UiAction::PickCity(CityId::Akka));
    harness.request(UiAction::PickCity(CityId::Rafah));
    harness.tick(1.0 / 60.0);
    harness.request(UiAction::Depart);

    // One enormous frame, the kind a stalled window produces, covering the
    // whole run from the Galilee coast to the southern hills.
    harness.tick(30.0);
    harness.tick(1.0 / 60.0);

    assert_eq!(harness.phase(), GamePhase::Arrived);

    // Every stop the route actually calls at, asked of the network rather
    // than written down here: which towns lie between Akka and Al-Khalil is
    // the network's business, and it changes as lines are added.
    let network = Network::new();
    let route = network.route(CityId::Akka, CityId::Rafah, 1918).expect("connected");
    let passport = harness.passport();
    for city in route.stops() {
        assert!(passport.is_stamped(city), "{city:?} was skipped over");
    }
    assert!(route.legs.len() > 4, "this needs to be a long run to be worth testing");
}

#[test]
fn the_clock_button_cycles_through_the_timetable() {
    let mut harness = Harness::new("the_clock_button_cycles_through_the_timetable");
    let first = harness.app.world().resource::<Clock>().clock_label();

    harness.request(UiAction::CycleClock);
    harness.tick(1.0 / 60.0);
    let second = harness.app.world().resource::<Clock>().clock_label();
    assert_ne!(first, second);

    // Three presets, so three more presses come back round.
    for _ in 0..2 {
        harness.request(UiAction::CycleClock);
        harness.tick(1.0 / 60.0);
    }
    assert_eq!(harness.app.world().resource::<Clock>().clock_label(), first);
}

#[test]
fn depart_sends_an_arrived_train_back_to_the_counter() {
    let mut harness = Harness::new("depart_sends_an_arrived_train_back_to_the_counter");
    harness.set_year(1918);
    hurry_up(&mut harness);
    harness.travel(CityId::Yafa, CityId::AlQuds);
    assert_eq!(harness.phase(), GamePhase::Arrived);

    // The same control the player just used to leave now books the next trip,
    // so an arrived train is never a dead end.
    harness.request(UiAction::ApproachGates);
    harness.tick(1.0 / 60.0);
    harness.tick(1.0 / 60.0);

    assert_eq!(harness.phase(), GamePhase::AtTheCounter);
    assert_eq!(harness.desk().origin, None);

    // And the counter really is open again.
    harness.travel(CityId::Haifa, CityId::Akka);
    assert!(harness.passport().is_stamped(CityId::Akka));
}

#[test]
fn the_calendar_is_wound_on_by_the_track_run() {
    let mut harness = Harness::new("the_calendar_is_wound_on_by_the_track_run");
    hurry_up(&mut harness);
    assert_eq!(harness.year(), yafa_core::FIRST_YEAR);

    // Yafa to Al-Quds is 86 km, short of the 120 that makes a year.
    harness.travel(CityId::Yafa, CityId::AlQuds);
    assert_eq!(harness.year(), 1892);

    // The return trip carries it over.
    harness.request(UiAction::BackToCounter);
    harness.tick(1.0 / 60.0);
    harness.travel(CityId::AlQuds, CityId::Yafa);
    assert_eq!(harness.year(), 1893);
}

#[test]
fn a_station_off_the_network_cannot_be_ticketed() {
    let mut harness = Harness::new("a_station_off_the_network_cannot_be_ticketed");

    // In 1892 the branch to Akka is twenty-one years away.
    harness.request(UiAction::PickCity(CityId::Akka));
    harness.tick(1.0 / 60.0);
    assert_eq!(harness.desk().origin, None, "no line runs there yet");
    assert_eq!(
        harness.app.world().resource::<Focus>().0,
        Some(CityId::Akka),
        "but its history can still be read"
    );

    // Once the years have run on, the same click books it.
    harness.set_year(1913);
    harness.request(UiAction::PickCity(CityId::Akka));
    harness.tick(1.0 / 60.0);
    assert_eq!(harness.desk().origin, Some(CityId::Akka));
}

#[test]
fn projected_track_stays_shut_however_late_it_gets() {
    let mut harness = Harness::new("projected_track_stays_shut_however_late_it_gets");
    harness.set_year(1948);

    // Only surveyed line runs to Safad, so no year ever opens it.
    harness.request(UiAction::PickCity(CityId::Safad));
    harness.tick(1.0 / 60.0);
    assert_eq!(harness.desk().origin, None);
}

#[test]
fn a_run_with_passengers_aboard_pays_for_itself() {
    let mut harness = Harness::new("a_run_with_passengers_aboard_pays_for_itself");
    hurry_up(&mut harness);
    harness.stop_traffic();
    let opening = harness.balance();
    assert_eq!(opening, yafa_core::Treasury::OPENING_BALANCE);

    // Seed the platforms by hand rather than waiting on the generator: this
    // test is about the money, not about who turns up.
    {
        let mut queues = harness.app.world_mut().resource_mut::<Queues>();
        for _ in 0..12 {
            queues.at_mut(CityId::Yafa).join(Passenger { destination: CityId::AlQuds });
        }
        for _ in 0..6 {
            queues.at_mut(CityId::Yafa).join(Passenger { destination: CityId::AlLadd });
        }
        // Bound somewhere this train does not go, so they stay put.
        for _ in 0..3 {
            queues.at_mut(CityId::Yafa).join(Passenger { destination: CityId::Haifa });
        }
    }

    harness.travel(CityId::Yafa, CityId::AlQuds);

    // 18 fares collected against 43 piastres of coal on an 86 km run.
    assert!(
        harness.balance() > opening,
        "the run should turn a profit: {} -> {}",
        opening,
        harness.balance()
    );

    let queues = harness.app.world().resource::<Queues>();
    assert_eq!(
        queues.at(CityId::Yafa).len(),
        3,
        "only those going the train's way should have boarded"
    );
}

#[test]
fn a_line_can_be_bought_and_stays_bought() {
    let mut harness = Harness::new("a_line_can_be_bought_and_stays_bought");
    harness.set_year(1918);

    // Yafa to Qaisariya is 55 km of surveyed line: 550 piastres.
    let segment = {
        let rail = harness.app.world().resource::<Rail>();
        rail.projected()
            .find(|s| (s.from, s.to) == (CityId::Qaisariya, CityId::Yafa))
            .expect("the coastal line is still on paper")
            .id
    };

    harness.credit(1_000);
    let before = harness.balance();
    harness.request(UiAction::OpenLine(segment));
    harness.tick(1.0 / 60.0);

    assert_eq!(harness.balance(), before - 550, "the line should cost ten a kilometre");

    let rail = harness.app.world().resource::<Rail>();
    assert!(rail.is_open(segment, 1918), "and carry traffic from the year it was laid");
    assert!(rail.station_is_open(CityId::Qaisariya, 1918));
}

#[test]
fn a_line_the_railway_cannot_afford_is_refused() {
    let mut harness = Harness::new("a_line_the_railway_cannot_afford_is_refused");
    harness.set_year(1918);

    // The longest line still on paper, which the opening balance cannot reach.
    // Several of the shorter ones it can, so the test has to name this one.
    let (segment, cost) = {
        let rail = harness.app.world().resource::<Rail>();
        let tariff = harness.app.world().resource::<Counting>().tariff;
        let longest =
            rail.projected().max_by_key(|s| s.track_m).expect("something is still on paper");
        (longest.id, tariff.track(longest.track_m))
    };
    let before = harness.balance();
    assert!(cost > before, "this test needs a line the railway cannot afford");

    harness.request(UiAction::OpenLine(segment));
    harness.tick(1.0 / 60.0);

    assert_eq!(harness.balance(), before, "a refused purchase must not move the balance");
    let rail = harness.app.world().resource::<Rail>();
    assert!(!rail.is_open(segment, 1918));
}
