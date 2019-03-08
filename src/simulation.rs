use std::cmp::Reverse;
use std::collections::BTreeSet;

use rand::seq::SliceRandom;

use crate::nhlapi;
use crate::nhlapi::schedule::Game;
use crate::nhlapi::standings::TeamRecord;
use crate::nhlapi::teams::Team;
use crate::Api;

#[derive(Debug, Copy, Clone)]
struct Entry {
    team_id: u32,
    division_id: u32,
    conference_id: u32,
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

pub fn pick_ideal_winner<'a>(
    api: &'a Api,
    my_team: &'a Team,
    records: &'a [TeamRecord],
    game: &'a Game,
    times: u32,
) -> &'a nhlapi::Team {
    let mut home_win_sim = Simulation::new(api, my_team, records);
    home_win_sim.give_team_win(game.home_team().id);
    home_win_sim.give_team_loss(game.away_team().id);
    let mut home_win_x = 0;
    for _ in 0..times {
        if home_win_sim.run() {
            home_win_x += 1;
        }
    }

    let mut away_win_sim = Simulation::new(api, my_team, records);
    away_win_sim.give_team_win(game.away_team().id);
    away_win_sim.give_team_loss(game.home_team().id);
    let mut away_win_x = 0;
    for _ in 0..times {
        if away_win_sim.run() {
            away_win_x += 1;
        }
    }

    eprintln!(
        "{} ({}) at {} ({})",
        game.away_team().name,
        away_win_x,
        game.home_team().name,
        home_win_x
    );

    if home_win_x > away_win_x {
        game.home_team()
    } else {
        game.away_team()
    }
}

struct Simulation<'a> {
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
                    conference_id: team.conference.id,
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

    pub fn run(&self) -> bool {
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
