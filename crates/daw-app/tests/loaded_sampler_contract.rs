//! End-to-end offline oracles: filesystem WAV -> application bank -> raw kernel.
#[path = "support/sample_files.rs"]
mod sample_files;
#[path = "support/wav_fixtures.rs"]
mod wav_fixtures;

use daw_app::sample_loading::{
    BankPreparationCoordinator, BankPreparationError, LoadError, LoadLimit, MAX_ENCODED_FILE_BYTES,
    MAX_RIFF_CHUNKS, SampleSelection,
};
use daw_engine::{
    AudioEngine, EngineCommand, EngineConfig, Gain, MAX_SAMPLE_FRAMES, SAMPLE_BANK_SLOTS, Sampler,
    Tempo, TransportState, TriggerError,
};
use sample_files::SampleFiles;
use std::{
    fs::{self, File},
    io::{Seek, SeekFrom, Write},
    path::Path,
    sync::atomic::AtomicBool,
};
use wav_fixtures::{chunk, format, riff, wav};

#[test]
fn prepared_wav_bank_drives_audioengine_transport_with_headroom_and_retained_pcm() {
    let files = SampleFiles::new();
    let path = files.write(
        "engine.wav",
        &encoded_wave(1, Header::Extensible40, 1, 800, &[8192; 32]),
    );
    let coordinator = BankPreparationCoordinator::default();
    let mut bank = None;
    coordinator
        .prepare(
            config(800, 2),
            &[select(0, &path)],
            None,
            &AtomicBool::new(false),
        )
        .unwrap()
        .install(&mut bank);
    fs::remove_file(path).unwrap();
    let mut engine = AudioEngine::with_sampler(bank.unwrap(), Tempo::default());
    let pointer = engine
        .sampler()
        .unwrap()
        .bank()
        .get(0)
        .unwrap()
        .samples()
        .as_ptr();
    engine.apply_command(EngineCommand::Play);
    engine.trigger_sample(0, Gain::new(1.0).unwrap()).unwrap();
    let mut attack = [f32::NAN; 8];
    engine.render_interleaved(&mut attack).unwrap();
    assert_eq!(
        attack,
        [
            0.00390625, 0.00390625, 0.0078125, 0.0078125, 0.01171875, 0.01171875, 0.015625,
            0.015625
        ]
    );
    engine.apply_command(EngineCommand::Pause);
    let mut release = [f32::NAN; 10];
    engine.render_interleaved(&mut release).unwrap();
    assert_eq!(
        release,
        [
            0.01171875, 0.01171875, 0.0078125, 0.0078125, 0.00390625, 0.00390625, 0.0, 0.0, 0.0,
            0.0
        ]
    );
    assert_eq!(engine.snapshot().transport, TransportState::Paused);
    engine.apply_command(EngineCommand::Stop);
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 0);
    assert_eq!(
        engine
            .sampler()
            .unwrap()
            .bank()
            .get(0)
            .unwrap()
            .samples()
            .as_ptr(),
        pointer
    );
}

