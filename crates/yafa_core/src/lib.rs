//! # Yafa Express - core
//!
//! The rules of the game, with no rendering engine attached: the historical
//! rail network, routing across it, the train's trip, the ticket counter and
//! the traveller's passport.
//!
//! Everything here addresses stations and track by `Copy` handles
//! ([`CityId`], [`SegmentId`]) rather than by reference. The network is built
//! once and borrowed immutably wherever it is read, so the presentation layer
//! can hold ticket state, journey state and passport state side by side
//! without ever tangling two mutable borrows of the same data.
//!
//! ```
//! use yafa_core::{CityId, Journey, JourneyConfig, JourneyEvent, Network, Passport};
//!
//! let network = Network::new();
//! let config = JourneyConfig::default();
//! let route = network.route(CityId::Yafa, CityId::AlQuds, 1892).expect("the 1892 line");
//!
//! let mut journey = Journey::board(route, &config);
//! let mut passport = Passport::new("Traveller");
//! let mut events = Vec::new();
//!
//! while !journey.is_finished() {
//!     events.clear();
//!     journey.advance(1.0 / 60.0, &config, &mut events);
//!     for event in &events {
//!         if let JourneyEvent::Reached { city, .. } = event {
//!             passport.stamp(*city);
//!         }
//!     }
//! }
//!
//! // Yafa hangs off the junction at Al-Ladd, so the traveller collects it too.
//! assert!(passport.is_stamped(CityId::AlLadd));
//! ```

#![forbid(unsafe_code)]

pub mod calendar;
pub mod city;
pub mod coast;
pub mod geo;
pub mod journey;
pub mod network;
pub mod passport;
pub mod ticket;

pub use calendar::{Calendar, FIRST_YEAR, METRES_PER_YEAR};
pub use city::{CITY_COUNT, City, CityId, Label, LabelSide, LineType};
pub use geo::{GeoPoint, MapPoint};
pub use journey::{Journey, JourneyConfig, JourneyEvent, Phase};
pub use network::{Leg, Link, Network, Opening, Route, Segment, SegmentId};
pub use passport::{LineProgress, Passport, Stamp, StampOutcome};
pub use ticket::{DeskAction, TicketDesk};
