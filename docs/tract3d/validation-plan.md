# Validation plan: model cuts against real vocal tracts

*Phase 1 design, 2026-11-09.* How the 3D model of [model-design.md](model-design.md) is compared
with MRI. Data and licences: [reference-data.md](reference-data.md). Source keys `[Rn]`:
[research.md](research.md).

## 1. Principles

1. **Measure both sides with the same code.** The MRI meshes and the model go through one
   reference pipeline (voxels → centreline → perpendicular cuts → band rule). The 1D resonances of
   both area functions come from the same solver. Observed reason: the 1D reduction of the DVTD
   meshes already sits +6.5 to +8.6 % (median) above the resonances measured on 3D prints of the
   same shapes, so only like-for-like comparisons are meaningful.
2. **Thresholds come from the data or a cited number.** Where neither exists,
   the result is reported without a pass line and the critique says so.
3. **A generic model is not one speaker.** In DVTD the same sound differs between the two speakers
   more than two different sounds differ within one speaker (§4). So there are two kinds of test:
   *can the model reproduce this speaker's shape* (fitted) and *do today's gestures produce a
   realistic shape* (default).
4. **Data stays local.** The pipeline reads `$TRACT3D_DATA`; reference area functions and images
   stay outside git. The repository gets the code, the error tables and the critique.

## 2. What is compared

| Set | Data | Sounds | Why these |
|---|---|---|---|
| Headline | DVTD, both speakers [R4] | /aː iː uː eː oː ə/ and /s ʃ ç x f l/ | corners and centre of the vowel space; every consonant DVTD has: groove (/s/), domed channel (/ʃ/), palatal and velar constrictions, labiodental, lateral |
| Full fit | DVTD, both speakers | all 22 sounds | the nearest-neighbour identification test needs every sound |
| /ɹ/ | USC 75-speaker [R22], 8 native-English speakers' sustained /ɹ/ | /ɹ/ | DVTD has no rhotic; bunched and retroflex variants both occur in this group |
| English check | USC, one male and one female speaker | /l s ʃ i ɑ u/ | the same shapes in English speakers, side by side with DVTD |
| Published numbers | [R12, R19, R23] | piriform fossae; repeat variation; formant discrimination | anatomy checks and thresholds |

## 3. The reference pipeline

A Rust port, inside the CLI (`chimera tract3d …`, §7), of the probe used in this research
(`$TRACT3D_DATA/_scripts/dvtd_slices.py`):

1. Read the closed inner-surface STL; voxelize at 0.5 mm by ray parity along one axis; fill holes.
2. Find the two openings of the open STL (boundary loops): the lower and smaller is the glottis,
   the other the lips.
