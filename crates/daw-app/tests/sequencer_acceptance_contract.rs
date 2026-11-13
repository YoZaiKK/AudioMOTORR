#[path = "support/sample_files.rs"]
mod sample_files;
#[path = "support/wav_fixtures.rs"]
mod wav_fixtures;

use daw_app::{
    Application, EngineConfig, TransportState,
    sample_loading::{BankPreparationCoordinator, SampleSelection},
    sampler_session::{PreparationState, SampleFileSelection},
};
use daw_engine::{
    AudioEngine, EngineCommand, Gain, MixerChannel, MixerSettings, Pattern, SequencerBoundary,
    SequencerCounters, SequencerEvent, SequencerSnapshot, Tempo,
};
use std::{
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

const RATE: u32 = 48_000;
const LENGTHS: [usize; 4] = [2048, 8000, 1024, 6000];
const ROUTES: [usize; 4] = [2, 0, 3, 1];
const GAINS: [f32; 4] = [0.5, 0.75, 0.25, 1.0];
const PANS: [f32; 4] = [-1.0, 0.5, 1.0, 0.0];

fn enabled(row: usize, step: usize) -> bool {
    match row {
        0 => step.is_multiple_of(4),
        1 => [4, 12].contains(&step),
        2 => step.is_multiple_of(2),
        3 => [3, 7, 11, 15].contains(&step),
        _ => unreachable!(),
    }
}
fn pattern() -> Pattern {
    let mut result = Pattern::default();
    for row in 0..4 {
        for step in 0..16 {
            result.set_step(row, step, enabled(row, step)).unwrap();
        }
    }
    result
}
fn settings() -> MixerSettings {
    let mut result = MixerSettings::default();
    for channel in 0..4 {
        result
            .set_channel(
                channel,
                MixerChannel::new(
                    f64::from(GAINS[channel]),
                    f64::from(PANS[channel]),
                    channel == 3,
                )
                .unwrap(),
            )
            .unwrap();
        result.set_route(channel, ROUTES[channel]).unwrap();
    }
    result
}
fn source(slot: usize, age: usize, side: usize) -> f32 {
    let side = if slot.is_multiple_of(2) { 0 } else { side };
    // Exactly representable in both PCM16 and float32, with distinct stereo sides.
    ((age * 7 + slot * 3 + side * 11) % 17) as f32 / 16.0 - 0.5
}
fn assets(files: &sample_files::SampleFiles, channels: u16) -> Vec<SampleFileSelection> {
    (0_usize..4)
        .map(|slot| {
            let width = if slot.is_multiple_of(2) { 1 } else { channels };
            let pcm16 = slot < 2;
            let mut bytes = Vec::new();
            for age in 0..LENGTHS[slot] {
                for side in 0..usize::from(width) {
                    let value = source(slot, age, side);
                    if pcm16 {
                        bytes.extend(((value * 32768.0) as i16).to_le_bytes());
                    } else {
                        bytes.extend(value.to_le_bytes());
                    }
                }
            }
            SampleFileSelection {
                slot,
                path: files.write(
                    &format!("reference-{slot}.wav"),
                    &wav_fixtures::wav(
                        if pcm16 { 1 } else { 3 },
                        if pcm16 { 16 } else { 32 },
                        width,
                        &bytes,
                        false,
                    ),
                ),
            }
        })
        .collect()
}

// Independent rational clock: BPM = numerator / 4. Compute absolute boundaries
// directly, without the engine's fixed-point scheduler or pattern event iterator.
fn boundary(ordinal: usize, numerator: u64) -> usize {
    (ordinal as u64 * u64::from(RATE) * 60).div_ceil(numerator) as usize
}
struct Reference {
    channels: usize,
    numerator: u64,
    frame: usize,
    ordinal: usize,
    voices: [Option<(usize, usize)>; 16],
    events: u64,
    last_event: Option<SequencerEvent>,
    last_boundary: Option<SequencerBoundary>,
}
struct Window {
    pcm: Vec<f32>,
    channels: [[f32; 2]; 4],
    master: [f32; 2],
}
impl Reference {
    fn new(channels: u16, numerator: u64) -> Self {
        Self {
            channels: usize::from(channels),
            numerator,
            frame: 0,
            ordinal: 0,
            voices: [None; 16],
            events: 0,
            last_event: None,
            last_boundary: None,
        }
    }
    fn render(&mut self, frames: usize) -> Window {
        let mut result = Window {
            pcm: Vec::with_capacity(frames * self.channels),
            channels: [[0.0; 2]; 4],
            master: [0.0; 2],
        };
        for _ in 0..frames {
            while boundary(self.ordinal, self.numerator) <= self.frame {
                let at = SequencerBoundary {
                    loop_index: (self.ordinal / 16) as u64,
                    step: self.ordinal % 16,
                    musical_frame: self.frame as u64,
                };
                self.last_boundary = Some(at);
                for row in 0..4 {
                    if enabled(row, at.step) {
                        let index = self.voices.iter().position(Option::is_none).unwrap();
                        self.voices[index] = Some((row, 0));
                        self.events += 1;
                        self.last_event = Some(SequencerEvent {
                            boundary: at,
                            row,
                            slot: row,
                            device_frame: self.frame as u64,
                            outcome: Ok(index),
                        });
                    }
                }
                self.ordinal += 1;
            }
            let mut sums = [[0.0_f32; 2]; 4];
            for voice in &mut self.voices {
                if let Some((slot, age)) = voice {
                    for side in 0..self.channels {
                        sums[ROUTES[*slot]][side] += source(*slot, *age, side);
                    }
                    *age += 1;
                    if *age == LENGTHS[*slot] {
                        *voice = None;
                    }
                }
            }
            for side in 0..self.channels {
                let mut sample = 0.0_f32;
                for channel in 0..4 {
                    let gain = if channel == 3 { 0.0 } else { GAINS[channel] };
                    let pan = if self.channels == 1 {
                        1.0
                    } else if side == 0 {
                        1.0 - PANS[channel].max(0.0)
                    } else {
                        1.0 + PANS[channel].min(0.0)
                    };
                    let contribution = ((sums[channel][side] * gain) * pan) / 16.0;
                    result.channels[channel][side] =
                        result.channels[channel][side].max(contribution.abs());
                    sample += contribution;
                }
                // Explicit 5 ms attack to the selected master gain 0.5.
                sample *= 0.5 * ((self.frame + 1) as f32 / 240.0).min(1.0);
                result.master[side] = result.master[side].max(sample.abs());
                result.pcm.push(sample);
            }
            self.frame += 1;
        }
        result
    }
    fn snapshot(&self) -> SequencerSnapshot {
        let denominator = u64::from(RATE) * 60;
        let position = self.frame as u64 * self.numerator;
        let steps = position / denominator;
        SequencerSnapshot {
            pattern: pattern(),
            musical_frames: self.frame as u64,
            loop_index: steps / 16,
            step: (steps % 16) as usize,
            phase_numerator: u128::from(position % denominator) * (1 << 46),
            phase_denominator: u128::from(denominator) * (1 << 46),
            last_boundary: self.last_boundary,
            counters: SequencerCounters {
                boundaries: self.ordinal as u64,
                attempted: self.events,
                started: self.events,
                missing_asset: 0,
                pool_full: 0,
            },
            last_event: self.last_event,
        }
    }
}
fn near(actual: f32, expected: f32) {
    assert!((actual - expected).abs() <= 1e-7, "{actual} != {expected}");
}

fn acceptance(channels: u16, numerator: u64, partition: usize, with_app: bool) {
    let files = sample_files::SampleFiles::new();
    let selections = assets(&files, channels);
    let config = EngineConfig::new(RATE, channels).unwrap();
    let bpm = numerator as f64 / 4.0;
    let prepared = BankPreparationCoordinator::default()
        .prepare(
            config,
            &selections
                .iter()
                .map(|s| SampleSelection {
                    slot: s.slot,
                    path: &s.path,
                })
                .collect::<Vec<_>>(),
            None,
            &AtomicBool::new(false),
        )
        .unwrap();
    let mut bank = None;
    prepared.install(&mut bank);
    let mut engine = AudioEngine::with_sequenced_sampler(
        bank.unwrap(),
        Tempo::new(bpm).unwrap(),
        settings(),
        pattern(),
    );
    let mut application = with_app.then(|| {
        let mut app = Application::new().unwrap();
        // Seed the document through its existing tempo command, even if the
        // original session closes before that command reaches a callback.
        app.initialize_manual(config).unwrap();
        app.transport_set_tempo(bpm).unwrap();
        app.shutdown_audio().unwrap();
        let id = app
            .prepare_samples(config, selections.clone())
            .unwrap()
            .preparation_id
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let state = app.sample_preparation();
            if state.state != PreparationState::Loading {
                assert_eq!(state.state, PreparationState::Ready, "{:?}", state.error);
                assert_eq!(state.occupied_slots, 15);
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        // Edits after preparation must be the settings used at installation.
        app.sequencer_set_pattern(pattern()).unwrap();
        for channel in 0..4 {
            app.mixer_set_route(channel, ROUTES[channel]).unwrap();
            app.mixer_set_channel(
                channel,
                f64::from(GAINS[channel]),
                f64::from(PANS[channel]),
                channel == 3,
            )
            .unwrap();
        }
        (app, id)
    });
    // Both independently decoded banks must work after every source is removed.
    for selection in selections {
        std::fs::remove_file(selection.path).unwrap();
    }
    let command_id = application.as_mut().map(|(app, id)| {
        let installed = app.initialize_manual_sequenced_sampler(*id).unwrap();
        assert_eq!(installed.engine.unwrap().transport, TransportState::Stopped);
        assert_eq!(installed.last_applied_command, None);
        assert_eq!(
            installed.sequencer.unwrap().counters,
            SequencerCounters::default()
        );
        app.signal_set_gain(0.5).unwrap();
        app.transport_play().unwrap().command_id
    });
    engine.apply_command(EngineCommand::SetGain(Gain::new(0.5).unwrap()));
    engine.apply_command(EngineCommand::Play);
    let mut reference = Reference::new(channels, numerator);
    let frames = boundary(16, numerator) + 257;
    for start in (0..frames).step_by(partition) {
        let count = partition.min(frames - start);
        let expected = reference.render(count);
        let mut actual = vec![f32::NAN; count * usize::from(channels)];
        engine.render_interleaved(&mut actual).unwrap();
        for (sample, expected) in actual.iter().zip(&expected.pcm) {
            near(*sample, *expected);
        }
        assert_eq!(engine.sequencer_snapshot().unwrap(), reference.snapshot());
        let mix = engine.mixer_snapshot().unwrap();
        assert_eq!(mix.meters.frames, count as u64);
        for (peak, expected) in mix.meters.channels.iter().zip(expected.channels) {
            near(peak.left, expected[0]);
            assert_eq!(peak.right.is_some(), channels == 2);
            near(peak.right.unwrap_or(0.0), expected[1]);
        }
        near(mix.meters.master.left, expected.master[0]);
        near(mix.meters.master.right.unwrap_or(0.0), expected.master[1]);
        assert_eq!(
            mix.voice_channels,
            reference.voices.map(|v| v.map(|(slot, _)| ROUTES[slot]))
        );
        if let Some((app, _)) = &application {
            let report = app.render_manual(count).unwrap();
            assert_eq!(report.frames, count);
            assert_eq!(report.silent, actual.iter().all(|s| *s == 0.0));
            let snapshot = app.get_engine_snapshot().unwrap();
            assert_eq!(snapshot.engine.unwrap(), engine.snapshot());
            assert_eq!(snapshot.sequencer, engine.sequencer_snapshot());
            assert_eq!(snapshot.mixer, Some(mix));
            assert_eq!(snapshot.last_applied_command, command_id);
            let sampler = snapshot.sampler.unwrap();
            assert_eq!(
                sampler.active_voices,
                reference.voices.iter().flatten().count()
            );
            assert_eq!((sampler.started, sampler.rejected), (0, 0));
            assert_eq!(sampler.last_trigger, None);
        }
    }
    assert_eq!(reference.events, 20); // 18 in the bar, then two at the next step 0.
    assert_eq!(reference.last_boundary.unwrap().loop_index, 1);
    if let Some((app, _)) = application {
        app.shutdown_audio().unwrap();
        assert_eq!(app.get_engine_snapshot().unwrap().sequencer, None);
        assert_eq!(app.get_pattern(), pattern());
        assert_eq!(app.get_mixer_settings(), settings());
    }
}

#[test]
fn decoded_beat_pcm_matches_independent_reference_across_formats_tempos_and_partitions() {
    for channels in [1, 2] {
        for numerator in [382, 509, 1200] {
            for partition in [1, 127, 256, 4096] {
                acceptance(channels, numerator, partition, false);
            }
        }
    }
}

#[test]
fn prepared_document_session_matches_reference_windows_events_voices_and_acknowledgements() {
    for channels in [1, 2] {
        for numerator in [382, 509, 1200] {
            for partition in [127, 256, 4096] {
                acceptance(channels, numerator, partition, true);
            }
        }
    }
}
