use crate::config::JobConfig;

pub struct Scheduler {
    job_config: JobConfig,
}

impl Scheduler {
    fn new(job_config: JobConfig) -> Self {
        Scheduler { job_config }
    }
}
