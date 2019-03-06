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
    wins: u32,
    losses: u32,
    ot: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Team {
    id: u32,
    name: String,
}

pub mod schedule {
    use chrono::{DateTime, NaiveDate, Utc};
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
    }

    pub fn today() -> reqwest::Result<Vec<Date>> {
        let root: Root = reqwest::get("https://statsapi.web.nhl.com/api/v1/schedule")?.json()?;
        Ok(root.dates)
    }
}

pub mod standings {
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
        team: Team,
        #[serde(rename = "leagueRecord")]
        league_record: LeagueRecord,

        #[serde(rename = "goalsAgainst")]
        goals_against: u32,
        #[serde(rename = "goalsScored")]
        goals_scored: u32,
        points: u32,
        row: u32,
        #[serde(rename = "gamesPlayed")]
        games_played: u32,

        #[serde(rename = "divisionRank", deserialize_with = "from_str")]
        division_rank: u32,
        #[serde(rename = "conferenceRank", deserialize_with = "from_str")]
        conference_rank: u32,
        #[serde(rename = "leagueRank", deserialize_with = "from_str")]
        league_rank: u32,
        #[serde(rename = "wildCardRank", deserialize_with = "from_str")]
        wildcard_rank: u32,
    }

    pub fn today() -> reqwest::Result<Vec<TeamRecord>> {
        let mut root: Root =
            reqwest::get("https://statsapi.web.nhl.com/api/v1/standings/byLeague")?.json()?;
        Ok(root.records.remove(0).team_records)
    }
}