3. Centreline: shortest path through the lumen voxels (2× coarser grid, 26-neighbour, edge cost
   length / (mean distance-to-wall + 0.2 mm)²), anchored on the openings, smoothed (30 passes of
   ¼-½-¼), **pinned to the midsagittal plane** (lateral coordinate set to the mean of the two
   openings'; without it the path slides into one /l/ channel), resampled every 2.5 mm.
4. Cut perpendicular to the centreline (0.25 mm in-plane sampling, ±40 mm), orient "up" away from
   the inside of the bend, keep the air regions entering the band ±6 mm across × ±30 mm sideways.
5. Per cut: area, width, height, channel count, mask; per sound: length, volume.

**Regression oracle for the port.** The probe's JSON for all 44 DVTD meshes is kept in
`$TRACT3D_DATA/dvtd/derived/`. The port must reproduce it: median area difference ≤ 3 % on cuts
above 0.3 cm², equal channel counts on ≥ 95 % of cuts, lengths within one step (2.5 mm). It may
then improve on the probe, with the change and its effect recorded.

**Model through the same pipeline.** The model is voxelized with `Tract3d::air` at 0.5 mm and
measured by steps 3–5, with its own glottis and lip-opening points.

## 4. Numbers that set the thresholds

Measured here on DVTD with `af_stats.py` and `af_formants.py`. Distance between two area
functions: **d = RMS over 101 points of normalised length of log2(A₁ / A₂)**, areas floored at
0.05 cm² (d = 1 means "off by a factor of two on average").

| Quantity | Male | Female | Use |
|---|---|---|---|
| Nearest *other* sound, same speaker: smallest | 0.31 (/ɛː/–/ɛ/) | 0.35 (/ɛ/–/œ/) | a fit closer than this cannot be mistaken for another sound |
| Nearest other sound: median | 0.59 | 0.59 | |
| Same sound across the two speakers, size-normalised: median (range) | 0.71 (0.43–1.45) | | how far apart two real people are |
| Same, raw | median 0.99 | | |
| 1D resonances of the cut area function vs measured 3D-print peaks | median +6.7 % F1, +6.5 % F2, +8.6 % F3 (n = 44) | | bias of the 1D reduction itself |

Published: repeated 3D MRI of the same task varies by about 8 % [R19] (≈ 0.11 in d units);
formant discrimination is about 14 Hz below 800 Hz and 1.5 % in the F2 region for trained
listeners [R23]; piriform fossae 2.1–2.9 cm³ and 1.6–2.0 cm deep [R12].

## 5. Tests and pass lines

### T-consistency: the fast cut measures what the reference measures

The model's own cuts (§6.2 of the design) against the reference pipeline run on the model's
voxels, for all presets and the headline sounds' fitted postures.
- Area: median |difference| ≤ 3 % on cuts above 0.3 cm²; channel count equal on ≥ 95 % of cuts.
- Resonances of the two area functions (same 1D solver): F1 within 14 Hz, F2 and F3 within 1.5 %
  [R23]. *Why:* both are measurements of one geometry, so their difference must stay below what a
  listener could hear.

### T-fit: can the model reproduce real shapes

Per DVTD speaker:
1. **Anatomy once:** body preset (male for speaker 1, female for speaker 2) with scale, pharynx
   factor and the palate/arch parameters fitted to that speaker's dental-cast scans and /ə/.
2. **Articulation per sound:** the 17 articulators and the 3D parameters fitted to that sound by
   Nelder–Mead (`chimera::util::optimize`) minimising d plus a cut-shape term (§6, M3).

Pass lines:
- d ≤ 0.30 median over the 22 sounds, ≤ 0.45 for every sound. *Why:* below the smallest distance
  between two different sounds of one speaker (0.31 / 0.35); 0.45 still sits under the median
  nearest-other distance (0.59).
- Identification: for ≥ 20 of 22 sounds, the reference sound nearest to the fitted model (by d)
  is the target sound.
- Constriction: channel count equal at the minimum-area cut for /l/ (2), /s/, /ʃ/, /ç/ (1);
  location within 1 cm along s.
- Lower pharynx: where a reference cut 0.5–3 cm above the glottis splits into separate regions
  (epilarynx plus fossae: three regions in most of the female speaker's sounds, two in most of
  the male's; not in every sound, e.g. the male /l/), the model's matching cut has the same
  count ±1.
- Resonances of fitted vs reference area function (same 1D solver): median |ΔF1–F3| ≤ 5 %, every
  value ≤ 10 %. *Why:* about three times the F2 discrimination threshold [R23], and inside the
  ~8 % repeat variation of 3D MRI [R19].

### T-default: do today's gestures make realistic shapes

No articulation fitting: the 17 articulators are the engine's calibrated targets for that sound
(`vowelTargets.generated.json`, consonant inventory), the 3D parameters come from `derive()`, the
anatomy from T-fit. German vowels map to the engine's nearest symbols (/iː/→i, /eː/→e, /ɛː/→ɛ,
/aː/→a, /oː/→o, /uː/→u, /yː/→y, /øː/→ø, /ə/→ə, …), listed in the results.
- Per sound: d ≤ max(1.0, d between the two DVTD speakers for that sound); median ≤ 0.71.
  *Why:* a default model should be as close to each speaker as the two speakers are to each
  other: "a third speaker".
- The `derive()` constants (§5 of the design) are fitted here, once, minimising the median
  default d over both speakers; the fitted values and the run are recorded.
- Resonance differences are reported without a pass line: the engine's targets were calibrated
  to formants through α·d^β; the gap to this geometry is phase 2's to close.

### T-features: the tongue shapes the research lists

Presets (design §4.1), checked by geometry:
- *grooved* /s/: single channel ≤ 0.2 cm² within 0.85–0.95 of the length, lower outline concave
  (groove index > 0, §6 M4). The line encloses both speakers' /s/.
- *domed* /ʃ/: minimum ≤ 0.15 cm² further back than /s/ (as in both speakers), with a long thin
  channel behind it.
- *lateral* /l/: two channels at the tip contact, 2–2.8 cm apart (a line that encloses both
  speakers' /l/), their area ratio
  following `sideAsym`.
- *retroflex* /ɹ/: midsagittal air under the tip (sublingual), palatal constriction.
- *bunched* /ɹ/: two constrictions (palatal and pharyngeal) and a large front cavity [R2].
- *flat*: no concavity anywhere along the tongue.
- *funnel*: groove width decreasing towards the tip with depth increasing.
Each is a yes/no check plus the measured value.

### T-anatomy: side cavities

Piriform fossa volume 2.1–2.9 cm³ each and depth 1.6–2.0 cm for the male template [R12] (female
and child scaled with the body); valleculae present off the midline and absent at z = 0.

### T-/ɹ/: English rhotics (USC)

For each of the 8 speakers: midsagittal plane plus coronal planes at two depths behind the
incisors, MRI next to the closer preset (bunched or retroflex) driven to that speaker's tip and
dorsum positions measured on the midsagittal image. Recorded per speaker: tip raised, sublingual
pocket, palatal-constriction position (cm behind the incisors), pharyngeal narrowing, coronal
tongue outline (concave / flat / convex). Pass: every feature of the preset matches the speakers
assigned to it. No area threshold: the volumes are not segmented (teeth read as air).

### T-continuity and cost

The 3D area function under articulator and parameter sweeps (like `tests/geometry_continuity.rs`)
has no step far above its neighbours; `compute` stays within the design's cost target, measured
with the load recorded.

## 6. Metrics

- **M1 area function:** d (raw and size-normalised), length error ΔL/L, volume error.
- **M2 constriction:** position of the minimum (s/L, search s ≥ 0.5 cm), log2 of the minimum-area
  ratio, channel count there.
- **M3 cut shape** at s/L = 0.15, 0.3, 0.45, 0.6, 0.75, 0.9 and at the constriction:
  intersection-over-union of the two cuts after scaling to equal area and aligning centroids;
  |log2| of the width/height ratio difference. Pass lines for M3 are set from the same measure
  between the two DVTD speakers for the same sound (computed in the first run and recorded):
  the fitted model must be at least as close to a speaker as the other speaker is.
- **M4 groove index:** on oral cuts, depth of the lower outline's midline dip below the higher of
  its two lateral maxima (cm); negative for a convex (domed) tongue.
- **M5 resonances:** F1–F3 of the area function by the engine's chain-matrix code
  (`chimera::airway::frequency`) on a straight tube network built from the area function.
- **M6 side cavities:** volume and depth per cavity from the model's descriptors.

## 7. Tooling

- `chimera tract3d reference <sound-dir> <out>`: the reference pipeline on one DVTD sound.
- `chimera tract3d model <preset|artic.json> <out> [--body male|female] [--anatomy fit.json]`.
- `chimera tract3d validate <dvtd-root> <out>`: all tests above that use DVTD; writes
  `table.json`, `table.md`, images.
- `chimera tract3d usc <volume.mat> <out> --planes …`: planes of a USC volume (MATLAB v5 reader
  for the one complex array) for the /ɹ/ and English comparisons.
- Images as PNG (the CLI already depends on `flate2`).
- The data root is an argument or `$TRACT3D_DATA`; nothing under it is read at build or test time.
  `cargo test` must pass on a machine without the data.

## 8. Evidence

Written to the stage's evidence directory, numbered:

- per speaker and sound: area functions overlaid (reference, fitted model, default model) with
  the midsagittal outlines side by side; one row of reference cuts above one row of model cuts at
  the M3 stations, same scale, area and channel count printed under each;
- the presets' cuts and 3D views (feature checks);
- USC /ɹ/ side-by-sides;
- `table.md` / `table.json` with every metric and pass line, and the revision and command that
  made them.

In git: `docs/tract3d/validation-results.md` with the tables (this project's numbers), the
critique, and the dataset citations. No reference areas or images.

## 9. The critique

For every sound that fails a line, and for the worst three that pass, say where along the tract
the model departs (s range, which cuts), how (too wide, wrong channel count, wrong shape), and
why, in one of these classes:

- *anatomy*: the speaker's palate, pharynx width or arch differs from the fitted preset;
- *missing freedom*: no parameter can make that shape (name the shape);
- *derivation*: `derive()` gives the wrong default for the articulators;
- *reference artefact*: smoothing over a constriction (the DVTD authors name it for /f/ [R4]),
  teeth, posture in the scanner [R21], the cut method at a bend;
- *speaker variation*: within the two-speaker spread of §4.

Known limits of the references to state in the critique: one male and one female German speaker;
sustained for 14 s in the scanner; smoothed segmentations; the nasal cavity excluded; the 1D reduction's
own bias (§4).
