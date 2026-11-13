use super::dto::{
    MixerChannelDto, MixerSettingsDto, PatternDto, ProjectSummaryDto, SequencerCountersDto,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDocumentDto {
    pub path: Option<String>,
    pub dirty: bool,
    pub summary: ProjectSummaryDto,
    pub pattern: PatternDto,
    pub mixer: MixerSettingsDto,
    pub assets: [Option<String>; 16],
}
impl From<daw_app::project_file::ProjectStatus> for ProjectDocumentDto {
    fn from(value: daw_app::project_file::ProjectStatus) -> Self {
        let doc = value.document;
        Self {
            path: value.path.map(|p| {
                let path = p.to_string_lossy();
                if let Some(rest) = path.strip_prefix("\\\\?\\UNC\\") {
                    format!("\\\\{rest}")
                } else {
                    path.strip_prefix("\\\\?\\").unwrap_or(&path).to_owned()
                }
            }),
            dirty: value.dirty,
            summary: ProjectSummaryDto {
                id: doc.id,
                title: doc.title,
                tempo: doc.bpm,
            },
            pattern: PatternDto {
                steps: doc.pattern.steps,
                slots: doc.pattern.slots,
            },
            mixer: MixerSettingsDto {
                channels: doc.mixer.channels.map(|c| MixerChannelDto {
                    gain: c.gain,
                    pan: c.pan,
                    muted: c.muted,
                }),
                routes: doc.mixer.routes,
            },
            assets: doc.assets,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum BeatTailDto {
    Cut,
    Finish,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BeatExportRequestDto {
    pub output_path: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub bars: u32,
    pub block_size: u16,
    pub tail: BeatTailDto,
}
impl BeatExportRequestDto {
    pub fn options(
        &self,
    ) -> Result<daw_app::project_render::BeatRenderOptions, daw_app::AudioError> {
        let invalid =
            |message| daw_app::AudioError::new(daw_app::ErrorCode::InvalidArgument, message);
        if self.output_path.is_empty()
            || self.output_path.len() > 32768
            || !(100..=192000).contains(&self.sample_rate)
            || ![1, 2].contains(&self.channels)
            || !(1..=4096).contains(&self.bars)
            || !(1..=4096).contains(&self.block_size)
        {
            return Err(invalid(
                "Invalid beat export path/rate/channels/bars/block size",
            ));
        }
        Ok(daw_app::project_render::BeatRenderOptions {
            config: daw_app::EngineConfig::new(self.sample_rate, self.channels)
                .map_err(|_| invalid("Invalid beat engine config"))?,
            bars: u64::from(self.bars),
            block_size: usize::from(self.block_size),
            tail: match self.tail {
                BeatTailDto::Cut => daw_app::project_render::TailPolicy::Cut,
                BeatTailDto::Finish => daw_app::project_render::TailPolicy::Finish,
            },
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct BeatExportDto {
    pub musical_frames: String,
    pub tail_frames: String,
    pub total_frames: String,
    pub counters: SequencerCountersDto,
}
impl From<daw_app::project_render::BeatRenderReport> for BeatExportDto {
    fn from(report: daw_app::project_render::BeatRenderReport) -> Self {
        let c = report.counters;
        Self {
            musical_frames: report.musical_frames.to_string(),
            tail_frames: report.tail_frames.to_string(),
            total_frames: report.total_frames.to_string(),
            counters: SequencerCountersDto {
                boundaries: c.boundaries.to_string(),
                attempted: c.attempted.to_string(),
                started: c.started.to_string(),
                missing_asset: c.missing_asset.to_string(),
                pool_full: c.pool_full.to_string(),
            },
        }
    }
}
