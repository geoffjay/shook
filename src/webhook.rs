use serde::Deserialize;

#[derive(Deserialize)]
struct Repository {
    url: String,
}

#[derive(Deserialize)]
struct Attributes {
    action: String,
    target_branch: String,
    source_branch: String,
    state: String,
    merge_status: String,
}

#[derive(Deserialize)]
pub struct Webhook {
    event_type: String,
    repository: Repository,
    object_attributes: Attributes,
}

impl Webhook {
    pub fn event_type(&self) -> String {
        self.event_type.clone()
    }

    pub fn repository_url(&self) -> String {
        self.repository.url.clone()
    }

    pub fn action(&self) -> String {
        self.object_attributes.action.clone()
    }

    pub fn target_branch(&self) -> String {
        self.object_attributes.target_branch.clone()
    }

    pub fn source_branch(&self) -> String {
        self.object_attributes.source_branch.clone()
    }

    pub fn state(&self) -> String {
        self.object_attributes.state.clone()
    }

    pub fn merge_status(&self) -> String {
        self.object_attributes.merge_status.clone()
    }
}
