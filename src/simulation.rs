use rand::seq::SliceRandom;

use crate::nhlapi::LeagueRecord;

#[derive(Debug, Copy, Clone)]
struct Entry {
    team_id: u32,
    division_id: u32,
    conference_id: u32,
    wins: u32,
    losses: u32,
    ot: u32,
    points: u32,
}

#[derive(Debug, Copy, Clone)]
enum Event {
    Win,
    Loss,
    Ot,
}

fn random_event(rec: &LeagueRecord) -> Event {
    [
        (Event::Win, rec.wins),
        (Event::Loss, rec.losses),
        (Event::Ot, rec.ot),
    ]
    .choose_weighted(&mut rand::thread_rng(), |x| x.1)
    .unwrap()
    .0
}

pub struct Simulation {}
