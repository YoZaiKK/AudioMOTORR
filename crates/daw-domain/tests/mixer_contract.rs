use daw_domain::{MIXER_CHANNELS, MixerChannel, MixerError, MixerSettings, Project, SAMPLE_SLOTS};

#[test]
fn selected_settings_are_bounded_copy_state_and_invalid_edits_are_atomic() {
    fn copy<T: Copy>() {}
    copy::<MixerSettings>();
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.01, 1.01] {
        assert_eq!(
            MixerChannel::new(value, 0.0, false),
            Err(MixerError::InvalidGain)
        );
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.01, 1.01] {
        assert_eq!(
            MixerChannel::new(1.0, value, false),
            Err(MixerError::InvalidPan)
        );
    }
    let mut settings = MixerSettings::default();
    for slot in 0..SAMPLE_SLOTS {
        assert_eq!(settings.route(slot), Ok(slot % MIXER_CHANNELS));
        for channel in 0..MIXER_CHANNELS {
            settings.set_route(slot, channel).unwrap();
            assert_eq!(settings.route(slot), Ok(channel));
        }
    }
    let before = settings;
    for invalid in [MIXER_CHANNELS, usize::MAX] {
        assert_eq!(
            settings.set_route(0, invalid),
            Err(MixerError::InvalidChannel)
        );
        assert_eq!(
            settings.set_channel(invalid, MixerChannel::default()),
            Err(MixerError::InvalidChannel)
        );
        assert_eq!(settings.channel(invalid), Err(MixerError::InvalidChannel));
        assert_eq!(settings, before);
    }
    for invalid in [SAMPLE_SLOTS, usize::MAX] {
        assert_eq!(settings.set_route(invalid, 0), Err(MixerError::InvalidSlot));
        assert_eq!(settings.route(invalid), Err(MixerError::InvalidSlot));
        assert_eq!(settings, before);
    }
    for gain in [0.0, 0.5, 1.0] {
        for pan in [-1.0, 0.0, 1.0] {
            let selected = MixerChannel::new(gain, pan, true).unwrap();
            assert_eq!(
                (selected.gain(), selected.pan(), selected.muted()),
                (gain, pan, true)
            );
        }
    }
}

#[test]
fn project_owns_editable_mixer_settings_without_shared_state() {
    let mut project = Project::default();
    let original = project.clone();
    assert_eq!(project.mixer(), &MixerSettings::default());
    project
        .mixer_mut()
        .set_channel(2, MixerChannel::new(0.5, -1.0, true).unwrap())
        .unwrap();
    project.mixer_mut().set_route(15, 2).unwrap();
    assert_eq!(original.mixer(), &MixerSettings::default());
    assert_ne!(project.mixer(), original.mixer());
    assert_eq!(project.id(), original.id());
    assert_eq!(project.tempo(), original.tempo());
}
