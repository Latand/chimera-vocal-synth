# Validation results

*Measured 2026-11-10 on the model as integrated (model-design.md §0.2), by
`chimera tract3d validate` with 3000 fit iterations per sound and 600 for each speaker's
anatomy (281 s on the shared machine, three jobs). How the comparison works:
[validation-plan.md](validation-plan.md); commands: [tools/tract3d](../../tools/tract3d/README.md).
Sections 2–8 keep their own dates.*

**In short.** The parametric model reproduces the 44 MRI vocal tracts of the
[Dresden Vocal Tract Dataset](https://doi.org/10.1038/s41597-020-00597-w) (DVTD; two German
speakers, 22 sounds each) with a median area-function distance d of 0.453 (male) and 0.447
(female), below the distance between the two speakers themselves (median 0.73), and with the
first three resonances of the fitted area functions a median 2.2 % and 1.9 % from the MRI's
(same 1D solver both sides). Every one of the 44 fits is measured by the voxel pipeline, and
the fast slicer agrees with the model's own voxels (area within 1.2 % on every sound, channel
counts equal on a median 99 % and 98 % of cuts). Of the plan's pass lines for a fit, the median
resonance error (≤ 5 %) and the male identification (20 of 22) now hold; the area-function
lines (median d ≤ 0.30, every sound ≤ 0.45), the female identification (16 of 22) and "every
resonance within 10 %" (5 male and 4 female sounds are outside it) are **not met**; the
reasons are below. The default shapes (the engine's own targets with `derive`) remain far from
MRI (median d 0.868 / 0.801); closing that is the work of the sound's port, on the targets.

d is the RMS over 101 points of normalised length of log2(A_model / A_MRI), areas floored at
0.05 cm² (d = 1 means a factor of two on average). Every number below is this project's
measurement against the cited datasets; the datasets' own values (areas, sizes, places) are
not written here: they stay with the data, outside the repository.

## 1. DVTD: area functions, cuts and resonances

The same pipeline measures MRI and model: 0.5 mm voxels, a centreline through the lumen pinned to
the midsagittal plane, cuts every 2.5 mm perpendicular to it, and the air regions within ±6 mm
across and ±30 mm sideways; a region under 0.01 cm² (less than one MRI voxel's face) is neither
area nor a channel, as in the fast slicer. The port of the research probe reproduces the probe's
output on all 44 meshes (worst per-mesh median area difference 1.71 %, lowest channel agreement
95.1 %, largest length difference one step, 0.25 cm; plan lines 3 %, 95 %, 0.25 cm).

Each speaker's anatomy (pharynx widths and corner shape, dental arches, palate vault, laryngeal
tube, piriform fossae, tongue width) is fitted once on /ə aː iː uː s l/; then each sound's 13
articulators, 11 tongue parameters and the lips' rounding are fitted. The objective is the
area-function distance, the tract's length, the cuts' proportions at six stations, a penalty for
openings thinner than 1.5 mm (the reference pipeline traces its centreline on a 1 mm grid; the
MRI voxels are 1.2 mm), and the distance of F1–F3 from the reference's (weight 0.8 per unit of
log ratio: a 10 % error of one resonance costs as much as 0.08 of d). A fit whose voxels have
no connected path from glottis to lips is backed off towards its starting posture until they
have one; in this run no fit needed it.

| Test (plan §5) | Line | Male | Female | Result |
|---|---|---:|---:|---|
| Fitted d, median of 22 | ≤ 0.30 | 0.453 | 0.447 | fail |
| Fitted d, worst sound | ≤ 0.45 | 0.903 | 1.158 | fail |
| Sounds with fitted d ≤ 0.45 | all 22 | 10/22 | 11/22 | fail |
| Measured by the voxel pipeline | 22 | 22/22 | 22/22 | pass |
| Identification (nearest MRI sound is the target) | ≥ 20/22 | 20/22 | 16/22 | male pass, female fail |
| F1–F3 of fitted vs MRI area function, median absolute error | ≤ 5 % | 2.2 % | 1.9 % | pass |
| F1–F3, every value | ≤ 10 % | 50 % | 61 % | fail (5 and 4 sounds outside) |
| Default d (engine targets + derive), median | ≤ 0.71 | 0.868 | 0.801 | fail |
| Default d per sound ≤ max(1.0, between-speaker d) | all 22 | 16/22 | 16/22 | fail |
| Fast slicer vs the model's voxels, median area difference (T-consistency) | ≤ 3 % | 0.7 % | 0.8 % | pass |
| … worst sound | ≤ 3 % | 0.9 % | 1.2 % | pass |
| … channel count equal, median over sounds (lowest sound) | ≥ 95 % | 99 % (93 %) | 98 % (93 %) | pass |

