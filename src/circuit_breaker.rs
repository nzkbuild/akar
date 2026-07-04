/// Prevents runaway retry loops by tripping after max_failures.
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub name: String,
    pub failure_count: usize,
    pub max_failures: usize,
    pub tripped: bool,
}

impl CircuitBreaker {
    pub fn new(name: &str, max_failures: usize) -> Self {
        CircuitBreaker {
            name: name.to_string(),
            failure_count: 0,
            max_failures,
            tripped: false,
        }
    }
}

/// Record a failure. Returns true if the breaker just tripped.
pub fn record_failure(cb: &mut CircuitBreaker) -> bool {
    if cb.tripped {
        return false;
    }
    cb.failure_count += 1;
    if cb.failure_count >= cb.max_failures {
        cb.tripped = true;
        return true;
    }
    false
}

/// Reset the breaker to initial state.
pub fn reset(cb: &mut CircuitBreaker) {
    cb.failure_count = 0;
    cb.tripped = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trips_at_max_failures() {
        let mut cb = CircuitBreaker::new("test", 3);
        assert!(!record_failure(&mut cb));
        assert!(!record_failure(&mut cb));
        assert!(record_failure(&mut cb)); // trips on 3rd
        assert!(cb.tripped);
    }

    #[test]
    fn does_not_trip_before_max() {
        let mut cb = CircuitBreaker::new("test", 3);
        record_failure(&mut cb);
        record_failure(&mut cb);
        assert!(!cb.tripped);
        assert_eq!(cb.failure_count, 2);
    }

    #[test]
    fn reset_clears_state() {
        let mut cb = CircuitBreaker::new("test", 2);
        record_failure(&mut cb);
        record_failure(&mut cb);
        assert!(cb.tripped);
        reset(&mut cb);
        assert!(!cb.tripped);
        assert_eq!(cb.failure_count, 0);
    }

    #[test]
    fn tripped_breaker_does_not_re_trip() {
        let mut cb = CircuitBreaker::new("test", 2);
        record_failure(&mut cb);
        record_failure(&mut cb); // trips
        let result = record_failure(&mut cb); // already tripped
        assert!(!result); // returns false — already was tripped
        assert!(cb.tripped);
    }
}
