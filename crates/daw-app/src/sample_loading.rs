//! Bounded, synchronous WAV input adapter. Call outside audio processing.
//! Bank preparation and its permit stay outside the audio service and IPC.
mod bank;
pub use bank::{
    BankPreparationCoordinator, BankPreparationError, MAX_ACTIVE_AND_PREPARING_DATA_BYTES,
    MAX_BANK_PCM_BYTES, MAX_PREPARATION_DATA_BYTES, MAX_PREPARATION_PCM_EXTRA_BYTES,
    PreparationUsage, PreparedBank, SampleSelection,
};
use daw_engine::{EngineConfig, MAX_SAMPLE_FRAMES, SampleBuffer, SampleError};
use std::{
    fmt,
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

pub const MAX_ENCODED_FILE_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_RIFF_CHUNKS: usize = 4096;
pub const MAX_DECODER_BUFFER_BYTES: usize = 64 * 1024;
pub const MAX_RESOURCE_PCM_BYTES: usize = MAX_SAMPLE_FRAMES * 2 * 4;
pub const CANCEL_CHECK_FRAMES: usize = 256;
const SCRATCH_BYTES: usize = CANCEL_CHECK_FRAMES * 2 * 4;
const _: () = assert!(SCRATCH_BYTES + 40 + 12 + 8 + 4 <= MAX_DECODER_BUFFER_BYTES);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoOperation {
    Open,
    Metadata,
    Read,
    Seek,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadLimit {
    EncodedBytes,
    Chunks,
    Frames,
    Arithmetic,
    PcmCapacity,
    BankPcm,
    PreparationData,
    ActiveAndPreparingData,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WavProblem {
    Header,
    RiffLength,
    ChunkBoundary,
    DuplicateFormat,
    DuplicateData,
    DataBeforeFormat,
    MissingFormatOrData,
    FormatSize,
    BlockAlignment,
    ByteRate,
    SampleRate,
    EmptyData,
    PartialFrame,
    Truncated,
    ChangedLength,
}
#[derive(Debug)]
pub enum LoadError {
    Cancelled,
    UnsupportedOutputChannels,
    NotRegularFile,
    Io {
        operation: IoOperation,
        error: io::Error,
    },
    Malformed(WavProblem),
    UnsupportedContainer,
    UnsupportedEncoding,
    UnsupportedLayout,
    UnsupportedChannels,
    UnsupportedChannelAssignment,
    SampleRateMismatch,
    StereoToMono,
    LimitExceeded(LoadLimit),
    InvalidPcm {
        sample_index: usize,
    },
    AllocationFailed,
    Resource(SampleError),
}
impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { operation, error } => write!(f, "WAV {operation:?} failed: {error}"),
            Self::Malformed(problem) => write!(f, "malformed WAV: {problem:?}"),
            Self::LimitExceeded(limit) => write!(f, "WAV limit exceeded: {limit:?}"),
            Self::InvalidPcm { sample_index } => write!(f, "invalid PCM sample {sample_index}"),
            Self::Resource(error) => error.fmt(f),
            other => write!(f, "WAV load failed: {other:?}"),
        }
    }
}
impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { error, .. } => Some(error),
            Self::Resource(error) => Some(error),
            _ => None,
        }
    }
}
fn cancelled(cancel: &AtomicBool) -> Result<(), LoadError> {
    if cancel.load(Ordering::Acquire) {
        Err(LoadError::Cancelled)
    } else {
        Ok(())
    }
}
fn io_error(operation: IoOperation, error: io::Error) -> LoadError {
    LoadError::Io { operation, error }
}
fn malformed(problem: WavProblem) -> LoadError {
    LoadError::Malformed(problem)
}
fn checked_add(a: u64, b: u64) -> Result<u64, LoadError> {
    a.checked_add(b)
        .ok_or(LoadError::LimitExceeded(LoadLimit::Arithmetic))
}

