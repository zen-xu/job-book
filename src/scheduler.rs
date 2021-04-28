use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Error, Write};
use std::process::{Command, Output, Stdio};

use tempfile::tempdir;

use crate::config::{self, JobConfig, ParallelTasks, ScriptConfig, TaskConfig, TaskLabels};
use crate::scheduler::StatusPhase::Running;

pub struct Scheduler {
    job_config: JobConfig,
    // task_group: TaskGroup,
}

impl Scheduler {
    fn new(job_config: JobConfig) -> Self {
        Scheduler { job_config }
    }
}

enum StatusPhase {
    Pending,
    Running,
    Succeeded,
    Failed,
    Skipped,
}

struct Status {
    phase: StatusPhase,
    state: HashMap<StatusPhase, i32>,
}

struct TaskGroup {
    task_configs: Vec<ParallelTasks>,
    tasks: Vec<TaskKind>,
    status: Status,
}

enum TaskKind {
    Task(Task),
    Template(TaskTemplate),
}

struct Task {
    name: String,
    labels: TaskLabels,
    script: ScriptConfig,
    status_phase: StatusPhase,
    failed_reason: String,
}

struct TaskTemplate {
    name: String,
    labels: TaskLabels,
    template: String,
    tasks_group: TaskGroup,
    status_phase: StatusPhase,
}

impl Task {
    fn run<T: Into<Stdio>>(&mut self, stdout_cfg: T) -> io::Result<Output> {
        let result = (|| {
            let dir = tempdir()?;
            let script = dir.path().join("script");
            let script_name = String::from(script.to_str().unwrap());
            let mut script = File::create(script)?;

            script.write_all(self.script.source.as_bytes())?;

            let bash_script = format!(
                r#"
cd {}
{} {} {}
"#,
                self.script.working_dir,
                self.script.executor,
                self.script.executor_opts,
                script_name
            );

            let child = Command::new("bash")
                .arg("-c")
                .stdout(stdout_cfg)
                .stderr(Stdio::piped())
                .arg(&bash_script)
                .spawn()?;

            self.status_phase = StatusPhase::Running;
            child.wait_with_output()
        })();

        match &result {
            Ok(result) => {
                if result.status.success() {
                    self.status_phase = StatusPhase::Succeeded;
                } else {
                    self.status_phase = StatusPhase::Failed;
                    self.failed_reason = String::from_utf8(result.stderr.clone()).unwrap();
                }
            }
            Err(err) => {
                self.status_phase = StatusPhase::Failed;
                self.failed_reason = err.to_string();
            }
        }

        return result;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task() {
        let mut task = Task {
            name: "echo".to_string(),
            labels: Default::default(),
            script: ScriptConfig {
                source: "echo hello".to_string(),
                executor: "bash".to_string(),
                executor_opts: "".to_string(),
                working_dir: ".".to_string(),
            },
            status_phase: StatusPhase::Pending,
            failed_reason: "".to_string(),
        };

        let result = task.run(Stdio::piped()).unwrap();
        assert_eq!(result.stdout.as_slice(), b"hello\n");

        match task.status_phase {
            StatusPhase::Succeeded => {
                // ok
            }
            _ => {
                panic!("task should be Succeeded");
            }
        }
    }

    #[test]
    fn test_failed_task() {
        let mut task = Task {
            name: "error".to_string(),
            labels: Default::default(),
            script: ScriptConfig {
                source: "echos hello".to_string(),
                executor: "bash".to_string(),
                executor_opts: "".to_string(),
                working_dir: ".".to_string(),
            },
            status_phase: StatusPhase::Pending,
            failed_reason: "".to_string(),
        };

        let result = task.run(Stdio::piped());

        match task.status_phase {
            StatusPhase::Failed => {
                assert!(task.failed_reason.contains("echos: command not found"))
            }
            _ => {
                panic!("task should be Failed");
            }
        }
    }
}
