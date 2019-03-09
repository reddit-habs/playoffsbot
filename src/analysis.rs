use std::cmp::Reverse;
use std::collections::BTreeSet;

use crate::nhlapi::{self, schedule::Game, standings::TeamRecord, teams::Team};
use crate::simulation;

pub struct Api {
    pub teams: Vec<Team>,
    pub past_standings: Vec<TeamRecord>,
    pub standings: Vec<TeamRecord>,
    pub results: nhlapi::schedule::Date,
    pub games: nhlapi::schedule::Date,
}

impl Api {
    pub fn download() -> Api {
        let teams = nhlapi::teams::get().expect("error getting teams");
        let past_standings = nhlapi::standings::yesterday().expect("error getting past standings");
        let standings = nhlapi::standings::today().expect("error getting standings");
        let results = nhlapi::schedule::yesterday().expect("error getting results");
        let games = nhlapi::schedule::today().expect("error getting games");

        Api {
            teams,
            past_standings,
            standings,
            results,
            games,
        }
    }

    pub fn get_team_by_abbrev(&self, abbrev: &str) -> &Team {
        let abbrev = abbrev.to_ascii_uppercase();
        self.teams
            .iter()
            .find(|t| t.abbrev == abbrev)
            .expect("team abbrev not found")
    }

    pub fn get_team_by_id(&self, team_id: u32) -> &Team {
        self.teams
            .iter()
            .find(|t| t.id == team_id)
            .expect("team id not found")
    }

    pub fn get_points(&self, team_id: u32, past: bool) -> u32 {
        if !past {
            self.standings
                .iter()
                .find(|t| t.team.id == team_id)
                .expect("team id not found")
                .points
        } else {
            self.past_standings
                .iter()
                .find(|t| t.team.id == team_id)
                .expect("team id not found")
                .points
        }
    }
}

pub struct Analyzer<'a> {
    api: &'a Api,
    my_team: &'a Team,
    own_conference_team_ids: BTreeSet<u32>,
}

impl Analyzer<'_> {
    pub fn new<'a>(api: &'a Api, my_team: &'a Team) -> Analyzer<'a> {
        let mut own_conference_team_ids = BTreeSet::new();
        for team in &api.teams {
            if team.conference.id == my_team.conference.id {
                own_conference_team_ids.insert(team.id);
            }
        }
        Analyzer {
            api,
            my_team,
            own_conference_team_ids,
        }
    }

    pub fn perform(&self) -> Analysis {
        let mut my_game = None;
        let mut games = vec![];
        let mut my_result = None;
        let mut results = vec![];

        for game in &self.api.games.games {
            let m = MatchupPre::create(self, game, false);
            if m.is_relevant(self) {
                if m.is_my_team_involed {
                    my_game = Some(m.pick_winner(self));
                } else {
                    games.push(m.pick_winner(self));
                }
            }
        }

        for game in &self.api.results.games {
            let m = MatchupPre::create(self, game, true);
            if m.is_relevant(self) {
                if m.is_my_team_involed {
                    my_result = Some(m.pick_winner(self));
                } else {
                    results.push(m.pick_winner(self));
                }
            }
        }

        let mut own_division_seed = vec![];
        let mut other_division_seed = vec![];
        let mut wildcard_seed = vec![];
        for record in &self.api.standings {
            if self.own_conference_team_ids.contains(&record.team.id) {
                let team = self.api.get_team_by_id(record.team.id);

                if team.division.id == self.my_team.division.id {
                    if own_division_seed.len() < 3 {
                        own_division_seed.push(Seed {
                            seed: own_division_seed.len() as u32 + 1,
                            record: record,
                        });
                    } else {
                        wildcard_seed.push(Seed {
                            seed: wildcard_seed.len() as u32 + 1,
                            record: record,
                        })
                    }
                } else {
                    if other_division_seed.len() < 3 {
                        other_division_seed.push(Seed {
                            seed: other_division_seed.len() as u32 + 1,
                            record: record,
                        });
                    } else {
                        wildcard_seed.push(Seed {
                            seed: wildcard_seed.len() as u32 + 1,
                            record: record,
                        })
                    }
                }
            }
        }

        let mut tops = vec![&own_division_seed[0], &other_division_seed[0]];
        tops.sort_unstable_by_key(|s| Reverse(s.record.points));

        let playoffs = vec![
            PlayoffMatchup::new(&tops[0].record, &wildcard_seed[1].record),
            PlayoffMatchup::new(&tops[1].record, &wildcard_seed[0].record),
            PlayoffMatchup::new(&own_division_seed[1].record, &own_division_seed[2].record),
            PlayoffMatchup::new(
                &other_division_seed[1].record,
                &other_division_seed[2].record,
            ),
        ];

        Analysis {
            my_team: self.my_team,
            my_game: my_game,
            games: games,
            my_result: my_result,
            results: results,
            own_division_seed,
            other_division_seed,
            wildcard_seed,
            playoffs,
        }
    }
}

