#[path = "support/wav_fixtures.rs"]
mod fixtures;
use daw_app::sample_loading::{
    CANCEL_CHECK_FRAMES, IoOperation, LoadError, LoadLimit, MAX_ENCODED_FILE_BYTES,
    MAX_RIFF_CHUNKS, WavProblem, decode_wav, load_wav_file,
};
use daw_engine::{EngineConfig, MAX_SAMPLE_FRAMES};
use fixtures::*;
use std::{
    cell::Cell,
    io::{self, Cursor, Read, Seek, SeekFrom},
    sync::atomic::{AtomicBool, Ordering},
};

fn target(channels: u16) -> EngineConfig {
    EngineConfig::new(48_000, channels).unwrap()
}
fn decode(bytes: Vec<u8>, channels: u16) -> Result<daw_engine::SampleBuffer, LoadError> {
    decode_wav(
        Cursor::new(bytes),
        target(channels),
        &AtomicBool::new(false),
    )
}
fn f32_bytes(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

#[test]
fn pcm16_endpoints_float_bits_and_channel_compatibility_are_exact() {
    let integers = [i16::MIN, -1, 0, i16::MAX];
    let encoded = integers
        .iter()
        .flat_map(|v| v.to_le_bytes())
        .collect::<Vec<_>>();
    for channels in [1, 2] {
        for extensible in [false, true] {
            let sample = decode(wav(1, 16, channels, &encoded, extensible), 2).unwrap();
            assert_eq!(
                sample.samples(),
                [-1.0, -1.0 / 32768.0, 0.0, 32767.0 / 32768.0]
            );
            assert_eq!(sample.config(), target(channels));
            assert_eq!(sample.frames(), 4 / usize::from(channels));
            let values = [-1.0, -0.0, 0.1, 1.0];
            let sample = decode(wav(3, 32, channels, &f32_bytes(&values), extensible), 2).unwrap();
            assert_eq!(
                sample
                    .samples()
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>(),
                values.map(f32::to_bits)
            );
        }
    }
    for tag in [1, 3] {
        let bits = if tag == 1 { 16 } else { 32 };
        let mut fmt = format(tag, bits, 1, 48_000, false);
        fmt.extend([0, 0]);
        assert!(decode(riff(&[chunk(b"fmt ", &fmt), chunk(b"data", &[0; 4])]), 1).is_ok());
    }
}

#[test]
fn extensible_precision_masks_and_subformats_are_checked() {
    for channels in [1, 2] {
        for mask in [0u32, 1, 2, 3, 4, 7] {
            for valid_bits in [0u16, 32, 24] {
                let mut fmt = format(3, 32, channels, 48_000, true);
                fmt[18..20].copy_from_slice(&valid_bits.to_le_bytes());
                fmt[20..24].copy_from_slice(&mask.to_le_bytes());
                let result = decode(riff(&[chunk(b"fmt ", &fmt), chunk(b"data", &[0; 8])]), 2);
                let allowed_mask = if channels == 1 {
                    [0, 1, 4].contains(&mask)
                } else {
                    [0, 3].contains(&mask)
                };
                assert_eq!(result.is_ok(), allowed_mask && valid_bits != 24);
            }
        }
    }
    let mut fmt = format(3, 32, 1, 48_000, true);
    fmt[39] ^= 1;
    assert!(matches!(
        decode(riff(&[chunk(b"fmt ", &fmt), chunk(b"data", &[0; 4])]), 1),
        Err(LoadError::UnsupportedEncoding)
    ));
    fmt[16] = 21;
    assert!(matches!(
        decode(riff(&[chunk(b"fmt ", &fmt), chunk(b"data", &[0; 4])]), 1),
        Err(LoadError::Malformed(WavProblem::FormatSize))
    ));
}

#[test]
fn unknown_odd_metadata_and_trailers_are_scanned_without_loading_their_bodies() {
    let bytes = riff(&[
        chunk(b"JUNK", &[9; 101]),
        chunk(b"fmt ", &format(3, 32, 1, 48_000, false)),
        chunk(b"data", &f32_bytes(&[0.25])),
        chunk(b"LIST", b"INFOodd"),
        chunk(b"fact", &[0; 4]),
    ]);
    let state = State::default();
    let cancel = AtomicBool::new(false);
    let sample = decode_wav(
        Probe::new(bytes, &state, &cancel, Fault::None),
        target(1),
        &cancel,
    )
    .unwrap();
    assert_eq!(sample.samples(), [0.25]);
    assert_eq!(state.read_bytes.get(), 12 + 5 * 8 + 16 + 4 + 4);
    assert!(state.dropped.get());
}

#[test]
fn malformed_layouts_and_boundaries_are_rejected_including_after_data() {
    let fmt = chunk(b"fmt ", &format(1, 16, 1, 48_000, false));
    let data = chunk(b"data", &[0; 2]);
    for (parts, problem) in [
        (
            vec![data.clone(), fmt.clone()],
            WavProblem::DataBeforeFormat,
        ),
        (
            vec![fmt.clone(), data.clone(), fmt.clone()],
            WavProblem::DuplicateFormat,
        ),
        (
            vec![fmt.clone(), data.clone(), data.clone()],
            WavProblem::DuplicateData,
        ),
        (vec![fmt.clone()], WavProblem::MissingFormatOrData),
        (
            vec![fmt.clone(), chunk(b"data", &[])],
            WavProblem::EmptyData,
        ),
        (
            vec![fmt.clone(), chunk(b"data", &[0])],
            WavProblem::PartialFrame,
        ),
        (
            vec![chunk(b"fmt ", &[0; 15]), data.clone()],
            WavProblem::FormatSize,
        ),
    ] {
        assert!(matches!(decode(riff(&parts),1), Err(LoadError::Malformed(p)) if p == problem));
    }
    let mut bad = riff(&[fmt.clone(), data.clone()]);
    bad[4] ^= 1;
    assert!(matches!(
        decode(bad, 1),
        Err(LoadError::Malformed(WavProblem::RiffLength))
    ));
    let mut bad = riff(&[fmt.clone(), data.clone()]);
    bad[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(
        decode(bad, 1),
        Err(LoadError::Malformed(WavProblem::ChunkBoundary))
    ));
    for name in [b"RIFX", b"RF64"] {
        let mut bad = riff(&[fmt.clone(), data.clone()]);
        bad[..4].copy_from_slice(name);
        assert!(matches!(
            decode(bad, 1),
            Err(LoadError::UnsupportedContainer)
        ));
    }
    for extra in [chunk(b"plst", &[]), chunk(b"LIST", b"wavl")] {
        assert!(matches!(
            decode(riff(&[fmt.clone(), data.clone(), extra]), 1),
            Err(LoadError::UnsupportedLayout)
        ));
    }
    // Even an incomplete trailer must be rejected after a valid data chunk.
    let mut bad = riff(&[fmt, data]);
    bad.push(0);
    let size = bad.len() as u32 - 8;
    bad[4..8].copy_from_slice(&size.to_le_bytes());
    assert!(matches!(
        decode(bad, 1),
        Err(LoadError::Malformed(WavProblem::ChunkBoundary))
    ));
}

#[test]
fn metadata_unsupported_formats_and_invalid_pcm_are_typed() {
    for (tag, bits) in [(1, 8), (1, 24), (1, 32), (3, 64), (6, 16)] {
        assert!(matches!(
            decode(wav(tag, bits, 1, &[0; 8], false), 1),
            Err(LoadError::UnsupportedEncoding)
        ));
    }
    for (offset, bytes, expected) in [
        (4, 0u32.to_le_bytes().to_vec(), WavProblem::SampleRate),
        (8, 1u32.to_le_bytes().to_vec(), WavProblem::ByteRate),
        (12, 1u16.to_le_bytes().to_vec(), WavProblem::BlockAlignment),
    ] {
        let mut fmt = format(1, 16, 1, 48_000, false);
        fmt[offset..offset + bytes.len()].copy_from_slice(&bytes);
        assert!(
            matches!(decode(riff(&[chunk(b"fmt ",&fmt),chunk(b"data",&[0;2])]),1),Err(LoadError::Malformed(p)) if p==expected)
        );
    }
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 1.01, -1.01] {
        assert!(matches!(
            decode(wav(3, 32, 1, &f32_bytes(&[0.0, value]), false), 1),
            Err(LoadError::InvalidPcm { sample_index: 1 })
        ));
    }
    assert!(matches!(
        decode(wav(1, 16, 2, &[0; 4], false), 1),
        Err(LoadError::StereoToMono)
    ));
    assert!(matches!(
        decode(wav(1, 16, 3, &[0; 6], false), 2),
        Err(LoadError::UnsupportedChannels)
    ));
    let bytes = riff(&[
        chunk(b"fmt ", &format(1, 16, 1, 44_100, false)),
        chunk(b"data", &[0; 2]),
    ]);
    assert!(matches!(
        decode(bytes, 1),
        Err(LoadError::SampleRateMismatch)
    ));
    let rate = u32::MAX;
    let bytes = riff(&[
        chunk(b"fmt ", &format(3, 32, 2, rate, false)),
        chunk(b"data", &[0; 8]),
    ]);
    assert!(matches!(
        decode_wav(
            Cursor::new(bytes),
            EngineConfig::new(rate, 2).unwrap(),
            &AtomicBool::new(false)
        ),
        Err(LoadError::LimitExceeded(LoadLimit::Arithmetic))
    ));
}