- **male**: fitted length error median 2.0 %, volume error median 12.3 %; F1/F2/F3 median
  absolute error 2.3 / 2.7 / 1.8 %; lower pharynx (0.5–3 cm above the glottis), where MRI splits
  into ≥ 2 regions the model's count is within ±1 on 35/47 cuts; cut IoU median 0.66 (between the
  two speakers 0.71), aspect error (absolute log2 ratio) median 0.47 (between speakers 0.43);
  headline set median fitted d 0.443, sounds with fitted d ≤ 0.30: 0.
- **female**: fitted length error median 0.0 %, volume error median 5.7 %; F1/F2/F3 median
  absolute error 2.0 / 1.4 / 2.1 %; lower pharynx within ±1 on 45/93 cuts; cut IoU median 0.71,
  aspect error median 0.36; headline set median fitted d 0.514, sounds with fitted d ≤ 0.30: 2.

### What changed since the reviewed head, and what it bought

The review of the previous head (commit e8687b1, the same command) found: one male fit (/ʃ/)
without a connected voxel path (21 of 22 measured), the female channel agreement at 92 % against
the 95 % line, identification 18 and 16 of 22 where 18 and 18 were published, and the fitted
rounded vowels' F1 a third too high. Three runs of the same command tell the changes apart
(the evidence folder keeps all three tables):

| | Reviewed head | + perimeter, region rule, connected fits, `lipRound` | + the resonance term (this document) |
|---|---|---|---|
| Measured, male / female | 21 / 22 | 22 / 22 | 22 / 22 |
| Fitted d, median | 0.420 / 0.438 | 0.407 / 0.424 | 0.453 / 0.447 |
| Identified | 18 / 16 | 19 / 18 | 20 / 16 |
| F1 error, median absolute | 9.9 / 9.1 % | 8.3 / 6.3 % | 2.3 / 2.0 % |
| F1–F3 error, median absolute | 6.3 / 4.1 % | 5.5 / 4.2 % | 2.2 / 1.9 % |
| Sounds with a resonance over 10 % off | 10 / 10 | 11 / 8 | 5 / 4 |
| Channel agreement, median | 96 / 92 % | 97 / 98 % | 99 / 98 % |

- **Channel agreement** rose because the two pipelines now use one rule: the voxel pipeline
  counted every speck that touched the band as a channel, the slicer dropped regions under
  0.01 cm². (The perimeter's fix changes the limit on thin openings only for round shapes, which
  were never near it.)
- **`lipRound`** lets a fit narrow the lips' opening below the engine's 1.1 cm floor. Alone it
  brought the male /u/ from d 0.891 to 0.553, and its F1 from +33 % to +24 %.
- **The resonance term.** The area distance weighs every centimetre of the tract alike, while
  F1 of a rounded vowel hangs on the lips' few millimetres and the cavity behind them. With the
  term, the rounded vowels' F1 comes to within the 10 % line for 12 of the 16 cases below, at a
  cost in d (median +0.05 / +0.02) and in the female identification (18 → 16): the fit trades
  shape for resonance where it cannot have both. Ruled out: half the weight (F1 5.6 / 4.0 %,
  male /u/ still +18 %) and bringing the term in round by round (male /u/ +25 %).

| Case | IPA | F1 error: reviewed head | with `lipRound` | with `lipRound` and the resonance term | Fitted d: reviewed head → now |
|---|---|---:|---:|---:|---|
| s1-04-boote-tense-o | /o/ | +14 % | +19 % | +9 % | 0.470 → 0.504 |
| s1-05-bude-tense-u | /u/ | +33 % | +24 % | +2 % | 0.891 → 0.784 |
| s1-07-hoehle-tense-oe | /ø/ | +23 % | +18 % | +7 % | 0.450 → 0.514 |
| s1-08-guete-tense-y | /y/ | +35 % | +33 % | +17 % | 0.670 → 0.738 |
| s1-18-offen-lax-o | /ɔ/ | +6 % | +6 % | +1 % | 0.331 → 0.378 |
| s1-19-butter-lax-u | /ʊ/ | +28 % | +19 % | +14 % | 0.580 → 0.631 |
| s1-20-muetter-lax-y | /ʏ/ | +10 % | +8 % | +3 % | 0.347 → 0.376 |
| s1-21-goetter-lax-oe | /œ/ | +7 % | +4 % | +3 % | 0.333 → 0.415 |
| s2-04-boote-tense-o | /o/ | +13 % | +14 % | +1 % | 0.578 → 0.569 |
| s2-05-bude-tense-u | /u/ | +14 % | +15 % | +0 % | 0.443 → 0.427 |
| s2-07-hoehle-tense-oe | /ø/ | +2 % | +1 % | +0 % | 0.352 → 0.382 |
| s2-08-guete-tense-y | /y/ | +7 % | +3 % | −0 % | 0.357 → 0.358 |
| s2-18-offen-lax-o | /ɔ/ | +28 % | +28 % | +11 % | 0.554 → 0.550 |
| s2-19-butter-lax-u | /ʊ/ | +15 % | +19 % | +13 % | 0.394 → 0.512 |
| s2-20-muetter-lax-y | /ʏ/ | +1 % | +0 % | +1 % | 0.291 → 0.274 |
| s2-21-goetter-lax-oe | /œ/ | +6 % | +6 % | +1 % | 0.432 → 0.382 |

