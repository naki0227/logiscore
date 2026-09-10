use crate::error::LogiscoreError;

const CODEWORD_BITS: usize = 13;
const DATA_POSITIONS: [usize; 8] = [3, 5, 6, 7, 9, 10, 11, 12];
const PARITY_POSITIONS: [usize; 4] = [1, 2, 4, 8];

pub(super) struct Decoded {
    pub data: Vec<u8>,
    pub corrected_codewords: usize,
}

pub(super) fn encode(data: &[u8]) -> Vec<bool> {
    let mut bits = Vec::with_capacity(data.len() * CODEWORD_BITS);
    for &byte in data {
        let mut codeword = [false; CODEWORD_BITS];
        for (bit_index, position) in DATA_POSITIONS.iter().enumerate() {
            codeword[position - 1] = (byte >> (7 - bit_index)) & 1 == 1;
        }
        for parity in PARITY_POSITIONS {
            codeword[parity - 1] = parity_value(&codeword, parity);
        }
        codeword[12] = codeword[..12]
            .iter()
            .fold(false, |parity, bit| parity ^ bit);
        bits.extend(codeword);
    }
    bits
}

pub(super) fn decode(bits: &[bool]) -> Result<Decoded, LogiscoreError> {
    if !bits.len().is_multiple_of(CODEWORD_BITS) {
        return Err(invalid_fec("Hamming codeword is truncated"));
    }
    let mut corrected_codewords = 0;
    let mut data = Vec::with_capacity(bits.len() / CODEWORD_BITS);
    for chunk in bits.chunks_exact(CODEWORD_BITS) {
        let mut codeword: [bool; CODEWORD_BITS] = chunk
            .try_into()
            .map_err(|_| invalid_fec("Hamming codeword is malformed"))?;
        let syndrome = PARITY_POSITIONS.into_iter().fold(0, |value, parity| {
            if parity_value(&codeword, parity) {
                value | parity
            } else {
                value
            }
        });
        let overall_mismatch = codeword.iter().fold(false, |parity, bit| parity ^ bit);
        match (syndrome, overall_mismatch) {
            (0, false) => {}
            (0, true) => corrected_codewords += 1,
            (position @ 1..=12, true) => {
                codeword[position - 1] = !codeword[position - 1];
                corrected_codewords += 1;
            }
            _ => return Err(invalid_fec("uncorrectable Hamming codeword")),
        }
        let byte = DATA_POSITIONS.iter().fold(0, |value, position| {
            (value << 1) | u8::from(codeword[position - 1])
        });
        data.push(byte);
    }
    Ok(Decoded {
        data,
        corrected_codewords,
    })
}

fn parity_value(codeword: &[bool; CODEWORD_BITS], parity_position: usize) -> bool {
    (1..=12)
        .filter(|position| position & parity_position != 0)
        .fold(false, |parity, position| parity ^ codeword[position - 1])
}

fn invalid_fec(message: &str) -> LogiscoreError {
    LogiscoreError::InvalidFec(message.to_owned())
}
