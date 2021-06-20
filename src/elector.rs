use crate::lock::Lock;
use std::time::Duration;

pub struct Elector {
    lock: Box<dyn Lock>,
    lease_duration: Duration,
    renew_duration: Duration,
    retry_period: Duration,
    name: String,
}

impl Elector {
    pub fn new(lock: Box<dyn Lock>) -> Self {
        todo!()
    }

    pub fn run(&self) {
        todo!("should we reinvent `context.Context` concept in Rust?")
    }
}
