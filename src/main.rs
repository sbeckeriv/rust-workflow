extern crate dotenv;
extern crate envy;
#[macro_use]
extern crate failure;
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
extern crate notify_rust;
extern crate regex;
use structopt::StructOpt;
mod aha;
mod github;

#[derive(StructOpt, Debug)]
pub struct Opt {
    #[structopt(short = "r", long = "repo", name = "repo")]
    repo: String,
    #[structopt(short = "d", long = "dryrun")]
    dry_run: bool,
    #[structopt(short = "s", long = "silent")]
    silent: bool,
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
}

#[derive(Deserialize, Debug)]
struct Env {
    github_api_token: String,
    aha_domain: String,
    aha_token: String,
    workflow_repo: String,
    workflow_login: String,
    workflow_email: String,
}

fn main() -> Result<(), failure::Error> {
    let opt = Opt::from_args();
    if !opt.silent {
        println!("{:?}", opt);
    }
    dotenv::dotenv().ok();
    env_logger::init();

    let config: Env = envy::from_env()?;

    let github = github::GithubEnv {
        github_api_token: config.github_api_token,
        workflow_repo: opt.repo.clone(),
        workflow_login: config.workflow_login,
        silent: opt.silent,
        verbose: opt.verbose,
    };
    let list = github::prs(github).unwrap();
    let aha = aha::Aha::new(
        config.aha_domain,
        config.aha_token,
        config.workflow_email,
        opt,
    );
    for pr in list {
        aha.sync_pr(pr).unwrap();
    }
    Ok(())
}
