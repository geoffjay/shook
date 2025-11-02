use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::env;
use std::io;
use std::path::Path;

type HmacSha256 = Hmac<Sha256>;

#[derive(Deserialize)]
struct User {
    login: Option<String>,
}

#[derive(Deserialize)]
struct Repository {
    name: Option<String>,
    full_name: Option<String>,
    clone_url: Option<String>,
    ssh_url: Option<String>,
    default_branch: Option<String>,
}

#[derive(Deserialize)]
struct PullRequestHead {
    r#ref: Option<String>,
    sha: Option<String>,
}

#[derive(Deserialize)]
struct PullRequestBase {
    r#ref: Option<String>,
    sha: Option<String>,
}

#[derive(Deserialize)]
struct PullRequest {
    number: Option<u64>,
    state: Option<String>,
    title: Option<String>,
    merged: Option<bool>,
    merged_at: Option<String>,
    head: PullRequestHead,
    base: PullRequestBase,
}

#[derive(Deserialize)]
pub struct Webhook {
    action: Option<String>,
    repository: Repository,
    pull_request: Option<PullRequest>,
    sender: Option<User>,
}

impl Webhook {
    pub fn action(&self) -> String {
        match &self.action {
            None => "undefined".to_string(),
            Some(value) => value.clone(),
        }
    }

    pub fn repository_name(&self) -> String {
        match &self.repository.name {
            None => "undefined".to_string(),
            Some(value) => value.clone(),
        }
    }

    pub fn repository_full_name(&self) -> String {
        match &self.repository.full_name {
            None => "undefined".to_string(),
            Some(value) => value.clone(),
        }
    }

    pub fn default_branch(&self) -> String {
        match &self.repository.default_branch {
            None => "main".to_string(),  // GitHub default
            Some(value) => value.clone(),
        }
    }

    pub fn clone_url(&self) -> String {
        match &self.repository.clone_url {
            None => "undefined".to_string(),
            Some(value) => value.clone(),
        }
    }

    pub fn ssh_url(&self) -> String {
        match &self.repository.ssh_url {
            None => "undefined".to_string(),
            Some(value) => value.clone(),
        }
    }

    pub fn is_merged(&self) -> bool {
        match &self.pull_request {
            None => false,
            Some(pr) => pr.merged.unwrap_or(false),
        }
    }

    pub fn pr_state(&self) -> String {
        match &self.pull_request {
            None => "undefined".to_string(),
            Some(pr) => match &pr.state {
                None => "undefined".to_string(),
                Some(value) => value.clone(),
            },
        }
    }

    pub fn target_branch(&self) -> String {
        match &self.pull_request {
            None => "undefined".to_string(),
            Some(pr) => match &pr.base.r#ref {
                None => "undefined".to_string(),
                Some(value) => value.clone(),
            },
        }
    }

    pub fn source_branch(&self) -> String {
        match &self.pull_request {
            None => "undefined".to_string(),
            Some(pr) => match &pr.head.r#ref {
                None => "undefined".to_string(),
                Some(value) => value.clone(),
            },
        }
    }

    pub fn pr_number(&self) -> u64 {
        match &self.pull_request {
            None => 0,
            Some(pr) => pr.number.unwrap_or(0),
        }
    }

    pub fn pr_title(&self) -> String {
        match &self.pull_request {
            None => "undefined".to_string(),
            Some(pr) => match &pr.title {
                None => "undefined".to_string(),
                Some(value) => value.clone(),
            },
        }
    }

    pub fn sender(&self) -> String {
        match &self.sender {
            None => "undefined".to_string(),
            Some(user) => match &user.login {
                None => "undefined".to_string(),
                Some(value) => value.clone(),
            },
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
        let project = self.repository_name();
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
                // Use HTTPS URL for GitHub (more compatible than SSH in most cases)
                let _ = match git2::Repository::clone(&self.clone_url(), &project) {
                    Ok(repo) => repo,
                    Err(e) => panic!("failed to clone repository: {}", e),
                };
            }
        }

        Ok(format!("/var/cache/shook/{}", project))
    }

    pub fn dump(&self) {
        let log = slog_scope::logger();

        debug!(log, "github webhook event"; "action" => self.action());
        debug!(log, "github webhook repository";
            "name" => self.repository_name(),
            "full_name" => self.repository_full_name(),
            "clone_url" => self.clone_url(),
            "ssh_url" => self.ssh_url(),
            "default_branch" => self.default_branch(),
        );

        if self.pull_request.is_some() {
            debug!(log, "github webhook pull_request";
                "number" => self.pr_number(),
                "state" => self.pr_state(),
                "merged" => self.is_merged(),
                "target_branch" => self.target_branch(),
                "source_branch" => self.source_branch(),
                "title" => self.pr_title(),
            );
        }

        debug!(log, "github webhook sender"; "login" => self.sender());
    }
}

