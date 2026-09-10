const POLYNOMIAL: u32 = 0xedb8_8320;

pub(super) fn checksum(data: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (POLYNOMIAL & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_standard_crc32_vector() {
        assert_eq!(checksum(b"123456789"), 0xcbf4_3926);
    }
}
