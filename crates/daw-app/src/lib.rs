//! Casos de uso independientes de Tauri. El servicio de audio se integra en la etapa 3.

/// Información de la aplicación que pueden reutilizar desktop y la futura CLI.
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
}

pub const fn app_info() -> AppInfo {
    AppInfo {
        name: "Rust DAW",
        version: env!("CARGO_PKG_VERSION"),
    }
}
