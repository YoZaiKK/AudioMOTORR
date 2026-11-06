import { isTauri } from "@tauri-apps/api/core";

export function App() {
  return (
    <main className="workspace">
      <header>
        <span className="eyebrow">AudioMOTORR · Etapa 2</span>
        <h1>Rust DAW</h1>
        <p>
          Motor de silencio y transporte verificados sin dispositivo de audio.
        </p>
      </header>
      <section aria-labelledby="status-title" className="status-panel">
        <h2 id="status-title">Estado de desarrollo</h2>
        <dl>
          <div>
            <dt>Interfaz</dt>
            <dd>React + TypeScript + Vite</dd>
          </div>
          <div>
            <dt>Entorno</dt>
            <dd>{isTauri() ? "Aplicación Tauri" : "Navegador"}</dd>
          </div>
          <div>
            <dt>Audio</dt>
            <dd>Pendiente de implementar</dd>
          </div>
        </dl>
        {!isTauri() && (
          <p className="notice">Vista de interfaz: requiere Tauri para audio</p>
        )}
        <p className="muted">
          El siguiente paso es conectar el motor al servicio de audio.
        </p>
      </section>
    </main>
  );
}
