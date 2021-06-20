use super::{ElectionRecord, Lock};
use async_trait::async_trait;
use k8s_openapi::api::coordination::v1::Lease;
use k8s_openapi::api::coordination::v1::LeaseSpec;
use kube::api::{Api, ObjectMeta, Patch, PatchParams, PostParams};
use kube::Client;
use std::fmt;

pub struct LeaseLock {
    ns: String,
    name: String,
    id: String,
    client: Client,
    lease: Option<Lease>,
}

#[async_trait]
impl Lock for LeaseLock {
    async fn create(&mut self, record: ElectionRecord) -> kube::Result<()> {
        let metadata = self.meta();
        let spec = Some(record.into());
        let data = Lease { metadata, spec };
        let lease = self.api().create(&PostParams::default(), &data).await?;
        self.lease = Some(lease);
        Ok(())
    }

    async fn get(&self) -> kube::Result<ElectionRecord> {
        let lease: Lease = self.api().get(&self.name).await?;
        // TODO: handle missing?
        Ok(lease.spec.unwrap().into())
    }

    async fn update(&mut self, record: ElectionRecord) -> kube::Result<()> {
        self.lease.as_mut().map(|l| l.spec = Some(record.into()));
        match self.lease.as_ref() {
            Some(lease) => {
                let patch = Patch::Apply(&lease);
                let lease = self
                    .api()
                    .patch(&self.name, &PatchParams::default(), &patch)
                    .await?;
                self.lease = Some(lease);
                Ok(())
            }
            None => todo!("lease not initialized, call get or create first"),
        }
    }

    fn lock_id(&self) -> &str {
        &self.id
    }
}

impl LeaseLock {
    #[inline]
    fn api(&self) -> Api<Lease> {
        Api::namespaced(self.client.clone(), &self.ns)
    }

    #[inline]
    fn meta(&self) -> ObjectMeta {
        let mut meta = ObjectMeta::default();
        meta.name = Some(self.name.to_owned());
        meta.namespace = Some(self.ns.to_owned());
        meta
    }

    pub fn new(ns: String, name: String, id: String, client: Client) -> Self {
        Self {
            ns,
            name,
            id,
            client,
            lease: None,
        }
    }
}

impl fmt::Display for LeaseLock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.ns, self.name)
    }
}

impl From<ElectionRecord> for LeaseSpec {
    fn from(x: ElectionRecord) -> Self {
        Self {
            acquire_time: x.acquire_time,
            holder_identity: x.holder_id,
            lease_duration_seconds: x.lease_duration_seconds,
            lease_transitions: x.lease_transitions,
            renew_time: x.renew_time,
        }
    }
}

impl From<LeaseSpec> for ElectionRecord {
    fn from(x: LeaseSpec) -> Self {
        Self {
            acquire_time: x.acquire_time,
            holder_id: x.holder_identity,
            lease_duration_seconds: x.lease_duration_seconds,
            lease_transitions: x.lease_transitions,
            renew_time: x.renew_time,
        }
    }
}
