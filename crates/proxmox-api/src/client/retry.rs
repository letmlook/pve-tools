//! Retry logic for transient PVE API errors

use std::future::Future;
use std::time::Duration;

use crate::error::{PveError, PveResult};

pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 500,
            max_delay_ms: 8000,
            multiplier: 2.0,
        }
    }
}

pub async fn with_retry<F, Fut, T>(config: RetryConfig, f: F) -> PveResult<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
{
    let mut delay = config.initial_delay_ms;
    let mut attempt = 0;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;
                if attempt >= config.max_attempts {
                    return Err(PveError::ServerError {
                        code: 0,
                        message: format!("retry exhausted after {} attempts: {}", attempt, e),
                    });
                }

                let is_transient = should_retry(&*e);
                if !is_transient {
                    return Err(PveError::ServerError {
                        code: 0,
                        message: format!("non-retryable error: {}", e),
                    });
                }

                tokio::time::sleep(Duration::from_millis(delay)).await;
                delay = (delay as f64 * config.multiplier) as u64;
                delay = delay.min(config.max_delay_ms);
            }
        }
    }
}

fn should_retry(e: &dyn std::error::Error) -> bool {
    let msg = e.to_string();
    // Retry on connection, timeout, and 5xx errors
    msg.contains("connection")
        || msg.contains("timeout")
        || msg.contains("503")
        || msg.contains("502")
        || msg.contains("504")
        || msg.contains("Too many requests")
        || msg.contains("connection refused")
        || msg.contains("broken pipe")
}