#[derive(Debug)]
pub struct Seed<'a> {
    pub seed: u32,
    pub record: &'a TeamRecord,
}

#[derive(Debug)]
pub struct PlayoffMatchup<'a> {
    pub high_team: &'a TeamRecord,
    pub low_team: &'a TeamRecord,
}

impl PlayoffMatchup<'_> {
    fn new<'a>(high_team: &'a TeamRecord, low_team: &'a TeamRecord) -> PlayoffMatchup<'a> {
        PlayoffMatchup {
            high_team,
            low_team,
        }
    }
}

#[derive(Debug)]
pub struct Analysis<'a> {
    pub my_team: &'a Team,
    pub my_result: Option<Matchup<'a>>,
    pub results: Vec<Matchup<'a>>,
    pub my_game: Option<Matchup<'a>>,
    pub games: Vec<Matchup<'a>>,
    pub own_division_seed: Vec<Seed<'a>>,
    pub other_division_seed: Vec<Seed<'a>>,
    pub wildcard_seed: Vec<Seed<'a>>,
    pub playoffs: Vec<PlayoffMatchup<'a>>,
}

#[derive(Debug)]
pub struct Matchup<'a> {
    pub game: &'a Game,
    pub is_result: bool,
    pub is_my_team_involed: bool,
    pub ideal_loser: &'a nhlapi::Team,
}

impl Matchup<'_> {
    pub fn cheer_for(&self) -> &nhlapi::Team {
        if self.game.home_team().id == self.ideal_loser.id {
            self.game.away_team()
        } else if self.game.away_team().id == self.ideal_loser.id {
            self.game.home_team()
        } else {
            panic!("invalid match loser")
        }
    }

    pub fn get_mood(&self) -> &str {
        if self.game.loser().id == self.ideal_loser.id {
            if self.game.overtime() {
                "Good"
            } else {
                "Great"
            }
        } else {
            "Bad"
        }
    }
}

struct MatchupPre<'a> {
    pub game: &'a Game,
    pub is_result: bool,
    pub is_my_team_involed: bool,
}

impl<'m> MatchupPre<'m> {
    pub fn create<'a>(a: &'a Analyzer, game: &'a Game, is_result: bool) -> MatchupPre<'a> {
        let is_my_team_involed =
            game.teams.away.team.id == a.my_team.id || game.teams.home.team.id == a.my_team.id;
        MatchupPre {
            game,
            is_result,
            is_my_team_involed,
        }
    }

    pub fn is_relevant(&self, a: &Analyzer) -> bool {
        self.is_my_team_involed
            || a.own_conference_team_ids
                .contains(&self.game.home_team().id)
            || a.own_conference_team_ids
                .contains(&self.game.away_team().id)
    }

    pub fn pick_winner(self, a: &'m Analyzer) -> Matchup<'m> {
        let home_team = self.game.home_team();
        let away_team = self.game.away_team();

        let ideal_loser = if self.is_my_team_involed {
            if a.my_team.id == home_team.id {
                away_team
            } else if a.my_team.id == away_team.id {
                home_team
            } else {
                panic!("unexpected case in pick_winner");
            }
        } else if a.own_conference_team_ids.contains(&home_team.id)
            && !a.own_conference_team_ids.contains(&away_team.id)
        {
            home_team
        } else if a.own_conference_team_ids.contains(&away_team.id)
            && !a.own_conference_team_ids.contains(&home_team.id)
        {
            away_team
        } else {
            if self.is_result {
                simulation::pick_ideal_loser(a.api, a.my_team, &a.api.past_standings, self.game)
            } else {
                simulation::pick_ideal_loser(a.api, a.my_team, &a.api.standings, self.game)
            }
        };

        Matchup {
            game: self.game,
            is_result: self.is_result,
            is_my_team_involed: self.is_my_team_involed,
            ideal_loser,
        }
    }
}
