use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
        /// working dir
        #[serde(default = "default_cwd")]
        cwd: String,
    },
}

fn default_cwd() -> String {
    ".".to_string()
}

fn default_executor() -> String {
    "bash".to_string()
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
                            cwd: ".".to_string(),
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
                            cwd: "/home".to_string(),
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
          cwd: /home
          labels: [second, third]
"#;
        let deserialized_config: JobConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config, deserialized_config);
    }
}
