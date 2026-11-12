#[path = "support/sample_files.rs"]
mod sample_files;
#[path = "support/wav_fixtures.rs"]
mod wav_fixtures;

use daw_app::sampler_session::{PreparationState, SampleFileSelection};
use daw_app::{Application, EngineConfig, SamplerTriggerError, TransportState};
use std::time::{Duration, Instant};

fn load_bank(
    app: &mut Application,
    files: &sample_files::SampleFiles,
    output_channels: u16,
    patterns: [&[f32]; 4],
) -> u64 {
    let selections: Vec<_> = patterns
        .into_iter()
        .enumerate()
        .map(|(slot, pattern)| {
            let frame: Vec<u8> = pattern
                .iter()
                .flat_map(|sample| sample.to_le_bytes())
                .collect();
            let path = files.write(
                &format!("channel-{slot}.wav"),
                &wav_fixtures::wav(3, 32, pattern.len() as u16, &frame.repeat(8192), false),
            );
            SampleFileSelection { slot, path }
        })
        .collect();
    let id = app
        .prepare_samples(
            EngineConfig::new(48_000, output_channels).unwrap(),
            selections.clone(),
        )
        .unwrap()
        .preparation_id
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let next = app.sample_preparation();
        if next.state != PreparationState::Loading {
            assert_eq!(next.state, PreparationState::Ready, "{:?}", next.error);
            assert_eq!(next.occupied_slots, 15);
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    // Every subsequent render must use the retained decoded bank.
    for selection in selections {
        std::fs::remove_file(selection.path).unwrap();
    }
    id
}

#[derive(Debug, Default, PartialEq)]
struct Peaks {
    channels: [[f32; 2]; 4],
    master: [f32; 2],
}
fn peaks(channels: [[f32; 2]; 4], master: [f32; 2]) -> Peaks {
    Peaks { channels, master }
}

fn render(app: &Application, frames: usize, partition: usize) -> Peaks {
    let mut result = Peaks::default();
    let mut previous = app.get_engine_snapshot().unwrap();
    let mut remaining = frames;
    while remaining > 0 {
        let count = remaining.min(partition);
        let report = app.render_manual(count).unwrap();
        assert_eq!(report.frames, count);
        let current = app.get_engine_snapshot().unwrap();
        let mixer = current.mixer.unwrap();
        assert_eq!(mixer.meters.frames, count as u64);
        assert_eq!(
            mixer.meters.sequence,
            previous.mixer.unwrap().meters.sequence + 1
        );
        assert_eq!(
            current.engine.unwrap().device_frames,
            previous.engine.unwrap().device_frames + count as u64
        );
        for (index, peak) in mixer.meters.channels.iter().enumerate() {
            result.channels[index][0] = result.channels[index][0].max(peak.left);
            result.channels[index][1] = result.channels[index][1].max(peak.right.unwrap_or(0.0));
        }
        result.master[0] = result.master[0].max(mixer.meters.master.left);
        result.master[1] = result.master[1].max(mixer.meters.master.right.unwrap_or(0.0));
        previous = current;
        remaining -= count;
    }
    result
}

#[test]
fn four_decoded_channels_match_independent_oracles_across_partitions_and_lifecycle() {
    for partition in [1, 127, 256, 4096] {
        let files = sample_files::SampleFiles::new();
        let mut app = Application::new().unwrap();
        app.mixer_set_channel(0, 0.5, -1.0, false).unwrap();
        app.mixer_set_channel(1, 1.0, 1.0, false).unwrap();
        app.mixer_set_channel(2, 0.25, 0.0, false).unwrap();
        let id = load_bank(
            &mut app,
            &files,
            2,
            [&[1.0], &[0.5, -0.25], &[-1.0], &[0.5]],
        );
        app.mixer_set_channel(3, 0.5, 0.0, true).unwrap();
        let installed = app.initialize_manual_mixed_sampler(id).unwrap();
        assert_eq!(installed.mixer.unwrap().settings, app.get_mixer_settings());
        assert_eq!(installed.engine.unwrap().transport, TransportState::Stopped);
        assert_eq!(installed.last_applied_command, None);
        app.signal_set_gain(0.5).unwrap();
        app.transport_play().unwrap();
        assert_eq!(render(&app, 240, partition), Peaks::default());
        for slot in 0..4 {
            app.trigger_sample(slot, 1.0).unwrap();
        }
        assert_eq!(
            render(&app, 513, partition),
            peaks(
                [
                    [1.0 / 32.0, 0.0],
                    [0.0, 1.0 / 64.0],
                    [1.0 / 64.0, 1.0 / 64.0],
                    [0.0, 0.0]
                ],
                [1.0 / 128.0, 1.0 / 64.0],
            )
        );
        assert_eq!(
            app.get_engine_snapshot()
                .unwrap()
                .sampler
                .unwrap()
                .active_voices,
            4
        );
        app.mixer_set_route(0, 1).unwrap();
        app.trigger_sample(0, 1.0).unwrap();
        assert_eq!(
            render(&app, 1, partition),
            peaks(
                [
                    [1.0 / 32.0, 0.0],
                    [0.0, 3.0 / 64.0],
                    [1.0 / 64.0, 1.0 / 64.0],
                    [0.0, 0.0]
                ],
                [1.0 / 128.0, 1.0 / 64.0],
            )
        );
        let captured = [Some(0), Some(1), Some(2), Some(3), Some(1)];
        assert_eq!(
            app.get_engine_snapshot()
                .unwrap()
                .mixer
                .unwrap()
                .voice_channels[..5],
            captured
        );
        app.mixer_set_channel(2, 0.25, 0.0, true).unwrap();
        render(&app, 240, partition);
        assert_eq!(render(&app, 1, partition).master, [1.0 / 64.0, 3.0 / 128.0]);
        app.transport_pause().unwrap();
        render(&app, 240, partition);
        let frozen = app
            .get_engine_snapshot()
            .unwrap()
            .engine
            .unwrap()
            .transport_frames;
        assert_eq!(render(&app, 17, partition), Peaks::default());
        app.mixer_set_channel(0, 0.25, 1.0, false).unwrap();
        app.mixer_set_channel(1, 0.5, -1.0, false).unwrap();
        app.mixer_set_channel(3, 0.5, 0.0, false).unwrap();
        app.mixer_set_route(0, 3).unwrap();
        assert_eq!(render(&app, 1, partition), Peaks::default());
        let paused = app.get_engine_snapshot().unwrap();
        assert_eq!(paused.engine.unwrap().transport, TransportState::Paused);
        assert_eq!(paused.engine.unwrap().transport_frames, frozen);
        assert_eq!(paused.mixer.unwrap().voice_channels[..5], captured);
        assert_eq!(paused.mixer.unwrap().channels[0].gain_remaining_frames, 0);
        assert_eq!(paused.mixer.unwrap().channels[0].pan_remaining_frames, 0);
        app.transport_play().unwrap();
        render(&app, 240, partition);
        assert_eq!(
            render(&app, 1, partition),
            peaks(
                [
                    [0.0, 1.0 / 64.0],
                    [3.0 / 64.0, 0.0],
                    [0.0, 0.0],
                    [1.0 / 64.0, 1.0 / 64.0]
                ],
                [1.0 / 32.0, 1.0 / 64.0],
            )
        );
        app.trigger_sample(0, 1.0).unwrap();
        assert_eq!(render(&app, 1, partition).master, [3.0 / 64.0, 1.0 / 32.0]);
        assert_eq!(
            app.get_engine_snapshot()
                .unwrap()
                .mixer
                .unwrap()
                .voice_channels[5],
            Some(3)
        );
        let before_empty = app.get_engine_snapshot().unwrap();
        app.render_manual(0).unwrap();
        assert_eq!(app.get_engine_snapshot().unwrap(), before_empty);
        app.transport_stop().unwrap();
        render(&app, 240, partition);
        assert_eq!(render(&app, 1, partition), Peaks::default());
        let stopped = app.get_engine_snapshot().unwrap();
        assert_eq!(stopped.engine.unwrap().transport, TransportState::Stopped);
        assert_eq!(stopped.sampler.unwrap().active_voices, 0);
        assert_eq!(stopped.sampler.unwrap().occupied_slots, 15);
        let settings = app.get_mixer_settings();
        app.shutdown_audio().unwrap();
        assert_eq!(app.get_mixer_settings(), settings);
        let id = load_bank(
            &mut app,
            &files,
            2,
            [&[1.0], &[0.5, -0.25], &[-1.0], &[0.5]],
        );
        let reopened = app.initialize_manual_mixed_sampler(id).unwrap();
        assert_eq!(reopened.mixer.unwrap().settings, settings);
        assert_eq!(reopened.mixer.unwrap().meters.sequence, 0);
        assert_eq!(reopened.mixer.unwrap().voice_channels, [None; 16]);
        assert_eq!(reopened.engine.unwrap().device_frames, 0);
        assert_eq!(reopened.last_applied_command, None);
        assert_eq!(render(&app, 1, partition), Peaks::default());
    }
}

#[test]
fn decoded_pool_is_global_and_muted_voices_complete_in_mono_and_stereo() {
    for output in [1, 2] {
        let files = sample_files::SampleFiles::new();
        let mut app = Application::new().unwrap();
        if output == 1 {
            for channel in 0..4 {
                app.mixer_set_channel(
                    channel,
                    1.0,
                    if channel % 2 == 0 { -1.0 } else { 1.0 },
                    false,
                )
                .unwrap();
            }
        }
        let id = load_bank(&mut app, &files, output, [&[1.0]; 4]);
        app.initialize_manual_mixed_sampler(id).unwrap();
        app.transport_play().unwrap();
        render(&app, 240, 4096);
        for voice in 0..16 {
            app.trigger_sample(voice % 4, 1.0).unwrap();
        }
        let measured = render(&app, 1, 4096);
        let side = if output == 2 { 1.0 } else { 0.0 };
        assert_eq!(measured, peaks([[0.25, side * 0.25]; 4], [1.0, side]));
        let mono_right = app
            .get_engine_snapshot()
            .unwrap()
            .mixer
            .unwrap()
            .meters
            .master
            .right;
        assert_eq!(mono_right, (output == 2).then_some(1.0));
        let rejected = app.trigger_sample(0, 1.0).unwrap();
        render(&app, 1, 4096);
        let full = app.get_engine_snapshot().unwrap().sampler.unwrap();
        assert_eq!(full.active_voices, 16);
        assert_eq!((full.started, full.rejected), (16, 1));
        assert_eq!(full.last_trigger.unwrap().command_id, rejected.command_id);
        assert_eq!(
            full.last_trigger.unwrap().result,
            Err(SamplerTriggerError::PoolFull)
        );
        for channel in 0..4 {
            app.mixer_set_channel(channel, 1.0, 0.0, true).unwrap();
        }
        render(&app, 240, 4096);
        assert_eq!(render(&app, 1, 4096), Peaks::default());
        assert_eq!(
            app.get_engine_snapshot()
                .unwrap()
                .sampler
                .unwrap()
                .active_voices,
            16
        );
        assert_eq!(render(&app, 8192, 4096), Peaks::default());
        assert_eq!(
            app.get_engine_snapshot()
                .unwrap()
                .sampler
                .unwrap()
                .active_voices,
            0
        );
        assert_eq!(
            app.get_engine_snapshot()
                .unwrap()
                .sampler
                .unwrap()
                .occupied_slots,
            15
        );
        for channel in 0..4 {
            app.mixer_set_channel(channel, 1.0, 0.0, false).unwrap();
        }
        render(&app, 240, 4096);
        app.trigger_sample(0, 1.0).unwrap();
        assert_eq!(render(&app, 1, 4096).master, [1.0 / 16.0, side / 16.0]);
    }
}

#[test]
fn decoded_opposite_channels_have_real_levels_while_the_master_cancels() {
    for output in [1, 2] {
        let files = sample_files::SampleFiles::new();
        let mut app = Application::new().unwrap();
        let id = load_bank(&mut app, &files, output, [&[0.5], &[-0.5], &[0.0], &[0.0]]);
        app.initialize_manual_mixed_sampler(id).unwrap();
        app.transport_play().unwrap();
        render(&app, 240, 4096);
        app.trigger_sample(0, 1.0).unwrap();
        app.trigger_sample(1, 1.0).unwrap();
        let side = if output == 2 { 1.0 / 32.0 } else { 0.0 };
        assert_eq!(
            render(&app, 1001, 4096),
            peaks(
                [
                    [1.0 / 32.0, side],
                    [1.0 / 32.0, side],
                    [0.0, 0.0],
                    [0.0, 0.0]
                ],
                [0.0, 0.0],
            )
        );
        assert_eq!(
            app.get_engine_snapshot()
                .unwrap()
                .mixer
                .unwrap()
                .meters
                .frames,
            1001
        );
        assert_eq!(
            app.get_engine_snapshot()
                .unwrap()
                .sampler
                .unwrap()
                .active_voices,
            2
        );
        assert!(app.render_manual(1).unwrap().silent);
    }
}
