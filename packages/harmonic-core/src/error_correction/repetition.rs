use crate::error::LogiscoreError;

pub(super) fn encode(data: &[u8], repetitions: usize) -> Vec<u8> {
    let mut output = Vec::with_capacity(data.len() * repetitions);
    for _ in 0..repetitions {
        output.extend_from_slice(data);
    }
    output
}

pub(super) fn decode(data: &[u8], repetitions: usize) -> Result<Vec<u8>, LogiscoreError> {
    if repetitions < 3 || repetitions.is_multiple_of(2) || !data.len().is_multiple_of(repetitions) {
        return Err(invalid_fec("invalid repetition frame"));
    }
    let copy_length = data.len() / repetitions;
    let mut output = Vec::with_capacity(copy_length);
    for byte_index in 0..copy_length {
        let mut byte = 0;
        for bit_index in 0..8 {
            let mask = 1 << bit_index;
            let votes = (0..repetitions)
                .filter(|copy| data[copy * copy_length + byte_index] & mask != 0)
                .count();
            if votes > repetitions / 2 {
                byte |= mask;
            }
        }
        output.push(byte);
    }
    Ok(output)
}

fn invalid_fec(message: &str) -> LogiscoreError {
    LogiscoreError::InvalidFec(message.to_owned())
}
