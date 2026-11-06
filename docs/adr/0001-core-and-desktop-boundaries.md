# ADR 0001: Core, audio, and desktop boundaries

Status: accepted; records implemented decisions as of 2026-11-06.

## Context

The initial target is a four-channel, 16-step groovebox that can later evolve
into a clip-based DAW. The current engine produces silence. Its transport and
resource contracts must be verifiable without a window or audio device, while
desktop must exercise the same application and engine.

## Decisions and consequences

| Decision                                                   | Reason and consequence                                                                                                                                                                                                                                         |
| ---------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Independent domain and engine                              | Device-free rendering uses caller-owned interleaved `f32` buffers. Both crates have no external dependencies; UI and device concerns stay outside the engine.                                                                                                  |
| Explicit manual backend                                    | The same realtime processor runs only when asked to render. Deterministic tests and CLI smoke need no timer or hardware; this does not prove native callback timing.                                                                                           |
| Optional CPAL 0.18.2                                       | `native-audio` is opt-in for audio/app/CLI and enabled by desktop. Headless builds omit CPAL and Tauri; disabled native APIs return an explicit error.                                                                                                         |
| One control thread owns the stream                         | Enumeration, construction, play, and destruction occur on that thread. Cloneable handles submit bounded requests; the callback owns its engine. Driver blocking stays outside rendering but can delay initialization/shutdown.                                 |
| Bounded rtrb 0.4.1 SPSC queues                             | Only Copy commands/snapshots cross the realtime boundary. Commands reject overflow, callbacks consume at most 32 per block, and telemetry may skip publication. Additional callers serialize through the controller rather than becoming extra SPSC producers. |
| JSON counters and IDs are strings                          | Rust `u64` values can exceed JavaScript's exact integer range. Decimal strings preserve precision; React compares sequences/acknowledgements using `BigInt`.                                                                                                   |
| Generate DTOs with ts-rs 12.0.1                            | Rust Serde DTOs define the wire contract. A checked-in TypeScript file is generated and checked deterministically, avoiding manually duplicated definitions. Generation requires desktop build prerequisites.                                                  |
| Distinguish document, accepted commands, and applied state | A document edit after acceptance does not assert playback changed. Snapshots confirm application; shutdown discards pending commands but preserves document tempo for restart.                                                                                 |

These are concrete boundaries in the current implementation. No generic future
graph, plugin interface, or placeholder instrument layer is introduced.

## Validation and limits

Engine, service, application, CLI, React, and mock-runtime IPC tests cover these
contracts. Windows native smoke evidence is dated in the
[stage 5 report](../../howToRunThis/resultados.md). Inspection of the callback
supports the no-allocation/no-blocking rule; allocator instrumentation, release
deadline measurements, and physical unplug/replug remain pending.

The [architecture](../architecture.md), [realtime contract](../realtime-contract.md),
and [development guide](../development.md) describe the resulting implementation.