#[test]
fn file_frame_and_chunk_limits_accept_exact_maximum_and_reject_one_over() {
    for frames in [MAX_SAMPLE_FRAMES, MAX_SAMPLE_FRAMES + 1] {
        let result = decode(wav(3, 32, 2, &vec![0; frames * 8], true), 2);
        if frames == MAX_SAMPLE_FRAMES {
            assert_eq!(result.unwrap().frames(), frames);
        } else {
            assert!(matches!(
                result,
                Err(LoadError::LimitExceeded(LoadLimit::Frames))
            ));
        }
    }
    let fixed = wav(1, 16, 1, &[0; 2], false).len();
    let filler = vec![0; MAX_ENCODED_FILE_BYTES as usize - fixed - 8];
    let mut max = riff(&[
        chunk(b"fmt ", &format(1, 16, 1, 48_000, false)),
        chunk(b"JUNK", &filler),
        chunk(b"data", &[0; 2]),
    ]);
    assert_eq!(max.len() as u64, MAX_ENCODED_FILE_BYTES);
    assert!(decode_wav(Cursor::new(&max), target(1), &AtomicBool::new(false)).is_ok());
    max.push(0);
    assert!(matches!(
        decode(max, 1),
        Err(LoadError::LimitExceeded(LoadLimit::EncodedBytes))
    ));
    for count in [MAX_RIFF_CHUNKS, MAX_RIFF_CHUNKS + 1] {
        let mut chunks = vec![chunk(b"fmt ", &format(1, 16, 1, 48_000, false))];
        chunks.extend((0..count - 2).map(|_| chunk(b"JUNK", &[])));
        chunks.push(chunk(b"data", &[0; 2]));
        let result = decode(riff(&chunks), 1);
        assert_eq!(result.is_ok(), count == MAX_RIFF_CHUNKS);
        if count > MAX_RIFF_CHUNKS {
            assert!(matches!(
                result,
                Err(LoadError::LimitExceeded(LoadLimit::Chunks))
            ));
        }
    }
}

