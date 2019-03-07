use std::collections::BTreeSet;

use nhlapi::schedule::Game;
use nhlapi::standings::TeamRecord;
use nhlapi::teams::Team;

mod nhlapi;
mod simulation;

pub struct Api {
    teams: Vec<Team>,
    past_standings: Vec<TeamRecord>,
    standings: Vec<TeamRecord>,
    results: nhlapi::schedule::Date,
    games: nhlapi::schedule::Date,
}

impl Api {
    pub fn get_team_by_abbrev(&self, abbrev: &str) -> &Team {
        let abbrev = abbrev.to_ascii_uppercase();
        self.teams
            .iter()
            .find(|t| t.abbrev == abbrev)
            .expect("team abbrev not found")
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

struct Analyzer<'a> {
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

        Analysis {
            my_team: self.my_team,
            my_game: my_game,
            games: games,
            my_result: my_result,
            results: results,
        }
    }
}

#[derive(Debug)]
struct Analysis<'a> {
    pub my_team: &'a Team,
    pub my_result: Option<Matchup<'a>>,
    pub results: Vec<Matchup<'a>>,
    pub my_game: Option<Matchup<'a>>,
    pub games: Vec<Matchup<'a>>,
}

#[derive(Debug)]
struct Matchup<'a> {
    pub game: &'a Game,
    pub is_result: bool,
    pub is_my_team_involed: bool,
    pub ideal_winner: &'a nhlapi::Team,
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

        let ideal_winner = if self.is_my_team_involed {
            if a.my_team.id == home_team.id {
                home_team
            } else if a.my_team.id == away_team.id {
                away_team
            } else {
                panic!("unexpected case in pick_winner");
            }
        } else if a.own_conference_team_ids.contains(&home_team.id)
            && !a.own_conference_team_ids.contains(&away_team.id)
        {
            away_team
        } else if a.own_conference_team_ids.contains(&away_team.id)
            && !a.own_conference_team_ids.contains(&home_team.id)
        {
            home_team
        } else {
            // TODO: simulation
            home_team
        };

        Matchup {
            game: self.game,
            is_result: self.is_result,
            is_my_team_involed: self.is_my_team_involed,
            ideal_winner: ideal_winner,
        }
    }
}

fn main() -> reqwest::Result<()> {
    let teams = nhlapi::teams::get()?;
    let past_standings = nhlapi::standings::yesterday()?;
    let standings = nhlapi::standings::today()?;
    let results = nhlapi::schedule::yesterday()?;
    let games = nhlapi::schedule::today()?;

    let api = Api {
        teams,
        past_standings,
        standings,
        results,
        games,
    };

    let a = Analyzer::new(&api, api.get_team_by_abbrev("mtl"));
    let xd = a.perform();

    println!("{:#?}", xd);

    Ok(())
}
