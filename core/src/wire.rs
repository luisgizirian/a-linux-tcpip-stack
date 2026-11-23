use crate::{Error, Reader, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MacAddress(pub [u8; 6]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ipv4Address(pub [u8; 4]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EthernetHeader {
    pub destination: MacAddress,
    pub source: MacAddress,
    pub ether_type: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ipv4Header<'a> {
    pub version: u8,
    pub header_length: usize,
    pub dscp: u8,
    pub ecn: u8,
    pub total_length: usize,
    pub identification: u16,
    pub dont_fragment: bool,
    pub more_fragments: bool,
    pub fragment_offset: usize,
    pub ttl: u8,
    pub protocol: u8,
    pub checksum: u16,
    pub source: Ipv4Address,
    pub destination: Ipv4Address,
    pub options: &'a [u8],
    pub payload: &'a [u8],
}

impl EthernetHeader {
    pub fn parse(reader: &mut Reader<'_>) -> Result<Self> {
        let destination = reader.take(6)?;
        let source = reader.take(6)?;
        let ether_type = reader.u16_be()?;
        Ok(Self {
            destination: MacAddress(destination.try_into().map_err(|_| Error::UnexpectedEof)?),
            source: MacAddress(source.try_into().map_err(|_| Error::UnexpectedEof)?),
            ether_type,
        })
    }
}

impl<'a> Ipv4Header<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self> {
        let mut reader = Reader::new(bytes);
        let version_ihl = reader.u8()?;
        let version = version_ihl >> 4;
        let ihl = usize::from(version_ihl & 0x0f) * 4;
        if version != 4 || ihl < 20 {
            return Err(Error::InvalidData);
        }

        let dscp_ecn = reader.u8()?;
        let total_length = usize::from(reader.u16_be()?);
        if total_length < ihl || total_length > bytes.len() {
            return Err(Error::InvalidData);
        }

        let identification = reader.u16_be()?;
        let flags_fragment = reader.u16_be()?;
        if flags_fragment & 0x8000 != 0 {
            return Err(Error::InvalidData);
        }
        let ttl = reader.u8()?;
        let protocol = reader.u8()?;
        let checksum = reader.u16_be()?;
        let source = reader.take(4)?;
        let destination = reader.take(4)?;
        let options = reader.take(ihl - 20)?;
        let payload = &bytes[ihl..total_length];

        Ok(Self {
            version,
            header_length: ihl,
            dscp: dscp_ecn >> 2,
            ecn: dscp_ecn & 0x03,
            total_length,
            identification,
            dont_fragment: flags_fragment & 0x4000 != 0,
            more_fragments: flags_fragment & 0x2000 != 0,
            fragment_offset: usize::from(flags_fragment & 0x1fff) * 8,
            ttl,
            protocol,
            checksum,
            source: Ipv4Address(source.try_into().map_err(|_| Error::UnexpectedEof)?),
            destination: Ipv4Address(destination.try_into().map_err(|_| Error::UnexpectedEof)?),
            options,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ipv4_packet() -> [u8; 24] {
        [
            0x45, 0x00, 0x00, 0x18, 0x12, 0x34, 0x40, 0x00, 0x40, 0x11, 0x00, 0x00, 192, 0, 2, 1,
            192, 0, 2, 2, 1, 2, 3, 4,
        ]
    }

    #[test]
    fn parses_ethernet_fields_without_alignment_assumptions() {
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 0x08, 0x00, 99];
        let mut reader = Reader::new(&bytes);
        let header = EthernetHeader::parse(&mut reader).unwrap();
        assert_eq!(header.destination, MacAddress([1, 2, 3, 4, 5, 6]));
        assert_eq!(header.source, MacAddress([7, 8, 9, 10, 11, 12]));
        assert_eq!(header.ether_type, 0x0800);
        assert_eq!(reader.remaining(), &[99]);
    }

    #[test]
    fn parses_ipv4_header_and_exposes_payload() {
        let bytes = ipv4_packet();
        let header = Ipv4Header::parse(&bytes).unwrap();
        assert_eq!(header.header_length, 20);
        assert_eq!(header.total_length, 24);
        assert!(header.dont_fragment);
        assert_eq!(header.fragment_offset, 0);
        assert_eq!(header.source, Ipv4Address([192, 0, 2, 1]));
        assert_eq!(header.destination, Ipv4Address([192, 0, 2, 2]));
        assert_eq!(header.payload, &[1, 2, 3, 4]);
    }

    #[test]
    fn rejects_invalid_ipv4_lengths_versions_and_reserved_flags() {
        let valid = ipv4_packet();

        let mut bad_version = valid;
        bad_version[0] = 0x65;
        assert_eq!(Ipv4Header::parse(&bad_version), Err(Error::InvalidData));

        let mut short_ihl = valid;
        short_ihl[0] = 0x44;
        assert_eq!(Ipv4Header::parse(&short_ihl), Err(Error::InvalidData));

        let mut declared_too_long = valid;
        declared_too_long[3] = 25;
        assert_eq!(
            Ipv4Header::parse(&declared_too_long),
            Err(Error::InvalidData)
        );

        let mut reserved_flag = valid;
        reserved_flag[6] |= 0x80;
        assert_eq!(Ipv4Header::parse(&reserved_flag), Err(Error::InvalidData));
    }

    #[test]
    fn rejects_every_truncated_ipv4_prefix() {
        let bytes = ipv4_packet();
        for length in 0..20 {
            assert!(Ipv4Header::parse(&bytes[..length]).is_err());
        }
    }
}
