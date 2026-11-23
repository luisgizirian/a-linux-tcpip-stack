use crate::{Error, Result};

const MAX_DELAY: u64 = u64::MAX / 2;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Instant(u64);

impl Instant {
    pub const fn new(ticks: u64) -> Self {
        Self(ticks)
    }

    pub const fn ticks(self) -> u64 {
        self.0
    }

    pub const fn wrapping_add(self, ticks: u64) -> Self {
        Self(self.0.wrapping_add(ticks))
    }

    pub const fn wrapping_sub(self, earlier: Self) -> u64 {
        self.0.wrapping_sub(earlier.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Deadline {
    expires_at: Instant,
}

impl Deadline {
    pub fn after(now: Instant, delay_ticks: u64) -> Result<Self> {
        if delay_ticks > MAX_DELAY {
            return Err(Error::TimerRange);
        }
        Ok(Self {
            expires_at: now.wrapping_add(delay_ticks),
        })
    }

    pub const fn expires_at(self) -> Instant {
        self.expires_at
    }

    pub const fn is_expired(self, now: Instant) -> bool {
        let distance = now.wrapping_sub(self.expires_at);
        distance <= MAX_DELAY
    }

    pub const fn remaining(self, now: Instant) -> u64 {
        if self.is_expired(now) {
            0
        } else {
            self.expires_at.wrapping_sub(now)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deadline_expires_at_exact_tick() {
        let deadline = Deadline::after(Instant::new(10), 5).unwrap();
        assert_eq!(deadline.remaining(Instant::new(14)), 1);
        assert!(!deadline.is_expired(Instant::new(14)));
        assert!(deadline.is_expired(Instant::new(15)));
        assert_eq!(deadline.remaining(Instant::new(20)), 0);
    }

    #[test]
    fn deadline_comparison_wraps() {
        let deadline = Deadline::after(Instant::new(u64::MAX - 2), 5).unwrap();
        assert!(!deadline.is_expired(Instant::new(u64::MAX)));
        assert!(deadline.is_expired(Instant::new(2)));
        assert_eq!(deadline.expires_at(), Instant::new(2));
    }

    #[test]
    fn ambiguous_half_range_delays_are_rejected() {
        assert_eq!(
            Deadline::after(Instant::new(0), MAX_DELAY + 1),
            Err(Error::TimerRange)
        );
    }
}
