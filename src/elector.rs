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

    #[inline]
    pub fn is_leader(&self) -> bool {
        self.observed_record
            .holder_identity
            .as_ref()
            .map(|id| id == self.leader_identity())
            .unwrap_or_default()
    }

    #[inline]
    pub fn leader_identity(&self) -> &str {
        self.cfg.lock.lock_identity()
    }

    pub fn check(&self, max_tolerable_expired_lease_duration: Duration) -> kube::Result<()> {
        // If we are more than timeout seconds after the lease duration that is
        // past the timeout on the lease renew. Time to start reporting
        // ourselves as unhealthy. We should have died but conditions like
        // deadlock can prevent this. (See kubernetes/client-go#70819)
        if self.is_leader()
            // TODO: handle duration unwrap
            && SystemTime::now().duration_since(self.observed_time).unwrap()
                > self.cfg.lease_duration + max_tolerable_expired_lease_duration
        {
            todo!("failed election to renew leadership on lease %s")
        }
        Ok(())
    }

    async fn acquire(&self) -> bool {
        todo!()
    }

    async fn renew(&self) {
        todo!()
    }

    async fn release(&mut self) -> kube::Result<bool> {
        if !self.is_leader() {
            return Ok(true);
        }

        let now = Some(MicroTime(SystemTime::now().into()));
        let new_record = ElectionRecord {
            acquire_time: now.clone(),
            lease_duration_seconds: Some(1),
            lease_transitions: self.observed_record.lease_transitions,
            renew_time: now,
            ..Default::default()
        };
        if let Err(e) = self.cfg.lock.update(new_record.clone()).await {
            todo!("Failed to release lock: %v");
            return Ok(false);
        }
        self.observed_record = new_record;
        self.observed_time = SystemTime::now();
        return Ok(true);
    }

    async fn acquire_or_renew(&mut self) -> bool {
        todo!()
    }

    fn report_transition_if_needed(&mut self) {
        todo!()
    }
}
