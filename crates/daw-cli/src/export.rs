use daw_app::offline::OfflineRender;
use daw_engine::EngineConfig;
use std::{
    collections::HashSet,
    error::Error,
    fs::OpenOptions,
    io::{self, BufWriter, Seek, Write},
    path::PathBuf,
};

// hound 3.5.1 writes a 68-byte WAVEFORMATEXTENSIBLE header for float32.
// RIFF size excludes its first 8 bytes. Data and RIFF sizes both use u32.
const RIFF_HEADER_SIZE: u64 = 60;

fn validate_wav_size(config: EngineConfig, frames: u64) -> io::Result<()> {
    let bytes_per_frame = u32::from(config.channels()) * 4;
    if config.sample_rate().checked_mul(bytes_per_frame).is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "WAV byte rate exceeds u32",
        ));
    }
    let max_frames = (u64::from(u32::MAX) - RIFF_HEADER_SIZE) / u64::from(bytes_per_frame);
    if frames > max_frames {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "WAV RIFF size limit exceeded; maximum {max_frames} frames for this configuration"
            ),
        ));
    }
    Ok(())
}

fn encode<W: Write + Seek>(render: OfflineRender, output: W) -> Result<(), Box<dyn Error>> {
    let config = render.config();
    validate_wav_size(config, render.frames())?;
    let mut wav = hound::WavWriter::new(
        output,
        hound::WavSpec {
            channels: config.channels(),
            sample_rate: config.sample_rate(),
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )?;
    render.render(|samples| {
        for &sample in samples {
            wav.write_sample(sample)?;
        }
        Ok::<_, hound::Error>(())
    })?;
    wav.finalize()?;
    Ok(())
}

pub fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    let (mut output, mut frames) = (None, None);
    let (mut sample_rate, mut channels, mut block_size) = (48_000u32, 2u16, 256usize);
    let (mut frequency, mut amplitude) = (440.0f64, 0.1f64);
    let mut seen = HashSet::new();
    let mut options = args.chunks_exact(2);
    for pair in &mut options {
        if !seen.insert(pair[0].as_str()) {
            return Err("Duplicate export option".into());
        }
        match pair[0].as_str() {
            "--output" => output = Some(PathBuf::from(&pair[1])),
            "--frames" => frames = Some(pair[1].parse::<u64>()?),
            "--sample-rate" => sample_rate = pair[1].parse()?,
            "--channels" => channels = pair[1].parse()?,
            "--block-size" => block_size = pair[1].parse()?,
            "--frequency" => frequency = pair[1].parse()?,
            "--amplitude" => amplitude = pair[1].parse()?,
            _ => return Err("Unknown export option".into()),
        }
    }
    if !options.remainder().is_empty() {
        return Err("Missing export option value".into());
    }
    let output = output
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or("export-wav requires --output PATH")?;
    let frames = frames.ok_or("export-wav requires --frames N")?;
    let config = EngineConfig::new(sample_rate, channels)?;
    let render = OfflineRender::new(config, frames, block_size, frequency, amplitude)?;
    // All parameters and container arithmetic are checked before touching the path.
    validate_wav_size(config, frames)?;
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .map_err(|error| {
            format!(
                "Cannot create {} (existing files are not overwritten): {error}",
                output.display()
            )
        })?;
    encode(render, BufWriter::new(file)).map_err(|error| {
        format!(
            "WAV export failed; partial output may remain at {}: {error}",
            output.display()
        )
    })?;
    println!(
        "export-wav OK: frames={frames}, sample_rate={sample_rate}, channels={channels}, format=f32, output={}",
        output.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::Cell,
        io::{Cursor, SeekFrom},
        rc::Rc,
    };

    #[derive(Clone, Copy)]
    enum Failure {
        Header,
        Samples,
        FinalSeek,
        Flush,
    }
    struct FaultWriter {
        bytes: Cursor<Vec<u8>>,
        failure: Failure,
        closed: Rc<Cell<bool>>,
    }
    impl Write for FaultWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if matches!(self.failure, Failure::Header)
                || (matches!(self.failure, Failure::Samples) && self.bytes.position() >= 68)
            {
                return Err(io::Error::other("injected write failure"));
            }
            self.bytes.write(bytes)
        }
        fn flush(&mut self) -> io::Result<()> {
            if matches!(self.failure, Failure::Flush) {
                Err(io::Error::other("injected flush failure"))
            } else {
                Ok(())
            }
        }
    }
    impl Seek for FaultWriter {
        fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
            if matches!(self.failure, Failure::FinalSeek) {
                return Err(io::Error::other("injected seek failure"));
            }
            self.bytes.seek(position)
        }
    }
    impl Drop for FaultWriter {
        fn drop(&mut self) {
            self.closed.set(true);
        }
    }

    #[test]
    fn initialization_sample_and_finalization_errors_are_returned_and_close_the_writer() {
        for failure in [
            Failure::Header,
            Failure::Samples,
            Failure::FinalSeek,
            Failure::Flush,
        ] {
            let closed = Rc::new(Cell::new(false));
            let writer = FaultWriter {
                bytes: Cursor::new(Vec::new()),
                failure,
                closed: closed.clone(),
            };
            let render =
                OfflineRender::new(EngineConfig::new(48_000, 2).unwrap(), 9, 4, 440.0, 0.1)
                    .unwrap();
            let error = encode(render, writer).unwrap_err();
            assert!(error.to_string().contains("injected"), "{error}");
            assert!(closed.get());
        }
    }

    #[test]
    fn riff_limit_and_byte_rate_are_checked_without_large_renders() {
        for channels in [1, 2] {
            let config = EngineConfig::new(48_000, channels).unwrap();
            let boundary = (u64::from(u32::MAX) - 60) / (u64::from(channels) * 4);
            assert!(validate_wav_size(config, boundary).is_ok());
            assert!(validate_wav_size(config, boundary + 1).is_err());
            assert!(validate_wav_size(config, u64::MAX).is_err());
        }
        assert!(validate_wav_size(EngineConfig::new(u32::MAX, 2).unwrap(), 0).is_err());
    }
}
