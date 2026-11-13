//! Independent beat render using the existing sequencer/sampler/mixer engine.
use crate::{
    Application,
    project_file::{ProjectResult, read_project, resolved_assets},
    sample_loading::{BankPreparationCoordinator, SampleSelection},
};
use daw_domain::{Pattern, Project};
use daw_engine::{AudioEngine, EngineCommand, EngineConfig, SequencerCounters};
use std::{path::Path, sync::atomic::AtomicBool};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TailPolicy {
    Cut,
    Finish,
}
impl TailPolicy {
    pub fn parse(value: &str) -> ProjectResult<Self> {
        match value {
            "cut" => Ok(Self::Cut),
            "finish" => Ok(Self::Finish),
            _ => Err("Tail must be cut or finish".into()),
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct BeatRenderOptions {
    pub config: EngineConfig,
    pub bars: u64,
    pub block_size: usize,
    pub tail: TailPolicy,
}
#[derive(Debug, Clone, Copy)]
pub struct BeatRenderReport {
    pub musical_frames: u64,
    pub tail_frames: u64,
    pub total_frames: u64,
    pub counters: SequencerCounters,
}
pub struct BeatRender {
    engine: AudioEngine,
    scratch: Vec<f32>,
    musical_frames: u64,
    tail: TailPolicy,
    maximum_tail: u64,
}
pub fn musical_frames(rate: u32, bpm: f64, bars: u64) -> ProjectResult<u64> {
    let tempo = daw_domain::Tempo::new(bpm)?;
    if !(1..=4096).contains(&bars) {
        return Err("Beat bars must be 1..4096".into());
    }
    let scale = 1_u128 << 48;
    let increment = (tempo.bpm() * scale as f64) as u128;
    let frames = (u128::from(bars) * 16 * u128::from(rate) * 15 * scale).div_ceil(increment);
    Ok(u64::try_from(frames)?)
}
impl BeatRender {
    /// Exclusive float32 WAV destination; all validation/decoding precedes creation.
    pub fn write_wav(self, path: &Path) -> ProjectResult<BeatRenderReport> {
        let config = self.config();
        let bytes_per_frame = u32::from(config.channels()) * 4;
        if config.sample_rate().checked_mul(bytes_per_frame).is_none()
            || self.frames_upper_bound() > (u64::from(u32::MAX) - 60) / u64::from(bytes_per_frame)
        {
            return Err("Beat WAV exceeds byte-rate or RIFF size limit".into());
        }
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| {
                format!(
                    "Cannot create beat WAV {} (existing files are not overwritten): {e}",
                    path.display()
                )
            })?;
        let result = (|| -> ProjectResult<BeatRenderReport> {
            let mut writer = hound::WavWriter::new(
                std::io::BufWriter::new(file),
                hound::WavSpec {
                    channels: config.channels(),
                    sample_rate: config.sample_rate(),
                    bits_per_sample: 32,
                    sample_format: hound::SampleFormat::Float,
                },
            )?;
            let report = self.render(|samples| {
                for &sample in samples {
                    writer.write_sample(sample)?;
                }
                Ok(())
            })?;
            writer.finalize()?;
            Ok(report)
        })();
        result.map_err(|e| {
            format!(
                "Beat WAV export failed; partial output may remain at {}: {e}",
                path.display()
            )
            .into()
        })
    }
    pub fn open(path: &Path, options: BeatRenderOptions) -> ProjectResult<Self> {
        let (project, path) = read_project(path)?;
        Self::from_project(&project, Some(&path), options)
    }
    pub fn from_project(
        project: &Project,
        path: Option<&Path>,
        options: BeatRenderOptions,
    ) -> ProjectResult<Self> {
        if !(1..=2).contains(&options.config.channels())
            || !(1..=4096).contains(&options.block_size)
        {
            return Err("Beat output requires mono/stereo and block 1..4096".into());
        }
        let musical_frames = musical_frames(
            options.config.sample_rate(),
            project.tempo().bpm(),
            options.bars,
        )?;
        for row in 0..4 {
            if (0..16).any(|step| project.pattern().step(row, step).unwrap()) {
                let slot = project.pattern().row_slot(row)?;
                if project.assets()[slot].is_none() {
                    return Err(format!(
                        "Enabled pattern row {} references empty slot {slot}",
                        row + 1
                    )
                    .into());
                }
            }
        }
        let files = resolved_assets(project, path)?;
        let selections: Vec<_> = files
            .iter()
            .map(|file| SampleSelection {
                slot: file.slot,
                path: &file.path,
            })
            .collect();
        let prepared = BankPreparationCoordinator::default().prepare(
            options.config,
            &selections,
            None,
            &AtomicBool::new(false),
        )?;
        let maximum_tail = (0..16)
            .filter_map(|slot| prepared.bank().get(slot))
            .map(|s| s.frames() as u64)
            .max()
            .unwrap_or(0);
        let mut bank = None;
        prepared.install(&mut bank);
        let engine = AudioEngine::with_sequenced_sampler(
            bank.unwrap(),
            project.tempo(),
            *project.mixer(),
            *project.pattern(),
        );
        Ok(Self {
            engine,
            scratch: vec![0.0; options.block_size * usize::from(options.config.channels())],
            musical_frames,
            tail: options.tail,
            maximum_tail,
        })
    }
    pub fn config(&self) -> EngineConfig {
        self.engine.snapshot().config
    }
    pub fn musical_frames(&self) -> u64 {
        self.musical_frames
    }
    pub fn frames_upper_bound(&self) -> u64 {
        self.musical_frames
            + if self.tail == TailPolicy::Finish {
                self.maximum_tail
            } else {
                0
            }
    }
    fn remaining_voices(&self) -> u64 {
        let sampler = self.engine.sampler().unwrap();
        sampler
            .snapshot()
            .voices
            .iter()
            .flatten()
            .map(|voice| {
                (sampler.bank().get(voice.sample_slot).unwrap().frames() - voice.frame) as u64
            })
            .max()
            .unwrap_or(0)
    }
    fn emit(
        &mut self,
        frames: u64,
        sink: &mut impl FnMut(&[f32]) -> ProjectResult<()>,
    ) -> ProjectResult<()> {
        let channels = usize::from(self.config().channels());
        let mut remaining = frames;
        while remaining > 0 {
            let count = remaining.min((self.scratch.len() / channels) as u64) as usize;
            let buffer = &mut self.scratch[..count * channels];
            self.engine.render_interleaved(buffer)?;
            sink(buffer)?;
            remaining -= count as u64;
        }
        Ok(())
    }
    pub fn render(
        mut self,
        mut sink: impl FnMut(&[f32]) -> ProjectResult<()>,
    ) -> ProjectResult<BeatRenderReport> {
        self.engine.apply_command(EngineCommand::Play);
        self.emit(self.musical_frames, &mut sink)?;
        let counters = self.engine.sequencer_snapshot().unwrap().counters;
        let tail_frames = if self.tail == TailPolicy::Finish {
            self.remaining_voices()
        } else {
            0
        };
        if tail_frames > 0 {
            self.engine.set_pattern(Pattern::default())?;
            self.emit(tail_frames, &mut sink)?;
        }
        Ok(BeatRenderReport {
            musical_frames: self.musical_frames,
            tail_frames,
            total_frames: self.musical_frames + tail_frames,
            counters,
        })
    }
}
impl Application {
    pub fn project_render(&self, options: BeatRenderOptions) -> ProjectResult<BeatRender> {
        BeatRender::from_project(&self.project, self.project_path.as_deref(), options)
    }
}
