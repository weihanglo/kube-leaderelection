use crate::{Lock, BoxFuture, Elector};
use std::time::Duration;
use crate::elector::{Callbacks, Config};

pub struct ElectorBuilder {
    lock: Box<dyn Lock>,
    lease_duration: Duration,
    renew_deadline: Duration,
    retry_period: Duration,
    cbs: Callbacks,
}

impl ElectorBuilder {
    pub fn new(lock: impl Lock + 'static) -> Self {
        Self {
            lock: Box::new(lock),
            // TODO: default durations
            lease_duration: Duration::from_secs(15),
            renew_deadline: Duration::from_secs(10),
            retry_period: Duration::from_secs(2),
            cbs: Callbacks::default(),
        }
    }

    pub fn build(self) -> Elector {
        Elector {
            cfg: Config {
                lock: self.lock,
                lease_duration: self.lease_duration,
                renew_deadline: self.renew_deadline,
                retry_period: self.retry_period,
                cbs: self.cbs,
            },
            // TODO: fix default observe record
            observed_record: None,
            // TODO: fix default observe time
            observed_time: None,
            reported_leader: Default::default(),
        }
    }

    pub fn lease_duration(&mut self, duration: Duration) -> &mut Self {
        self.lease_duration = duration;
        self
    }

    pub fn renew_deadline(&mut self, duration: Duration) -> &mut Self {
        self.renew_deadline = duration;
        self
    }

    pub fn retry_period(&mut self, duration: Duration) -> &mut Self {
        self.retry_period = duration;
        self
    }

    pub fn on_started_leading(&mut self, f: fn() -> BoxFuture) -> &mut Self {
        self.cbs.on_started_leading = f;
        self
    }

    pub fn on_stopped_leading(&mut self, f: fn() -> BoxFuture) -> &mut Self {
        self.cbs.on_stopped_leading = f;
        self
    }

    /// Callback parameter `&str` is the elector's identity.
    pub fn on_new_leader(&mut self, f: fn(&str) -> BoxFuture) -> &mut Self {
        self.cbs.on_new_leader = f;
        self
    }
}

