use std::time::Duration;

/// Bounded exponential reconnect delay without polling or busy retries.
#[derive(Clone, Copy, Debug)]
pub struct RetryBackoff {
    attempts: u32,
    initial: Duration,
    maximum: Duration,
}

impl RetryBackoff {
    /// Creates a bounded reconnect backoff policy.
    ///
    /// # Panics
    ///
    /// Panics when the initial delay is zero or greater than the maximum.
    #[must_use]
    pub fn new(initial: Duration, maximum: Duration) -> Self {
        assert!(!initial.is_zero(), "initial retry delay must be nonzero");
        assert!(initial <= maximum, "initial retry delay exceeds maximum");
        Self {
            attempts: 0,
            initial,
            maximum,
        }
    }

    /// Returns the next retry delay and increments the failed-attempt count.
    pub fn next_delay(&mut self) -> Duration {
        let multiplier = 1_u32.checked_shl(self.attempts.min(31)).unwrap_or(u32::MAX);
        self.attempts = self.attempts.saturating_add(1);
        self.initial.saturating_mul(multiplier).min(self.maximum)
    }

    /// Resets the policy after a successful peer connection.
    pub const fn reset(&mut self) {
        self.attempts = 0;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::RetryBackoff;

    #[test]
    fn caps_and_resets_retry_delay() {
        let mut backoff = RetryBackoff::new(Duration::from_secs(1), Duration::from_secs(4));

        assert_eq!(backoff.next_delay(), Duration::from_secs(1));
        assert_eq!(backoff.next_delay(), Duration::from_secs(2));
        assert_eq!(backoff.next_delay(), Duration::from_secs(4));
        assert_eq!(backoff.next_delay(), Duration::from_secs(4));
        backoff.reset();
        assert_eq!(backoff.next_delay(), Duration::from_secs(1));
    }
}
