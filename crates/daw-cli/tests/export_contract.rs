use daw_engine::{AudioEngine, EngineCommand, EngineConfig, Tempo};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

#[test]
fn actual_mono_stereo_float_exports_load_with_exact_samples_and_owned_resources() {
    use daw_app::sample_loading::load_wav_file;
    use std::sync::atomic::AtomicBool;
    let directory = TestDirectory::new();
    let cancel = AtomicBool::new(false);
    for channels in [1u16, 2] {
        let path = directory.0.join(format!("load-{channels}.wav"));
        let channel_text = channels.to_string();
        let result = export(
            &path,
            &[
                "--frames",
                "4801",
                "--block-size",
                "127",
                "--channels",
                &channel_text,
                "--frequency",
                "443.25",
                "--amplitude",
                "0.125",
            ],
        );
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let expected = read_samples(&path, channels, 48_000, 4801);
        let resource =
            load_wav_file(&path, EngineConfig::new(48_000, 2).unwrap(), &cancel).unwrap();
        assert_eq!(resource.frames(), 4801);
        assert_eq!(resource.config().channels(), channels);
        fs::remove_file(&path).unwrap();
        assert_eq!(
            resource
                .samples()
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>(),
            expected.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
    }
    assert!(load_wav_file(&directory.0, EngineConfig::new(48_000, 2).unwrap(), &cancel).is_err());
}
struct TestDirectory(PathBuf);
impl TestDirectory {
    fn new() -> Self {
        loop {
            let path = std::env::temp_dir().join(format!(
                "audiomotorr-export-{}-{}",
                std::process::id(),
                NEXT_DIR.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("{error}"),
            }
        }
    }
}
impl Drop for TestDirectory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn export(path: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_daw-cli"))
        .args(["export-wav", "--output"])
        .arg(path)
        .args(args)
        .output()
        .unwrap()
}

fn read_samples(path: &Path, channels: u16, rate: u32, frames: u64) -> Vec<f32> {
    let mut reader = hound::WavReader::open(path).unwrap();
    assert_eq!(reader.spec().channels, channels);
    assert_eq!(reader.spec().sample_rate, rate);
    assert_eq!(reader.spec().bits_per_sample, 32);
    assert_eq!(reader.spec().sample_format, hound::SampleFormat::Float);
    assert_eq!(u64::from(reader.duration()), frames);
    let raw = fs::read(path).unwrap();
    assert_eq!(&raw[..4], b"RIFF");
    assert_eq!(&raw[8..12], b"WAVE");
    assert_eq!(
        u64::from(u32::from_le_bytes(raw[4..8].try_into().unwrap())) + 8,
        raw.len() as u64
    );
    assert_eq!(raw.len() as u64, 68 + frames * u64::from(channels) * 4);
    reader
        .samples::<f32>()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

#[test]
fn wav_roundtrips_exact_engine_samples_across_channels_partitions_and_empty_runs() {
    let directory = TestDirectory::new();
    for channels in [1, 2] {
        for frames in [0usize, 1, 4801] {
            let mut reference = AudioEngine::with_sine(
                EngineConfig::new(48_000, channels).unwrap(),
                Tempo::default(),
                440.0,
                0.1,
            )
            .unwrap();
            reference.apply_command(EngineCommand::Play);
            let mut expected = vec![0.0; frames * usize::from(channels)];
            reference.render_interleaved(&mut expected).unwrap();
            for block in [1, 127, 256, 4096] {
                let path = directory
                    .0
                    .join(format!("signal {channels}-{frames}-{block}.wav"));
                let result = export(
                    &path,
                    &[
                        "--frames",
                        &frames.to_string(),
                        "--block-size",
                        &block.to_string(),
                        "--channels",
                        &channels.to_string(),
                    ],
                );
                assert!(
                    result.status.success(),
                    "{}",
                    String::from_utf8_lossy(&result.stderr)
                );
                assert!(String::from_utf8_lossy(&result.stdout).contains("export-wav OK"));
                assert_eq!(
                    read_samples(&path, channels, 48_000, frames as u64),
                    expected
                );
            }
        }
    }
}

#[test]
fn custom_signal_and_default_stereo_configuration_are_supported() {
    let directory = TestDirectory::new();
    let path = directory.0.join("custom.wav");
    let result = export(
        &path,
        &[
            "--frames",
            "1007",
            "--sample-rate",
            "44100",
            "--channels",
            "1",
            "--frequency",
            "997.25",
            "--amplitude",
            "0.25",
        ],
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut reference = AudioEngine::with_sine(
        EngineConfig::new(44_100, 1).unwrap(),
        Tempo::default(),
        997.25,
        0.25,
    )
    .unwrap();
    reference.apply_command(EngineCommand::Play);
    let mut expected = vec![0.0; 1007];
    reference.render_interleaved(&mut expected).unwrap();
    assert_eq!(read_samples(&path, 1, 44_100, 1007), expected);
    let path = directory.0.join("default.wav");
    assert!(export(&path, &["--frames", "127"]).status.success());
    assert!(
        read_samples(&path, 2, 48_000, 127)
            .iter()
            .any(|sample| sample.abs() > 0.05)
    );
    let path = directory.0.join("zero-amplitude.wav");
    assert!(
        export(&path, &["--frames", "127", "--amplitude", "0"])
            .status
            .success()
    );
    assert!(
        read_samples(&path, 2, 48_000, 127)
            .iter()
            .all(|sample| *sample == 0.0)
    );
}

#[test]
fn existing_destinations_and_invalid_paths_fail_without_overwriting() {
    let directory = TestDirectory::new();
    let existing = directory.0.join("existing.wav");
    fs::write(&existing, b"preserve this content").unwrap();
    assert!(!export(&existing, &["--frames", "4801"]).status.success());
    assert_eq!(fs::read(existing).unwrap(), b"preserve this content");
    assert!(!export(&directory.0, &["--frames", "0"]).status.success());
    let missing = directory.0.join("missing-parent").join("signal.wav");
    assert!(!export(&missing, &["--frames", "1"]).status.success());
    assert!(!missing.exists());
}

#[test]
fn invalid_parameters_and_unrepresentable_sizes_fail_before_creating_a_file() {
    let directory = TestDirectory::new();
    for (index, args) in [
        vec![],
        vec!["--frames"],
        vec!["--frames", "-1"],
        vec!["--frames", "1", "--seconds", "1"],
        vec!["--frames", "1", "--frames", "2"],
        vec!["--frames", "1", "--block-size", "0"],
        vec!["--frames", "1", "--block-size", "4097"],
        vec!["--frames", "1", "--channels", "0"],
        vec!["--frames", "1", "--channels", "3"],
        vec!["--frames", "1", "--sample-rate", "0"],
        vec!["--frames", "1", "--frequency", "NaN"],
        vec!["--frames", "1", "--frequency", "24000"],
        vec!["--frames", "1", "--amplitude", "inf"],
        vec!["--frames", "1", "--amplitude", "-0.1"],
        vec!["--frames", "1", "--amplitude", "1.1"],
        vec!["--frames", "18446744073709551615"],
        vec!["--frames", "536870905"],
        vec!["--frames", "0", "--sample-rate", "4294967295"],
    ]
    .iter()
    .enumerate()
    {
        let path = directory.0.join(format!("invalid-{index}.wav"));
        let result = export(&path, args);
        assert!(!result.status.success(), "args accepted: {args:?}");
        assert!(!String::from_utf8_lossy(&result.stdout).contains("export-wav OK"));
        assert!(!path.exists(), "invalid request created {path:?}");
    }
    assert!(
        !Command::new(env!("CARGO_BIN_EXE_daw-cli"))
            .args(["export-wav", "--frames", "1"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(!export(Path::new(""), &["--frames", "1"]).status.success());
}
