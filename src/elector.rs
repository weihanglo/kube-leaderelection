use crate::lock::{ElectionRecord, Lock};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::MicroTime;
use std::time::{Duration, SystemTime};

pub struct Config {
    lock: Box<dyn Lock>,
    lease_duration: Duration,
    renew_duration: Duration,
    retry_period: Duration,
    callbacks: Box<dyn Callbacks>,
    name: String,
}

pub trait Callbacks {
    fn on_started_leading(&self);
    fn on_stopped_leading(&self);
    fn on_new_leader(&self, identity: &str);
}

pub struct Elector {
    cfg: Config,
    observed_record: ElectionRecord,
    // TODO: monotonic time or system/OS time?
    observed_time: SystemTime,
    // TODO: unnecessary?
    reported_leader: String,
}

impl Elector {
    pub fn new(cfg: Config) -> kube::Result<Self> {
        todo!()
    }

    pub async fn run(&self) {
        todo!("should we reinvent `context.Context` concept in Rust?")
    }

    pub async fn run_or_die(&self) {
        todo!("should we reinvent `context.Context` concept in Rust?")
    }

    pub fn is_leader(&self) -> bool {
        todo!()
    }
    pub fn leader_identity(&self) -> &str {
        self.cfg.lock.lock_identity()
    }

    pub fn check(&self) -> kube::Result<()> {
        todo!()
    }

    async fn acquire(&self) -> bool {
        todo!()
    }

    async fn renew(&self) {
        todo!()
    }

    async fn release(&self) -> bool {
        todo!()
    }

    async fn acquire_or_renew(&mut self) -> bool {
        todo!()
    }

    fn report_transition_if_needed(&mut self) {
        todo!()
    }
}