struct Input<'a, R> {
    reader: R,
    length: u64,
    position: u64,
    cancel: &'a AtomicBool,
}
impl<'a, R: Read + Seek> Input<'a, R> {
    fn new(mut reader: R, cancel: &'a AtomicBool) -> Result<Self, LoadError> {
        cancelled(cancel)?;
        let length = reader.seek(SeekFrom::End(0));
        cancelled(cancel)?;
        let length = length.map_err(|e| io_error(IoOperation::Seek, e))?;
        if length > MAX_ENCODED_FILE_BYTES {
            return Err(LoadError::LimitExceeded(LoadLimit::EncodedBytes));
        }
        if length < 12 {
            return Err(malformed(WavProblem::Header));
        }
        let mut input = Self {
            reader,
            length,
            position: length,
            cancel,
        };
        input.seek_to(0)?;
        Ok(input)
    }
    fn seek_to(&mut self, position: u64) -> Result<(), LoadError> {
        cancelled(self.cancel)?;
        if position > self.length {
            return Err(malformed(WavProblem::ChunkBoundary));
        }
        let result = self.reader.seek(SeekFrom::Start(position));
        cancelled(self.cancel)?;
        let actual = result.map_err(|e| io_error(IoOperation::Seek, e))?;
        if actual != position {
            return Err(io_error(
                IoOperation::Seek,
                io::ErrorKind::InvalidData.into(),
            ));
        }
        self.position = position;
        Ok(())
    }
    fn read_exact(&mut self, mut output: &mut [u8]) -> Result<(), LoadError> {
        if checked_add(self.position, output.len() as u64)? > self.length {
            return Err(malformed(WavProblem::ChunkBoundary));
        }
        while !output.is_empty() {
            cancelled(self.cancel)?;
            let result = self.reader.read(output);
            cancelled(self.cancel)?;
            match result {
                Ok(0) => return Err(malformed(WavProblem::Truncated)),
                Ok(n) if n <= output.len() => {
                    self.position += n as u64;
                    output = &mut output[n..];
                }
                Ok(_) => {
                    return Err(io_error(
                        IoOperation::Read,
                        io::ErrorKind::InvalidData.into(),
                    ));
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    return Err(malformed(WavProblem::Truncated));
                }
                Err(e) => return Err(io_error(IoOperation::Read, e)),
            }
        }
        Ok(())
    }
    fn unchanged_length(&mut self) -> Result<(), LoadError> {
        cancelled(self.cancel)?;
        let result = self.reader.seek(SeekFrom::End(0));
        cancelled(self.cancel)?;
        if result.map_err(|e| io_error(IoOperation::Seek, e))? != self.length {
            return Err(malformed(WavProblem::ChangedLength));
        }
        self.position = self.length;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Encoding {
    Pcm16,
    Float32,
}
#[derive(Clone, Copy)]
struct Format {
    config: EngineConfig,
    encoding: Encoding,
    frame_bytes: u16,
}
struct Metadata {
    format: Format,
    data_offset: u64,
    frames: usize,
    samples: usize,
}
fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}
fn parse_format(bytes: &[u8], target: EngineConfig) -> Result<Format, LoadError> {
    if ![16, 18, 40].contains(&bytes.len()) {
        return Err(malformed(WavProblem::FormatSize));
    }
    let tag = u16_at(bytes, 0);
    let channels = u16_at(bytes, 2);
    if !(1..=2).contains(&channels) {
        return Err(LoadError::UnsupportedChannels);
    }
    let rate = u32_at(bytes, 4);
    if rate == 0 {
        return Err(malformed(WavProblem::SampleRate));
    }
    if rate != target.sample_rate() {
        return Err(LoadError::SampleRateMismatch);
    }
    if channels == 2 && target.channels() == 1 {
        return Err(LoadError::StereoToMono);
    }
    let bits = u16_at(bytes, 14);
    let kind = match tag {
        1 | 3 => {
            if bytes.len() == 40 || (bytes.len() == 18 && u16_at(bytes, 16) != 0) {
                return Err(malformed(WavProblem::FormatSize));
            }
            u32::from(tag)
        }
        0xfffe => {
            if bytes.len() != 40 || u16_at(bytes, 16) != 22 {
                return Err(malformed(WavProblem::FormatSize));
            }
            let valid = u16_at(bytes, 18);
            if valid != 0 && valid != bits {
                return Err(LoadError::UnsupportedEncoding);
            }
            let mask = u32_at(bytes, 20);
            if !(if channels == 1 {
                [0, 1, 4].contains(&mask)
            } else {
                [0, 3].contains(&mask)
            }) {
                return Err(LoadError::UnsupportedChannelAssignment);
            }
            if bytes[28..40] != [0, 0, 0x10, 0, 0x80, 0, 0, 0xaa, 0, 0x38, 0x9b, 0x71] {
                return Err(LoadError::UnsupportedEncoding);
            }
            u32_at(bytes, 24)
        }
        _ => return Err(LoadError::UnsupportedEncoding),
    };
    let encoding = match (kind, bits) {
        (1, 16) => Encoding::Pcm16,
        (3, 32) => Encoding::Float32,
        _ => return Err(LoadError::UnsupportedEncoding),
    };
    let frame_bytes = channels * (bits / 8);
    if u16_at(bytes, 12) != frame_bytes {
        return Err(malformed(WavProblem::BlockAlignment));
    }
    let byte_rate = rate
        .checked_mul(u32::from(frame_bytes))
        .ok_or(LoadError::LimitExceeded(LoadLimit::Arithmetic))?;
    if u32_at(bytes, 8) != byte_rate {
        return Err(malformed(WavProblem::ByteRate));
    }
    let config =
        EngineConfig::new(rate, channels).map_err(|_| malformed(WavProblem::SampleRate))?;
    Ok(Format {
        config,
        encoding,
        frame_bytes,
    })
}
fn preflight<R: Read + Seek>(
    input: &mut Input<'_, R>,
    target: EngineConfig,
) -> Result<Metadata, LoadError> {
    let mut header = [0; 12];
    input.read_exact(&mut header)?;
    if &header[..4] != b"RIFF" {
        return Err(LoadError::UnsupportedContainer);
    }
    if &header[8..12] != b"WAVE" {
        return Err(malformed(WavProblem::Header));
    }
    if checked_add(u64::from(u32_at(&header, 4)), 8)? != input.length {
        return Err(malformed(WavProblem::RiffLength));
    }
    let mut format = None;
    let mut data = None;
    let mut count = 0;
    let mut fmt_bytes = [0; 40];
    while input.position < input.length {
        cancelled(input.cancel)?;
        count += 1;
        if count > MAX_RIFF_CHUNKS {
            return Err(LoadError::LimitExceeded(LoadLimit::Chunks));
        }
        let mut chunk = [0; 8];
        input.read_exact(&mut chunk)?;
        let length = u64::from(u32_at(&chunk, 4));
        let body = input.position;
        let end = checked_add(body, length)?;
        let padded_end = checked_add(end, length % 2)?;
        if padded_end > input.length {
            return Err(malformed(WavProblem::ChunkBoundary));
        }
        match &chunk[..4] {
            b"fmt " => {
                if format.is_some() {
                    return Err(malformed(WavProblem::DuplicateFormat));
                }
                if ![16, 18, 40].contains(&length) {
                    return Err(malformed(WavProblem::FormatSize));
                }
                input.read_exact(&mut fmt_bytes[..length as usize])?;
                format = Some(parse_format(&fmt_bytes[..length as usize], target)?);
            }
            b"data" => {
                if format.is_none() {
                    return Err(malformed(WavProblem::DataBeforeFormat));
                }
                if data.is_some() {
                    return Err(malformed(WavProblem::DuplicateData));
                }
                data = Some((body, length));
            }
            b"plst" => return Err(LoadError::UnsupportedLayout),
            b"LIST" if length >= 4 => {
                let mut subtype = [0; 4];
                input.read_exact(&mut subtype)?;
                if &subtype == b"wavl" {
                    return Err(LoadError::UnsupportedLayout);
                }
            }
            _ => (),
        }
        input.seek_to(padded_end)?;
    }
    let (format, (data_offset, data_bytes)) = format
        .zip(data)
        .ok_or(malformed(WavProblem::MissingFormatOrData))?;
    if data_bytes == 0 {
        return Err(malformed(WavProblem::EmptyData));
    }
    if data_bytes % u64::from(format.frame_bytes) != 0 {
        return Err(malformed(WavProblem::PartialFrame));
    }
    let frames = usize::try_from(data_bytes / u64::from(format.frame_bytes))
        .map_err(|_| LoadError::LimitExceeded(LoadLimit::Arithmetic))?;
    if frames > MAX_SAMPLE_FRAMES {
        return Err(LoadError::LimitExceeded(LoadLimit::Frames));
    }
    let samples = frames
        .checked_mul(usize::from(format.config.channels()))
        .ok_or(LoadError::LimitExceeded(LoadLimit::Arithmetic))?;
    if samples
        .checked_mul(4)
        .ok_or(LoadError::LimitExceeded(LoadLimit::Arithmetic))?
        > MAX_RESOURCE_PCM_BYTES
    {
        return Err(LoadError::LimitExceeded(LoadLimit::PcmCapacity));
    }
    Ok(Metadata {
        format,
        data_offset,
        frames,
        samples,
    })
}
fn reserve_pcm(samples: usize) -> Result<Vec<f32>, LoadError> {
    let mut pcm = Vec::new();
    pcm.try_reserve_exact(samples)
        .map_err(|_| LoadError::AllocationFailed)?;
    if pcm.capacity() != samples {
        return Err(LoadError::LimitExceeded(LoadLimit::PcmCapacity));
    }
    Ok(pcm)
}
fn decode_with_reservation<R: Read + Seek>(
    reader: R,
    target: EngineConfig,
    cancel: &AtomicBool,
    reserve: impl FnOnce(usize) -> Result<Vec<f32>, LoadError>,
) -> Result<SampleBuffer, LoadError> {
    cancelled(cancel)?;
    if !(1..=2).contains(&target.channels()) {
        return Err(LoadError::UnsupportedOutputChannels);
    }
    let mut input = Input::new(reader, cancel)?;
    let metadata = preflight(&mut input, target)?;
    input.unchanged_length()?;
    input.seek_to(metadata.data_offset)?;
    cancelled(cancel)?;
    let mut pcm = reserve(metadata.samples)?;
    let mut scratch = [0; SCRATCH_BYTES];
    let mut frames_left = metadata.frames;
    while frames_left != 0 {
        cancelled(cancel)?;
        let frames = frames_left.min(CANCEL_CHECK_FRAMES);
        let byte_count = frames * usize::from(metadata.format.frame_bytes);
        input.read_exact(&mut scratch[..byte_count])?;
        let width = match metadata.format.encoding {
            Encoding::Pcm16 => 2,
            Encoding::Float32 => 4,
        };
        for bytes in scratch[..byte_count].chunks_exact(width) {
            let value = match metadata.format.encoding {
                Encoding::Pcm16 => f32::from(i16::from_le_bytes([bytes[0], bytes[1]])) / 32768.0,
                Encoding::Float32 => f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            };
            if !value.is_finite() || !(-1.0..=1.0).contains(&value) {
                return Err(LoadError::InvalidPcm {
                    sample_index: pcm.len(),
                });
            }
            pcm.push(value);
        }
        frames_left -= frames;
    }
    input.unchanged_length()?;
    cancelled(cancel)?;
    let resource = SampleBuffer::new(metadata.format.config, pcm).map_err(LoadError::Resource)?;
    cancelled(cancel)?;
    Ok(resource)
}

/// Owns/drops its seekable reader on every path. Returns one immutable resource,
/// not a bank/session. Does not create Application or open audio.
pub fn decode_wav<R: Read + Seek>(
    reader: R,
    target: EngineConfig,
    cancel: &AtomicBool,
) -> Result<SampleBuffer, LoadError> {
    decode_with_reservation(reader, target, cancel, reserve_pcm)
}

/// Opens a regular file and delegates to the bounded reader. Cancellation does
/// not interrupt a blocked OS call; it is checked before and after calls return.
pub fn load_wav_file(
    path: &Path,
    target: EngineConfig,
    cancel: &AtomicBool,
) -> Result<SampleBuffer, LoadError> {
    load_wav_file_with_reservation(path, target, cancel, reserve_pcm)
}

fn load_wav_file_with_reservation(
    path: &Path,
    target: EngineConfig,
    cancel: &AtomicBool,
    reserve: impl FnOnce(usize) -> Result<Vec<f32>, LoadError>,
) -> Result<SampleBuffer, LoadError> {
    cancelled(cancel)?;
    if !(1..=2).contains(&target.channels()) {
        return Err(LoadError::UnsupportedOutputChannels);
    }
    let file = File::open(path);
    cancelled(cancel)?;
    let file = file.map_err(|e| io_error(IoOperation::Open, e))?;
    let metadata = file.metadata();
    cancelled(cancel)?;
    if !metadata
        .map_err(|e| io_error(IoOperation::Metadata, e))?
        .is_file()
    {
        return Err(LoadError::NotRegularFile);
    }
    decode_with_reservation(file, target, cancel, reserve)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn reservation_failure_is_typed_and_reader_is_dropped() {
        struct Tracked<'a> {
            cursor: Cursor<Vec<u8>>,
            dropped: &'a AtomicBool,
        }
        impl Read for Tracked<'_> {
            fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
                self.cursor.read(b)
            }
        }
        impl Seek for Tracked<'_> {
            fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
                self.cursor.seek(p)
            }
        }
        impl Drop for Tracked<'_> {
            fn drop(&mut self) {
                self.dropped.store(true, Ordering::Relaxed);
            }
        }
        let mut wav = b"RIFF\x26\0\0\0WAVEfmt \x10\0\0\0\x01\0\x01\0\x80\xbb\0\0\0\x77\x01\0\x02\0\x10\0data\x02\0\0\0".to_vec();
        wav.extend([0, 0]);
        let dropped = AtomicBool::new(false);
        let result = decode_with_reservation(
            Tracked {
                cursor: Cursor::new(wav),
                dropped: &dropped,
            },
            EngineConfig::new(48_000, 1).unwrap(),
            &AtomicBool::new(false),
            |_| Err(LoadError::AllocationFailed),
        );
        assert!(matches!(result, Err(LoadError::AllocationFailed)));
        assert!(dropped.load(Ordering::Relaxed));
    }
}