#[derive(Debug, Clone, Copy)]
enum Header {
    Classic16,
    Classic18,
    Extensible40,
}
const HEADERS: [Header; 3] = [Header::Classic16, Header::Classic18, Header::Extensible40];
const LONG_FRAMES: usize = 4103;
const TOTAL_FRAMES: usize = 8205;
// at, slot, gain in quarter units. The driver splits at these explicit triggers;
// this does not introduce within-block scheduling into the kernel.
const EVENTS: [(usize, usize, i64); 6] = [
    (0, 3, 2),
    (127, 3, 1),
    (256, 3, 4),
    (4096, 3, 2),
    (4096, 15, 4),
    (4103, 15, 1),
];
fn config(rate: u32, channels: u16) -> EngineConfig {
    EngineConfig::new(rate, channels).unwrap()
}
fn select(slot: usize, path: &Path) -> SampleSelection<'_> {
    SampleSelection { slot, path }
}
fn encoded_wave(tag: u16, header: Header, channels: u16, rate: u32, units: &[i16]) -> Vec<u8> {
    let bits = if tag == 1 { 16 } else { 32 };
    let mut fmt = format(
        tag,
        bits,
        channels,
        rate,
        matches!(header, Header::Extensible40),
    );
    if matches!(header, Header::Classic18) {
        fmt.extend([0, 0]);
    }
    let data: Vec<_> = if tag == 1 {
        units.iter().flat_map(|value| value.to_le_bytes()).collect()
    } else {
        units
            .iter()
            .flat_map(|value| (f32::from(*value) / 32768.0).to_le_bytes())
            .collect()
    };
    riff(&[
        chunk(b"JUNK", &[1, 2, 3]),
        chunk(b"fmt ", &fmt),
        chunk(b"LIST", b"INFO"),
        chunk(b"data", &data),
        chunk(b"note", &[4]),
    ])
}
fn load_player(paths: &[SampleSelection<'_>], output: EngineConfig) -> Sampler {
    let coordinator = BankPreparationCoordinator::default();
    let mut bank = None;
    coordinator
        .prepare(output, paths, None, &AtomicBool::new(false))
        .unwrap()
        .install(&mut bank);
    Sampler::new(bank.unwrap())
}
fn units(slot: usize, frame: usize, channel: usize) -> i16 {
    if slot == 3 {
        (((frame * 5 + channel * 7) % 17) as i16 - 8) * 1024
    } else {
        const LEFT: [i16; 7] = [8192, 0, -16384, 4096, 0, 16384, -8192];
        const RIGHT: [i16; 7] = [-4096, 8192, 0, -8192, 16384, 0, 4096];
        if channel == 0 {
            LEFT[frame]
        } else {
            RIGHT[frame]
        }
    }
}
fn oracle(source_channels: u16, output_channels: u16) -> Vec<f32> {
    // Integer accumulation avoids deriving the oracle from a second Sampler or
    // from the decoder's output, and all results are exact binary fractions.
    (0..TOTAL_FRAMES)
        .flat_map(|frame| {
            (0..usize::from(output_channels)).map(move |channel| {
                let mut sum = 0i64;
                for (at, slot, gain) in EVENTS {
                    let length = if slot == 3 { LONG_FRAMES } else { 7 };
                    if frame >= at && frame - at < length {
                        let source_channel = if source_channels == 1 { 0 } else { channel };
                        sum += i64::from(units(slot, frame - at, source_channel)) * gain;
                    }
                }
                sum as f32 / 131072.0
            })
        })
        .collect()
}
fn run_timeline(sampler: &mut Sampler, partition: usize) -> Vec<f32> {
    let channels = usize::from(sampler.bank().config().channels());
    let mut output = Vec::with_capacity(TOTAL_FRAMES * channels);
    let mut scratch = vec![f32::NAN; partition * channels];
    let mut frame = 0;
    let mut next_event = 0;
    while frame < TOTAL_FRAMES {
        while next_event < EVENTS.len() && EVENTS[next_event].0 == frame {
            let (_, slot, quarters) = EVENTS[next_event];
            sampler
                .trigger(slot, Gain::new(quarters as f64 / 4.0).unwrap())
                .unwrap();
            next_event += 1;
        }
        let next_at = EVENTS.get(next_event).map_or(TOTAL_FRAMES, |event| event.0);
        let count = partition.min(next_at - frame).min(TOTAL_FRAMES - frame);
        assert!(count > 0);
        sampler
            .render_interleaved(&mut scratch[..count * channels])
            .unwrap();
        output.extend_from_slice(&scratch[..count * channels]);
        frame += count;
    }
    assert_eq!(sampler.snapshot().rendered_frames, TOTAL_FRAMES as u64);
    assert_eq!(sampler.snapshot().active_voices(), 0);
    output
}

#[test]
fn wav_formats_and_channel_mappings_match_integer_timeline_oracle_across_partitions() {
    for tag in [1, 3] {
        for header in HEADERS {
            for (source_channels, output_channels) in [(1, 1), (1, 2), (2, 2)] {
                let files = SampleFiles::new();
                let source = |slot, frames| {
                    (0..frames)
                        .flat_map(|frame| {
                            (0..usize::from(source_channels))
                                .map(move |channel| units(slot, frame, channel))
                        })
                        .collect::<Vec<_>>()
                };
                let long = source(3, LONG_FRAMES);
                let short = source(15, 7);
                let long_path = files.write(
                    "long.wav",
                    &encoded_wave(tag, header, source_channels, 48_000, &long),
                );
                let short_path = files.write(
                    "short.wav",
                    &encoded_wave(tag, header, source_channels, 48_000, &short),
                );
                let paths = [select(15, &short_path), select(3, &long_path)];
                let output = config(48_000, output_channels);
                let mut players: Vec<_> = (0..4).map(|_| load_player(&paths, output)).collect();
                fs::remove_file(long_path).unwrap();
                fs::remove_file(short_path).unwrap();
                let expected = oracle(source_channels, output_channels);
                for (sampler, partition) in players.iter_mut().zip([1, 127, 256, 4096]) {
                    let resource = sampler.bank().get(3).unwrap();
                    assert_eq!(resource.config(), config(48_000, source_channels));
                    assert_eq!(resource.frames(), LONG_FRAMES);
                    for (actual, expected) in resource.samples().iter().zip(&long) {
                        assert_eq!(actual.to_bits(), (f32::from(*expected) / 32768.0).to_bits());
                    }
                    let actual = run_timeline(sampler, partition);
                    for (index, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
                        assert_eq!(
                            actual.to_bits(),
                            expected.to_bits(),
                            "tag={tag}, header={header:?}, source={source_channels}, output={output_channels}, partition={partition}, sample={index}"
                        );
                    }
                    assert_eq!(actual.len(), expected.len());
                }
            }
        }
    }
}

#[test]
fn empty_and_full_mixed_format_banks_render_exactly_and_keep_raw_summing_policy() {
    for output_channels in [1u16, 2] {
        let output = config(48_000, output_channels);
        let mut empty = load_player(&[], output);
        assert_eq!(
            empty.trigger(0, Gain::new(1.0).unwrap()),
            Err(TriggerError::MissingAsset)
        );
        let mut silence = vec![f32::NAN; 127 * usize::from(output_channels)];
        empty.render_interleaved(&mut silence).unwrap();
        assert!(silence.iter().all(|value| value.to_bits() == 0));
        let files = SampleFiles::new();
        let paths: Vec<_> = (0..SAMPLE_BANK_SLOTS)
            .map(|slot| {
                let channels = if output_channels == 2 && slot % 2 == 1 {
                    2
                } else {
                    1
                };
                let left = (slot as i16 + 1) * 1024;
                let samples = if channels == 1 {
                    vec![left]
                } else {
                    vec![left, -left / 2]
                };
                files.write(
                    &format!("slot-{slot}.wav"),
                    &encoded_wave(
                        if slot % 2 == 0 { 1 } else { 3 },
                        HEADERS[slot % 3],
                        channels,
                        48_000,
                        &samples,
                    ),
                )
            })
            .collect();
        let selections: Vec<_> = paths
            .iter()
            .enumerate()
            .rev()
            .map(|(slot, path)| select(slot, path))
            .collect();
        let mut sampler = load_player(&selections, output);
        for path in paths {
            fs::remove_file(path).unwrap();
        }
        for slot in 0..SAMPLE_BANK_SLOTS {
            assert_eq!(
                sampler.trigger(slot, Gain::new((slot % 4 + 1) as f64 / 4.0).unwrap()),
                Ok(slot)
            );
        }
        let before = sampler.snapshot();
        assert_eq!(
            sampler.trigger(0, Gain::new(1.0).unwrap()),
            Err(TriggerError::PoolFull)
        );
        assert_eq!(sampler.snapshot(), before);
        let mut rendered = vec![f32::NAN; 2 * usize::from(output_channels)];
        sampler.render_interleaved(&mut rendered).unwrap();
        let expected = if output_channels == 1 {
            vec![2.8125, 0.0]
        } else {
            vec![2.8125, 0.1875, 0.0, 0.0]
        };
        assert_eq!(rendered, expected);
        assert_eq!(sampler.snapshot().active_voices(), 0);
        assert!(sampler.bank().get(15).is_some());
    }
}

#[test]
fn every_pcm16_code_loads_and_plays_at_matching_44100_rate_without_conversion() {
    let files = SampleFiles::new();
    let codes: Vec<_> = (i16::MIN..=i16::MAX).collect();
    for header in HEADERS {
        let path = files.write("pcm16-all.wav", &encoded_wave(1, header, 1, 44_100, &codes));
        let mut sampler = load_player(&[select(0, &path)], config(44_100, 2));
        fs::remove_file(path).unwrap();
        sampler.trigger(0, Gain::new(1.0).unwrap()).unwrap();
        let mut frame = 0;
        let mut scratch = [f32::NAN; 254];
        while frame < codes.len() + 3 {
            let count = 127.min(codes.len() + 3 - frame);
            sampler
                .render_interleaved(&mut scratch[..count * 2])
                .unwrap();
            for (offset, stereo) in scratch[..count * 2].chunks_exact(2).enumerate() {
                let index = frame + offset;
                let expected = if index < codes.len() {
                    ((index as f64 - 32768.0) / 32768.0) as f32
                } else {
                    0.0
                };
                assert_eq!(stereo[0].to_bits(), expected.to_bits());
                assert_eq!(stereo[1].to_bits(), expected.to_bits());
            }
            frame += count;
        }
        assert_eq!(sampler.bank().get(0).unwrap().frames(), 65536);
        assert_eq!(sampler.snapshot().active_voices(), 0);
    }
}

#[test]
fn real_file_exact_limits_play_and_one_over_limits_preserve_existing_playable_bank() {
    let files = SampleFiles::new();
    let output = config(48_000, 2);
    let mut beacons = vec![0; MAX_SAMPLE_FRAMES * 2];
    beacons[0] = 4096;
    beacons[MAX_SAMPLE_FRAMES * 2 - 2] = 16384;
    beacons[MAX_SAMPLE_FRAMES * 2 - 1] = -8192;
    let max_frames = files.write(
        "max-frames.wav",
        &encoded_wave(3, Header::Extensible40, 2, 48_000, &beacons),
    );
    let mut largest = load_player(&[select(0, &max_frames)], output);
    fs::remove_file(max_frames).unwrap();
    largest.trigger(0, Gain::new(1.0).unwrap()).unwrap();
    let mut scratch = [f32::NAN; 8192];
    let mut frame = 0;
    while frame < MAX_SAMPLE_FRAMES + 1 {
        let count = 4096.min(MAX_SAMPLE_FRAMES + 1 - frame);
        largest
            .render_interleaved(&mut scratch[..count * 2])
            .unwrap();
        for (offset, pair) in scratch[..count * 2].chunks_exact(2).enumerate() {
            let expected = match frame + offset {
                0 => [0.125, 0.0],
                last if last == MAX_SAMPLE_FRAMES - 1 => [0.5, -0.25],
                _ => [0.0, 0.0],
            };
            assert_eq!(pair, expected);
        }
        frame += count;
    }
    assert_eq!(largest.snapshot().active_voices(), 0);

    let max_file = files.0.join("max-file.wav");
    {
        let mut file = File::create(&max_file).unwrap();
        file.write_all(b"RIFF").unwrap();
        file.write_all(&(MAX_ENCODED_FILE_BYTES as u32 - 8).to_le_bytes())
            .unwrap();
        file.write_all(b"WAVE").unwrap();
        file.write_all(&chunk(b"fmt ", &format(1, 16, 1, 48_000, false)))
            .unwrap();
        file.write_all(b"JUNK").unwrap();
        file.write_all(&(MAX_ENCODED_FILE_BYTES as u32 - 54).to_le_bytes())
            .unwrap();
        file.seek(SeekFrom::Start(MAX_ENCODED_FILE_BYTES - 10))
            .unwrap();
        file.write_all(&chunk(b"data", &8192i16.to_le_bytes()))
            .unwrap();
    }
    assert_eq!(
        fs::metadata(&max_file).unwrap().len(),
        MAX_ENCODED_FILE_BYTES
    );
    let mut max_bytes = load_player(&[select(0, &max_file)], output);
    fs::remove_file(max_file).unwrap();
    max_bytes.trigger(0, Gain::new(1.0).unwrap()).unwrap();
    let mut tiny = [f32::NAN; 4];
    max_bytes.render_interleaved(&mut tiny).unwrap();
    assert_eq!(tiny, [0.25, 0.25, 0.0, 0.0]);

    let mut chunks = vec![chunk(b"fmt ", &format(1, 16, 1, 48_000, false))];
    chunks.extend((0..MAX_RIFF_CHUNKS - 2).map(|_| chunk(b"JUNK", &[])));
    chunks.push(chunk(b"data", &8192i16.to_le_bytes()));
    let max_chunks = files.write("max-chunks.wav", &riff(&chunks));
    let mut active = load_player(&[select(4, &max_chunks)], output);
    fs::remove_file(max_chunks).unwrap();
    active.trigger(4, Gain::new(1.0).unwrap()).unwrap();
    let before = active.snapshot();
    let pointer = active.bank().get(4).unwrap().samples().as_ptr();
    chunks.push(chunk(b"JUNK", &[]));
    let excess_chunks = files.write("excess-chunks.wav", &riff(&chunks));
    let excess_frames = files.write(
        "excess-frames.wav",
        &wav(1, 16, 1, &vec![0; (MAX_SAMPLE_FRAMES + 1) * 2], false),
    );
    let excess_file = files.0.join("excess-file.wav");
    File::create(&excess_file)
        .unwrap()
        .set_len(MAX_ENCODED_FILE_BYTES + 1)
        .unwrap();
    let good = files.write("good.wav", &wav(1, 16, 1, &8192i16.to_le_bytes(), false));
    let coordinator = BankPreparationCoordinator::default();
    for (path, limit) in [
        (excess_chunks, LoadLimit::Chunks),
        (excess_frames, LoadLimit::Frames),
        (excess_file, LoadLimit::EncodedBytes),
    ] {
        let result = coordinator.prepare(
            output,
            &[select(0, &good), select(15, &path)],
            Some(active.bank()),
            &AtomicBool::new(false),
        );
        assert!(matches!(result, Err(BankPreparationError::Asset {
            slot: 15, error: LoadError::LimitExceeded(actual)
        }) if actual == limit));
        assert_eq!(active.snapshot(), before);
        assert_eq!(active.bank().get(4).unwrap().samples().as_ptr(), pointer);
        drop(
            coordinator
                .prepare(output, &[], Some(active.bank()), &AtomicBool::new(false))
                .unwrap(),
        );
    }
    active.render_interleaved(&mut tiny).unwrap();
    assert_eq!(tiny, [0.25, 0.25, 0.0, 0.0]);
}