**Still outside the 10 % line**, with what the run's own files show of each:

- the male /y/ (+17 %) and both speakers' /ʊ/ (+14 %, +13 %): the fitted pharynx is narrower
  than the MRI's (its mean area 0.65, 0.77 and 0.79 of the MRI's), so the cavity behind the lips
  is too small for the F1. The anatomy's pharynx width is shared by all of a speaker's sounds.
- both /f/ (−50 %, −61 %): the fitted area function ends in a closure (its last cuts hold no
  air), where the MRI has an opening beside the teeth.
- the male /ʃ/ (+30 %): the fitted channel is 2.9 times the MRI's at its narrowest (the 1.5 mm
  limit); the male /s/ (−11 %): one fitted cut is under 0.01 cm², so it counts as shut.
- the female /s/ (+16 %) and /ɔ/ (+11 %): not explained (their narrowest cuts are within a
  quarter of the MRI's, and the /ɔ/'s pharynx within 3 %).

The first build, for comparison (its run of the same pipeline, 2026-11-09): fitted d 0.683 /
0.732, identification 17 / 14 of 22, F1–F3 error 12.4 % / 7.7 %, slicer area error 5.9 % /
4.6 % with channel agreement 72 % / 65 %, default d 1.08 / 1.12, compute 11.2 ms.

### Per sound

German tense and lax vowels map to the engine's nearest symbol (/aː a/ → /a/, /ɛː ɛ/ → /ɛ/, long
vowels → short symbol). "Between speakers" is d between the two speakers' MRI for the same sound
after scaling areas to equal means. The F errors compare the fitted model's area function with the
MRI's through the same 1D chain-matrix solver; they measure the geometry. The engine's voice does
not use this geometry yet. The constriction column is the model's narrowest cut against the
MRI's: how far along the tract it lies from the MRI's (as a share of the length), how many times
its area, and its channel count against the MRI's. The table is `table.md` of the run, as written.

