use super::*;
use proptest::prelude::*;

#[test]
fn protected_frame_roundtrips_all_byte_values() {
    let input = (0..=u8::MAX).collect::<Vec<_>>();
    let protected = protect(&input).unwrap();
    let recovered = recover(&protected).unwrap();
    assert_eq!(recovered.into_data(), input);
}

#[test]
fn protected_frame_roundtrips_empty_data() {
    assert!(recover(&protect(&[]).unwrap())
        .unwrap()
        .into_data()
        .is_empty());
}

#[test]
fn repetition_recovers_different_copy_damage() {
    let input = b"burst protection";
    let mut protected = protect(input).unwrap();
    let copy_length = protected.len() / REPETITIONS;
    protected[2] ^= 0xff;
    protected[copy_length + 5] ^= 0xff;
    protected[copy_length * 2 + 8] ^= 0xff;
    assert_eq!(recover(&protected).unwrap().into_data(), input);
}

#[test]
fn repetition_recovers_a_contiguous_burst_in_one_copy() {
    let input = b"interleaved burst protection";
    let mut protected = protect(input).unwrap();
    for byte in &mut protected[3..11] {
        *byte ^= 0xff;
    }
    assert_eq!(recover(&protected).unwrap().into_data(), input);
}

#[test]
fn secded_corrects_one_post_vote_bit() {
    let input = b"single bit";
    let mut protected = protect(input).unwrap();
    let copy_length = protected.len() / REPETITIONS;
    for copy in 0..2 {
        protected[copy * copy_length] ^= 0x80;
    }
    let recovered = recover(&protected).unwrap();
    assert_eq!(recovered.data, input);
    assert_eq!(recovered.corrected_codewords(), 1);
}

#[test]
fn secded_rejects_two_bits_in_one_codeword() {
    let input = b"double bit";
    let mut protected = protect(input).unwrap();
    let copy_length = protected.len() / REPETITIONS;
    for copy in 0..2 {
        protected[copy * copy_length] ^= 0x80;
        protected[copy * copy_length + 1] ^= 0x80;
    }
    assert!(recover(&protected).is_err());
}

#[test]
fn crc_rejects_an_undetected_three_bit_hamming_error() {
    let mut protected = protect(b"CRC fallback").unwrap();
    let copy_length = protected.len() / REPETITIONS;
    for copy in 0..2 {
        for byte_offset in 0..3 {
            protected[copy * copy_length + byte_offset] ^= 0x80;
        }
    }
    assert!(matches!(
        recover(&protected),
        Err(LogiscoreError::InvalidFec(message)) if message.contains("CRC-32")
    ));
}

#[test]
fn malformed_or_truncated_frames_are_rejected() {
    assert!(recover(&[]).is_err());
    let mut protected = protect(b"truncated").unwrap();
    protected.pop();
    assert!(recover(&protected).is_err());
}

#[test]
fn adaptive_fec_profiles_roundtrip_and_increase_redundancy() {
    let input = b"adaptive FEC";
    let balanced = protect_for_profile(input, 1).unwrap();
    let conversation = protect_for_profile(input, 2).unwrap();
    let noisy = protect_for_profile(input, 3).unwrap();
    assert_eq!(
        recover_for_profile(&balanced, 1).unwrap().into_data(),
        input
    );
    assert_eq!(
        recover_for_profile(&conversation, 2).unwrap().into_data(),
        input
    );
    assert_eq!(recover_for_profile(&noisy, 3).unwrap().into_data(), input);
    assert_eq!(conversation.len(), balanced.len());
    assert!(noisy.len() > balanced.len());
    assert!(recover_for_profile(&noisy, 1).is_err());
}

#[test]
fn noisy_profile_survives_two_damaged_repetition_copies() {
    let input = b"five-copy majority";
    let mut protected = protect_for_profile(input, 3).unwrap();
    let copy_length = protected.len() / 5;
    for copy in 0..2 {
        for byte in &mut protected[copy * copy_length..(copy + 1) * copy_length] {
            *byte ^= 0xff;
        }
    }
    assert_eq!(
        recover_for_profile(&protected, 3).unwrap().into_data(),
        input
    );
}

proptest! {
    #[test]
    fn arbitrary_payloads_roundtrip(data in prop::collection::vec(any::<u8>(), 0..512)) {
        prop_assert_eq!(recover(&protect(&data)? )?.into_data(), data);
    }

    #[test]
    fn arbitrary_single_post_vote_bit_is_recovered(
        data in prop::collection::vec(any::<u8>(), 0..256),
        selector in any::<usize>(),
    ) {
        let mut protected = protect(&data)?;
        let copy_length = protected.len() / REPETITIONS;
        let bit_offset = selector % (copy_length * 8);
        let byte_offset = bit_offset / 8;
        let mask = 1 << (7 - bit_offset % 8);
        for copy in 0..2 {
            protected[copy * copy_length + byte_offset] ^= mask;
        }
        prop_assert_eq!(recover(&protected)?.into_data(), data);
    }
}
