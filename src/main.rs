extern crate dotenv;
extern crate envy;
#[macro_use]
extern crate failure;
extern crate graphql_client;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate structopt;
#[macro_use]
extern crate prettytable;
extern crate regex;
mod aha;
mod github;

use structopt::StructOpt;

#[derive(StructOpt)]
struct Command {
    #[structopt(name = "repository")]
    repo: String,
}

#[derive(Deserialize, Debug)]
struct Env {
    github_api_token: String,
    aha_domain: String,
    aha_token: String,
    workflow_repo: String,
    workflow_login: String,
}

fn main() -> Result<(), failure::Error> {
    dotenv::dotenv().ok();
    env_logger::init();

    let config: Env = envy::from_env()?;

    let github = github::GithubEnv {
        github_api_token: config.github_api_token,
        workflow_repo: config.workflow_repo,
        workflow_login: config.workflow_login,
    };
    let list = github::prs(github);
    //for pr in list
    let aha = aha::Aha::new(config.aha_domain, config.aha_token);
    let feature = aha.get_feature("HIVE-6".to_string());
    println!("{:?}", feature);
    if let Ok(ok_feature) = feature {
        // set user
        // set github url
        // set status matches
        println!("ok {:?}", ok_feature);
    } else {

    }
    Ok(())
}
