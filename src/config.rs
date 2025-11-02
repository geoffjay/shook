use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;
use std::str;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    GitLab,
    GitHub,
}

impl Default for Provider {
    fn default() -> Self {
        Provider::GitLab
    }
}

#[derive(Clone, Deserialize)]
pub struct Project {
    pub name: String,
    pub token: String,
    #[serde(default)]
    pub provider: Provider,
    env: Option<HashMap<String, String>>,
    commands: Vec<String>,
}

#[derive(Deserialize)]
pub struct Config {
    projects: Vec<Project>,
}

pub fn should_deploy(branch: String, action: String, status: String) -> bool {
    branch == "main" && action == "merge" && status == "merged"
}

impl Project {
    pub fn env(&self) -> HashMap<String, String> {
        match &self.env {
            None => HashMap::new(),
            Some(value) => value.clone(),
        }
    }
}

impl Config {
    pub fn get_project(&self, project: String) -> Option<Project> {
        for item in &self.projects {
            if item.name.clone() == project {
                return Some(item.clone());
            }
        }
        None
    }

    /// Process the list of configured commands. There's lifetime issues if this
    /// is on the project, so it's here because the config is kept as app data
    /// that's passed into handlers.
    pub async fn execute_commands(&self, project: Project) {
        let log = slog_scope::logger();

        debug!(log, "command processor"; "project_name" => project.name.clone());
        for command in project.commands.iter() {
            let output = Command::new("bash")
                .arg("-c")
                .arg(command)
                .envs(project.env())
                .output()
                .expect("failed to execute command");

            debug!(log, "processor"; "status" => format!("{:?}", output.status));
            debug!(log, "processor"; "stdout" => str::from_utf8(&output.stdout).unwrap());
            debug!(log, "processor"; "stderr" => str::from_utf8(&output.stderr).unwrap());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_deploy() {
        assert!(should_deploy(
            "main".to_string(),
            "merge".to_string(),
            "merged".to_string()
        ));
        assert!(!should_deploy(
            "staging".to_string(),
            "merge".to_string(),
            "merged".to_string()
        ));
        assert!(!should_deploy(
            "main".to_string(),
            "open".to_string(),
            "merged".to_string()
        ));
        assert!(!should_deploy(
            "main".to_string(),
            "merge".to_string(),
            "closed".to_string()
        ));
    }

    #[test]
    fn it_deserializes() {
        let input = r#"
          projects:
            - name: sample
              token: really-gud-secret
              env:
                LOG: /tmp/sample.log
              commands:
                - touch $LOG
                - echo test >> $LOG
        "#;
        let config = serde_yaml::from_str::<Config>(input).unwrap();
        let project = &config.projects[0];
        let env = project.env();

        assert_eq!(project.name, "sample".to_string());
        assert_eq!(project.token, "really-gud-secret".to_string());
        assert_eq!(project.provider, Provider::GitLab); // default provider
        assert_eq!(env.len(), 1);
        assert_eq!(
            env.get(&"LOG".to_string()),
            Some(&"/tmp/sample.log".to_string())
        );
    }

    #[test]
    fn it_deserializes_with_github_provider() {
        let input = r#"
          projects:
            - name: github-project
              token: github-secret-token
              provider: github
              env:
                LOG: /tmp/github.log
              commands:
                - touch $LOG
                - echo github >> $LOG
        "#;
        let config = serde_yaml::from_str::<Config>(input).unwrap();
        let project = &config.projects[0];

        assert_eq!(project.name, "github-project".to_string());
        assert_eq!(project.token, "github-secret-token".to_string());
        assert_eq!(project.provider, Provider::GitHub);
    }
}