| Case | IPA | Default d | Fitted d | Between speakers (scaled) | Nearest reference | F1/F2/F3 error % (fitted) | Constriction, model against MRI: place (Δ s/L), area (×), channels | Mean cut IoU |
|---|---|---:|---:|---:|---|---|---|---:|
| s1-01-bahn-tense-a | a | 1.161 | 0.392 | 0.64 | itself | 0 / 4 / 5 | +0.02, ×1.43, equal | 0.52 |
| s1-02-beet-tense-e | e | 0.797 | 0.328 | 0.47 | itself | -0 / 1 / 1 | -0.03, ×1.15, equal | 0.57 |
| s1-03-tiere-tense-i | i | 0.706 | 0.406 | 0.47 | itself | 0 / 2 / 2 | +0.01, ×1.16, equal | 0.62 |
| s1-04-boote-tense-o | o | 0.885 | 0.504 | 0.78 | itself | 9 / 3 / 3 | -0.38, ×0.62, equal | 0.69 |
| s1-05-bude-tense-u | u | 1.026 | 0.784 | 0.93 | itself | 2 / 9 / 0 | +0.03, ×0.51, equal | 0.65 |
| s1-06-laehmung-tense-ae | ɛ | 0.791 | 0.450 | 0.64 | itself | 1 / 0 / -0 | -0.01, ×1.25, equal | 0.60 |
| s1-07-hoehle-tense-oe | ø | 0.725 | 0.514 | 0.71 | itself | 7 / 5 / 10 | +0.04, ×0.78, equal | 0.68 |
| s1-08-guete-tense-y | y | 1.199 | 0.738 | 0.77 | itself | 17 / 4 / 4 | -0.25, ×1.51, equal | 0.66 |
| s1-09-los-l | l | 0.832 | 0.455 | 0.73 | itself | 3 / 4 / 2 | +0.01, ×0.51, -1 | 0.57 |
| s1-10-fahrt-f | f | 1.196 | 0.903 | 0.82 | itself | -50 / -22 / -2 | +0.03, ×0.00, -2 | 0.73 |
| s1-11-bass-s | s | 0.947 | 0.432 | 0.73 | itself | -11 / 3 / 1 | -0.01, ×0.00, -1 | 0.60 |
| s1-12-schoen-sh | ʃ | 1.399 | 0.514 | 1.45 | itself | 30 / 3 / 3 | +0.03, ×2.90, +2 | 0.67 |
| s1-13-ich-c | ç | 0.927 | 0.359 | 0.91 | itself | 2 / 2 / 4 | +0.00, ×1.24, equal | 0.60 |
| s1-14-ach-x | x | 1.490 | 0.496 | 0.56 | s1-01-bahn-tense-a | -9 / -1 / 3 | -0.03, ×1.12, equal | 0.60 |
| s1-15-bass-lax-a | a | 1.176 | 0.440 | 0.49 | itself | -2 / 4 / 2 | +0.02, ×1.48, equal | 0.50 |
| s1-16-bett-lax-ae | ɛ | 0.852 | 0.458 | 0.56 | itself | 1 / 1 / 2 | +0.03, ×1.46, equal | 0.63 |
| s1-17-mit-lax-i | ɪ | 0.750 | 0.456 | 0.77 | itself | 1 / 0 / -0 | -0.04, ×0.80, equal | 0.65 |
| s1-18-offen-lax-o | ɔ | 0.555 | 0.378 | 0.97 | itself | 1 / 1 / 1 | -0.39, ×0.87, equal | 0.59 |
| s1-19-butter-lax-u | ʊ | 0.971 | 0.631 | 0.83 | s1-04-boote-tense-o | 14 / 4 / 0 | +0.04, ×0.68, equal | 0.67 |
| s1-20-muetter-lax-y | ʏ | 0.525 | 0.376 | 0.49 | itself | 3 / 2 / 1 | +0.04, ×0.93, equal | 0.74 |
| s1-21-goetter-lax-oe | œ | 0.544 | 0.415 | 0.63 | itself | 3 / 2 / 3 | +0.03, ×0.74, equal | 0.67 |
| s1-22-ehe-schwa | ə | 0.725 | 0.322 | 0.90 | itself | -1 / 0 / -0 | +0.03, ×0.83, equal | 0.66 |
| s2-01-bahn-tense-a | a | 1.018 | 0.569 | 0.64 | s2-15-bass-lax-a | -7 / 2 / -0 | +0.33, ×1.73, -2 | 0.58 |
| s2-02-beet-tense-e | e | 0.810 | 0.342 | 0.47 | itself | -1 / 1 / 1 | -0.04, ×1.23, equal | 0.63 |
| s2-03-tiere-tense-i | i | 0.828 | 0.301 | 0.47 | itself | -1 / -4 / -2 | -0.09, ×0.95, equal | 0.65 |
| s2-04-boote-tense-o | o | 0.651 | 0.569 | 0.78 | itself | 1 / -1 / -10 | +0.10, ×0.64, -1 | 0.72 |
| s2-05-bude-tense-u | u | 0.666 | 0.427 | 0.93 | itself | 0 / -0 / -4 | -0.42, ×0.73, -2 | 0.69 |
| s2-06-laehmung-tense-ae | ɛ | 0.563 | 0.543 | 0.64 | s2-16-bett-lax-ae | -2 / 1 / 4 | +0.02, ×3.18, equal | 0.73 |
| s2-07-hoehle-tense-oe | ø | 0.650 | 0.382 | 0.71 | itself | 0 / 1 / 5 | +0.97, ×1.53, equal | 0.76 |
| s2-08-guete-tense-y | y | 0.792 | 0.358 | 0.77 | itself | -0 / 0 / 3 | +0.03, ×1.79, equal | 0.72 |
| s2-09-los-l | l | 0.729 | 0.327 | 0.73 | itself | -3 / 2 / 2 | +0.01, ×0.13, equal | 0.60 |
| s2-10-fahrt-f | f | 1.330 | 1.158 | 0.82 | s2-08-guete-tense-y | -61 / -21 / 5 | +0.93, ×0.00, -1 | 0.72 |
| s2-11-bass-s | s | 1.005 | 0.560 | 0.73 | itself | 16 / 2 / 2 | -0.02, ×0.98, +3 | 0.60 |
| s2-12-schoen-sh | ʃ | 1.347 | 0.922 | 1.43 | s2-08-guete-tense-y | 9 / 2 / 1 | -0.03, ×0.21, +1 | 0.64 |
| s2-13-ich-c | ç | 1.032 | 0.330 | 0.91 | itself | -0 / 1 / -0 | +0.02, ×0.77, equal | 0.68 |
| s2-14-ach-x | x | 1.261 | 0.529 | 0.56 | itself | -8 / -1 / 0 | +0.50, ×1.80, -1 | 0.66 |
| s2-15-bass-lax-a | a | 1.062 | 0.467 | 0.49 | itself | -7 / 1 / -4 | +0.31, ×1.15, -1 | 0.61 |
| s2-16-bett-lax-ae | ɛ | 0.407 | 0.305 | 0.56 | itself | 3 / 3 / 4 | +0.00, ×2.40, equal | 0.70 |
| s2-17-mit-lax-i | ɪ | 0.442 | 0.233 | 0.77 | itself | 2 / 5 / 1 | +0.00, ×1.57, -1 | 0.70 |
| s2-18-offen-lax-o | ɔ | 0.831 | 0.550 | 0.97 | itself | 11 / 3 / -5 | -0.02, ×1.24, equal | 0.68 |
| s2-19-butter-lax-u | ʊ | 0.961 | 0.512 | 0.83 | itself | 13 / 5 / -12 | +0.08, ×0.81, -1 | 0.72 |
| s2-20-muetter-lax-y | ʏ | 0.416 | 0.274 | 0.49 | itself | 1 / 1 / 0 | +0.02, ×1.87, equal | 0.77 |
| s2-21-goetter-lax-oe | œ | 0.515 | 0.382 | 0.63 | s2-16-bett-lax-ae | 1 / 2 / 1 | +0.00, ×2.20, equal | 0.77 |
| s2-22-ehe-schwa | ə | 0.623 | 0.499 | 0.90 | s2-16-bett-lax-ae | 0 / 0 / 1 | -0.02, ×4.29, equal | 0.66 |


