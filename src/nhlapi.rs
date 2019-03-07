//! Docs: https://gitlab.com/dword4/nhlapi

use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone)]
pub struct Season {
    pub begin: u32,
    pub end: u32,
}

impl<'de> Deserialize<'de> for Season {
    fn deserialize<D>(deserializer: D) -> Result<Season, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.len() == 8 && s.chars().all(|c| c.is_digit(10)) {
            Ok(Season {
                begin: s[..4].parse().unwrap(),
                end: s[4..].parse().unwrap(),
            })
        } else {
            Err(serde::de::Error::custom("invalid season"))
        }
    }
}
impl Serialize for Season {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:04}{:04}", self.begin, self.end))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LeagueRecord {
    pub wins: u32,
    pub losses: u32,
    pub ot: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Team {
    pub id: u32,
    pub name: String,
}

pub mod schedule {
    use chrono::{DateTime, Local, NaiveDate, Utc};
    use serde::{Deserialize, Serialize};

    use super::{LeagueRecord, Season, Team};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    struct Root {
        pub dates: Vec<Date>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Date {
        pub date: NaiveDate,
        pub games: Vec<Game>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Game {
        #[serde(rename = "gamePk")]
        pub game_pk: u64,
        #[serde(rename = "gameType")]
        pub game_type: String,
        pub season: Season,
        #[serde(rename = "gameDate")]
        pub game_date: DateTime<Utc>,
        pub teams: Teams,
    }

    impl Game {
        pub fn home_team(&self) -> &Team {
            &self.teams.home.team
        }
        pub fn away_team(&self) -> &Team {
            &self.teams.away.team
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Teams {
        pub away: TeamRecord,
        pub home: TeamRecord,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct TeamRecord {
        pub team: Team,
        #[serde(rename = "leagueRecord")]
        pub league_record: LeagueRecord,
        pub score: u32,
    }

    pub fn get(date: &NaiveDate) -> reqwest::Result<Date> {
        let date = format!("{}", date.format("%Y-%m-%d"));

        let client = reqwest::Client::new();
        let mut root: Root = client
            .get("https://statsapi.web.nhl.com/api/v1/schedule")
            .query(&[("date", date)])
            .send()?
            .json()?;
        Ok(root.dates.remove(0))
    }

    pub fn get_range(begin: &NaiveDate, end: &NaiveDate) -> reqwest::Result<Vec<Date>> {
        let begin = format!("{}", begin.format("%Y-%m-%d"));
        let end = format!("{}", end.format("%Y-%m-%d"));

        let client = reqwest::Client::new();
        let root: Root = client
            .get("https://statsapi.web.nhl.com/api/v1/schedule")
            .query(&[("startDate", begin), ("endDate", end)])
            .send()?
            .json()?;
        Ok(root.dates)
    }

    pub fn today() -> reqwest::Result<Date> {
        get(&Local::today().naive_local())
    }

    pub fn yesterday() -> reqwest::Result<Date> {
        get(&Local::today().naive_local().pred())
    }
}

pub mod standings {
    use chrono::{Local, NaiveDate};
    use serde::{Deserialize, Serialize};

    use super::{from_str, LeagueRecord, Team};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    struct Root {
        pub records: Vec<Records>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    struct Records {
        #[serde(rename = "teamRecords")]
        pub team_records: Vec<TeamRecord>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct TeamRecord {
        pub team: Team,
        #[serde(rename = "leagueRecord")]
        pub league_record: LeagueRecord,

        #[serde(rename = "goalsAgainst")]
        pub goals_against: u32,
        #[serde(rename = "goalsScored")]
        pub goals_scored: u32,
        pub points: u32,
        pub row: u32,
        #[serde(rename = "gamesPlayed")]
        pub games_played: u32,

        #[serde(rename = "divisionRank", deserialize_with = "from_str")]
        pub division_rank: u32,
        #[serde(rename = "conferenceRank", deserialize_with = "from_str")]
        pub conference_rank: u32,
        #[serde(rename = "leagueRank", deserialize_with = "from_str")]
        pub league_rank: u32,
        #[serde(rename = "wildCardRank", deserialize_with = "from_str")]
        pub wildcard_rank: u32,
    }

    pub fn get(date: &NaiveDate) -> reqwest::Result<Vec<TeamRecord>> {
        let date = format!("{}", date.format("%Y-%m-%d"));
        let client = reqwest::Client::new();
        let mut root: Root = client
            .get("https://statsapi.web.nhl.com/api/v1/standings/byLeague")
            .query(&[("date", date)])
            .send()?
            .json()?;
        Ok(root.records.remove(0).team_records)
    }

    pub fn today() -> reqwest::Result<Vec<TeamRecord>> {
        get(&Local::today().naive_local())
    }

    pub fn yesterday() -> reqwest::Result<Vec<TeamRecord>> {
        get(&Local::today().naive_local().pred())
    }
}

pub mod teams {
    use std::cmp;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    struct Root {
        teams: Vec<Team>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Team {
        pub id: u32,
        #[serde(rename = "name")]
        pub full_name: String,
        #[serde(rename = "abbreviation")]
        pub abbrev: String,
        #[serde(rename = "teamName")]
        pub name: String,
        #[serde(rename = "locationName")]
        pub location: String,
        pub division: Division,
        pub conference: Conference,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Division {
        pub id: u32,
        pub name: String,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Conference {
        pub id: u32,
        pub name: String,
    }

    pub fn get() -> reqwest::Result<Vec<Team>> {
        let client = reqwest::Client::new();
        let root: Root = client
            .get("https://statsapi.web.nhl.com/api/v1/teams")
            .send()?
            .json()?;
        Ok(root.teams)
    }
}
