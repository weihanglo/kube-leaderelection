use async_trait::async_trait;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::MicroTime;
use kube::Client;

mod lease_lock;

pub use lease_lock::LeaseLock;

/// Roughly equivalent to [`k8s_openapi::api::coordination::v1::LeaseSpec`].
pub struct ElectionRecord {
    pub acquire_time: Option<MicroTime>,
    pub holder_identity: Option<String>,
    pub lease_duration_seconds: Option<i32>,
    pub lease_transitions: Option<i32>,
    pub renew_time: Option<MicroTime>,
}

pub enum LockKind {
    Endpoint,
    ConfigMap,
    Lease,
}

#[async_trait]
pub trait Lock {
    async fn create(&mut self, record: ElectionRecord) -> kube::Result<()>;
    async fn get(&self) -> kube::Result<ElectionRecord>;
    async fn update(&mut self, record: ElectionRecord) -> kube::Result<()>;
    fn lock_identity(&self) -> &str;
}

impl LockKind {
    pub fn new(self, ns: String, name: String, identity: String, client: Client) -> Box<dyn Lock> {
        use LockKind::*;
        match self {
            Endpoint => todo!(),
            ConfigMap => todo!(),
            Lease => Box::new(LeaseLock::new(ns, name, identity, client)),
        }
    }
}