#[derive(Default)]
struct State {
    dropped: Cell<bool>,
    read_bytes: Cell<usize>,
    max_read: Cell<usize>,
    ends: Cell<usize>,
    data_reads: Cell<usize>,
}
#[derive(Clone, Copy)]
enum Fault {
    None,
    Read,
    Seek,
    Truncate,
    ChangedLength,
    CancelHeader,
    CancelReserve,
    CancelData,
    CancelSecondData,
    CancelFinal,
    PartialInterrupted,
}
struct Probe<'a> {
    cursor: Cursor<Vec<u8>>,
    state: &'a State,
    cancel: &'a AtomicBool,
    fault: Fault,
    interrupted: bool,
}
impl<'a> Probe<'a> {
    fn new(bytes: Vec<u8>, state: &'a State, cancel: &'a AtomicBool, fault: Fault) -> Self {
        Self {
            cursor: Cursor::new(bytes),
            state,
            cancel,
            fault,
            interrupted: false,
        }
    }
}
impl Read for Probe<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let data = self.state.ends.get() >= 2;
        if matches!(self.fault, Fault::Read) && data {
            return Err(io::ErrorKind::Other.into());
        }
        if matches!(self.fault, Fault::Truncate) && data {
            return Ok(0);
        }
        if matches!(self.fault, Fault::PartialInterrupted) && !self.interrupted {
            self.interrupted = true;
            return Err(io::ErrorKind::Interrupted.into());
        }
        let wanted = if matches!(self.fault, Fault::PartialInterrupted) {
            1
        } else {
            bytes.len()
        };
        self.state
            .max_read
            .set(self.state.max_read.get().max(bytes.len()));
        let n = self.cursor.read(&mut bytes[..wanted])?;
        if data {
            self.state.data_reads.set(self.state.data_reads.get() + 1);
        }
        self.state.read_bytes.set(self.state.read_bytes.get() + n);
        if matches!(self.fault, Fault::CancelHeader)
            || (data && matches!(self.fault, Fault::CancelData))
            || (self.state.data_reads.get() == 2 && matches!(self.fault, Fault::CancelSecondData))
        {
            self.cancel.store(true, Ordering::Release);
        }
        Ok(n)
    }
}
impl Seek for Probe<'_> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        if matches!(self.fault, Fault::Seek) {
            return Err(io::ErrorKind::Other.into());
        }
        let value = self.cursor.seek(pos)?;
        if matches!(pos, SeekFrom::End(_)) {
            let ends = self.state.ends.get() + 1;
            self.state.ends.set(ends);
            if (ends == 2 && matches!(self.fault, Fault::CancelReserve))
                || (ends == 3 && matches!(self.fault, Fault::CancelFinal))
            {
                self.cancel.store(true, Ordering::Release);
            }
            if ends == 3 && matches!(self.fault, Fault::ChangedLength) {
                return Ok(value + 1);
            }
        }
        Ok(value)
    }
}
impl Drop for Probe<'_> {
    fn drop(&mut self) {
        self.state.dropped.set(true);
    }
}

