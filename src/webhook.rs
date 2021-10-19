use serde::Deserialize;
use std::env;
use std::io;
use std::path::Path;

#[derive(Deserialize)]
struct Repository {
    url: Option<String>,
}

#[derive(Deserialize)]
struct Project {
    path_with_namespace: Option<String>,
    git_ssh_url: Option<String>,
    git_http_url: Option<String>,
}

#[derive(Deserialize)]
struct Attributes {
    action: Option<String>,
    target_branch: Option<String>,
    source_branch: Option<String>,
    state: Option<String>,
    merge_status: Option<String>,
}

#[derive(Deserialize)]
pub struct Webhook {
    event_type: Option<String>,
    project: Project,
    repository: Repository,
    object_attributes: Attributes,
}

impl Webhook {
    pub fn event_type(&self) -> String {
        match &self.event_type {
            None => "undefined".to_string(),
            Some(value) => value.clone(),
        }
    }

    pub fn project_namespace(&self) -> String {
        match &self.project.path_with_namespace {
            None => "undefined".to_string(),
            Some(value) => {
                let parts = value.split('/').collect::<Vec<&str>>();
                parts[0].to_string()
            }
        }
    }

    pub fn project_name(&self) -> String {
        match &self.project.path_with_namespace {
            None => "undefined".to_string(),
            Some(value) => {
                let parts = value.split('/').collect::<Vec<&str>>();
                parts[parts.len() - 1].to_string()
            }
        }
    }

    pub fn ssh_url(&self) -> String {
        match &self.project.git_ssh_url {
            None => "undefined".to_string(),
            Some(value) => value.clone(),
        }
    }

    pub fn http_url(&self) -> String {
        match &self.project.git_http_url {
            None => "undefined".to_string(),
            Some(value) => value.clone(),
        }
    }

    pub fn repository_url(&self) -> String {
        match &self.repository.url {
            None => "undefined".to_string(),
            Some(value) => value.clone(),
        }
    }

    pub fn action(&self) -> String {
        match &self.object_attributes.action {
            None => "undefined".to_string(),
            Some(value) => value.clone(),
        }
    }

    pub fn target_branch(&self) -> String {
        match &self.object_attributes.target_branch {
            None => "undefined".to_string(),
            Some(value) => value.clone(),
        }
    }

    pub fn source_branch(&self) -> String {
        match &self.object_attributes.source_branch {
            None => "undefined".to_string(),
            Some(value) => value.clone(),
        }
    }

    pub fn state(&self) -> String {
        match &self.object_attributes.state {
            None => "undefined".to_string(),
            Some(value) => value.clone(),
        }
    }

    pub fn merge_status(&self) -> String {
        match &self.object_attributes.merge_status {
            None => "undefined".to_string(),
            Some(value) => value.clone(),
        }
    }

    pub fn clone_repository(&self) -> Result<String, io::Error> {
        let project = "test".to_string();
        let root = Path::new("/var/cache/shook/");
        env::set_current_dir(&root)?;

        let _ = match git2::Repository::clone(&self.repository_url(), &project) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to clone repository: {}", e),
        };

        Ok(format!("/var/cache/shook/{}", project))
    }

    pub fn dump(&self) {
        let log = slog_scope::logger();

        debug!(log, "webhook event"; "event_type" => self.event_type());
        debug!(log, "webhook project";
            "name" => self.project_name(),
            "namespace" => self.project_namespace(),
            "ssh_url" => self.ssh_url(),
            "http_url" => self.http_url(),
        );
        debug!(log, "webhook repository"; "url" => self.repository_url());
        debug!(log, "webhook attributes";
            "action" => self.action(),
            "target_branch" => self.target_branch(),
            "source_branch" => self.source_branch(),
            "state" => self.state(),
            "merge_status" => self.merge_status(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_deserializes() {
        let input = r#"{
            "event_type": "merge_request",
            "project": {
                "path_with_namespace": "user/repo",
                "git_ssh_url": "git@example.com/user/repo.git",
                "git_http_url": "https://example.com/user/repo.git"
            },
            "repository": {
                "url": "git@example.com/user/repo.git"
            },
            "object_attributes": {
                "action": "merge",
                "target_branch": "main",
                "source_branch": "staging",
                "state": "merge",
                "merge_status": "merged"
            }
        }"#;
        let webhook = serde_json::from_str::<Webhook>(input).unwrap();

        assert_eq!(webhook.event_type(), "merge_request".to_string());
        assert_eq!(webhook.project_name(), "repo".to_string());
        assert_eq!(webhook.project_namespace(), "user".to_string());
        assert_eq!(
            webhook.ssh_url(),
            "git@example.com/user/repo.git".to_string()
        );
        assert_eq!(
            webhook.http_url(),
            "https://example.com/user/repo.git".to_string()
        );
        assert_eq!(
            webhook.repository_url(),
            "git@example.com/user/repo.git".to_string()
        );
        assert_eq!(webhook.action(), "merge".to_string());
        assert_eq!(webhook.target_branch(), "main".to_string());
        assert_eq!(webhook.source_branch(), "staging".to_string());
        assert_eq!(webhook.state(), "merge".to_string());
        assert_eq!(webhook.merge_status(), "merged".to_string());
    }

    #[test]
    fn it_deserializes_with_missing_fields() {
        let input = r#"{
            "project": {
            },
            "repository": {
                "url": "git@example.com/user/repo.git"
            },
            "object_attributes": {
                "target_branch": "main",
                "source_branch": "staging"
            }
        }"#;
        let webhook = serde_json::from_str::<Webhook>(input).unwrap();

        assert_eq!(webhook.event_type(), "undefined".to_string());
        assert_eq!(webhook.ssh_url(), "undefined".to_string());
        assert_eq!(webhook.http_url(), "undefined".to_string());
        assert_eq!(
            webhook.repository_url(),
            "git@example.com/user/repo.git".to_string()
        );
        assert_eq!(webhook.action(), "undefined".to_string());
        assert_eq!(webhook.target_branch(), "main".to_string());
        assert_eq!(webhook.source_branch(), "staging".to_string());
        assert_eq!(webhook.state(), "undefined".to_string());
        assert_eq!(webhook.merge_status(), "undefined".to_string());
    }
}
