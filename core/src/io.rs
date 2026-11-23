use crate::{Error, Instant, Result};

pub trait Clock {
    fn now(&self) -> Instant;
}

pub trait PacketSource {
    fn receive(&mut self, destination: &mut [u8]) -> Result<usize>;
}

pub trait PacketSink {
    fn transmit(&mut self, packet: &[u8]) -> Result<()>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceBudget {
    limit: usize,
    used: usize,
}

impl ResourceBudget {
    pub const fn new(limit: usize) -> Self {
        Self { limit, used: 0 }
    }

    pub const fn limit(self) -> usize {
        self.limit
    }

    pub const fn used(self) -> usize {
        self.used
    }

    pub const fn available(self) -> usize {
        self.limit - self.used
    }

    pub fn reserve(&mut self, amount: usize) -> Result<()> {
        if amount > self.available() {
            return Err(Error::ResourceExhausted);
        }
        self.used += amount;
        Ok(())
    }

    pub fn release(&mut self, amount: usize) -> Result<()> {
        if amount > self.used {
            return Err(Error::InvalidData);
        }
        self.used -= amount;
        Ok(())
    }
}

#[cfg(feature = "test-support")]
pub struct ManualClock {
    now: core::cell::Cell<u64>,
}

#[cfg(feature = "test-support")]
impl ManualClock {
    pub const fn new(now: u64) -> Self {
        Self {
            now: core::cell::Cell::new(now),
        }
    }

    pub fn advance(&self, ticks: u64) {
        self.now.set(self.now.get().wrapping_add(ticks));
    }
}

#[cfg(feature = "test-support")]
impl Clock for ManualClock {
    fn now(&self) -> Instant {
        Instant::new(self.now.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reservation_failures_leave_budget_unchanged() {
        let mut budget = ResourceBudget::new(4);
        assert_eq!(budget.reserve(3), Ok(()));
        assert_eq!(budget.reserve(2), Err(Error::ResourceExhausted));
        assert_eq!(budget.used(), 3);
        assert_eq!(budget.available(), 1);
    }

    #[test]
    fn release_cannot_underflow() {
        let mut budget = ResourceBudget::new(4);
        assert_eq!(budget.reserve(2), Ok(()));
        assert_eq!(budget.release(3), Err(Error::InvalidData));
        assert_eq!(budget.used(), 2);
    }
}
