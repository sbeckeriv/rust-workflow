pub struct Aha {
    pub domain: String,
    pub client: reqwest::Client,
}

impl Aha {
    pub fn new(domain: String, auth_token: String) -> Aha {
        let mut headers = reqwest::header::HeaderMap::new();
        let mut auth =
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", auth_token)).unwrap();
        auth.set_sensitive(true);
        headers.insert(reqwest::header::AUTHORIZATION, auth);
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("Rust aha api v1 (Becker@aha.io)"),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        let client = reqwest::Client::builder()
            .gzip(true)
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(50))
            .build()
            .unwrap();
        Aha {
            client: client,
            domain: domain,
        }
    }

    pub fn get_requirement(&self, url: String) -> Result<Requirements, serde_json::Error> {
        let uri = format!("https://{}.aha.io/api/v1/requirements/{}", self.domain, url);
        let response = self.client.get(&uri).send();
        let content = response.unwrap().text();
        println!("{:?}", content);
        let requirement: Result<RequirementRootInterface, _> =
            serde_json::from_str(&content.unwrap_or("".to_string()));
        if let Ok(req) = requirement {
            Ok(req.requirement)
        } else {
            let ex: Result<Requirements, serde_json::Error> = Err(requirement.unwrap_err());
            ex
        }
    }

    pub fn get_feature(&self, url: String) -> Result<Feature, serde_json::Error> {
        let uri = format!("https://{}.aha.io/api/v1/features/{}", self.domain, url);
        let response = self.client.get(&uri).send();
        let content = response.unwrap().text();
        println!("{:?}", content);
        let feature: Result<FeatureRootInterface, _> =
            serde_json::from_str(&content.unwrap_or("".to_string()));
        if let Ok(fe) = feature {
            Ok(fe.feature)
        } else {
            let ex: Result<Feature, serde_json::Error> = Err(feature.unwrap_err());
            ex
        }
    }
}

#[derive(Serialize, Debug, Deserialize)]
pub struct AssignedToUser {
    id: String,
    name: String,
    email: String,
    created_at: String,
    updated_at: String,
    default_assignee: bool,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct Attachments {
    id: String,
    download_url: String,
    created_at: String,
    updated_at: String,
    content_type: String,
    file_name: String,
    file_size: i64,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct CustomFields {
    key: String,
    name: String,
    value: String,
    #[serde(rename = "type")]
    _type: String,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct Description {
    id: String,
    body: String,
    created_at: String,
    attachments: Vec<Attachments>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct Feature {
    id: String,
    name: String,
    reference_num: String,
    position: i64,
    score: i64,
    created_at: String,
    updated_at: String,
    start_date: Option<String>,
    due_date: Option<String>,
    product_id: String,
    workflow_kind: WorkflowKind,
    workflow_status: WorkflowStatus,
    description: Description,
    attachments: Vec<Attachments>,
    integration_fields: Vec<IntegrationFields>,
    url: String,
    resource: String,
    release: Release,
    master_feature: Option<MasterFeature>,
    created_by_user: Owner,
    assigned_to_user: Option<AssignedToUser>,
    requirements: Vec<Requirements>,
    initiative: Option<Initiative>,
    goals: Vec<Goals>,
    comments_count: i64,
    score_facts: Vec<ScoreFacts>,
    tags: Vec<String>,
    full_tags: Vec<FullTags>,
    custom_fields: Vec<CustomFields>,
    feature_links: Vec<FeatureLinks>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct ScoreFacts {
    name: String,
    value: i64,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct Feature1 {
    id: String,
    reference_num: String,
    name: String,
    created_at: String,
    url: String,
    resource: String,
    product_id: String,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct FeatureLinks {
    link_type: String,
    link_type_id: String,
    created_at: String,
    parent_record: Feature1,
    child_record: Feature1,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct FullTags {
    id: i64,
    name: String,
    color: String,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct Goals {
    id: String,
    name: String,
    url: String,
    resource: String,
    created_at: String,
    description: Description,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct Initiative {
    id: String,
    name: String,
    url: String,
    resource: String,
    created_at: String,
    description: Description,
    integration_fields: Vec<IntegrationFields>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct IntegrationFields {
    id: String,
    name: String,
    value: Option<String>,
    integration_id: String,
    service_name: String,
    created_at: String,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct MasterFeature {
    id: String,
    reference_num: String,
    name: String,
    created_at: String,
    url: String,
    resource: String,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct Owner {
    id: String,
    name: String,
    email: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct Project {
    id: String,
    reference_prefix: String,
    name: String,
    product_line: bool,
    created_at: String,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct Release {
    id: String,
    reference_num: String,
    name: String,
    start_date: String,
    release_date: String,
    parking_lot: bool,
    created_at: String,
    product_id: String,
    integration_fields: Vec<IntegrationFields>,
    url: String,
    resource: String,
    owner: Owner,
    project: Project,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct Requirements {
    id: String,
    name: String,
    reference_num: String,
    position: i64,
    created_at: String,
    updated_at: String,
    release_id: String,
    workflow_status: WorkflowStatus,
    url: String,
    resource: String,
    description: Description,
    feature: Feature1,
    assigned_to_user: Option<AssignedToUser>,
    created_by_user: Owner,
    attachments: Vec<Attachments>,
    custom_fields: Vec<CustomFields>,
    integration_fields: Vec<IntegrationFields>,
    comments_count: i64,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct RequirementRootInterface {
    requirement: Requirements,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct FeatureRootInterface {
    feature: Feature,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct WorkflowKind {
    id: String,
    name: String,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct WorkflowStatus {
    id: String,
    name: String,
    position: i64,
    complete: bool,
    color: String,
}
