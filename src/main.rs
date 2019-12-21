#![allow(dead_code)]

mod analysis;
mod generate;
mod markdown;
mod nhlapi;
mod simulation;

use std::fs::File;
use std::io::prelude::*;

use chrono::{Datelike, Local, NaiveDate, TimeZone};
use failure::Error;
use ordinal::Ordinal;
use serde::{Deserialize, Serialize};

use analysis::{Analyzer, Api};
use generate::MarkdownGenerator;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    client_id: String,
    client_secret: String,
    username: String,
    password: String,
    user_agent: String,
    playoffs: Vec<String>,
    #[serde(default)]
    test: bool,
}

fn get_season_year(today: &NaiveDate) -> i32 {
    if today.month() < 7 {
        today.year()
    } else {
        today.year() + 1
    }
}

fn main() -> Result<(), Error> {
    let api = Api::download();

    let config_file = File::open("config.json")?;
    let config: Config = serde_json::from_reader(config_file)?;

    for abbrev in config.playoffs {
        let team = api.get_team_by_abbrev(&abbrev);
        let analyzer = Analyzer::new(&api, team);
        let an = analyzer.perform();

        let today = Local::today().naive_local();
        let season_end = Local.ymd(get_season_year(&today), 5, 1).naive_local();

        let schedule = nhlapi::schedule::get_range(team.id, &today, &season_end)?;

        let gen = MarkdownGenerator::new(&api, &an, &schedule, &team);
        let doc = gen.markdown();

        if config.test {
            let mut file = File::create(&format!("{}.md", team.abbrev))?;
            write!(file, "{}", doc.as_str())?;
        } else {
            let mut reddit = orca::App::new("tankbot", "1.0", "sbstp")?;
            reddit.authorize_script(
                &config.client_id,
                &config.client_secret,
                &config.username,
                &config.password,
            )?;

            let today = Local::today();
            reddit.submit_self(
                &team.subreddit,
                &format!(
                    "Playoffs Race: {} {}, {}",
                    today.format("%B"),
                    Ordinal(today.day()),
                    today.format("%Y")
                ),
                doc.as_str(),
                false,
            )?;
        }
    }

    Ok(())
}

#[test]
fn test_get_season_year() {
    assert_eq!(
        get_season_year(&Local.ymd(2019, 03, 15).naive_local()),
        2019
    );
    assert_eq!(
        get_season_year(&Local.ymd(2018, 11, 15).naive_local()),
        2019
    );
}
