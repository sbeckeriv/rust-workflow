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

use graphql_client::*;
use structopt::StructOpt;

type URI = String;
type HTML = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.graphql",
    query_path = "src/query_1.graphql",
    response_derives = "Debug"
)]
struct RepoView;

#[derive(StructOpt)]
struct Command {
    #[structopt(name = "repository")]
    repo: String,
}

#[derive(Deserialize, Debug)]
struct Env {
    github_api_token: String,
}

fn parse_repo_name(repo_name: &str) -> Result<(&str, &str, &str), failure::Error> {
    let mut parts = repo_name.split('/');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(owner), Some(name), Some(login)) => Ok((owner, name, login)),
        _ => Err(format_err!("wrong format for the repository name param (we expect something like facebook/graphql/(optional name if not we use the org)"))
    }
}

fn main() -> Result<(), failure::Error> {
    dotenv::dotenv().ok();
    env_logger::init();

    let config: Env = envy::from_env()?;

    let args = Command::from_args();

    let repo = args.repo;
    let (owner, name, user) = parse_repo_name(&repo).unwrap_or(("sbeckeriv-org", "testtest", "sbeckeriv"));

    let q = RepoView::build_query(repo_view::Variables {
        owner: owner.to_string(),
        name: name.to_string(),
        username: user.to_string(),
    });

    let client = reqwest::Client::new();

    println!("{:?}", q);
    let mut res = client
        .post("https://api.github.com/graphql")
        .bearer_auth(config.github_api_token)
        .json(&q)
        .send()?;

    let response_body: Response<repo_view::ResponseData> = res.json()?;
    info!("{:?}", response_body);

    if let Some(errors) = response_body.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
    }

    let response_data: repo_view::ResponseData = response_body.data.expect("missing response data");

    let mut table = prettytable::Table::new();
    println!("{:?}", response_data);
    for pr in &response_data
        .user.expect("missing user")
        .organization.expect("missing org")
        .repository
        .expect("missing repository")
        .pull_requests
        .nodes
        .expect("pr nodes is null")
    {
        if let Some(pr) = pr {
            table.add_row(row!(pr.title, pr.body_text));
        }
    }

    table.printstd();
    Ok(())
}
