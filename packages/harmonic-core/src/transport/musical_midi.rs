use std::collections::BTreeMap;

use midly::{Format, MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};

use crate::error::LogiscoreError;
use crate::musical::{MusicalSymbolCodec, TonalContext, Voicing};
use crate::transport::Transport;

const TRACK_NAME: &[u8] = b"Logiscore v2 Musical";
const TICKS_PER_BEAT: u16 = 480;
const NOTE_DURATION: u32 = 360;
const REST_DURATION: u32 = 120;
const VELOCITY: u8 = 80;
const PROGRAMS: [u8; 4] = [32, 42, 41, 40];
const MAX_PACKET_BYTES: usize = 64 * 1024;
const MAX_SYMBOLS: usize = MAX_PACKET_BYTES * 2;

#[derive(Debug, Clone, Copy)]
pub struct MusicalMidiTransport {
    codec: MusicalSymbolCodec,
}

impl MusicalMidiTransport {
    pub fn c_major() -> Self {
        Self {
            codec: MusicalSymbolCodec::new(TonalContext::C_MAJOR),
        }
    }
}

impl Default for MusicalMidiTransport {
    fn default() -> Self {
        Self::c_major()
    }
}

impl Transport for MusicalMidiTransport {
    fn encode(&self, packet: &[u8]) -> Result<Vec<u8>, LogiscoreError> {
        if packet.len() > MAX_PACKET_BYTES {
            return Err(LogiscoreError::InvalidPacket(format!(
                "Musical MIDI packet exceeds {MAX_PACKET_BYTES} bytes"
            )));
        }
        let symbols = packet
            .iter()
            .flat_map(|byte| [byte >> 4, byte & 0x0f])
            .collect::<Vec<_>>();
        let voicings = self.codec.encode(&symbols).map_err(musical_error)?;
        build_midi(&voicings)
    }

    fn decode(&self, transport_data: &[u8]) -> Result<Vec<u8>, LogiscoreError> {
        let voicings = parse_midi(transport_data)?;
        let symbols = self.codec.decode(&voicings).map_err(musical_error)?;
        if symbols.len() % 2 != 0 {
            return Err(LogiscoreError::InvalidMidi(
                "musical symbol count must be even".into(),
            ));
        }
        Ok(symbols
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| (pair[0] << 4) | pair[1])
            .collect())
    }
}

fn build_midi(voicings: &[Voicing]) -> Result<Vec<u8>, LogiscoreError> {
    let mut track = Vec::new();
    write_meta(&mut track, 0, 0x03, TRACK_NAME);
    write_meta(&mut track, 0, 0x51, &[0x07, 0xA1, 0x20]);
    for (channel, program) in PROGRAMS.into_iter().enumerate() {
        write_vlq(&mut track, 0);
        track.extend_from_slice(&[0xC0 | channel as u8, program]);
    }
    for (event_index, voicing) in voicings.iter().enumerate() {
        for (voice, note) in voicing.notes().into_iter().enumerate() {
            write_vlq(
                &mut track,
                if voice == 0 && event_index > 0 {
                    REST_DURATION
                } else {
                    0
                },
            );
            track.extend_from_slice(&[0x90 | voice as u8, note, VELOCITY]);
        }
        for (voice, note) in voicing.notes().into_iter().enumerate() {
            write_vlq(&mut track, if voice == 0 { NOTE_DURATION } else { 0 });
            track.extend_from_slice(&[0x80 | voice as u8, note, 0]);
        }
    }
    write_meta(&mut track, 0, 0x2f, &[]);
    let track_length = u32::try_from(track.len())
        .map_err(|_| LogiscoreError::InvalidMidi("musical track exceeds u32 length".into()))?;
    let mut midi = Vec::with_capacity(22 + track.len());
    midi.extend_from_slice(b"MThd");
    midi.extend_from_slice(&6u32.to_be_bytes());
    midi.extend_from_slice(&0u16.to_be_bytes());
    midi.extend_from_slice(&1u16.to_be_bytes());
    midi.extend_from_slice(&TICKS_PER_BEAT.to_be_bytes());
    midi.extend_from_slice(b"MTrk");
    midi.extend_from_slice(&track_length.to_be_bytes());
    midi.extend_from_slice(&track);
    Ok(midi)
}

