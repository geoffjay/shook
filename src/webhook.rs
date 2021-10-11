use serde::Deserialize;

#[derive(Deserialize)]
struct Repository {
    url: Option<String>,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_deserializes() {
        let input = r#"{
            "event_type": "merge_request",
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
