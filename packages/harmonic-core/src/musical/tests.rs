use super::*;

fn c_major() -> TonalContext {
    TonalContext::new(0, Mode::Major).unwrap()
}

#[test]
fn tonal_context_rejects_pitch_class_outside_octave() {
    assert_eq!(
        TonalContext::new(12, Mode::Major),
        Err(MusicalError::InvalidRoot(12))
    );
}

#[test]
fn tonal_context_transposes_scale_pitch_classes() {
    let d_dorian = TonalContext::new(2, Mode::Dorian).unwrap();
    assert_eq!(d_dorian.scale_pitch_classes(), [2, 4, 5, 7, 9, 11, 0]);
}

#[test]
fn chord_planner_repeats_i_v_vi_iv() {
    let planner = ChordPlanner::new(c_major());
    let degrees = (0..8)
        .map(|event| planner.chord_at(event).degree())
        .collect::<Vec<_>>();
    assert_eq!(degrees, [1, 5, 6, 4, 1, 5, 6, 4]);
    assert_eq!(planner.chord_at(0).pitch_classes(), [0, 4, 7]);
}

#[test]
fn every_progression_step_produces_16_unique_valid_candidates() {
    let planner = ChordPlanner::new(c_major());
    for event in 0..4 {
        let chord = planner.chord_at(event);
        let set = CandidateSet::generate(chord, None).unwrap();
        let mut unique = set.candidates().to_vec();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 16);
        assert!(set
            .candidates()
            .iter()
            .all(|candidate| candidate.is_valid_for(chord)));
    }
}

#[test]
fn candidate_generation_is_deterministic_with_previous_voicing() {
    let planner = ChordPlanner::new(c_major());
    let first = CandidateSet::generate(planner.chord_at(0), None).unwrap();
    let previous = first.encode_symbol(11).unwrap();
    let left = CandidateSet::generate(planner.chord_at(1), Some(previous)).unwrap();
    let right = CandidateSet::generate(planner.chord_at(1), Some(previous)).unwrap();
    assert_eq!(left, right);
}

#[test]
fn previous_voicing_is_preferred_when_the_chord_is_unchanged() {
    let chord = ChordPlanner::new(c_major()).chord_at(0);
    let initial = CandidateSet::generate(chord, None).unwrap();
    let previous = initial.encode_symbol(15).unwrap();
    let guided = CandidateSet::generate(chord, Some(previous)).unwrap();
    assert_eq!(guided.candidates()[0], previous);
}

#[test]
fn all_four_bit_symbols_roundtrip() {
    let chord = ChordPlanner::new(c_major()).chord_at(0);
    let set = CandidateSet::generate(chord, None).unwrap();
    for symbol in 0..16 {
        let voicing = set.encode_symbol(symbol).unwrap();
        assert_eq!(set.decode_symbol(voicing).unwrap(), symbol);
    }
    assert_eq!(set.encode_symbol(16), Err(MusicalError::InvalidSymbol(16)));
}

#[test]
fn symbol_stream_roundtrips_across_chord_and_voice_leading_state() {
    let codec = MusicalSymbolCodec::new(c_major());
    let symbols = (0..16).chain((0..16).rev()).collect::<Vec<_>>();
    let voicings = codec.encode(&symbols).unwrap();
    assert_eq!(codec.decode(&voicings).unwrap(), symbols);
}

#[test]
fn voices_are_strictly_ordered_and_inside_their_ranges() {
    let chord = ChordPlanner::new(c_major()).chord_at(0);
    let set = CandidateSet::generate(chord, None).unwrap();
    for voicing in set.candidates() {
        let notes = voicing.notes();
        assert!(notes.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(notes
            .into_iter()
            .zip(VOICES)
            .all(|(note, voice)| voice.contains(note)));
    }
}

#[test]
fn candidate_count_can_change_without_losing_constraints() {
    let planner = ChordPlanner::new(c_major());
    for count in [8, 16, 32] {
        let set = CandidateSet::generate_with_count(planner.chord_at(0), None, count).unwrap();
        assert_eq!(set.candidates().len(), count);
        assert!(set
            .candidates()
            .iter()
            .all(|voicing| voicing.is_valid_for(planner.chord_at(0))));
    }
    assert_eq!(
        CandidateSet::generate_with_count(planner.chord_at(0), None, 12),
        Err(MusicalError::InvalidCandidateCount(12))
    );
}

#[test]
fn rhythmic_codec_roundtrips_multiple_symbol_channels() {
    let codec = RhythmicMusicalCodec::new(c_major());
    let bytes = (0..=u8::MAX).collect::<Vec<_>>();
    let events = codec.encode(&bytes).unwrap();
    assert_eq!(codec.decode(&events, bytes.len()).unwrap(), bytes);
    assert!(events
        .iter()
        .all(|event| [240, 360, 480, 720].contains(&event.duration_ticks())));
}

#[test]
fn rhythmic_codec_uses_fewer_events_than_four_bit_baseline() {
    let codec = RhythmicMusicalCodec::new(c_major());
    let bytes = vec![0x5a; 120];
    let events = codec.encode(&bytes).unwrap();
    assert!(events.len() < bytes.len() * 2);
    assert_eq!(codec.decode(&events, bytes.len()).unwrap(), bytes);
}

#[test]
fn rhythmic_codec_handles_empty_payload() {
    let codec = RhythmicMusicalCodec::new(c_major());
    assert!(codec.encode(&[]).unwrap().is_empty());
    assert!(codec.decode(&[], 0).unwrap().is_empty());
}
