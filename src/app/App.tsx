import { isTauri } from "@tauri-apps/api/core";

export function App() {
  return (
    <main className="workspace">
      <header>
        <span className="eyebrow">AudioMOTORR · Etapa 3</span>
        <h1>Rust DAW</h1>
        <p>
          Servicio de audio disponible mediante la CLI. Interfaz pendiente de
          conectar.
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
            <dd>Backends manual y nativo; controles pendientes</dd>
          </div>
        </dl>
        {!isTauri() && (
          <p className="notice">Vista de interfaz: requiere Tauri para audio</p>
        )}
        <p className="muted">
          El siguiente paso es conectar la interfaz al servicio de audio.
        </p>
      </section>
    </main>
  );
}