## 2. The tongue shapes and the side cavities (T-features, T-anatomy)

Checked by geometry on the presets (male template), with the lines of the plan; the same checks run
in `cargo test` (`the_presets_make_the_tongue_shapes_the_research_lists`).

| Preset | Feature | Measured | Line | Result |
|---|---|---|---|---|
| flat | no concavity along the tongue | largest surface concavity 0.000 cm | ≤ 0.05 cm | pass |
| ə | piriform fossa volume | 2.63 cm³ each | 2.1–2.9 cm³ (male; scaled by size³) [R12] | pass |
| ə | piriform fossa depth | 1.75 cm | 1.6–2.0 cm (male; scaled by size) [R12] | pass |
| ə | valleculae off the midline only | pocket outline points at z = 0: 0, at z = 0.5 cm: 48 | absent at z = 0, present at 0.5 cm | pass |
| s | narrow single channel | 0.114 cm², 1 channel(s) at s/L 0.90 | ≤ 0.20 cm², 1 channel, s/L 0.85–0.95 (encloses both DVTD speakers' /s/) | pass |
| s | grooved (concave) tongue behind the channel | groove index 0.54 cm | > 0 (concave lower outline) | pass |
| ʃ | constriction behind /s/ | 0.026 cm² at s/L 0.78 (/s/ at 0.90) | ≤ 0.15 cm², further back than /s/ (as in both DVTD speakers) | pass |
| ʃ | long thin channel behind it | 6 cuts within 2.5 cm behind, under 0.6 cm high | ≥ 4 (≥ 1 cm of thin channel) | pass |
| ʃ | domed blade without a groove | largest surface concavity 0.00 cm over the blade | ≤ 0.05 cm | pass |
| l | two lateral channels at the tip contact | 2 channels at s/L 0.81, 2.00 cm apart | 2 channels, 2.0–2.8 cm apart (encloses both DVTD speakers' /l/) | pass |
| l | channel areas follow sideAsym | right/left 0.43/0.40 cm² with sideAsym +0.15 | right > left when sideAsym > 0 | pass |
| funnel | groove narrows and deepens towards the tip | t 0.60: 1.26 wide, 0.24 deep; t 0.75: 0.99 wide, 0.28 deep; t 0.90: 0.74 wide, 0.34 deep | width falls and depth rises from t 0.6 to 0.9 | pass |
| cupped | margins raised above the midline | surface rise 0.40 cm | > 0.1 cm | pass |
| bunched | pharyngeal and palatal constrictions | 2.46 cm² at s/L 0.26, 0.45 cm² at s/L 0.62; 4.43 cm² between | both below 60 % of the area between them | pass |
| bunched | open along its whole length | narrowest cut 0.45 cm², 0 cut(s) without air | every cut ≥ 0.15 cm² (an approximant, no closure) | pass |
| bunched | large front cavity | 14.6 cm³ in front of the palatal constriction | ≥ 10 cm³ | pass |
| bunched | tip low | tip 0.95 cm below the upper incisors' edge | ≥ 0.2 cm below | pass |
| retroflex | air under the raised tip (sublingual) | 3.83 cm of air below the tip in the midsagittal plane | ≥ 0.3 cm | pass |
| retroflex | palatal constriction at the curled tip | 0.61 cm² at s/L 0.79 | ≤ 0.8 cm² at s/L 0.7–0.92 | pass |

The presets are demonstration postures, tuned with `chimera tract3d tune` until their checks pass;
none is fitted to MRI (the /ʃ/ preset's channel is tighter than either speaker's). The bunched
/ɹ/ is open along its whole length since the review (its narrowest cut 0.45 cm², where the
reviewed preset had nine cuts without air), and the check refuses a closure. The female and
child presets pass 17 and 16 of the 19 checks: the adult lines do not scale with them (/l/
channels 1.85 and 1.55 cm apart against the 2.0–2.8 cm line; /s/ narrows to two and three
channels; the child's bunched /ɹ/ front cavity is 6.4 cm³ in a smaller mouth). They are
reported; the tests assert the male lines only.

Piriform fossae: 2.63 cm³ each and 1.75 cm deep in the default male model, measured from the
model's own pockets, within the 2.1–2.9 cm³ and 1.6–2.0 cm of four speakers'
volumetric MRI ([doi:10.1121/1.417990](https://doi.org/10.1121/1.417990)). Valleculae: present
off the midline, absent at z = 0 (the median glossoepiglottic fold).

## 3. English /ɹ/ and the USC volumes (T-/ɹ/)

26 volumes of the [USC 75-speaker set](https://doi.org/10.1038/s41597-021-00976-x) (CC BY 4.0,
1.5625 mm): /ɹ/ of 8 native English speakers, and /i ɑ u s ʃ l ɝ ə/ and a clenched posture of
two of them. Each panel puts the MRI's midsagittal and two coronal planes next to the model's
midsagittal air and a coronal cut, at the same scale. The planes are not registered to the model
and the volumes are not segmented (teeth look like air), so no area is measured.

Visual classification of the 8 sustained /ɹ/ (1.56 mm images, judged by eye):
5 bunched (dorsum high under the palate over a long stretch, tip low behind the lower teeth, a
large front cavity), 2 with the tip and blade raised over an air pocket under the tongue, 1
unclear. Both kinds occur, as the literature reports
([doi:10.1121/1.2902168](https://doi.org/10.1121/1.2902168)). The bunched preset shows what the
bunched speakers show (two constrictions, tip low, front cavity of 10.4 cm³, root drawn back); the
retroflex preset what the tip-up speakers show (air under the tip, the constriction at the
curled tip). Not established: where each speaker's constriction lies (needs registration of the
incisors and palate), the coronal outline of the tongue body in /ɹ/, and any area.

## 4. Cost

Native release build, one male model, 65 laminae, 73 cuts: median 1.51 ms per compute over the
15 presets (50 calls each), slowest preset 2.58 ms; the display meshes 0.85 ms for 9881
vertices (single crowns and gums included), measured while the machine's load average was
about 8. Design target ≤ 5 ms per compute: met. (The reviewed head: 3.13 ms and 7.18 ms under a
load of about 30, 3304 vertices.) In the browser every screen now asks the engine for the
posture's view each time the posture changes (`tract3dView`: positions as a float buffer,
without JSON); its timing is in [model-design.md §0.2](model-design.md#02-integration-2026-11-10-one-head-and-tract-on-every-screen).

## 5. Critique: where the model departs from MRI and why

Classes as in [validation-plan.md §9](validation-plan.md).

- **Anatomy, fixed per speaker: the larynx region.** The fit drives the piriform fossae to about
  the smallest the parameters allow (width 0.31–0.33 cm, bound 0.3; depth 1.01–1.09 cm, bound 1.0;
  both speakers) and the male hypopharynx to its widest (half-width 2.40 cm, bound 2.4). In the
  DVTD cuts just above the glottis the fossae are smaller, or merged with the hypopharynx, than
  an ellipsoid of the published volume beside the laryngeal tube. The two references disagree
  (DVTD's smoothed segmentation, 1.2 × 1.2 × 1.8 mm voxels; the four speakers of the volumetric
  study); the defaults follow the published volumes, the fit follows DVTD. Where the MRI splits
  into separate regions there, the model's count is within ±1 on 35/47 (male) and 45/93 (female)
  cuts. In 11 of 44 fits the model's narrowest cut lies in the lowest 15 % of the tract: the
  laryngeal tube, whose profile is the engine's epilarynx and stays unfitted.
- **Lip rounding: a freedom since the review, and its limit.** `lipRound` and the resonance
  term brought the rounded vowels' F1 from up to +35 % to within 10 % for 12 of 16 cases (§1).
  The four left (male /y/ and /ʊ/, female /ʊ/ and /ɔ/) no longer hang on the lips: three of
  them have a fitted pharynx narrower than the MRI's (next point).
- **Missing freedom: pharynx depth and width.** The model's depth in the oropharynx is the
  engine's midsagittal gap (bounded by its tongue-root range), and its width a single half-width
  per level shared by all of a speaker's sounds. For the sounds whose MRI pharynx is widest the
  fit stays under it (mean pharynx area 0.65–0.79 of the MRI's for the male /y/ and both /ʊ/).
- **Reference artefact and lip geometry: /f/.** Both speakers' /f/ is the worst fit (d 0.90 and
  1.16). The DVTD authors note that smoothing widened the /f/ constriction of their meshes
  ([doi:10.1038/s41597-020-00597-w](https://doi.org/10.1038/s41597-020-00597-w)); the model's
  labiodental closure leaves the lip end shut in the voxels (its last cuts hold no air), so its
  1D resonances are those of a closed tube (F1 −50 % and −61 %).
- **Resolution: sibilants.** Unconstrained, an earlier fit narrowed the male /s/ and /ʃ/ and the
  female /ʃ/ to slits about 1 mm thick, which a pipeline at this resolution cannot follow; with
  the 1.5 mm limit (now on the true hydraulic thickness: the perimeter was a staircase 28 % long
  on a circle) the fitted channels are wider than the MRI's (the male /ʃ/ 2.9 times at its
  narrowest). The model also makes sub-mm slivers between a tongue margin and the teeth that a
  smoothed segmentation does not have; both pipelines drop regions under 0.01 cm².
- **Derivation.** The default shapes come from the engine's targets, which were calibrated to
  formants through its 1D area law, without any geometric reference: median default d
  0.868 / 0.801. Fitting the eight `derive` constants to all 44 sounds (first build) moved the
  median default d only in the third decimal, with constants that differ between runs: what
  limits the defaults is the midline targets. The constants were left as they were; the targets
  move with the sound's port.
- **Speaker variation.** The two DVTD speakers differ by a median scaled d of 0.73 for the same
  sound, and their cuts by IoU 0.71; the fitted model's cuts reach IoU 0.66 / 0.71 against the
  speaker it was fitted to. The plan's fit lines (0.30 / 0.45) ask for a model closer to one
  speaker than two sounds of that speaker are to each other; this model meets the weaker
  standard of being closer to each speaker than the other speaker is.

Known limits of the references: one male and one female German speaker; sounds sustained for
14 s in the scanner; smoothed segmentations without the nasal cavity; the 1D reduction itself sits
6.5–8.6 % above the 3D prints' measured resonances (research.md §5). USC /ɹ/: unsegmented, 7 s,
unregistered here.

## 6. Not established

- The area-function lines of T-fit (median d ≤ 0.30, worst ≤ 0.45), the female identification
  (16 of 22 against ≥ 20), "every resonance within 10 %" (9 of 44 sounds are outside it) and
  T-default (median ≤ 0.71).
- Anatomy from the dental-cast scans: the arches and vault are fitted to the area functions
  instead; the casts were not measured.
- Registration of the USC planes, and any measured /ɹ/ feature.
- Descriptors for the sublingual and buccal spaces (they are geometry, they have no numbers yet).
- Any effect of the 3D geometry on the sound: the voice still uses the engine's 1D area law
  (the next lane, by the operator's order of work). The operator's amendment changed the
  voice's acoustics themselves (§7, §8).
- The head's skin against any reference: it is a display mesh (CC0) registered round the
  tract, judged by eye against the operator's reference picture.

## 7. Low frequencies: the engine's acoustics against DVTD's measured prints

*Operator amendment, 2026-11-09. `chimera tract3d acoustics` (tools/tract3d/README.md).*

DVTD 3D-printed each of its 44 tracts and measured the volume-velocity transfer function
U_lips/U_glottis with the glottis closed, in room air (345 m/s), on hard plastic walls. The
same MRI area functions (cut by this project's reference pipeline, §1) went through the
engine's 1D acoustics with the voice's own section length (2.5 mm), perimeter factors and
radiation load, walls rigid like the prints, the engine's frequency axis read at c = 345 m/s.
Peaks: at least 4 dB of prominence, measured curves smoothed over ±3 Hz.

| Engine physics | F1 error, median signed / absolute | F2 error, median signed / absolute | F1 bandwidth, engine / measured, median | 100–1000 Hz curve, RMS dB difference, median |
|---|---:|---:|---:|---:|
| before | +5.5 % / 5.5 % | +5.6 % / 5.6 % | 0.65 | 2.91 |
| after | +4.7 % / 4.7 % | +4.8 % / 5.1 % | 0.80 | 2.53 |

n = 44. The places that fitted worst were the narrow necks into wide cavities: the 11 sounds
whose F1 sat 9 % or more above the print's (/s/ of both speakers, female /ʃ/, /y u ʊ ç f/ and
the female /l/) moved from a median +10.5 % to +8.3 % (female /s/ +16.9 → +10.6 %, male /s/
+15.4 → +10.8 %, female /ʃ/ +12.9 → +8.8 %, male /y/ +11.1 → +7.8 %, male /u/ +9.4 → +7.0 %).

What changed (on in the voice's tract, off for creatures):
- **The inertance of abrupt area changes.** A plane-wave tube treats a step between two
  sections as a plain junction. The flow out of a narrow neck drags the air beyond it, which
  adds a length δ = 0.82·r·(1 − 1.35α + 0.31α³) to the neck (α the ratio of the radii,
  Kergomard & Garcia 1987, J. Sound Vib. 114:465); it vanishes for gentle steps (α ≳ 0.9) and
  lowers F1 where a constriction opens into a cavity (/u/, /y/, the sibilants, rounded lips).
- **The thermal boundary layer.** The engine had the viscous loss at the walls only; the heat
  exchange beside it (Kirchhoff; Flanagan 1972, eq. 3.33) is a shunt conductance per section of
  similar size. It raised the rigid-wall F1 bandwidth from 0.65 to 0.80 of the prints'.

What stays: about 5 % of F1 above the prints, the same on all sounds, which this 1D chain does
not resolve (the research stage's own chain-matrix solver showed +6.7 %, research.md §5). It
is the 1D reduction itself: cross-sections perpendicular to a curved centreline, no transverse
modes, the side branches folded into the area. Phase 2's 3D solver is the reference for that.
The remaining bandwidth gap (0.80) includes the prints' own losses (their plastic walls and the
measuring setup) as well as the model's. In the voice, the yielding walls dominate the
bandwidths at F1 (§8).

The fitted model's F1–F3 errors in §1 (rounded vowels, /f/) compare the model's area function
with the MRI's through the same solver; they are not affected by these terms and are unchanged.

## 8. The walls and the soft palate in the voice

*Operator amendment, 2026-11-09. `chimera tract3d walls` on the voice's own airway
(`HumanAirway`, glottis closed), the calibrated vowels, male and female.*

The voice's tract had yielding walls before: one tissue for every section (mass 1.5 g/cm²,
resistance 1600 dyn·s/cm³, stiffness 3·10⁵ dyn/cm³: its own resonance at 71 Hz) over the whole
perimeter. Now each section's wall is the tissue there, at control rate: the tongue, the
pharyngeal constrictors, the laryngeal walls, the cheeks and the lips by region, with the hard
palate and the teeth rigid; the stiffnesses put each tissue's resonance near the ~20 Hz Hanna,
Smith & Wolfe measured through the lips of ten subjects
([doi:10.1121/1.4948754](https://doi.org/10.1121/1.4948754)), the masses span their 1–2 cm
effective wall thickness, and the resistances (1.3× the 1000–1600 dyn·s/cm³ in use since
Flanagan and Stevens) keep the F1 bandwidths of the close vowels inside their 50–90 Hz. The
soft palate with the uvula is a membrane between the top of the pharynx and the nasopharynx.

Same postures, three acoustics: rigid walls; the walls as they were; the tissue walls, the soft
palate's membrane and the §7 terms.

| Measure | Male: rigid / before / after | Female: rigid / before / after | Human (Hanna et al. 2016) |
|---|---|---|---|
| Tissue resonance R0 through the sealed lips, /ə/ | – / 60 / 31 Hz | – / 61 / 31 Hz | ≈ 20 Hz |
| Anti-resonance R̄0 (wall mass with the air's compliance), /ə/ | – / 237 / 187 Hz | – / 250 / 195 Hz | ≈ 200 Hz |
| F1, median of 10 vowels | 444 / 503 / 482 Hz | 494 / 552 / 533 Hz | |
| F1 bandwidth, median of 10 vowels | 18 / 49 / 48 Hz | 20 / 47 / 46 Hz | 50–90 Hz (men), 70–90 Hz (women) |
| F1 bandwidth /i/, /u/, /a/ | 26, 29, 23 / 81, 82, 38 / 78, 86, 39 Hz | 20, 26, 27 / 59, 68, 40 / 58, 71, 41 Hz | |
| Nose / mouth at F1 through the closed velum, /i/, /u/, /a/ | – | – | (no measurement found) |
| … after | −25, −23, −70 dB | −29, −25, −71 dB | |

- **What the walls do.** They raise F1 above the rigid tube's (walls give a little under the
  pulse of pressure) and widen it; the effect falls off with frequency (B2, B3 move by a few Hz).
  The tissue's own resonance (R0) is sub-audio; between it and F1 the walls' mass sets the
  lowest F1 a closed tract can have (R̄0, the "voice bar" of voiced stops).
- **What changed.** R0 moved from 60 Hz (the stiff old walls) to 31 Hz, R̄0 from 237–250 Hz
  to 187–195 Hz, both towards the measured 20 and 200 Hz. The same articulation now has its
  F1 4 % (male) and 3 % (female) lower; the vowel table was recalibrated so the voice's vowels
  keep their target formants (median formant error after calibration: male F1 0.9 %, F2 0.8 %;
  the base calibration rerun gave 0.6 % and 0.5 %).
- **The soft palate and the uvula.** With the port shut, the soft palate moves with the
  pressure difference across it, and the close vowels (small mouth opening, high pressure in
  the mouth at F1) drive the nasal cavity through it: the nose radiates 23–25 dB below the
  mouth at F1 for /u/ and /i/ (male), 70 dB below for /a/. This is the path by which oral vowels
  reach the nose without any air path; no published measurement of its size was found here, so
  the numbers are the model's.
- **Not matched.** The F1 bandwidths of the open vowels (/a/ 39 Hz male, 41 Hz female) and the
  female vowels' median (46 Hz) stay below the measured ranges; raising the resistance further
  moved R0 and R̄0 away from theirs. Hanna's bandwidths are of impedance resonances through the
  lips, a close but different quantity from the transfer function's.
- **Evidence (local):** `walls.png` (transfer functions 0–1.5 kHz rigid / before / after with
  the nose's share; impedance through the sealed lips), `walls.md`, `walls.json`.
