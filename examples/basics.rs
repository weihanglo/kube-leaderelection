use kube::Client;
use kube_leaderelection::{ElectorBuilder, LockKind};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    log::info!("Leader election demo started!");

    let client = Client::try_default().await?;
    let lock = LockKind::Lease.new(
        "default".to_string(),
        "lease-lock".to_string(),
        "unique-id".to_string(),
        client.clone(),
    );

    let mut elector = ElectorBuilder::new(lock)
        .on_started_leading(|| {
            let name = std::env::var("NAME").unwrap_or_default();
            log::info!(">>>> `{}` has just started leading", name);
        })
        .on_stopped_leading(|| {
            let name = std::env::var("NAME").unwrap_or_default();
            log::info!(">>>> `{}` has just stopped leading", name);
        })
        .on_new_leader(|leader| {
            log::info!(">>>> lock `{}` just electecd as new leader", leader);
        })
        .build();

    elector.run().await;

    Ok(())
}
