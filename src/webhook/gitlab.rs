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
    default_branch: Option<String>,
    git_ssh_url: Option<String>,
    git_http_url: Option<String>,
    path_with_namespace: Option<String>,
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

    pub fn default_branch(&self) -> String {
        match &self.project.default_branch {
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

    fn fast_forward(&self, path: &Path) -> Result<(), git2::Error> {
        let repo = git2::Repository::open(path)?;

        repo.find_remote("origin")?
            .fetch(&[self.default_branch()], None, None)?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        let analysis = repo.merge_analysis(&[&fetch_commit])?;
        if analysis.0.is_up_to_date() {
            Ok(())
        } else if analysis.0.is_fast_forward() {
            let refname = format!("refs/heads/{}", self.default_branch());
            let mut reference = repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-Forward")?;
            repo.set_head(&refname)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        } else {
            Err(git2::Error::from_str("Fast-forward only!"))
        }
    }

    fn reset(&self, path: &Path) {
        let repo = match git2::Repository::open(path) {
            Ok(repo) => repo,
            Err(e) => panic!("Failed to open: {}", e),
        };
        repo.reset(
            &repo.revparse_single("HEAD").unwrap(),
            git2::ResetType::Hard,
            None,
        )
        .unwrap();
    }

    pub fn clone_repository(&self) -> Result<String, io::Error> {
        let project = self.project_name();
        let base = "/var/cache/shook/".to_string();
        let path = format!("{}{}", base, project);
        let repo_path = Path::new(&path);

        match repo_path.exists() && repo_path.is_dir() {
            true => {
                let root = Path::new(&path);
                env::set_current_dir(&root)?;
                self.reset(repo_path);
                if let Err(e) = self.fast_forward(repo_path) {
                    panic!("Failed to pull: {}", e)
                }
            }
            false => {
                let root = Path::new(&base);
                env::set_current_dir(&root)?;
                let _ = match git2::Repository::clone(&self.repository_url(), &project) {
                    Ok(repo) => repo,
                    Err(e) => panic!("failed to clone repository: {}", e),
                };
            }
        }

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
