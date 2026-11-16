use super::sample_files::SampleFiles;
use daw_domain::{Clip, MixerChannel, PlaybackMode, Project, Scene, SceneAction, Tempo};

pub const LENGTHS: [usize; 4] = [137, 211, 73, 113];
pub fn source(slot: usize, age: usize, side: usize) -> f32 {
    let side = if slot.is_multiple_of(2) { 0 } else { side };
    ((age * 7 + slot * 3 + side * 5) % 29) as f32 / 32.0 - 0.4375
}
pub fn project(files: &SampleFiles, rate: u32, channels: u16, bpm: f64) -> Project {
    let mut project = Project::default();
    project.set_title("Escena portátil").unwrap();
    project.set_tempo(Tempo::new(bpm).unwrap());
    for slot in 0_usize..4 {
        let path = files.0.join(format!("source-{slot}.wav"));
        let sides = if slot.is_multiple_of(2) { 1 } else { channels };
        let mut writer = hound::WavWriter::create(
            &path,
            hound::WavSpec {
                channels: sides,
                sample_rate: rate,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            },
        )
        .unwrap();
        for age in 0..LENGTHS[slot] {
            for side in 0..sides as usize {
                writer.write_sample(source(slot, age, side)).unwrap();
            }
        }
        writer.finalize().unwrap();
        project
            .set_asset(slot, Some(path.to_string_lossy().into()))
            .unwrap();
        project
            .mixer_mut()
            .set_channel(
                slot,
                MixerChannel::new(
                    [1.0, 0.5, 0.75, 0.25][slot],
                    [-0.5, 1.0, -1.0, 0.0][slot],
                    slot == 2,
                )
                .unwrap(),
            )
            .unwrap();
    }
    let mut scene = Scene::default();
    for (row, slot) in [15, 7, 0, 3].into_iter().enumerate() {
        let mut clip = Clip::new([0, 1, 2, 0][row]).unwrap();
        for step in 0_usize..16 {
            let enabled = match row {
                0 => [0, 15].contains(&step),
                1 => [0, 4, 12].contains(&step),
                2 => step.is_multiple_of(2),
                3 => [0, 7].contains(&step),
                _ => unreachable!(),
            };
            clip.set_step(step, enabled).unwrap();
        }
        project.launcher_mut().set_clip(row, slot, clip).unwrap();
        scene
            .set_action(row, SceneAction::Launch(slot as u8))
            .unwrap();
    }
    project.launcher_mut().set_scene(7, scene).unwrap();
    project.set_initial_scene(7).unwrap();
    project.set_playback_mode(PlaybackMode::Launcher);
    project.pattern_mut().set_step(0, 1, true).unwrap(); // Intentionally differs from the scene.
    project
}
