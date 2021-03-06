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
    #[serde(default)]
    pub ot: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Team {
    pub id: u32,
    pub name: String,
}

pub mod schedule {
    use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};
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
        pub linescore: LineScore,
    }

    impl Game {
        pub fn home_team(&self) -> &Team {
            &self.teams.home.team
        }

        pub fn away_team(&self) -> &Team {
            &self.teams.away.team
        }

        pub fn winner(&self) -> &Team {
            if self.teams.home.score > self.teams.away.score {
                self.home_team()
            } else {
                self.away_team()
            }
        }

        pub fn loser(&self) -> &Team {
            if self.teams.home.score > self.teams.away.score {
                self.away_team()
            } else {
                self.home_team()
            }
        }

        pub fn local_date<T>(&self, tz: &T) -> String
        where
            T: TimeZone,
            <T as TimeZone>::Offset: std::fmt::Display,
        {
            self.game_date.with_timezone(tz).format("%A, %B %d").to_string()
        }

        pub fn local_time<T>(&self, tz: &T) -> String
        where
            T: TimeZone,
            <T as TimeZone>::Offset: std::fmt::Display,
        {
            self.game_date.with_timezone(tz).format("%H:%M").to_string()
        }

        pub fn overtime(&self) -> bool {
            self.linescore.current_period > 3
        }

        pub fn shootout(&self) -> bool {
            self.linescore.current_period > 4
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

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct LineScore {
        #[serde(rename = "currentPeriod")]
        pub current_period: u32,
        pub periods: Vec<Period>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Period {
        #[serde(rename = "periodType")]
        pub period_type: String,
    }

    pub fn get(date: &NaiveDate) -> attohttpc::Result<Date> {
        let sdate = format!("{}", date.format("%Y-%m-%d"));

        let root: Root = attohttpc::get("https://statsapi.web.nhl.com/api/v1/schedule?expand=schedule.linescore")
            .params(&[("date", sdate)])
            .send()?
            .json()?;

        // We there are no games scheduled for a day, for instance around Christmas,
        // there is no dates returned. To make this problem transparent we return
        // a mock date with no games.
        Ok(root.dates.into_iter().next().unwrap_or_else(|| Date {
            date: date.clone(),
            games: vec![],
        }))
    }

    pub fn get_range(team_id: u32, begin: &NaiveDate, end: &NaiveDate) -> attohttpc::Result<Vec<Date>> {
        let begin = format!("{}", begin.format("%Y-%m-%d"));
        let end = format!("{}", end.format("%Y-%m-%d"));

        let root: Root = attohttpc::get("https://statsapi.web.nhl.com/api/v1/schedule?expand=schedule.linescore")
            .params(&[
                ("teamId", format!("{}", team_id)),
                ("startDate", begin),
                ("endDate", end),
            ])
            .send()?
            .json()?;
        Ok(root.dates)
    }

    pub fn today() -> attohttpc::Result<Date> {
        get(&Local::today().naive_local())
    }

    pub fn yesterday() -> attohttpc::Result<Date> {
        get(&Local::today().naive_local().pred())
    }
}

pub mod standings {
    use chrono::{Local, NaiveDate};
    use serde::{Deserialize, Serialize};

    use super::{from_str, LeagueRecord, Team};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    struct Root {
        pub records: Vec<RootRecords>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    struct RootRecords {
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

        pub records: Records,
    }

    impl TeamRecord {
        pub fn format(&self) -> String {
            format!(
                "{}-{}-{}",
                self.league_record.wins, self.league_record.losses, self.league_record.ot
            )
        }

        pub fn last10(&self) -> Option<String> {
            self.records
                .overall_records
                .iter()
                .find(|x| x.kind == "lastTen")
                .map(|x| format!("{}-{}-{}", x.wins, x.losses, x.ot))
        }

        pub fn point_percent(&self) -> String {
            format!("{:.3}", self.points as f64 / (self.games_played * 2) as f64)
        }

        pub fn point_82(&self) -> String {
            format!("{:.0}", (self.points as f64 / self.games_played as f64) * 82.0)
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Records {
        #[serde(rename = "overallRecords")]
        overall_records: Vec<Record>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Record {
        wins: u32,
        losses: u32,
        #[serde(default)]
        ot: u32,
        #[serde(rename = "type")]
        kind: String,
    }

    pub fn get(date: &NaiveDate) -> attohttpc::Result<Vec<TeamRecord>> {
        let date = format!("{}", date.format("%Y-%m-%d"));
        let mut root: Root =
            attohttpc::get("https://statsapi.web.nhl.com/api/v1/standings/byLeague?expand=standings.record")
                .params(&[("date", date)])
                .send()?
                .json()?;
        Ok(root.records.remove(0).team_records)
    }

    pub fn today() -> attohttpc::Result<Vec<TeamRecord>> {
        get(&Local::today().naive_local())
    }

    pub fn yesterday() -> attohttpc::Result<Vec<TeamRecord>> {
        get(&Local::today().naive_local().pred())
    }
}

pub mod teams {
    const SUBREDDITS: &'static str = "\
        anaheimducks
        coyotes
        bostonbruins
        sabres
        calgaryflames
        canes
        hawks
        coloradoavalanche
        bluejackets
        dallasstars
        detroitredwings
        edmontonoilers
        floridapanthers
        losangeleskings
        wildhockey
        habs
        predators
        devils
        newyorkislanders
        rangers
        ottawasenators
        flyers
        penguins
        sanjosesharks
        stlouisblues
        tampabaylightning
        leafs
        canucks
        goldenknights
        caps
        winnipegjets";

    use std::str::FromStr;

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
        pub venue: Venue,
        #[serde(default)]
        pub subreddit: String,
    }

    impl Team {
        pub fn timezone(&self) -> chrono_tz::Tz {
            chrono_tz::Tz::from_str(&self.venue.timezone.id).expect("Unknown timezone")
        }

        pub fn timezone_code(&self) -> &str {
            &self.venue.timezone.code
        }
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

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Venue {
        pub id: Option<u32>,
        pub name: String,
        pub city: String,
        #[serde(rename = "timeZone")]
        pub timezone: TimeZone,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct TimeZone {
        pub id: String,
        pub offset: i32,
        #[serde(rename = "tz")]
        pub code: String,
    }

    pub fn get() -> attohttpc::Result<Vec<Team>> {
        let mut root: Root = attohttpc::get("https://statsapi.web.nhl.com/api/v1/teams")
            .send()?
            .json()?;

        root.teams
            .sort_unstable_by(|left, right| left.full_name.cmp(&right.full_name));

        for (sub, team) in SUBREDDITS.lines().zip(root.teams.iter_mut()) {
            team.subreddit = sub.trim().to_string();
        }

        Ok(root.teams)
    }
}
