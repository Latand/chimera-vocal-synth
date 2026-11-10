# tract3d: a parametric 3D vocal tract, checked against MRI

*Phase 1 research and design, 2026-11-09, base revision 2e567e2.*

**Status (phase 1 built).** The model is `crates/chimera/src/anatomy/tract3d/` (what differs from
the design: [model-design.md §0](model-design.md#0-as-built-phase-1)), the lab is `?mode=lab`, the
comparison with MRI is `chimera tract3d` ([tools/tract3d](../../tools/tract3d/README.md)), and its
numbers and critique are in [validation-results.md](validation-results.md).

**Operator amendment (2026-11-09).** The lab's 3D view now sits inside a head with skin,
subcutaneous fat, muscle, bones and cartilages, cut open with its layers' cut faces; the
uvula moves in the voice as part of the soft palate's vibrating membrane; the engine's
low-frequency acoustics were checked against DVTD's measured 3D-print transfer functions and
corrected; the walls of the voice's tract vibrate by tissue. What changed and the numbers:
[model-design.md §0.1](model-design.md#01-operator-amendment-2026-11-09-head-uvula-low-frequencies-walls),
[validation-results.md §7–8](validation-results.md#7-low-frequencies-the-engines-acoustics-against-dvtds-measured-prints).

**Integration (2026-11-10).** The 3D tract is now what every human mode shows: Play, Speak,
Sing, Beatbox and the duet's heads draw the engine's 3D tract inside a head whose skin is a
mesh baked from an openly licensed (CC0) base mesh, moved by the live voice's jaw and lips; the
lab is audible (Listen, Say, a start prompt); the review's findings on the geometry and the
fits are answered, with what is still outside a pass line said in the results (four rounded
vowels' F1, the male /y/ the furthest at +17 %). What changed: [model-design.md §0.2](model-design.md#02-integration-2026-11-10-one-head-and-tract-on-every-screen);
the head's source and licence: [head-asset.md](head-asset.md); the numbers:
[validation-results.md](validation-results.md). **The voice does not read the 3D tract yet**:
by the operator's order of work the shapes came first, and the port that computes every human
sound from the 3D geometry is the next lane (below, deferred).

## The requirement

The operator's request (2026-11-09, in Ukrainian; the verbatim text is kept in the task record),
restated: study how the synthesizer works and set up a more faithful simulation of the timbre and
sound it produces; meanwhile build the most detailed, realistic and parametrically controllable
throat, mouth and the rest; make the tongue far more flexible, able to form the various funnels
and grooves real tongues make; research this in detail, build the 3D parts separately and show
what they look like; check and critique the 3D cross-sections against real human data where such
data exists, because that is how the sound gets as realistic and rich as possible; and make it
possible to see the shape and hear the result.

Operator decisions the same day: code in the project's public fork; **this lane is phase 1**
(research, parametric 3D elements, cross-sections checked against MRI, shown to the operator);
phase 2 computes sound from the geometry (real-time areas into the existing FDTD plus an offline 3D
wave solver as reference); a parametric tongue surface with many controls along its length (no
biomechanical FEM); MRI datasets only locally, git only published numbers with citations; a separate "lab" mode
in the same app.

## Documents

| File | What |
|---|---|
| [research.md](research.md) | anatomy and literature per part of the tract; what the engine does today; observations made on the downloaded data |
| [reference-data.md](reference-data.md) | every dataset opened: URL, licence, access, formats, sounds, planes, git-usable numbers, what was downloaded |
| [model-design.md](model-design.md) | the elements and their parameters, mapping from today's articulators, cuts and area function, API, lab mode, phase 2 outline |
| [validation-plan.md](validation-plan.md) | reference pipeline, metrics, tests and pass lines with their justification, evidence, critique |
| [validation-results.md](validation-results.md) | the measured numbers against DVTD and USC, the feature checks, cost, and the critique |
| [head-asset.md](head-asset.md) | the head's skin mesh: source, licence (CC0), what the generator does |

## Validation against the requirement

| Requirement | Where it is met |
|---|---|
| detailed, realistic, parametric pharynx, mouth and the rest, as separate 3D parts | design §4: tongue, palate, velum, pharynx walls, larynx with epilarynx, epiglottis, piriform fossae and valleculae, lips, teeth and jaw, cheeks; each with its own mesh and parameters |
| a far more flexible tongue: funnels, grooves | design §4.1: five stations of side elevation and groove depth, groove width and taper, asymmetry, tip width; presets for grooved, funnel, cupped, lateral, bunched and retroflex /ɹ/, flat |
| researched in detail | research.md, reference-data.md: 27 sources, 6 datasets opened, 2 downloaded and measured |
| check and critique of 3D cross-sections against real people | validation-plan.md: DVTD (2 speakers, 22 sounds) and USC (/ɹ/, 8 speakers); same pipeline both sides; pass lines from the data; written critique |
| see the shape and hear the result | design §9 and §0.2: the 3D head and tract on every human mode's face screen; Lab mode (3D view, cuts, area function, parameters, Listen and Say for the posture) |
| more faithful timbre, richest sound | phase 2 (design §10); phase 1 makes area and shape along the centreline computable from the 3D geometry and shows how it differs from what the engine uses today |

## Build plan (one lane)

Each step ends with its tests green and is committed on its own. Steps 1–6 are the minimum that
meets every acceptance item; step 7 is polish within the same lane if time remains.

0. **Toolchain and baseline.** Rust through user-local rustup (`--no-modify-path`, stable +
   `wasm32-unknown-unknown`), `bun install`; run `bun run test` and `bun run typecheck` on the
   base and keep the logs as the baseline.
1. **Engine module, geometry.** `chimera::anatomy::tract3d`: parameter specs, laminae from
   `HumanFrame` plus element profiles (tongue, palate, velum, pharynx, larynx with epiglottis,
   piriform fossae and valleculae, lips, teeth and jaw, cheeks), `derive()` with starting
   constants. Tests: the z = 0 lamina equals the 2D outlines; mirror symmetry at symmetric
   parameters; parameter extremes give valid polygons; side cavities absent at z = 0 where the
   design says so.
2. **Cuts.** Centreline, perpendicular cuts, coronal/axial/midsagittal planes, band rule,
   components and tags, area function, cavity descriptors, `air()`. Tests: a synthetic tube cuts
   to πr² within 1 %; /l/ preset gives two channels at the contact; three regions in the lower
   pharynx; continuity sweep; no allocation in `compute` after warm-up; the cost target measured.
3. **Meshes and presets.** Fixed-count element meshes; the preset table of design §4.1.
4. **Browser API.** `tract3dSpecs` and `tract3d` JSON calls, TS types and wrappers; measured call
   time.
5. **Validation tooling and run.** `chimera tract3d reference|model|validate|usc`; the port checked
   against the probe's 44 outputs; anatomy fits from the dental casts; T-consistency, T-fit,
   T-default (fitting the `derive()` constants), T-features, T-anatomy, T-/ɹ/; evidence images
   and tables; `validation-results.md` with the critique.
6. **Lab mode.** Mode and `?mode=lab`; TRACT, CUT, AREA, SHAPE and SOUND screens as in design §9;
   follow-live from telemetry; layout lint and e2e cover the new mode.
7. **Docs and checks.** AGENTS.md map and checks table gain the tract3d module and the
   validation command; `bun run test`, `bun run typecheck`, `bun run lint:layout`,
   `bun run build && bun run preview` with `bun tools/e2e.ts`; goldens untouched (no
   `CHIMERA_BLESS`).

If the lane runs long, trim the end of each step and keep every step: fewer presets,
fewer USC speakers (keep at least one bunched and one retroflex), the full-22 identification test
before the headline set.

## Deferred — not currently justified

- **Sound from the 3D geometry, the gesture records, the planner:** the next lane, by the
  operator's order of work (2026-11-09: the 3D shapes first, then the port in which every human
  sound is computed from the 3D geometry; design §10). Until then the voice uses its 1D area
  law with the amendment's walls, soft palate, losses and inertances, and the lab's AREA screen
  shows the difference between the two area functions. With it come the vowel and consonant
  recalibration, the golden re-bless and the audio-thread benchmark of that change.
- **The offline 3D wave solver and "precise render":** phase 2; phase 1 leaves `air()` and the
  voxelizer as hooks.
- **The nasal cavity and sinuses in 3D:** not in the acceptance list; the 1D nasal branch stays.
- **Biomechanical (FEM) tongue:** excluded by the operator.
- **Individual tooth crowns beyond the arch and incisors:** interdental spaces have very small
  acoustic effect [research R10]; arch-level teeth plus the incisors suffice.
- **Head-posture (supine versus upright) correction:** the critique reports it; the model leaves it out.
- **The French static 3D set and its segmentations, the Aalto set, the gated USC 17-speaker set:**
  overlap with DVTD/USC, licence or access (reference-data.md).
- **Segmenting the USC volumes for area functions:** teeth read as air; DVTD covers the
  quantitative part.
- **Published area-function tables (1991–2008):** not obtainable here; add by hand later with
  table and page citations if wanted.
- **Child anatomy beyond body scaling:** no child 3D reference was found.
- **From the operator amendment (2026-11-09):** walls from the 3D geometry's own perimeter and
  tissue per cut (the voice reads regions of the 2D frame); the uvula's flow-driven vibration
  (a uvular trill or snore, a source like the vocal folds). Both belong with the sound's port.
- **The head's look beyond the mesh:** a skin texture (pores, brows, lashes), eyes that open,
  hair; the single crowns' cusps and the gums are drawn, not scanned.

## Notes

- Copies of older branches in the fork (`t3/tackle-issue-4`, `t3/layout-lint`,
  `t3/good-evening-demo`, …) touch `api.rs`, `human.rs`, `App.tsx`, `store.ts` and `engine.ts`;
  they are pre-squash versions of work already on `main` and have no open pull requests, so no
  live change conflicts with this plan.
- The probe scripts, raw pages and downloaded data are in the local data folder
  (`$TRACT3D_DATA`, outside the repository) with a README of sources, licences and checksums.
