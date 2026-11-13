//! Device-free inspection/export of versioned musical projects.
use daw_app::{
    project_file::{ProjectResult, read_project},
    project_render::{BeatRender, BeatRenderOptions, TailPolicy},
};
use daw_engine::EngineConfig;
use std::{collections::HashSet, path::PathBuf};
pub fn run(command: &str, args: &[String]) -> ProjectResult<()> {
    let (mut project, mut output) = (None, None);
    let (mut rate, mut channels, mut bars, mut block, mut tail) =
        (48_000, 2, 1, 256, TailPolicy::Finish);
    let mut seen = HashSet::new();
    let mut pairs = args.chunks_exact(2);
    for pair in &mut pairs {
        if !seen.insert(pair[0].as_str()) {
            return Err("Duplicate project command option".into());
        }
        match (command, pair[0].as_str()) {
            (_, "--project") => project = Some(PathBuf::from(&pair[1])),
            ("export-beat", "--output") => output = Some(PathBuf::from(&pair[1])),
            ("export-beat", "--sample-rate") => rate = pair[1].parse()?,
            ("export-beat", "--channels") => channels = pair[1].parse()?,
            ("export-beat", "--bars") => bars = pair[1].parse()?,
            ("export-beat", "--block-size") => block = pair[1].parse()?,
            ("export-beat", "--tail") => tail = TailPolicy::parse(&pair[1])?,
            _ => return Err("Unknown project command option".into()),
        }
    }
    if !pairs.remainder().is_empty() {
        return Err("Missing project command option value".into());
    }
    let project = project
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or("--project PATH is required")?;
    if command == "project-check" {
        let (document, path) = read_project(&project)?;
        println!(
            "project-check OK: version=1, id={}, title={:?}, bpm={}, assets={}, path={}",
            document.id().get(),
            document.title(),
            document.tempo().bpm(),
            document.assets().iter().flatten().count(),
            path.display()
        );
        return Ok(());
    }
    let output = output
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or("--output PATH is required")?;
    if !(100..=192_000).contains(&rate)
        || ![1, 2].contains(&channels)
        || !(1..=4096).contains(&bars)
        || !(1..=4096).contains(&block)
    {
        return Err("Invalid beat export rate/channels/bars/block size".into());
    }
    let render = BeatRender::open(
        &project,
        BeatRenderOptions {
            config: EngineConfig::new(rate, channels)?,
            bars,
            block_size: block,
            tail,
        },
    )?;
    let report = render.write_wav(&output)?;
    println!(
        "export-beat OK: sample_rate={rate}, channels={channels}, bars={bars}, tail={tail:?}, musical_frames={}, tail_frames={}, frames={}, notes={:?}, output={}",
        report.musical_frames,
        report.tail_frames,
        report.total_frames,
        report.counters,
        output.display()
    );
    Ok(())
}
