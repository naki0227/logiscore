use crate::error::LogiscoreError;

pub(super) fn encode(
    bits: &[bool],
    codeword_bits: usize,
    depth: usize,
) -> Result<Vec<bool>, LogiscoreError> {
    validate(bits, codeword_bits, depth)?;
    let codeword_count = bits.len() / codeword_bits;
    let mut output = Vec::with_capacity(bits.len());
    for block_start in (0..codeword_count).step_by(depth) {
        let block_size = depth.min(codeword_count - block_start);
        for bit_index in 0..codeword_bits {
            for codeword_offset in 0..block_size {
                output.push(bits[(block_start + codeword_offset) * codeword_bits + bit_index]);
            }
        }
    }
    Ok(output)
}

pub(super) fn decode(
    bits: &[bool],
    codeword_bits: usize,
    depth: usize,
    codeword_count: usize,
) -> Result<Vec<bool>, LogiscoreError> {
    validate(bits, codeword_bits, depth)?;
    if bits.len() / codeword_bits != codeword_count {
        return Err(invalid_fec("interleave codeword count mismatch"));
    }
    let mut output = vec![false; bits.len()];
    let mut input_offset = 0;
    for block_start in (0..codeword_count).step_by(depth) {
        let block_size = depth.min(codeword_count - block_start);
        for bit_index in 0..codeword_bits {
            for codeword_offset in 0..block_size {
                output[(block_start + codeword_offset) * codeword_bits + bit_index] =
                    bits[input_offset];
                input_offset += 1;
            }
        }
    }
    Ok(output)
}

fn validate(bits: &[bool], codeword_bits: usize, depth: usize) -> Result<(), LogiscoreError> {
    if codeword_bits == 0 || depth == 0 || !bits.len().is_multiple_of(codeword_bits) {
        return Err(invalid_fec("invalid interleave dimensions"));
    }
    Ok(())
}

fn invalid_fec(message: &str) -> LogiscoreError {
    LogiscoreError::InvalidFec(message.to_owned())
}
