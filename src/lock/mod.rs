use async_trait::async_trait;
use kube::Client;

pub struct ElectionRecord;

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
            Lease => todo!(),
        }
    }
}
