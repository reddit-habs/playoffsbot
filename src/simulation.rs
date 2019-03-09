use std::cmp::Reverse;
use std::collections::BTreeSet;

use rand::seq::SliceRandom;

use crate::nhlapi;
use crate::nhlapi::schedule::Game;
use crate::nhlapi::standings::TeamRecord;
use crate::nhlapi::teams::Team;
use crate::Api;

pub const TIMES: u32 = 50_000;

#[derive(Debug, Copy, Clone)]
struct Entry {
    team_id: u32,
    division_id: u32,
    wins: u32,
    losses: u32,
    ot: u32,
    games_played: u32,
    points: u32,
}

#[derive(Debug, Copy, Clone)]
enum Event {
    Win,
    Loss,
    Ot,
}

impl Event {
    fn points(&self) -> u32 {
        match self {
            Event::Win => 2,
            Event::Loss => 0,
            Event::Ot => 1,
        }
    }
}

fn random_event(base: &Entry) -> Event {
    [
        (Event::Win, base.wins),
        (Event::Loss, base.losses),
        (Event::Ot, base.ot),
    ]
    .choose_weighted(&mut rand::thread_rng(), |x| x.1)
    .unwrap()
    .0
}

pub fn odds_for_team<'a>(api: &'a Api, team: &'a Team, past: bool) -> f64 {
    let sim = if !past {
        Simulation::new(api, team, &api.standings)
    } else {
        Simulation::new(api, team, &api.past_standings)
    };
    let x = sim.run_for(TIMES);
    x as f64 / TIMES as f64
}

pub fn pick_ideal_loser<'a>(
    api: &'a Api,
    my_team: &'a Team,
    records: &'a [TeamRecord],
    game: &'a Game,
) -> &'a nhlapi::Team {
    let mut home_win_sim = Simulation::new(api, my_team, records);
    home_win_sim.give_team_win(game.home_team().id);
    home_win_sim.give_team_loss(game.away_team().id);
    let home_win_x = home_win_sim.run_for(TIMES);

    let mut away_win_sim = Simulation::new(api, my_team, records);
    away_win_sim.give_team_win(game.away_team().id);
    away_win_sim.give_team_loss(game.home_team().id);
    let away_win_x = away_win_sim.run_for(TIMES);

    if home_win_x > away_win_x {
        game.away_team()
    } else {
        game.home_team()
    }
}

pub struct Simulation<'a> {
    my_team: &'a Team,
    base: Vec<Entry>,
}

impl Simulation<'_> {
    pub fn new<'a>(api: &'a Api, my_team: &'a Team, records: &'a [TeamRecord]) -> Simulation<'a> {
        let mut base = Vec::new();
        for record in records {
            let team = api.get_team_by_id(record.team.id);
            if team.conference.id == my_team.conference.id {
                base.push(Entry {
                    team_id: team.id,
                    division_id: team.division.id,
                    wins: record.league_record.wins,
                    losses: record.league_record.losses,
                    ot: record.league_record.ot,
                    games_played: record.games_played,
                    points: record.points,
                });
            }
        }
        Simulation { my_team, base }
    }

    pub fn give_team_win(&mut self, team_id: u32) {
        if let Some(entry) = self.base.iter_mut().find(|x| x.team_id == team_id) {
            entry.wins += 1;
            entry.points += 2;
            entry.games_played += 1;
        }
    }

    pub fn give_team_loss(&mut self, team_id: u32) {
        if let Some(entry) = self.base.iter_mut().find(|x| x.team_id == team_id) {
            entry.losses += 1;
            entry.games_played += 1;
        }
    }

    /// Run the simulation for `times` times, and return the number of times
    /// `self.my_team` made the playoffs.
    pub fn run_for(&self, times: u32) -> u32 {
        let mut x = 0;
        for _ in 0..times {
            if self.run() {
                x += 1
            }
        }
        x
    }

    fn run(&self) -> bool {
        let mut entries = self.base.clone();
        for (base, entry) in self.base.iter().zip(entries.iter_mut()) {
            while entry.games_played < 82 {
                let event = random_event(base);
                entry.games_played += 1;
                entry.points += event.points();
                match event {
                    Event::Win => entry.wins += 1,
                    Event::Loss => entry.losses += 1,
                    Event::Ot => entry.ot += 1,
                }
            }
        }

        entries.sort_unstable_by_key(|e| Reverse((e.points, e.wins)));

        let top_3_teams: BTreeSet<u32> = entries
            .iter()
            .filter(|x| x.division_id == self.my_team.division.id)
            .take(3)
            .map(|x| x.team_id)
            .chain(
                entries
                    .iter()
                    .filter(|x| x.division_id != self.my_team.division.id)
                    .take(3)
                    .map(|x| x.team_id),
            )
            .collect();

        let wildcard: BTreeSet<u32> = entries
            .iter()
            .filter(|x| !top_3_teams.contains(&x.team_id))
            .take(2)
            .map(|x| x.team_id)
            .collect();

        top_3_teams.contains(&self.my_team.id) || wildcard.contains(&self.my_team.id)
    }
}
