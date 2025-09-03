use std::time::Duration;
use tokio::time::sleep;

pub struct SyncTrier {
    try_count: u32,
    timeout: Duration,
    multiplier: f32,
    max_try_count: u32,
}

impl SyncTrier {

    pub fn new(timeout_sec: u64, multiplier: f32, max_try_count: u32) -> Self {
        Self {
            try_count: 0,
            timeout: Duration::from_secs(timeout_sec),
            multiplier,
            max_try_count,
        }
    }

    pub async fn iterate(&mut self) -> bool {
        self.increment();

        if self.is_exceeded() {
            return false;
        }

        if self.should_wait() {
            sleep(self.next_timeout())
                .await;
        }

        return true;
    }

    pub fn is_last(&self) -> bool {
        self.try_count == self.max_try_count
    }

    pub fn is_failed(&self) -> bool {
        return self.is_exceeded();
    }

    pub fn is_exceeded(&self) -> bool {
        self.try_count > self.max_try_count
    }

    pub fn try_count(&self) -> u32 {
        self.try_count
    }

    pub fn retry_count(&self) -> u32 {
        if self.try_count == 0 {
            return 0;
        }

        self.try_count - 1
    }

    pub fn reset(&mut self) {
        self.try_count = 0;
    }

    pub fn fail(&mut self) {
        self.try_count = self.max_try_count + 1;
    }

    fn increment(&mut self) {
        self.try_count += 1;
    }

    fn should_wait(&self) -> bool {
        self.try_count > 1 && self.try_count <= self.max_try_count
    }

    fn next_timeout(&self) -> Duration {
        let timeout = self.timeout.mul_f32(self.multiplier);

        if self.try_count > self.max_try_count {
            return Duration::from_secs(0);
        }

        return timeout;
    }
}
