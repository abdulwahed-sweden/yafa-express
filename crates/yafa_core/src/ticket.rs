//! The ticket counter: turning two clicks into a route.

use crate::city::CityId;
use crate::network::{Network, Route};

/// What the clerk did with a station the traveller pointed at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeskAction {
    /// The station became the departure.
    SetOrigin(CityId),
    /// The station became the destination.
    SetDestination(CityId),
    /// The traveller pointed at a station already written on the ticket, and
    /// the clerk crossed it out.
    Cleared(CityId),
    /// Both boxes were full, so a new ticket was started.
    Restarted(CityId),
}

/// The two boxes on a blank ticket.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TicketDesk {
    pub origin: Option<CityId>,
    pub destination: Option<CityId>,
}

impl TicketDesk {
    pub const fn new() -> Self {
        Self { origin: None, destination: None }
    }

    /// Handles a click on a station, filling "From" and then "To".
    pub fn select(&mut self, city: CityId) -> DeskAction {
        match (self.origin, self.destination) {
            // Nothing written yet.
            (None, _) => {
                self.origin = Some(city);
                DeskAction::SetOrigin(city)
            }
            // Clicking the departure again rubs it out.
            (Some(origin), _) if origin == city => {
                self.origin = self.destination.take();
                match self.origin {
                    Some(promoted) => {
                        // The old destination slides up into the From box.
                        self.destination = None;
                        let _ = promoted;
                    }
                    None => self.destination = None,
                }
                DeskAction::Cleared(city)
            }
            // Clicking the destination again rubs that out instead.
            (Some(_), Some(destination)) if destination == city => {
                self.destination = None;
                DeskAction::Cleared(city)
            }
            // Departure set, destination free: fill it in.
            (Some(_), None) => {
                self.destination = Some(city);
                DeskAction::SetDestination(city)
            }
            // Ticket already complete: start a fresh one from here.
            (Some(_), Some(_)) => {
                self.origin = Some(city);
                self.destination = None;
                DeskAction::Restarted(city)
            }
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Both boxes filled with different stations.
    pub fn is_ready(&self) -> bool {
        matches!((self.origin, self.destination), (Some(a), Some(b)) if a != b)
    }

    /// Is this station written on the ticket at all?
    pub fn mentions(&self, city: CityId) -> bool {
        self.origin == Some(city) || self.destination == Some(city)
    }

    /// Issues the ticket, if it is complete and the stations are connected.
    pub fn issue(&self, network: &Network) -> Option<Route> {
        let (origin, destination) = (self.origin?, self.destination?);
        network.route(origin, destination)
    }

    /// The line printed across the top of the ticket.
    pub fn summary(&self, network: &Network) -> String {
        match (self.origin, self.destination) {
            (None, _) => "Select a departure station".to_owned(),
            (Some(origin), None) => {
                format!("From {} - select a destination", network.city(origin).name)
            }
            (Some(origin), Some(destination)) => {
                format!("{} to {}", network.city(origin).name, network.city(destination).name)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_clicks_fill_the_ticket() {
        let mut desk = TicketDesk::new();
        assert_eq!(desk.select(CityId::Yafa), DeskAction::SetOrigin(CityId::Yafa));
        assert!(!desk.is_ready());
        assert_eq!(desk.select(CityId::AlQuds), DeskAction::SetDestination(CityId::AlQuds));
        assert!(desk.is_ready());
    }

    #[test]
    fn a_third_click_starts_a_new_ticket() {
        let mut desk = TicketDesk::new();
        desk.select(CityId::Yafa);
        desk.select(CityId::AlQuds);
        assert_eq!(desk.select(CityId::Haifa), DeskAction::Restarted(CityId::Haifa));
        assert_eq!(desk.origin, Some(CityId::Haifa));
        assert_eq!(desk.destination, None);
    }

    #[test]
    fn clicking_a_written_station_rubs_it_out() {
        let mut desk = TicketDesk::new();
        desk.select(CityId::Yafa);
        assert_eq!(desk.select(CityId::Yafa), DeskAction::Cleared(CityId::Yafa));
        assert_eq!(desk, TicketDesk::new());

        desk.select(CityId::Yafa);
        desk.select(CityId::AlQuds);
        assert_eq!(desk.select(CityId::AlQuds), DeskAction::Cleared(CityId::AlQuds));
        assert_eq!(desk.origin, Some(CityId::Yafa));
        assert_eq!(desk.destination, None);
    }

    #[test]
    fn a_ticket_to_nowhere_is_never_ready() {
        let desk = TicketDesk { origin: Some(CityId::Yafa), destination: Some(CityId::Yafa) };
        assert!(!desk.is_ready());
        assert!(desk.issue(&Network::new()).is_none());
    }

    #[test]
    fn issuing_produces_the_shortest_route() {
        let net = Network::new();
        let mut desk = TicketDesk::new();
        desk.select(CityId::Akka);
        desk.select(CityId::Yafa);
        let route = desk.issue(&net).expect("Akka to Yafa is connected");
        assert_eq!(route.origin(), CityId::Akka);
        assert_eq!(route.destination(), CityId::Yafa);
        assert_eq!(route.total_m, net.route(CityId::Akka, CityId::Yafa).unwrap().total_m);
    }
}