/// Verify the GitHub webhook signature using HMAC-SHA256
pub fn verify_signature(secret: &str, body: &[u8], signature: &str) -> bool {
    // GitHub signature format: "sha256=<hex_digest>"
    if !signature.starts_with("sha256=") {
        return false;
    }

    let signature_hex = &signature[7..];

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(body);

    let result = mac.finalize();
    let expected = hex::encode(result.into_bytes());

    // Use constant-time comparison to prevent timing attacks
    expected == signature_hex
}

/// Check if this webhook event should trigger a deployment
pub fn should_deploy_github(action: String, merged: bool, target_branch: String) -> bool {
    // GitHub sends "closed" action when PR is merged
    action == "closed" && merged && target_branch == "main"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_deserializes_pr_event() {
        let input = r#"{
            "action": "closed",
            "repository": {
                "name": "test-repo",
                "full_name": "user/test-repo",
                "clone_url": "https://github.com/user/test-repo.git",
                "ssh_url": "git@github.com:user/test-repo.git",
                "default_branch": "main"
            },
            "pull_request": {
                "number": 123,
                "state": "closed",
                "title": "Test PR",
                "merged": true,
                "merged_at": "2023-01-01T00:00:00Z",
                "head": {
                    "ref": "feature-branch",
                    "sha": "abc123"
                },
                "base": {
                    "ref": "main",
                    "sha": "def456"
                }
            },
            "sender": {
                "login": "testuser"
            }
        }"#;

        let webhook = serde_json::from_str::<Webhook>(input).unwrap();

        assert_eq!(webhook.action(), "closed".to_string());
        assert_eq!(webhook.repository_name(), "test-repo".to_string());
        assert_eq!(webhook.repository_full_name(), "user/test-repo".to_string());
        assert_eq!(webhook.clone_url(), "https://github.com/user/test-repo.git".to_string());
        assert_eq!(webhook.ssh_url(), "git@github.com:user/test-repo.git".to_string());
        assert_eq!(webhook.default_branch(), "main".to_string());
        assert!(webhook.is_merged());
        assert_eq!(webhook.pr_state(), "closed".to_string());
        assert_eq!(webhook.target_branch(), "main".to_string());
        assert_eq!(webhook.source_branch(), "feature-branch".to_string());
        assert_eq!(webhook.pr_number(), 123);
        assert_eq!(webhook.pr_title(), "Test PR".to_string());
        assert_eq!(webhook.sender(), "testuser".to_string());
    }

    #[test]
    fn it_verifies_signature() {
        let secret = "test_secret";
        let body = b"test payload";
        // This is the actual HMAC-SHA256 signature for "test payload" with secret "test_secret"
        let valid_signature = "sha256=fb9fb46a0a4c5edf7c9f524414be12d1eef6847c7b34dac98757920731e51169";

        assert!(verify_signature(secret, body, valid_signature));
        assert!(!verify_signature(secret, body, "sha256=invalid"));
        assert!(!verify_signature(secret, body, "invalid_format"));
    }

    #[test]
    fn it_should_deploy() {
        assert!(should_deploy_github(
            "closed".to_string(),
            true,
            "main".to_string()
        ));
        assert!(!should_deploy_github(
            "opened".to_string(),
            false,
            "main".to_string()
        ));
        assert!(!should_deploy_github(
            "closed".to_string(),
            false,
            "main".to_string()
        ));
        assert!(!should_deploy_github(
            "closed".to_string(),
            true,
            "develop".to_string()
        ));
    }
}