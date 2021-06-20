use async_trait::async_trait;
use rand::random;
use std::future::Future;
use std::time::Duration;
use tokio::time;

#[async_trait]
pub trait Backoff {
    async fn tick(&mut self);
}

pub async fn jitter_until(
    f: impl Future,
    period: Duration,
    factor: f64,
    sliding: bool,
    shutdown: impl Future,
) {
    struct Jitter(f64, Duration);
    #[async_trait]
    impl Backoff for Jitter {
        async fn tick(&mut self) {
            let duration = self.1 + self.1.mul_f64(self.0).mul_f64(random());
            time::sleep(duration).await;
        }
    }

    if factor <= 0.0 {
        return repeat_until(f, period, sliding, shutdown).await;
    }

    let jitter = Jitter(factor, period);
    backoff_until(f, jitter, sliding, shutdown).await
}

pub async fn repeat_until(f: impl Future, period: Duration, sliding: bool, shutdown: impl Future) {
    struct Repeat(time::Interval);
    #[async_trait]
    impl Backoff for Repeat {
        async fn tick(&mut self) {
            self.0.tick().await;
        }
    }

    let mut interval = Repeat(time::interval(period));
    interval.tick().await;
    backoff_until(f, interval, sliding, shutdown).await
}

pub async fn backoff_until(
    f: impl Future,
    backoff: impl Backoff,
    sliding: bool,
    shutdown: impl Future,
) {
    let mut backoff = backoff;
    if sliding {
        backoff.tick().await;
    }
    tokio::pin!(f);
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            _ = &mut shutdown => {
                break
            }
            _ = &mut f => {
                backoff.tick().await;
            }
        }
    }
}
