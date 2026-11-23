use crate::{Error, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnedPacket<const CAPACITY: usize> {
    bytes: [u8; CAPACITY],
    length: usize,
}

impl<const CAPACITY: usize> OwnedPacket<CAPACITY> {
    pub const fn new() -> Self {
        Self {
            bytes: [0; CAPACITY],
            length: 0,
        }
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        let mut packet = Self::new();
        packet.write(bytes)?;
        Ok(packet)
    }

    pub fn capacity(&self) -> usize {
        CAPACITY
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn remaining_capacity(&self) -> usize {
        CAPACITY - self.length
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.length]
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.bytes[..self.length]
    }

    pub fn spare_capacity_mut(&mut self) -> &mut [u8] {
        &mut self.bytes[self.length..]
    }

    pub fn write(&mut self, bytes: &[u8]) -> Result<()> {
        let new_length = self
            .length
            .checked_add(bytes.len())
            .ok_or(Error::BufferFull)?;
        if new_length > CAPACITY {
            return Err(Error::BufferFull);
        }

        self.bytes[self.length..new_length].copy_from_slice(bytes);
        self.length = new_length;
        Ok(())
    }

    pub fn truncate(&mut self, length: usize) -> Result<()> {
        if length > self.length {
            return Err(Error::InvalidData);
        }
        self.length = length;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.length = 0;
    }
}

impl<const CAPACITY: usize> Default for OwnedPacket<CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    pub fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.position..]
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn take(&mut self, length: usize) -> Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(Error::UnexpectedEof)?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or(Error::UnexpectedEof)?;
        self.position = end;
        Ok(bytes)
    }

    pub fn skip(&mut self, length: usize) -> Result<()> {
        self.take(length).map(|_| ())
    }

    pub fn u8(&mut self) -> Result<u8> {
        let bytes = self.take(1)?;
        Ok(bytes[0])
    }

    pub fn u16_be(&mut self) -> Result<u16> {
        let bytes = self.take(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    pub fn u16_le(&mut self) -> Result<u16> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub fn u32_be(&mut self) -> Result<u32> {
        let bytes = self.take(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn u64_be(&mut self) -> Result<u64> {
        let bytes = self.take(8)?;
        Ok(u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }
}

pub struct Writer<'a> {
    bytes: &'a mut [u8],
    position: usize,
}

impl<'a> Writer<'a> {
    pub fn new(bytes: &'a mut [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn remaining(&self) -> &[u8] {
        &self.bytes[self.position..]
    }

    pub fn write(&mut self, bytes: &[u8]) -> Result<()> {
        let end = self
            .position
            .checked_add(bytes.len())
            .ok_or(Error::BufferFull)?;
        let destination = self
            .bytes
            .get_mut(self.position..end)
            .ok_or(Error::BufferFull)?;
        destination.copy_from_slice(bytes);
        self.position = end;
        Ok(())
    }

    pub fn u8(&mut self, value: u8) -> Result<()> {
        self.write(&[value])
    }

    pub fn u16_be(&mut self, value: u16) -> Result<()> {
        self.write(&value.to_be_bytes())
    }

    pub fn u32_be(&mut self, value: u32) -> Result<()> {
        self.write(&value.to_be_bytes())
    }

    pub fn u64_be(&mut self, value: u64) -> Result<()> {
        self.write(&value.to_be_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_packet_enforces_capacity_without_panicking() {
        let mut packet = OwnedPacket::<3>::new();
        assert_eq!(packet.write(&[1, 2]), Ok(()));
        assert_eq!(packet.write(&[3]), Ok(()));
        assert_eq!(packet.write(&[4]), Err(Error::BufferFull));
        assert_eq!(packet.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn reader_never_advances_on_truncated_input() {
        let mut reader = Reader::new(&[1, 2, 3]);
        assert_eq!(reader.u16_be(), Ok(0x0102));
        assert_eq!(reader.u16_be(), Err(Error::UnexpectedEof));
        assert_eq!(reader.position(), 2);
        assert_eq!(reader.remaining(), &[3]);
    }

    #[test]
    fn writer_never_partially_writes() {
        let mut storage = [0; 3];
        let mut writer = Writer::new(&mut storage);
        assert_eq!(writer.write(&[1, 2]), Ok(()));
        assert_eq!(writer.write(&[3, 4]), Err(Error::BufferFull));
        assert_eq!(writer.position(), 2);
        assert_eq!(writer.remaining(), &[0]);
    }

    #[test]
    fn every_reader_truncation_is_reported() {
        for length in 0..16 {
            let bytes = [0xa5; 16];
            let mut reader = Reader::new(&bytes[..length]);
            assert!(reader.take(length.saturating_add(1)).is_err());
            assert_eq!(reader.position(), 0);
        }
    }
}
