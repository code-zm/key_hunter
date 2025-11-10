use governor::{Quota, RateLimiter as GovernorRateLimiter};
use nonzero_ext::*;
use std::num::NonZeroU32;
use std::time::Duration;
use tokio::time::sleep;

/// Rate limiter for API requests
pub struct RateLimiter {
    limiter: GovernorRateLimiter<
        governor::state::direct::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::DefaultClock,
    >,
    delay: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter with requests per second
    pub fn new(requests_per_second: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap());
        Self {
            limiter: GovernorRateLimiter::direct(quota),
            delay: Duration::from_secs(0),
        }
    }

    /// Create a new rate limiter with a minimum delay between requests
    pub fn with_delay(delay: Duration) -> Self {
        let quota = Quota::per_second(nonzero!(1u32));
        Self {
            limiter: GovernorRateLimiter::direct(quota),
            delay,
        }
    }

    /// Wait until a request is allowed
    pub async fn wait(&self) {
        // Wait for rate limiter
        while self.limiter.check().is_err() {
            sleep(Duration::from_millis(100)).await;
        }

        // Additional delay if configured
        if !self.delay.is_zero() {
            sleep(self.delay).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(10);
        limiter.wait().await;
        // Should not panic
    }

    #[tokio::test]
    async fn test_rate_limiter_with_delay() {
        let limiter = RateLimiter::with_delay(Duration::from_millis(100));
        let start = std::time::Instant::now();
        limiter.wait().await;
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(100));
    }
}
