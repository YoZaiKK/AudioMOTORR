//! Versioned document I/O. Never call from audio processing.
use crate::{
    Application, AudioError, AudioServiceState, EngineConfig, ErrorCode,
    sampler_session::{PreparationSnapshot, SampleFileSelection},
};
use daw_domain::{MixerChannel, Project, ProjectId, Tempo};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub type ProjectResult<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
pub const MAX_PROJECT_BYTES: usize = 1024 * 1024;
static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ChannelDocument {
    pub gain: f64,
    pub pan: f64,
    pub muted: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PatternDocument {
    pub steps: [[bool; 16]; 4],
    pub slots: [u8; 4],
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MixerDocument {
    pub channels: [ChannelDocument; 4],
    pub routes: [u8; 16],
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProjectDocument {
    pub format: String,
    pub version: u32,
    pub id: String,
    pub title: String,
    pub bpm: f64,
    pub pattern: PatternDocument,
    pub mixer: MixerDocument,
    pub assets: [Option<String>; 16],
}
impl ProjectDocument {
    pub fn from_project(project: &Project) -> Self {
        Self {
            format: "audiomotorr-project".into(),
            version: 1,
            id: project.id().get().to_string(),
            title: project.title().into(),
            bpm: project.tempo().bpm(),
            pattern: PatternDocument {
                steps: std::array::from_fn(|row| {
                    std::array::from_fn(|step| project.pattern().step(row, step).unwrap())
                }),
                slots: std::array::from_fn(|row| project.pattern().row_slot(row).unwrap() as u8),
            },
            mixer: MixerDocument {
                channels: std::array::from_fn(|channel| {
                    let c = project.mixer().channel(channel).unwrap();
                    ChannelDocument {
                        gain: c.gain(),
                        pan: c.pan(),
                        muted: c.muted(),
                    }
                }),
                routes: std::array::from_fn(|slot| project.mixer().route(slot).unwrap() as u8),
            },
            assets: project.assets().clone(),
        }
    }
    pub fn into_project(self) -> ProjectResult<Project> {
        if self.format != "audiomotorr-project" || self.version != 1 {
            return Err(
                "Unsupported project format/version; expected audiomotorr-project v1".into(),
            );
        }
        let id = self.id.parse::<u64>()?;
        if id.to_string() != self.id {
            return Err("Project ID must be a canonical decimal string".into());
        }
        let mut project = Project::new(ProjectId::new(id)?, &self.title, Tempo::new(self.bpm)?)?;
        for row in 0..4 {
            project
                .pattern_mut()
                .set_row_slot(row, usize::from(self.pattern.slots[row]))?;
            for step in 0..16 {
                project
                    .pattern_mut()
                    .set_step(row, step, self.pattern.steps[row][step])?;
            }
            let c = &self.mixer.channels[row];
            project
                .mixer_mut()
                .set_channel(row, MixerChannel::new(c.gain, c.pan, c.muted)?)?;
        }
        for slot in 0..16 {
            project
                .mixer_mut()
                .set_route(slot, usize::from(self.mixer.routes[slot]))?;
            if let Some(path) = &self.assets[slot] {
                validate_relative(path)?;
            }
            project.set_asset(slot, self.assets[slot].clone())?;
        }
        Ok(project)
    }
}
pub fn validate_relative(path: &str) -> ProjectResult<()> {
    if path.len() > 4096
        || path.contains(['\\', ':', '\0'])
        || path
            .split('/')
            .any(|c| c.is_empty() || c == "." || c == "..")
    {
        return Err(format!("Invalid relative asset reference: {path:?}").into());
    }
    Ok(())
}
fn asset_metadata(path: &Path) -> ProjectResult<()> {
    let metadata =
        fs::metadata(path).map_err(|e| format!("Cannot read asset {}: {e}", path.display()))?;
    if !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > crate::sample_loading::MAX_ENCODED_FILE_BYTES
    {
        return Err(format!("Asset is not a nonempty bounded file: {}", path.display()).into());
    }
    Ok(())
}
pub fn resolved_assets(
    project: &Project,
    project_path: Option<&Path>,
) -> ProjectResult<Vec<SampleFileSelection>> {
    let base = match project_path.and_then(Path::parent) {
        Some(parent) => parent.to_owned(),
        None => std::env::current_dir()?,
    };
    Ok(project
        .assets()
        .iter()
        .enumerate()
        .filter_map(|(slot, path)| {
            path.as_ref().map(|path| SampleFileSelection {
                slot,
                path: base.join(path),
            })
        })
        .collect())
}
pub fn read_project(path: &Path) -> ProjectResult<(Project, PathBuf)> {
    let path = fs::canonicalize(path)
        .map_err(|e| format!("Cannot open project {}: {e}", path.display()))?;
    let mut bytes = Vec::new();
    File::open(&path)?
        .take((MAX_PROJECT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_PROJECT_BYTES {
        return Err("Project exceeds 1 MiB".into());
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    if value.get("version").and_then(serde_json::Value::as_u64) != Some(1) {
        return Err("Unsupported project version; only v1 is supported".into());
    }
    let document: ProjectDocument = serde_json::from_slice(&bytes)?;
    let project = document.into_project()?;
    for selection in resolved_assets(&project, Some(&path))? {
        asset_metadata(&selection.path)?;
    }
    Ok((project, path))
}
struct Writes {
    parent: PathBuf,
    temporary: Option<PathBuf>,
    directory: Option<PathBuf>,
}
impl Drop for Writes {
    fn drop(&mut self) {
        if let Some(path) = &self.temporary {
            let _ = fs::remove_file(path);
        }
        if let Some(path) = &self.directory {
            // Only the fresh directory this operation exclusively created is owned.
            if path.parent() == Some(self.parent.as_path()) {
                let _ = fs::remove_dir_all(path);
            }
        }
    }
}
pub fn save_project(
    project: &Project,
    previous_path: Option<&Path>,
    destination: &Path,
    overwrite: bool,
) -> ProjectResult<(Project, PathBuf)> {
    let name = destination
        .file_name()
        .ok_or("Project destination requires a file name")?;
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent =
        fs::canonicalize(parent).map_err(|e| format!("Cannot access project directory: {e}"))?;
    let destination = parent.join(name);
    if !overwrite && destination.try_exists()? {
        return Err("Project destination exists; overwrite must be explicit".into());
    }
    let mut writes = Writes {
        parent: parent.clone(),
        temporary: None,
        directory: None,
    };
    let mut next = project.clone();
    let selections = resolved_assets(project, previous_path)?;
    if !selections.is_empty() {
        let folder = loop {
            let name = format!(
                "audiomotorr-assets-{}-{}",
                std::process::id(),
                NEXT_FILE.fetch_add(1, Ordering::Relaxed)
            );
            let path = parent.join(&name);
            match fs::create_dir(&path) {
                Ok(()) => {
                    writes.directory = Some(path);
                    break name;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.into()),
            }
        };
        for selection in selections {
            asset_metadata(&selection.path)?;
            let relative = format!("{folder}/slot-{}.wav", selection.slot);
            let mut input = File::open(&selection.path)?
                .take(crate::sample_loading::MAX_ENCODED_FILE_BYTES + 1);
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(parent.join(&relative))?;
            let copied = std::io::copy(&mut input, &mut output)?;
            if copied == 0 || copied > crate::sample_loading::MAX_ENCODED_FILE_BYTES {
                return Err("Asset grew beyond its encoded size limit".into());
            }
            output.sync_all()?;
            next.set_asset(selection.slot, Some(relative))?;
        }
    }
    let bytes = serde_json::to_vec_pretty(&ProjectDocument::from_project(&next))?;
    if bytes.len() > MAX_PROJECT_BYTES {
        return Err("Project exceeds 1 MiB".into());
    }
    let mut temporary = loop {
        let path = parent.join(format!(
            ".audiomotorr-project-{}-{}.tmp",
            std::process::id(),
            NEXT_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => {
                writes.temporary = Some(path);
                break file;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.into()),
        }
    };
    temporary.write_all(&bytes)?;
    temporary.write_all(b"\n")?;
    temporary.sync_all()?;
    drop(temporary);
    let source = writes.temporary.as_ref().unwrap();
    if overwrite {
        fs::rename(source, &destination)?;
    } else {
        fs::hard_link(source, &destination)?;
    }
    writes.directory = None; // committed JSON owns its newly copied resources
    Ok((next, destination))
}
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectStatus {
    pub path: Option<PathBuf>,
    pub dirty: bool,
    pub document: ProjectDocument,
}
fn app_error(error: impl std::fmt::Display) -> AudioError {
    AudioError::new(ErrorCode::InvalidArgument, error.to_string())
}
impl Application {
    pub fn project_status(&self) -> ProjectStatus {
        ProjectStatus {
            path: self.project_path.clone(),
            dirty: self.saved_project.as_ref() != Some(&self.project),
            document: ProjectDocument::from_project(&self.project),
        }
    }
    pub fn project_set_title(&mut self, title: &str) -> crate::Result<ProjectStatus> {
        self.project.set_title(title).map_err(app_error)?;
        Ok(self.project_status())
    }
    pub fn save_project(&mut self, path: &Path, overwrite: bool) -> crate::Result<ProjectStatus> {
        let title = self.project.title().to_owned();
        self.save_project_titled(path, overwrite, &title)
    }
    pub fn save_project_titled(
        &mut self,
        path: &Path,
        overwrite: bool,
        title: &str,
    ) -> crate::Result<ProjectStatus> {
        let mut candidate = self.project.clone();
        candidate.set_title(title).map_err(app_error)?;
        let (project, path) =
            save_project(&candidate, self.project_path.as_deref(), path, overwrite)
                .map_err(app_error)?;
        self.project = project;
        self.project_path = Some(path);
        self.saved_project = Some(self.project.clone());
        Ok(self.project_status())
    }
    pub fn open_project(&mut self, path: &Path) -> crate::Result<ProjectStatus> {
        self.sample_preparation();
        self.samples.ensure_idle()?;
        if self.get_engine_snapshot()?.state != AudioServiceState::Uninitialized {
            return Err(AudioError::new(
                ErrorCode::Busy,
                "Close audio before opening a project",
            ));
        }
        let (project, path) = read_project(path).map_err(app_error)?;
        self.project = project;
        self.project_path = Some(path);
        self.saved_project = Some(self.project.clone());
        Ok(self.project_status())
    }
    pub fn prepare_project_samples(
        &mut self,
        config: EngineConfig,
    ) -> crate::Result<PreparationSnapshot> {
        let files =
            resolved_assets(&self.project, self.project_path.as_deref()).map_err(app_error)?;
        self.start_sample_preparation(config, files, false)
    }
}
