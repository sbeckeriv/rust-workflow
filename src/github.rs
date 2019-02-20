pub struct GithubEnv {
    pub github_api_token: String,
    pub workflow_repo: String,
    pub workflow_login: String,
}
use graphql_client::*;
type HTML = String;
type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.graphql",
    query_path = "src/query_1.graphql",
    response_derives = "Debug"
)]
struct RepoView;
#[derive(Debug)]
pub struct PullRequest {
    pub number: i64,
    pub name: String,
    pub url: String,
    pub labels: Vec<String>,
}
impl PullRequest {
    pub fn status_for_labels(&self) -> Option<String> {
        self.labels
            .iter()
            .map(|label| match label.as_ref() {
                "In development" => Some("".to_string()),
                "Needs code review" => Some("".to_string()),
                "Needs PM review" => Some("".to_string()),
                "Ready" => Some("Ready to ship".to_string()),
                _ => None,
            })
            .filter(|label| label.is_some())
            .nth(1)
            .unwrap_or(None)
    }
}

fn parse_repo_name(repo_name: &str) -> Result<(&str, &str), failure::Error> {
    let mut parts = repo_name.split('/');
    match (parts.next(), parts.next()) {
        (Some(owner), Some(name)) => Ok((owner, name)),
        _ => Err(format_err!("wrong format for the repository name param (we expect something like facebook/graphql/(optional name if not we use the org)"))
    }
}

pub fn prs(config: GithubEnv) -> Result<Vec<PullRequest>, failure::Error> {
    let (owner, name) =
        parse_repo_name(&config.workflow_repo).unwrap_or(("sbeckeriv-org", "testtest"));

    let q = RepoView::build_query(repo_view::Variables {
        owner: owner.to_string(),
        name: name.to_string(),
        username: config.workflow_login.to_string(),
    });

    let client = reqwest::Client::new();

    //println!("{:?}", q);
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
    let mut branches: Vec<PullRequest> = Vec::new();
    let mut table = prettytable::Table::new();
    //println!("{:?}", response_data);
    for pr in &response_data
        .user
        .expect("missing user")
        .organization
        .expect("missing org")
        .repository
        .expect("missing repository")
        .pull_requests
        .nodes
        .expect("pr nodes is null")
    {
        if let Some(pr) = pr {
            let mut ref_head = "".to_string();
            let mut label_names: Vec<String> = Vec::new();
            if let Some(head) = &pr.head_ref {
                ref_head = head.name.clone();
            }
            if let Some(edges) = &pr.labels {
                for labels in &edges.edges {
                    for label in labels {
                        if let Some(l) = label {
                            if let Some(node) = &l.node {
                                label_names.push(node.name.clone());
                            }
                        }
                    }
                }
            }

            table.add_row(row!(
                pr.title,
                pr.body_text,
                ref_head,
                label_names.join(","),
                pr.url,
            ));
            let pull = PullRequest {
                number: pr.number,
                url: pr.url.clone(),
                name: ref_head,
                labels: label_names,
            };
            branches.push(pull);
        }
    }

    table.printstd();
    Ok(branches)
}