#[test]
fn cancellation_and_injected_io_failures_close_owned_readers() {
    for fault in [
        Fault::Read,
        Fault::Seek,
        Fault::Truncate,
        Fault::ChangedLength,
        Fault::CancelHeader,
        Fault::CancelReserve,
        Fault::CancelData,
        Fault::CancelSecondData,
        Fault::CancelFinal,
    ] {
        let cancel = AtomicBool::new(false);
        let state = State::default();
        let result = decode_wav(
            Probe::new(
                wav(3, 32, 2, &vec![0; 600 * 8], false),
                &state,
                &cancel,
                fault,
            ),
            target(2),
            &cancel,
        );
        match fault {
            Fault::Read => assert!(matches!(
                result,
                Err(LoadError::Io {
                    operation: IoOperation::Read,
                    ..
                })
            )),
            Fault::Seek => assert!(matches!(
                result,
                Err(LoadError::Io {
                    operation: IoOperation::Seek,
                    ..
                })
            )),
            Fault::Truncate => assert!(matches!(
                result,
                Err(LoadError::Malformed(WavProblem::Truncated))
            )),
            Fault::ChangedLength => assert!(matches!(
                result,
                Err(LoadError::Malformed(WavProblem::ChangedLength))
            )),
            _ => assert!(matches!(result, Err(LoadError::Cancelled))),
        }
        assert!(state.dropped.get());
        assert!(state.max_read.get() <= CANCEL_CHECK_FRAMES * 2 * 4);
        if matches!(fault, Fault::CancelSecondData) {
            assert_eq!(state.data_reads.get(), 2);
        }
    }
    let state = State::default();
    let cancel = AtomicBool::new(true);
    assert!(matches!(
        decode_wav(
            Probe::new(vec![], &state, &cancel, Fault::None),
            target(1),
            &cancel
        ),
        Err(LoadError::Cancelled)
    ));
    assert_eq!(state.ends.get(), 0);
    assert!(state.dropped.get());
}

#[test]
fn partial_reads_interrupted_reads_and_invalid_target_are_handled() {
    let state = State::default();
    let cancel = AtomicBool::new(false);
    let sample = decode_wav(
        Probe::new(
            wav(1, 16, 1, &[0; 2], false),
            &state,
            &cancel,
            Fault::PartialInterrupted,
        ),
        target(1),
        &cancel,
    )
    .unwrap();
    assert_eq!(sample.samples(), [0.0]);
    assert!(state.dropped.get());
    let state = State::default();
    assert!(matches!(
        decode_wav(
            Probe::new(vec![], &state, &cancel, Fault::None),
            target(6),
            &cancel
        ),
        Err(LoadError::UnsupportedOutputChannels)
    ));
    assert_eq!(state.ends.get(), 0);
    let missing =
        std::env::temp_dir().join(format!("audiomotorr-no-such-wav-{}", std::process::id()));
    assert!(matches!(
        load_wav_file(&missing, target(1), &cancel),
        Err(LoadError::Io {
            operation: IoOperation::Open,
            ..
        })
    ));
    cancel.store(true, Ordering::Release);
    assert!(matches!(
        load_wav_file(&missing, target(1), &cancel),
        Err(LoadError::Cancelled)
    ));
}
