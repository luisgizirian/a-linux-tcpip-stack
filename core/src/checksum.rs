pub fn internet_checksum(chunks: &[&[u8]]) -> u16 {
    let mut sum = 0u32;
    let mut pending_high = None;

    for chunk in chunks {
        for &byte in *chunk {
            if let Some(high) = pending_high.take() {
                sum = sum.wrapping_add(u32::from(u16::from_be_bytes([high, byte])));
            } else {
                pending_high = Some(byte);
            }
        }
    }

    if let Some(high) = pending_high {
        sum = sum.wrapping_add(u32::from(high) << 8);
    }

    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }

    !(sum as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_rfc_1071_even_length_example() {
        let bytes = [0x00, 0x01, 0xf2, 0x03, 0xf4, 0xf5, 0xf6, 0xf7];
        assert_eq!(internet_checksum(&[&bytes]), 0x220d);
    }

    #[test]
    fn pads_an_odd_number_of_bytes_with_zero() {
        assert_eq!(
            internet_checksum(&[&[0x01, 0x02, 0xff]]),
            internet_checksum(&[&[0x01, 0x02, 0xff, 0x00]])
        );
    }

    #[test]
    fn pairs_bytes_across_scatter_gather_boundaries() {
        let contiguous = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        assert_eq!(
            internet_checksum(&[&contiguous]),
            internet_checksum(&[&contiguous[..1], &contiguous[1..3], &contiguous[3..]])
        );
    }

    #[test]
    fn checksum_field_of_containing_packet_verifies_as_zero() {
        let header = [
            0x45, 0x00, 0x00, 0x73, 0x00, 0x00, 0x40, 0x00, 0x40, 0x11, 0xb8, 0x61, 0xc0, 0xa8,
            0x00, 0x01, 0xc0, 0xa8, 0x00, 0xc7,
        ];
        assert_eq!(internet_checksum(&[&header]), 0);
    }
}
