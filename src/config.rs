use std::collections::HashMap;
use std::{
    self,
    fmt::{self, Display},
    fs,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Message(String),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct JobConfig {
    /// job name
    pub name: String,
    /// entrypoint is a template reference to the starting point of the job.
    pub entrypoint: String,
    /// job level limits the max total parallel tasks that can execute at the same time
    pub parallelism: Option<i32>,
    /// a series of templates
    pub templates: HashMap<String, Template>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Template {
    /// template level limits the max total parallel tasks that can execute at the same time
    pub parallelism: Option<i32>,
    /// a series of sequential/parallel tasks
    pub tasks: Vec<Vec<TaskConfig>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct TaskLabels(Vec<String>);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskConfig {
    /// job name
    #[serde(default)]
    pub name: String,
    /// job labels
    #[serde(default)]
    pub labels: TaskLabels,
    #[serde(flatten)]
    pub kind: TaskKind,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskKind {
    Template {
        /// template name
        template: String,
    },
    Script {
        /// script body
        script: String,
        /// script language
        #[serde(default = "default_executor")]
        executor: String,
        #[serde(default)]
        executor_args: Vec<String>,
        /// working dir
        #[serde(default = "default_working_dir")]
        working_dir: String,
    },
}

fn default_working_dir() -> String {
    ".".to_string()
}

fn default_executor() -> String {
    "bash".to_string()
}

impl JobConfig {
    pub fn from_yaml(path: &str) -> Result<JobConfig, Box<dyn std::error::Error>> {
        return Self::from_str(&fs::read_to_string(path)?);
    }

    pub fn from_str(yaml: &str) -> Result<JobConfig, Box<dyn std::error::Error>> {
        let config: JobConfig = serde_yaml::from_str(yaml)?;

        // check entrypoint
        if !config.templates.contains_key(&config.entrypoint) {
            return Err(Error::Message(format!(
                "invalid entrypoint, no template names '{}'",
                config.entrypoint
            ))
            .into());
        }

        // check TaskKind template
        for template in config.templates.values() {
            for task_group in &template.tasks {
                for task in task_group {
                    match task.kind {
                        TaskKind::Template { template: ref t } => {
                            if !config.templates.contains_key(t) {
                                return Err(Error::Message(format!(
                                    "invalid template, no template names '{}'",
                                    t
                                ))
                                .into());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        return Ok(config);
    }
}

#[cfg(test)]
mod tests {
    use crate::collection;

    use super::*;

    #[test]
    fn test_default_deserialize_yaml() {
        let config = JobConfig {
            name: "job".to_string(),
            parallelism: None,
            entrypoint: "main".to_string(),
            templates: collection! {
                "main".to_string() => Template {
                    parallelism: None,
                    tasks: vec![vec![TaskConfig {
                        name: "".to_string(),
                        labels: TaskLabels(vec![]),
                        kind: TaskKind::Script {
                            script: "echo hello".to_string(),
                            executor: "bash".to_string(),
                            executor_args: vec![],
                            working_dir: ".".to_string(),
                        },
                    }]],
                }
            },
        };

        let yaml_str = r#"
---
name: job
entrypoint: main
templates:
  main:
    tasks:
    - - script: echo hello
"#;
        let deserialized_config: JobConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config, deserialized_config);
    }

    #[test]
    fn test_deserialize_yaml() {
        let config = JobConfig {
            name: "job".to_string(),
            entrypoint: "main".to_string(),
            parallelism: Some(2),
            templates: collection! {
                "main".to_string() => Template {
                    parallelism: None,
                    tasks: vec![vec![TaskConfig {
                        name: "".to_string(),
                        labels: TaskLabels(vec!["first".to_string()]),
                        kind: TaskKind::Template {
                            template: "run_it".to_string(),
                        },
                    }]],
                },
                "run_it".to_string() => Template {
                    parallelism: None,
                    tasks: vec![vec![TaskConfig {
                        name: "".to_string(),
                        labels: TaskLabels(vec!["second".to_string(), "third".to_string()]),
                        kind: TaskKind::Script {
                            script: "print(\"hello\")".to_string(),
                            executor: "python".to_string(),
                            executor_args: vec!["-u".to_string()],
                            working_dir: "/home".to_string(),
                        },
                    }]],
                },
            },
        };

        let yaml_str = r#"
---
name: job
entrypoint: main
parallelism: 2
templates:
  main:
    tasks:
      - - template: run_it
          labels: [first]
  run_it:
    tasks:
      - - script: "print(\"hello\")"
          executor: python
          executor_args: [-u]
          working_dir: /home
          labels: [second, third]
"#;
        let deserialized_config: JobConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config, deserialized_config);
    }

    #[test]
    fn test_from_str() {
        let yaml_str = r#"
---
name: job
entrypoint: main
templates:
  main2:
    tasks:
    - - script: echo hello
        "#;
        if let Err(err) = JobConfig::from_str(&yaml_str) {
            assert_eq!(
                err.to_string(),
                "invalid entrypoint, no template names 'main'".to_string()
            );
        } else {
            panic!("should not find template")
        }

        let yaml_str = r#"
---
name: job
entrypoint: main
parallelism: 2
templates:
  main:
    tasks:
      - - template: run_it2
          labels: [first]
  run_it:
    tasks:
      - - script: "print(\"hello\")"
          executor: python
          executor_args: [-u]
          working_dir: /home
          labels: [second, third]
        "#;
        if let Err(err) = JobConfig::from_str(&yaml_str) {
            assert_eq!(
                err.to_string(),
                "invalid template, no template names 'run_it2'".to_string()
            );
        } else {
            panic!("should not find template")
        }
    }
}
