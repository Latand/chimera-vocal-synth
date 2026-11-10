# CHIMERA

A physical-modeling voice synthesizer: a Rust engine (wasm in an AudioWorklet, native for tests
and the CLI) under a React UI. See the README.

## Working here

- Do what was asked, the simplest way. Side work nobody asked for (experiments, tooling, paid
  runs) is a proposal with its cost. If the work grows well past the estimate, stop and say so.
- Before repeating or reversing earlier work, read its history (`git log -- <path>`, `git log -S`).
- Commit small. The subject names what changed and the trap, in searchable words; the body gives
  the why, what you ruled out, what you ran (with numbers) and what you didn't test.
- Keep each fixed bug's reproduction as a test.
- Code outranks this file: fix the file when they disagree. When something breaks unnoticed, add
  a rule here, with a check.

## Map

- `crates/chimera/`: the engine (no I/O). `crates/chimera-wasm/`: the browser's C ABI.
  `crates/chimera-cli/`: `chimera render | compare csd | diagnose | calibrate | tract3d`.
- `crates/chimera/src/anatomy/tract3d/`: the parametric 3D tract (lab mode; phase 2's acoustics).
  `chimera tract3d` compares it with MRI kept outside the repository (`tools/tract3d/`).
- `src/wasm/`: the UI's facade over the engine. `src/worklet/`: the AudioWorklet. `src/ui/`: React
  and the canvas renderers.
- `src/ui/render/head/`: the 3D head every human screen draws (the lab, each mode's face screen,
  the duet): the engine's tract meshes inside a skin mesh that follows the engine's jaw and lips.
- `tests/`: bun tests on the wasm build. `tools/`: e2e, layout lint, spectrogram, speech judge.
- `docs/`: architecture, the engine contract (`rust-engine.md`), the UI look, the singer judge,
  and the hill-climb rules.

## Rules

- **The engine's contract** (`docs/rust-engine.md`): transcendental math only through
  `chimera::math`, no allocation while rendering, output independent of block size. `cargo test`
  enforces each.
- **Any change to the sound or the plans fails the golden tests, by design.** Re-bless with
  `CHIMERA_BLESS=1 cargo test --workspace --no-fail-fast`, run twice. Human-voice audio, plans
  (including the beat sounds' gestures and the beat player), the tract geometry and the human
  airway's transfer functions re-bless; see `crates/chimera/tests/golden/README.md` for the rest.
- **`*.generated.json` tables are regenerated (`chimera calibrate`, `tools/gen-*`), never edited.**
  Vowel calibration isn't bit-reproducible, so rerun it only when the formants should move.
  `src/ui/render/head/head.generated.bin` is the same kind: `bun tools/gen-head.ts <makehuman
  checkout>` bakes it from CC0 assets kept outside the repository (`docs/tract3d/head-asset.md`).
- **`src/wasm/chimera.wasm` is a gitignored build product.** After a Rust change, `bun test tests`
  alone tests the old engine: use `bun run test`. Don't share a cargo `target/` between checkouts.
- **Gesture records are defined twice**, in `crates/chimera-wasm/src/lib.rs` and
  `src/wasm/wire.ts`. Change both and bump `LAYOUT_VERSION`.
- **The wasm loads before the UI.** A module that calls the engine at import must not be reachable
  from `src/main.tsx`'s static imports. Check: `bun tools/e2e.ts` against `bun run preview`.
- **DOM content flows in `.screen-body`, never absolutely positioned.** Check: `bun run lint:layout`.
- **Nothing copyrighted goes in git.** `public/shows/` is gitignored; never ship CSD audio or
  anything derived from it.
- **The 3D tract does not reach the voice yet**: every human screen draws it (`tract3dView`), the
  lab and the CLI measure it, and its changes leave the goldens alone. MRI data and comparison
  images stay outside git, and so do the reference's own measurements (areas, sizes, places):
  only this project's error numbers and pass lines, with the dataset's citation, go in
  `docs/tract3d/validation-results.md` (`chimera tract3d validate` writes its `table.md` that way).
  The tract reads `HumanFrame`'s outlines by position, so a change to how `human.rs` builds them
  must update `tract3d/parts.rs`. Check: `cargo test -p chimera --test tract3d`.
- **What a view draws of the tract comes from the helpers the laminae are cut from**
  (`tract3d/parts.rs`); a surface with a formula of its own in `mesh.rs` drifts from the cuts
  (the fossae did, by 3 mm). Check: the same test (`drawn_side_cavities_and_walls_are_the_sliced_geometry`).
- **The head's skin is display only and follows the engine**: its lips are fitted to
  `Tract3d::display`'s lip line and its jaw to the engine's turn (`src/ui/render/head/rig.ts`).
  Check: `bun test tests/head.test.ts`.
- **Voice tuning follows `docs/hill-climb/README.md`** and never edits its judges.

## Checks

| When you touch | Run |
|---|---|
| anything | `bun run test` and `bun run typecheck` |
| `crates/chimera/` sound | re-bless goldens (above); `cargo run --release -q -p chimera --example bench` against the previous commit |
| `crates/chimera-wasm/`, `src/wasm/`, `src/worklet/`, `src/main.tsx` | `bun run build && bun run preview`, then `bun tools/e2e.ts` |
| `src/ui/`, `src/show/`, CSS | `bun run lint:layout` |
| `crates/chimera/src/anatomy/tract3d/` | its tests (in `bun run test`); with the local data, `chimera tract3d validate` per `tools/tract3d/README.md`, and update `docs/tract3d/validation-results.md` |
| `src/ui/render/head/`, the 3D views | `bun test tests/head.test.ts`; `bun tools/e2e.ts` (the head drawn in each mode, the lab's voice); frames with `bun tools/tract3d/lab-capture.ts`, looked at |
