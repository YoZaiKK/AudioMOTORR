use std::process::Command;

#[test]
fn smoke_checks_exact_frames_for_multiple_partitions_and_zero_frames() {
    for block in ["1", "127", "256", "4096"] {
        let output = Command::new(env!("CARGO_BIN_EXE_daw-cli"))
            .args(["smoke", "--frames", "4801", "--block-size", block])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("frames=4801"));
    }
    assert!(
        Command::new(env!("CARGO_BIN_EXE_daw-cli"))
            .args(["smoke", "--frames", "0"])
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn invalid_cli_arguments_fail() {
    for args in [
        vec!["smoke", "--block-size", "0"],
        vec!["smoke", "--block-size", "4097"],
        vec!["smoke", "--frames", "-1"],
        vec!["smoke", "--frames"],
        vec!["smoke", "--frames", "1", "--seconds", "1"],
        vec!["devices", "--frames", "1"],
        vec!["smoke", "--frames", "1", "--frames", "2"],
        vec!["smoke", "--seconds", "18446744073709551615"],
    ] {
        assert!(
            !Command::new(env!("CARGO_BIN_EXE_daw-cli"))
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
}
