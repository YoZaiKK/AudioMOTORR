# Preparar el entorno

Entorno comprobado: Windows nativo, MSVC x64, Windows SDK y WebView2 Runtime.
Instalar esos prerrequisitos antes de compilar desktop. El repositorio no los
instala ni modifica silenciosamente.

Versiones fijadas: Rust 1.90.0, Node 22.23.4 y npm 10.9.9. Consultar
rust-toolchain.toml, .node-version y package.json si se actualizan posteriormente.

```powershell
Set-Location C:\proyectos\rust\MOTORUST
rustup toolchain install 1.90.0 --profile minimal --component rustfmt --component clippy
npm ci
npm run doctor
```

Cambiar la ruta si el repositorio está en otro directorio. `npm ci` usa el lockfile
y reinstala node_modules. `doctor` solo inspecciona herramientas y prerrequisitos.
Esperar diagnósticos correctos; atender cualquier MISSING o UNKNOWN antes del build.

Para trabajar solo con el núcleo, Rust basta: no se necesitan ventana, WebView2 ni
dispositivo de audio. La [guía de pruebas](04-pruebas.md) proporciona los comandos
headless; `npm run check` también comprueba desktop y necesita sus prerrequisitos.

El proyecto tiene un Cargo.lock y un package-lock.json en la raíz. No crear
workspaces Cargo anidados ni cambiar el gestor npm.

La [guía técnica de desarrollo](../docs/development.md) distingue build desktop,
mock runtime y hardware, y documenta la secuencia de CI. Windows tiene evidencia
local. La base 1ef9bd0 pasó también Ubuntu headless y Windows desktop en
[Actions](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42314334447).
Linux nativo/macOS siguen sin comprobación; los cambios locales posteriores
requieren su propia verificación.
