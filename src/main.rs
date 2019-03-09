#![allow(dead_code)]

mod analysis;
mod generate;
mod markdown;
mod nhlapi;
mod simulation;

use analysis::{Analyzer, Api};
use generate::MarkdownGenerator;

fn main() -> reqwest::Result<()> {
    let api = Api::download();

    // let teams = ["mtl", "car", "cbj"];
    // for abbrev in &teams {
    //     let team = api.get_team_by_abbrev(abbrev);
    //     println!(
    //         "{} {:.3}",
    //         team.abbrev,
    //         simulation::odds_for_team(&api, team, 50_000)
    //     );
    // }

    let analyzer = Analyzer::new(&api, api.get_team_by_abbrev("mtl"));
    let an = analyzer.perform();
    let gen = MarkdownGenerator::new(&api, &an);
    println!("{}", gen.markdown().as_str());

    Ok(())
}
