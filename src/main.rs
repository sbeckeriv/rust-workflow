extern crate dirs;
extern crate dotenv;
extern crate envy;
extern crate termion;
#[macro_use]
extern crate failure;
extern crate env_logger;
extern crate log;
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
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use structopt::StructOpt;
mod aha;
mod github;

#[derive(StructOpt, Debug)]
pub struct Opt {
    #[structopt(short = "r", long = "repo", name = "repo")]
    repo: Option<String>,
    #[structopt(short = "d", long = "dryrun")]
    dry_run: bool,
    #[structopt(short = "s", long = "silent")]
    silent: bool,
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
    #[structopt(short = "c", long = "config")]
    config_file: Option<String>,
    #[structopt(short = "g", long = "generate")]
    generate: bool,
    #[structopt(short = "p", long = "prs")]
    pr_status: bool,
    #[structopt(long = "closed")]
    closed: bool,
}
#[derive(Debug, Deserialize)]
struct Config {
    aha: Option<AhaConfig>,
    global_integer: Option<u64>,
    repos: Option<Vec<RepoConfig>>,
}

#[derive(Debug, Deserialize)]
struct RepoConfig {
    name: String,
    username: String,
    labels: Option<HashMap<String, String>>,
}
#[derive(Debug, Deserialize)]
struct AhaConfig {
    domain: String,
    email: String,
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
    if opt.verbose {
        println!("{:?}", opt);
    }
    let home_dir = dirs::home_dir().expect("Could not find home path");

    let path_name = match &opt.config_file {
        Some(path) => path.clone(),
        None => format!("{}/.aha_workflow", home_dir.display()),
    };

    if opt.verbose {
        println!("{:?}", path_name);
    }
    let config_path = fs::canonicalize(&path_name);
    let config_info: Option<Config> = match config_path {
        Ok(path) => {
            if opt.verbose {
                println!("found {:?}", path_name);
            }
            let display = path.display();
            let mut file = match File::open(&path) {
                Err(why) => panic!("couldn't open {}: {}", display, why.description()),
                Ok(file) => file,
            };

            // Read the file contents into a string, returns `io::Result<usize>`
            let mut s = String::new();
            match file.read_to_string(&mut s) {
                Err(why) => panic!("couldn't read {}: {}", display, why.description()),
                Ok(_) => (),
            }
            Some(toml::from_str(&s)?)
        }
        Err(e) => {
            if !opt.silent {
                println!("did not find {:?}, {}", path_name, e);
            }
            None
        }
    };

    //dotenv::dotenv().ok();
    let my_path = format!("{}/.env", home_dir.display());
    dotenv::from_path(my_path).ok();
    env_logger::init();

    let mut config: Env = envy::from_env()?;

    match config_info.as_ref() {
        Some(c) => match c.aha.as_ref() {
            Some(a) => {
                config.aha_domain = a.domain.clone();
                config.workflow_email = a.email.clone();
            }
            _ => (),
        },
        _ => (),
    }

    if opt.verbose {
        println!("config updated");
    }
    let repos = match config_info {
        Some(c) => c.repos.unwrap(),
        None => vec![RepoConfig {
            name: opt
                .repo
                .clone()
                .expect("Did not pass in required repo param"),
            username: config.workflow_login,
            labels: None,
        }],
    };

    if opt.verbose {
        println!("{:?}", repos);
    }

    let silent = opt.silent.clone();
    let verbose = opt.verbose.clone();

    let aha = aha::Aha::new(
        config.aha_domain,
        config.aha_token,
        config.workflow_email,
        &opt,
    );

    if opt.pr_status {
        for repo in repos {
            let github = github::GithubEnv {
                github_api_token: config.github_api_token.clone(),
                workflow_repo: repo.name.clone(),
                workflow_login: repo.username.clone(),
                silent,
                verbose: verbose.clone(),
            };
            let response_body = github::pr_data(&github, "".to_string(), !opt.closed);
            github::pr_table(&response_body);
        }
    } else if opt.generate {
        let feature = aha.generate().unwrap()["feature"].take();
        println!(
            "{} {}",
            feature["reference_num"].as_str().unwrap(),
            feature["url"].as_str().unwrap()
        );
        println!(
            "git stash; git co master; git co -b {}-{}",
            feature["reference_num"].as_str().unwrap(),
            feature["name"].as_str().unwrap().to_lowercase().replacen(
                char::is_whitespace,
                "-",
                300
            )
        );
    } else {
        for repo in repos {
            let labels = repo.labels;
            let github = github::GithubEnv {
                github_api_token: config.github_api_token.clone(),
                workflow_repo: repo.name.clone(),
                workflow_login: repo.username.clone(),
                silent,
                verbose: verbose.clone(),
            };
            let list = github::prs(github).unwrap();
            for pr in list {
                aha.sync_pr(pr, labels.clone()).unwrap();
            }
        }
    }
    Ok(())
}
