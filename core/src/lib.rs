#![no_std]
#![forbid(unsafe_code)]

pub mod buffer;
pub mod checksum;
pub mod error;
pub mod io;
pub mod time;
pub mod wire;

pub use buffer::{OwnedPacket, Reader, Writer};
pub use checksum::internet_checksum;
pub use error::Error;
pub use io::{Clock, PacketSink, PacketSource, ResourceBudget};
pub use time::{Deadline, Instant};

pub type Result<T> = core::result::Result<T, Error>;