fn parse_midi(bytes: &[u8]) -> Result<Vec<Voicing>, LogiscoreError> {
    let smf =
        Smf::parse(bytes).map_err(|error| LogiscoreError::MidiParseError(error.to_string()))?;
    if smf.header.format != Format::SingleTrack || smf.tracks.len() != 1 {
        return Err(LogiscoreError::InvalidMidi(
            "musical MIDI must contain exactly one track".into(),
        ));
    }
    if !matches!(smf.header.timing, Timing::Metrical(value) if value.as_int() == TICKS_PER_BEAT) {
        return Err(LogiscoreError::InvalidMidi(
            "musical MIDI must use 480 ticks per beat".into(),
        ));
    }
    let mut found_marker = false;
    let mut tick = 0u64;
    let mut chords = BTreeMap::<u64, [Option<u8>; 4]>::new();
    for event in &smf.tracks[0] {
        tick = tick
            .checked_add(u64::from(event.delta.as_int()))
            .ok_or_else(|| LogiscoreError::InvalidMidi("absolute tick overflow".into()))?;
        match event.kind {
            TrackEventKind::Meta(MetaMessage::TrackName(name)) if name == TRACK_NAME => {
                found_marker = true;
            }
            TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOn { key, vel },
            } if vel.as_int() > 0 => {
                let voice = usize::from(channel.as_int());
                if voice >= 4 {
                    return Err(LogiscoreError::InvalidMidi(
                        "musical note uses an unsupported channel".into(),
                    ));
                }
                if !chords.contains_key(&tick) && chords.len() >= MAX_SYMBOLS {
                    return Err(LogiscoreError::InvalidMidi(
                        "musical MIDI exceeds the symbol limit".into(),
                    ));
                }
                let notes = chords.entry(tick).or_default();
                if notes[voice].replace(key.as_int()).is_some() {
                    return Err(LogiscoreError::InvalidMidi(
                        "duplicate voice at musical event".into(),
                    ));
                }
            }
            _ => {}
        }
    }
    if !found_marker {
        return Err(LogiscoreError::InvalidMidi(
            "missing Logiscore v2 Musical marker".into(),
        ));
    }
    chords
        .into_values()
        .map(|notes| {
            let notes = notes
                .map(|note| {
                    note.ok_or_else(|| {
                        LogiscoreError::InvalidMidi("incomplete four-voice event".into())
                    })
                })
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
                .map_err(|_| LogiscoreError::InvalidMidi("incomplete four-voice event".into()))?;
            Ok(Voicing::from_notes(notes))
        })
        .collect()
}

fn write_meta(track: &mut Vec<u8>, delta: u32, kind: u8, data: &[u8]) {
    write_vlq(track, delta);
    track.extend_from_slice(&[0xff, kind]);
    write_vlq(track, data.len() as u32);
    track.extend_from_slice(data);
}

fn write_vlq(output: &mut Vec<u8>, mut value: u32) {
    let mut buffer = [0u8; 4];
    let mut index = 3;
    buffer[index] = (value & 0x7f) as u8;
    while {
        value >>= 7;
        value > 0
    } {
        index -= 1;
        buffer[index] = ((value & 0x7f) as u8) | 0x80;
    }
    output.extend_from_slice(&buffer[index..]);
}

fn musical_error(error: crate::musical::MusicalError) -> LogiscoreError {
    LogiscoreError::InvalidMidi(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn musical_transport_roundtrips_every_byte_value() {
        let original = (0..=u8::MAX).collect::<Vec<_>>();
        let transport = MusicalMidiTransport::default();
        let midi = transport.encode(&original).unwrap();
        assert_eq!(transport.decode(&midi).unwrap(), original);
    }

    #[test]
    fn musical_transport_is_deterministic_and_valid_smf() {
        let transport = MusicalMidiTransport::default();
        let first = transport.encode(b"Logiscore").unwrap();
        let second = transport.encode(b"Logiscore").unwrap();
        assert_eq!(first, second);
        assert!(Smf::parse(&first).is_ok());
    }

    #[test]
    fn musical_transport_does_not_encode_data_in_velocity() {
        let midi = MusicalMidiTransport::default().encode(&[0x0f]).unwrap();
        let smf = Smf::parse(&midi).unwrap();
        let velocities = smf.tracks[0].iter().filter_map(|event| match event.kind {
            TrackEventKind::Midi {
                message: MidiMessage::NoteOn { vel, .. },
                ..
            } if vel.as_int() > 0 => Some(vel.as_int()),
            _ => None,
        });
        assert!(velocities.eq([VELOCITY; 8]));
    }

    #[test]
    fn musical_transport_rejects_packet_over_limit() {
        let packet = vec![0; MAX_PACKET_BYTES + 1];
        assert!(MusicalMidiTransport::default().encode(&packet).is_err());
    }
}
