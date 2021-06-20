use crate::lock::{ElectionRecord, Lock};
use crate::wait::{jitter_until, repeat_until};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::MicroTime;
use std::time::{Duration, SystemTime};

const JITTER_FACTOR: f64 = 1.2;

pub struct Config {
    lock: Box<dyn Lock>,
    lease_duration: Duration,
    renew_deadline: Duration,
    retry_period: Duration,
    cbs: Callbacks,
    name: String,
}

type BoxFuture = Box<dyn std::future::Future<Output = ()> + Send + Unpin>;

pub struct Callbacks {
    pub on_started_leading: fn() -> BoxFuture,
    pub on_stopped_leading: fn() -> BoxFuture,
    /// Callback parameter `&str` is Elector's identity.
    pub on_new_leader: fn(&str) -> BoxFuture,
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

    pub async fn run(&mut self) {
        // TODO: should we reinvent `context.Context` concept in Rust?
        if !self.acquire().await {
            (self.cfg.cbs.on_stopped_leading)().await;
        }
        tokio::spawn((self.cfg.cbs.on_started_leading)());
        self.renew().await;
    }

    #[inline]
    pub fn is_leader(&self) -> bool {
        self.observed_record
            .holder_id
            .as_ref()
            .map(|id| id == self.leader_id())
            .unwrap_or_default()
    }

    #[inline]
    pub fn leader_id(&self) -> &str {
        self.cfg.lock.lock_id()
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
            return Err(kube::Error::RequestValidation(format!("failed election to renew leadership on lease {}", self.cfg.name)))
        }
        Ok(())
    }

    async fn acquire(&mut self) -> bool {
        // TODO: should we reinvent `context.Context` concept in Rust?
        use tokio::{sync, time};
        let (cancel, timeout) = sync::oneshot::channel();
        let deadline = time::timeout(Duration::from_secs(5), timeout);
        let period = self.cfg.retry_period;

        let mut succeeded = false;

        let f = async {
            succeeded = self.acquire_or_renew().await;
            if succeeded {
                tracing::info!("successfully acquired lease {}", self.cfg.lock);
                // TODO: handle receiver drop
                cancel.send(()).unwrap();
            } else {
                tracing::info!("failed to acquire lease {}", self.cfg.lock);
            }
        };

        jitter_until(f, period, JITTER_FACTOR, true, deadline).await;

        return succeeded;
    }

    async fn renew(&mut self) {
        // TODO: should we reinvent `context.Context` concept in Rust?
        use tokio::{sync, time};
        let (cancel, timeout) = sync::oneshot::channel();
        let deadline = time::timeout(Duration::from_secs(5), timeout);
        let period = self.cfg.retry_period;

        let f = async {
            // Run immediately
            let mut is_timeout = false;
            if !self.acquire_or_renew().await {
                let poll_deadline = time::sleep(self.cfg.renew_deadline);
                let (ready, rx) = sync::oneshot::channel();
                let f = async {
                    if self.acquire_or_renew().await {
                        ready.send(()).unwrap();
                    }
                };
                let stop = async {
                    tokio::select! {
                        // either reaching the renew deadline
                        _ = poll_deadline => {
                            is_timeout = true
                        }
                        // or poll is ready
                        _ = rx => {}
                    }
                };
                // Polling `acquire_or_renew` until yielding `true`
                repeat_until(f, period, true, stop).await;
            }

            self.report_transition_if_needed();

            if is_timeout {
                tracing::info!("failed to renew lease {}: {}", self.cfg.lock, "timeout");
                // TODO: handle receiver drop
                cancel.send(()).unwrap();
            } else {
                tracing::info!("successfully renewed lease {}", self.cfg.lock);
            }
        };

        repeat_until(f, period, true, deadline).await;

        // TODO: release on cancel?
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
            tracing::error!("Failed to release lock: {}", e);
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
            holder_id: Some(self.cfg.lock.lock_id().to_string()),
            ..Default::default()
        };

        // 1. Obtain or create the `ElectionRecord`
        //
        // - Do not propagate error here. Just log them.
        // - Create new record if error emerges, except error 404.
        match self.cfg.lock.get().await {
            Err(Error::Api(ErrorResponse { code, .. })) if code == 404 => {
                if let Err(e) = self.cfg.lock.create(new_record.clone()).await {
                    tracing::error!("error initially creating leader election record: {}", e);
                    return false;
                } else {
                    self.observed_record = new_record;
                    self.observed_time = SystemTime::now();
                    return true;
                }
            }
            Err(e) => {
                tracing::error!("error retrieving resource lock {}: {}", self.cfg.lock, e);
                return false;
            }
            Ok(old_record) => {
                // 2. Record obtained, check the Identity & Time
                if self.observed_record != old_record {
                    self.observed_record = old_record.clone();
                    self.observed_time = SystemTime::now();
                }

                if old_record
                    .holder_id
                    .as_ref()
                    .map(|id| id.len() > 0)
                    .unwrap_or_default()
                    && self.observed_time + self.cfg.lease_duration > now
                    && !self.is_leader()
                {
                    let id = old_record.holder_id.unwrap_or_default();
                    tracing::info!("lock is held by {} and has not yet expired", id);
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
                    tracing::error!("Failed to update lock: {}", e);
                    return false;
                }

                self.observed_record = new_record;
                self.observed_time = SystemTime::now();
                return true;
            }
        }
    }

    fn report_transition_if_needed(&mut self) {
        if self
            .observed_record
            .holder_id
            .as_ref()
            .map(|id| id == &self.reported_leader)
            .unwrap_or_default()
        {
            return;
        }
        self.reported_leader = self.observed_record.holder_id.clone().unwrap_or_default();

        tokio::spawn((self.cfg.cbs.on_new_leader)(&self.reported_leader));
    }
}
