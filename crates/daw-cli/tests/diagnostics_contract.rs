use std::process::Command;
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_daw-cli"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn diagnostic_help_and_feature_or_profile_gates_do_not_open_hardware() {
    let help = run(&["--help"]);
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(
        help.status.success()
            && text.contains("sampler-bench")
            && text.contains("sampler-native-check")
            && text.contains("mixer-bench")
            && text.contains("mixer-native-check")
    );
    #[cfg(not(any(feature = "offline-diagnostics", feature = "rt-diagnostics")))]
    for name in [
        "sampler-bench",
        "sampler-native-check",
        "mixer-bench",
        "mixer-native-check",
    ] {
        let output = run(&[name]);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Diagnostic commands require"));
    }
    #[cfg(all(
        debug_assertions,
        any(feature = "offline-diagnostics", feature = "rt-diagnostics")
    ))]
    {
        for name in ["sampler-bench", "mixer-bench", "mixer-native-check"] {
            let output = run(&[name]);
            assert!(!output.status.success());
            assert!(String::from_utf8_lossy(&output.stderr).contains("Diagnostics require"));
        }
    }
}

#[cfg(all(not(debug_assertions), feature = "offline-diagnostics"))]
#[test]
fn release_mixer_commands_report_mixed_path_and_gate_native_io() {
    for channels in ["1", "2"] {
        let output = run(&["mixer-bench", "--channels", channels, "--iterations", "9"]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            text.lines()
                .filter(|line| line.starts_with("mixer-bench completed:"))
                .count(),
            12
        );
        assert!(
            text.contains("source=mixed_sampler")
                && text.contains("allocations: 0")
                && text.contains("deallocations: 4")
                && text.contains("musical_peaks=")
                && text.contains("voices=16")
        );
    }
    for args in [
        vec!["mixer-bench", "--voices", "4"],
        vec!["mixer-bench", "--iterations", "0"],
        vec!["mixer-bench", "--channels", "3"],
        vec!["mixer-bench", "--voices", "1", "--voices", "16"],
        vec!["mixer-bench", "--iterations"],
        vec!["mixer-native-check", "--input", "missing.wav"],
        vec![
            "mixer-native-check",
            "--input",
            "missing.wav",
            "--device",
            "explicit",
            "--seconds",
            "6",
        ],
        vec![
            "mixer-native-check",
            "--input",
            "missing.wav",
            "--device",
            "explicit",
            "--voices",
            "4",
        ],
    ] {
        assert!(!run(&args).status.success());
    }
    #[cfg(not(feature = "native-audio"))]
    {
        let output = run(&[
            "mixer-native-check",
            "--input",
            "missing.wav",
            "--device",
            "explicit",
        ]);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("NativeAudioDisabled"));
    }
}
#[cfg(all(not(debug_assertions), feature = "offline-diagnostics"))]
#[test]
fn release_cli_reports_offline_matrix_and_rejects_bad_options_before_io() {
    let output = run(&["sampler-bench", "--iterations", "9"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        text.lines()
            .filter(|line| line.starts_with("sampler-bench completed:"))
            .count(),
        12
    );
    assert!(
        text.contains("allocations: 0")
            && text.contains("deallocations: 0")
            && text.contains("voices=16")
            && text.contains("p99_ns=")
    );
    for args in [
        vec!["sampler-bench", "--voices", "2"],
        vec!["sampler-native-check", "--input", "does-not-exist.wav"],
        vec![
            "sampler-native-check",
            "--input",
            "x",
            "--device",
            "x",
            "--seconds",
            "6",
        ],
    ] {
        assert!(!run(&args).status.success());
    }
    #[cfg(not(feature = "native-audio"))]
    {
        let output = run(&[
            "sampler-native-check",
            "--input",
            "does-not-exist.wav",
            "--device",
            "does-not-exist",
        ]);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("NativeAudioDisabled"));
    }
}
