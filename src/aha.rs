use super::github;
use super::Opt;
use notify_rust::Notification;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

pub struct Aha<'a> {
    pub domain: String,
    pub client: reqwest::Client,
    pub user_email: String,
    pub opt: &'a Opt,
}

trait UpdateJson {
    fn to_json(&self) -> FeatureUpdate;
}

impl<'a> Aha<'a> {
    pub fn status_for_labels(
        &self,
        labels: Vec<String>,
        config_labels: Option<HashMap<String, String>>,
    ) -> Option<String> {
        let mut default_labels = HashMap::new();
        default_labels.insert("In development".to_string(), "In development".to_string());
        default_labels.insert(
            "Needs code review".to_string(),
            "In code review".to_string(),
        );
        default_labels.insert("Needs PM review".to_string(), "In PM review".to_string());
        default_labels.insert("Ready".to_string(), "Ready to ship".to_string());
        labels
            .iter()
            .map(|label| {
                let default = default_labels.get(label);
                let x = match &config_labels {
                    Some(c) => c.get(label).or_else(|| default),
                    None => default,
                };
                match x {
                    Some(c) => Some(c.clone()),
                    None => None,
                }
            })
            .filter(|label| label.is_some())
            .nth(1)
            .unwrap_or(None)
    }
    pub fn new(domain: String, auth_token: String, email: String, opt: &Opt) -> Aha {
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
            user_email: email,
            opt: opt,
        }
    }

    pub fn sync_pr(
        &self,
        pr: github::PullRequest,
        labels: Option<HashMap<String, String>>,
    ) -> Result<(), failure::Error> {
        if let Some((source, key)) = self.type_from_name(&pr.name) {
            if self.opt.verbose {
                println!("matched {} {} {}", pr.name, source, key);
            }
            if source == "feature" {
                match self.get_json(key.clone(), "feature".to_string()) {
                    Ok(feature) => self
                        .update_feature(key.clone(), pr, feature, labels)
                        .unwrap(),
                    Err(error) => println!("Error feature: {}", error),
                }
            } else if source == "requirement" {
                match self.get_json(key.clone(), "requirement".to_string()) {
                    Ok(requirement) => self
                        .update_requirement(key.clone(), pr, requirement, labels)
                        .unwrap(),
                    Err(error) => println!("Error feature: {}", error),
                }
            }
        } else {
            if self.opt.verbose {
                println!("Did not match {}", pr.name);
            }
        }
        Ok(())
    }

    pub fn update_requirement(
        &self,
        key: String,
        pr: github::PullRequest,
        current: Value,
        labels: Option<HashMap<String, String>>,
    ) -> Result<(), serde_json::Error> {
        let uri = format!("https://{}.aha.io/api/v1/requirements/{}", self.domain, key);
        let assigned = if current["assigned_to_user"].is_null() {
            Some(self.user_email.clone())
        } else {
            None
        };

        let count = current["custom_fields"]
            .as_array()
            .unwrap()
            .iter()
            .by_ref()
            .filter(|cf| cf["name"] == "Pull Request")
            .count();

        let custom = if count == 0 {
            Some(CustomFieldGithub {
                github_url: pr.url.clone(),
            })
        } else {
            None
        };

        let mut status = if let Some(wf) = self.status_for_labels(pr.labels, labels) {
            Some(WorkflowStatusUpdate { name: wf })
        } else {
            None
        };
        let current_status = &current["workflow_status"]["name"];
        if status.is_none()
            && (current_status == "Ready to develop" || current_status == "Under consideration")
        {
            status = Some(WorkflowStatusUpdate {
                name: "In code review".to_string(),
            })
        }

        let feature = FeatureUpdate {
            assigned_to_user: assigned,
            custom_fields: custom,
            workflow_status: status,
        };

        let json_string = serde_json::to_string(&feature)?;

        if self.opt.verbose {
            println!("puting requirement json: {}", json_string);
        }

        if !self.opt.silent && json_string.len() > 4 {
            Notification::new()
                .summary(&format!("Updating requirement {}", key))
                .body(&format!("{}\n{}", current["url"], pr.url.clone()))
                .icon("firefox")
                .timeout(0)
                .show()
                .unwrap();
        }
        if !self.opt.dry_run && json_string.len() > 4 {
            let response = self.client.put(&uri).json(&feature).send();
            let content = response.unwrap().text();
            let text = &content.unwrap_or("".to_string());
            if self.opt.verbose {
                println!("updated requirement {:?}", text);
            }
            let feature: Result<Value, _> = serde_json::from_str(&text);

            if let Ok(_) = feature {
                Ok(())
            } else {
                if self.opt.verbose {
                    println!("json failed to parse {:?}", text);
                }
                let ex: Result<(), serde_json::Error> = Err(feature.unwrap_err());
                ex
            }
        } else {
            Ok(())
        }
    }

    pub fn update_feature(
        &self,
        key: String,
        pr: github::PullRequest,
        current: Value,
        labels: Option<HashMap<String, String>>,
    ) -> Result<(), serde_json::Error> {
        let uri = format!("https://{}.aha.io/api/v1/features/{}", self.domain, key);
        let assigned = if current["assigned_to_user"].is_null() {
            Some(self.user_email.clone())
        } else {
            None
        };
        let custom = CustomFieldGithub {
            github_url: pr.url.clone(),
        };
        let mut status = if let Some(wf) = self.status_for_labels(pr.labels, labels) {
            Some(WorkflowStatusUpdate { name: wf })
        } else {
            None
        };

        let current_status = &current["workflow_status"]["name"];

        if status.is_none()
            && (current_status == "Ready to develop" || current_status == "Under consideration")
        {
            status = Some(WorkflowStatusUpdate {
                name: "In code review".to_string(),
            })
        }

        let feature = FeatureUpdate {
            assigned_to_user: assigned,
            custom_fields: Some(custom),
            workflow_status: status,
        };

        let json_string = serde_json::to_string(&feature)?;
        if self.opt.verbose {
            println!("puting feature json: {}", json_string);
        }
        if !self.opt.silent && json_string.len() > 4 {
            Notification::new()
                .summary(&format!("Updating requirement {}", key))
                .body(&format!("{}\n{}", current["url"], pr.url.clone()))
                .icon("firefox")
                .timeout(0)
                .show()
                .unwrap();
        }
        if !self.opt.dry_run && json_string.len() > 4 {
            let response = self.client.put(&uri).json(&feature).send();
            let content = response.unwrap().text();
            let text = &content.unwrap_or("".to_string());
            if self.opt.verbose {
                println!("updated feature {:?}", text);
            }
            let feature: Result<Value, _> = serde_json::from_str(&text);

            if let Ok(_) = feature {
                Ok(())
            } else {
                if self.opt.verbose {
                    println!("json failed to parse {:?}", text);
                }
                let ex: Result<(), serde_json::Error> = Err(feature.unwrap_err());
                ex
            }
        } else {
            Ok(())
        }
    }

    pub fn type_from_name(&self, name: &str) -> Option<(String, String)> {
        //could return enum
        let req = Regex::new(r"^([A-Z]+-\d+-\d+)").unwrap();
        let fet = Regex::new(r"^([A-Z]{1,}-\d{1,})").unwrap();
        let rc = req.captures(&name.trim());
        let fc = fet.captures(&name.trim());
        if let Some(rc) = rc {
            Some(("requirement".to_string(), rc[0].to_string()))
        } else if let Some(fc) = fc {
            Some(("feature".to_string(), fc[0].to_string()))
        } else {
            None
        }
    }

    pub fn get_json(&self, url: String, base: String) -> Result<Value, serde_json::Error> {
        let uri = format!("https://{}.aha.io/api/v1/{}/{}", self.domain, base, url);
        if self.opt.verbose {
            println!("{} url: {}", base, uri);
        }
        let response = self.client.get(&uri).send();
        let content = response.unwrap().text();
        if self.opt.verbose {
            println!("{} text {:?}", base, content);
        }
        let feature: Result<Value, _> = serde_json::from_str(&content.unwrap_or("".to_string()));
        if let Ok(mut fe) = feature {
            Ok(fe[base].take())
        } else {
            let ex: Result<Value, serde_json::Error> = Err(feature.unwrap_err());
            ex
        }
    }
}

// keep
#[derive(Serialize, Debug, Deserialize)]
pub struct FeatureUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    assigned_to_user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_fields: Option<CustomFieldGithub>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workflow_status: Option<WorkflowStatusUpdate>,
}
//keep
#[derive(Serialize, Debug, Deserialize)]
pub struct WorkflowStatusUpdate {
    name: String,
}
// kepp
#[derive(Serialize, Debug, Deserialize)]
pub struct CustomFieldGithub {
    #[serde(rename = "pull_request")]
    github_url: String,
}
