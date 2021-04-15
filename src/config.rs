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
    pub templates: Vec<Template>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Template {
    /// template name
    pub name: String,
    /// template level limits the max total parallel tasks that can execute at the same time
    pub parallelism: Option<i32>,
    /// a series of sequential/parallel tasks
    pub tasks: Vec<Vec<TaskConfig>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskConfig {
    /// job name
    #[serde(default)]
    pub name: String,
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
        #[serde(default)]
        language: Language,
        /// working dir
        #[serde(default = "default_cwd")]
        cwd: String,
    },
}

fn default_cwd() -> String {
    ".".to_string()
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Bash,
    Python,
    Ruby,
    Javascript,
}

impl Default for Language {
    fn default() -> Self {
        Language::Bash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_deserialize_yaml() {
        let config = JobConfig {
            name: "job".to_string(),
            parallelism: None,
            entrypoint: "main".to_string(),
            templates: vec![Template {
                name: "main".to_string(),
                parallelism: None,
                tasks: vec![vec![TaskConfig {
                    name: "".to_string(),
                    kind: TaskKind::Script {
                        script: "echo hello".to_string(),
                        language: Language::Bash,
                        cwd: ".".to_string(),
                    },
                }]],
            }],
        };

        let yaml_str = r#"
---
name: job
entrypoint: main
templates:
- name: main
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
            templates: vec![
                Template {
                    name: "main".to_string(),
                    parallelism: None,
                    tasks: vec![vec![TaskConfig {
                        name: "".to_string(),
                        kind: TaskKind::Template {
                            template: "run_it".to_string(),
                        },
                    }]],
                },
                Template {
                    name: "run_it".to_string(),
                    parallelism: None,
                    tasks: vec![vec![TaskConfig {
                        name: "".to_string(),
                        kind: TaskKind::Script {
                            script: "print(\"hello\")".to_string(),
                            language: Language::Python,
                            cwd: "/home".to_string(),
                        },
                    }]],
                },
            ],
        };
        let yaml_str = r#"
---
name: job
entrypoint: main
parallelism: 2
templates:
- name: main
  tasks:
  - - template: run_it
- name: run_it
  tasks:
  - - script: print("hello")
      language: python
      cwd: /home
"#;
        let deserialized_config: JobConfig = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(config, deserialized_config);
    }
}
