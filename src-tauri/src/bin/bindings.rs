use std::{error::Error, path::PathBuf};
fn main() -> Result<(), Box<dyn Error>> {
    let output =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/shared/api/generated/api.ts");
    let text = rust_daw_desktop_lib::ipc::dto::bindings_text();
    if std::env::args().any(|arg| arg == "--check") {
        if std::fs::read_to_string(&output)?.replace("\r\n", "\n") != text {
            return Err("Generated bindings are stale. Run npm run bindings:generate".into());
        }
    } else {
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(output, text)?;
    }
    Ok(())
}
