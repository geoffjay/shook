use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;
use std::str;

#[derive(Deserialize)]
struct Project {
    name: String,
    env: Option<HashMap<String, String>>,
    commands: Vec<String>,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub token: String,
    projects: Vec<Project>,
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
    pub async fn execute_commands(&self, project_name: String) {
        let log = slog_scope::logger();

        debug!(log, "run commands for project"; "project_name" => project_name.clone());

        for project in self.projects.iter() {
            if project.name.clone() == project_name.clone() {
                debug!(log, "processor"; "project_name" => project.name.clone());
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_deserializes() {
        let input = r#"
          projects:
            - name: sample
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
        assert_eq!(env.len(), 1);
        assert_eq!(
            env.get(&"LOG".to_string()),
            Some(&"/tmp/sample.log".to_string())
        );
    }
}
