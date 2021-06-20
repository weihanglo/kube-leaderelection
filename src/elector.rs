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
        use kube::{error::ErrorResponse, Error};

        let now = SystemTime::now();
        let mut new_record = ElectionRecord {
            acquire_time: Some(MicroTime(now.clone().into())),
            renew_time: Some(MicroTime(now.clone().into())),
            lease_duration_seconds: Some(self.cfg.lease_duration.as_secs() as i32),
            // TODO: handle (as_secs -> u64) -> i32
            holder_identity: Some(self.cfg.lock.lock_identity().to_string()),
            ..Default::default()
        };

        // 1. Obtain or create the `ElectionRecord`
        //
        // - Do not propagate error here. Just log them.
        // - Create new record if error emerges, except error 404.
        match self.cfg.lock.get().await {
            Err(Error::Api(ErrorResponse { code, .. })) if code == 404 => {
                todo!("error retrieving resource lock %v: %v");
                return false;
            }
            Err(_) => {
                if let Ok(_) = self.cfg.lock.create(new_record.clone()).await {
                    self.observed_record = new_record;
                    self.observed_time = SystemTime::now();
                    return true;
                } else {
                    todo!("error initially creating leader election record: %v");
                    return false;
                }
            }
            Ok(old_record) => {
                // 2. Record obtained, check the Identity & Time
                if self.observed_record != old_record {
                    self.observed_record = old_record.clone();
                    self.observed_time = SystemTime::now();
                }
                if old_record
                    .holder_identity
                    .map(|id| id.len() > 0)
                    .unwrap_or_default()
                    && self.observed_time + self.cfg.lease_duration > now
                    && !self.is_leader()
                {
                    todo!("lock is held by %v and has not yet expired");
                    return false;
                }

                // 3. We're going to try to update. The leaderElectionRecord is
                // set to it's default here. Let's correct it before updating.
                new_record.lease_transitions = old_record.lease_transitions;
                if self.is_leader() {
                    new_record.acquire_time = old_record.acquire_time;
                } else {
                    new_record.lease_transitions.map(|t| t + 1);
                }

                // Update the lock itself
                if let Err(e) = self.cfg.lock.update(new_record.clone()).await {
                    todo!("Failed to update lock: %v");
                    return false;
                }

                self.observed_record = new_record;
                self.observed_time = SystemTime::now();
                return true;
            }
        }
    }

    fn report_transition_if_needed(&mut self) {
        todo!()
    }
}
