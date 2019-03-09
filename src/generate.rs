use std::iter;

use crate::analysis::{Analysis, Api, Matchup, PlayoffMatchup, Seed};
use crate::markdown::*;
use crate::nhlapi::{self, standings::TeamRecord};
use crate::simulation;

pub struct MarkdownGenerator<'a> {
    api: &'a Api,
    an: &'a Analysis<'a>,
}

impl MarkdownGenerator<'_> {
    pub fn new<'a>(api: &'a Api, an: &'a Analysis<'a>) -> MarkdownGenerator<'a> {
        MarkdownGenerator { api, an }
    }

    fn fmt_team(&self, team: &nhlapi::Team) -> String {
        let team = self.api.get_team_by_id(team.id);
        format!("[](/r/{}){}", team.subreddit, team.abbrev)
    }

    fn fmt_vs(&self, home_team: &nhlapi::Team, away_team: &nhlapi::Team) -> String {
        format!(
            "{} at {}",
            self.fmt_team(away_team),
            self.fmt_team(home_team)
        )
    }

    fn fmt_seed(&self, record: &TeamRecord) -> String {
        format!(
            "{} ({})",
            self.fmt_team(&record.team),
            record.conference_rank
        )
    }

    fn make_result_table<'a>(&self, matchups: impl Iterator<Item = &'a Matchup<'a>>) -> Table {
        let mut table = Table::new(&["Game", "Score", "Overtime"]);
        for m in matchups {
            table.add(&[
                self.fmt_vs(m.game.home_team(), m.game.away_team()),
                format!(
                    "{}-{} {}",
                    m.game.teams.home.score,
                    m.game.teams.away.score,
                    self.fmt_team(m.game.winner())
                ),
                m.get_mood().to_string(),
            ]);
        }
        table
    }

    fn make_game_table<'a>(&self, games: impl Iterator<Item = &'a Matchup<'a>>) -> Table {
        let mut table = Table::new(&["Game", "Cheer for", "Time"]);
        for m in games {
            table.add(&[
                self.fmt_vs(m.game.home_team(), m.game.away_team()),
                self.fmt_team(m.cheer_for()),
                m.game.local_time(),
            ]);
        }
        table
    }

    fn make_standings_table(&self, seeds: &[Seed], wildcard: bool) -> Table {
        let mut table = Table::new(&[
            "Place", "Team", "GP", "Record", "Points", "ROW", "L10", "P%", "P-82",
        ]);
        for (index, seed) in seeds.iter().enumerate() {
            let record = &seed.record;

            if index == 2 && wildcard {
                table.add(&["-", "-", "-", "-", "-", "-", "-", "-", "-"]);
            }

            table.add(&[
                format!("{}", seed.seed),
                self.fmt_team(&record.team),
                format!("{}", record.games_played),
                record.format(),
                format!("{}", record.points),
                format!("{}", record.row),
                record.last10().unwrap_or("".into()),
                record.point_percent(),
                record.point_82(),
            ]);
        }
        table
    }

    fn make_playoffs_table(&self, playoffs: &[PlayoffMatchup]) -> Table {
        let mut table = Table::new(&["High seed", "", "Low seed"]);
        for pm in playoffs {
            table.add(&[
                self.fmt_seed(&pm.high_team),
                "vs".to_string(),
                self.fmt_seed(&pm.low_team),
            ]);
        }
        table
    }

    pub fn markdown(&self) -> Document {
        let mut doc = Document::new();
        doc.add(H1::new("Playoffs race!"));

        let today_odds = simulation::odds_for_team(self.api, self.an.my_team, false);

        doc.add(Paragraph::new(format!(
            "Playoffs odds today: {:.1}%",
            today_odds * 100.0
        )));

        //
        // Last night
        //
        doc.add(H2::new("Last night's race"));

        doc.add(List::from(&["Our race:"]));
        if let Some(my_result) = &self.an.my_result {
            doc.add(self.make_result_table(iter::once(my_result)));
        } else {
            doc.add(Paragraph::new("Nothing"));
        }

        doc.add(List::from(&["Outside of town"]));
        doc.add(self.make_result_table(self.an.results.iter()));

        //
        // Standings
        //
        doc.add(H2::new("Standings"));
        doc.add(self.make_standings_table(&self.an.own_division_seed, false));
        doc.add(self.make_standings_table(&self.an.other_division_seed, false));
        doc.add(self.make_standings_table(&self.an.wildcard_seed, true));

        //
        // Playoffs matchups
        //
        doc.add(H2::new("Playoffs matchups"));
        doc.add(self.make_playoffs_table(&self.an.playoffs));

        //
        // Tonight
        //
        doc.add(H2::new("Tonight's race"));

        doc.add(List::from(&["Our race:"]));
        if let Some(my_game) = &self.an.my_game {
            doc.add(self.make_game_table(iter::once(my_game)));
        } else {
            doc.add(Paragraph::new("Nothing"));
        }

        doc.add(List::from(&["Outside of town"]));
        doc.add(self.make_game_table(self.an.games.iter()));

        doc
    }
}
