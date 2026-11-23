use core::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    UnexpectedEof,
    InvalidData,
    BufferFull,
    ResourceExhausted,
    TimerRange,
    WouldBlock,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnexpectedEof => "packet is truncated",
            Self::InvalidData => "packet contains invalid data",
            Self::BufferFull => "packet buffer is full",
            Self::ResourceExhausted => "resource budget is exhausted",
            Self::TimerRange => "timer duration cannot be represented",
            Self::WouldBlock => "operation would block",
        })
    }
}
