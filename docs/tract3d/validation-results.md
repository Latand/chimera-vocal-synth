# Phase 1 validation results

*Measured 2026-11-09 on the model at commit b682d3a (geometry 1be63db), by
`chimera tract3d validate` with 3000 fit iterations per sound and 600 for each
speaker's anatomy (268 s on the shared machine). How the comparison works:
[validation-plan.md](validation-plan.md); commands: [tools/tract3d](../../tools/tract3d/README.md).*

**In short.** The parametric model reproduces the 44 MRI vocal tracts of the
[Dresden Vocal Tract Dataset](https://doi.org/10.1038/s41597-020-00597-w) (DVTD; two German
speakers, 22 sounds each) with a median area-function distance d of 0.438 (male) and
0.442 (female), against 0.68 and 0.73 for the first build. That sits below the distance between
the two speakers themselves (median 0.73): the fitted model is closer to each speaker than the
other speaker is. The pass lines the plan set for a fit (median d ≤ 0.30, every sound ≤ 0.45,
≥ 20/22 identified, F1–F3 within 5 % median and 10 % everywhere) are **not met**; the reasons are
below, sound by sound. The fast slicer measures what the reference pipeline measures (area
within 1.2 % on every sound), the tongue makes every shape the research lists, and the side
cavities have the published size. The default shapes (the engine's own targets with `derive`)
remain far from MRI (median d 0.845 / 0.774); closing that is phase 2's work on the
targets.

d is the RMS over 101 points of normalised length of log2(A_model / A_MRI), areas floored at
0.05 cm² (d = 1 means a factor of two on average). Every number below is this project's
measurement against the cited datasets; no dataset values are copied here.

## 1. DVTD: area functions, cuts and resonances

The same pipeline measures MRI and model: 0.5 mm voxels, a centreline through the lumen pinned to
the midsagittal plane, cuts every 2.5 mm perpendicular to it, and the air regions within ±6 mm
across and ±30 mm sideways. The port of the research probe reproduces the probe's output on all
44 meshes (worst per-mesh median area difference 1.71 %, lowest channel agreement 95.1 %,
largest length difference one step, 0.25 cm; plan lines 3 %, 95 %, 0.25 cm).

Each speaker's anatomy (pharynx widths and corner shape, dental arches, palate vault, laryngeal
tube, piriform fossae, tongue width) is fitted once on /ə aː iː uː s l/; then each sound's 13
articulators and 11 tongue parameters are fitted to its area function, length and cut
proportions, with a penalty for openings thinner than 1.5 mm (the reference pipeline traces its
centreline on a 1 mm grid, so a thinner slit cannot be followed; the MRI voxels are 1.2 mm).

| Test (plan §5) | Line | Male | Female | Result |
|---|---|---:|---:|---|
| Fitted d, median of 22 | ≤ 0.30 | 0.438 | 0.442 | fail |
| Fitted d, worst sound | ≤ 0.45 | 1.032 | 1.064 | fail |
| Sounds with fitted d ≤ 0.45 | all 22 | 12/22 | 12/22 | fail |
| Measured by the voxel pipeline | 22 | 22/22 | 22/22 | pass |
| Identification (nearest MRI sound is the target) | ≥ 20/22 | 18/22 | 18/22 | fail |
| F1–F3 of fitted vs MRI area function, median absolute error | ≤ 5 % | 6.9 % | 4.6 % | fail |
| F1–F3, every value | ≤ 10 % | 234 % | 58 % | fail |
| Default d (engine targets + derive), median | ≤ 0.71 | 0.845 | 0.774 | fail |
| Default d per sound ≤ max(1.0, between-speaker d) | all 22 | 18/22 | 17/22 | fail |
| Fast slicer vs the model's voxels, median area difference (T-consistency) | ≤ 3 % | 0.7 % | 0.8 % | pass |
| … worst sound | ≤ 3 % | 1.0 % | 1.2 % | pass |
| … channel count equal, median over sounds | ≥ 95 % | 96 % | 95 % | pass |

- **male**: port vs the research probe: worst median area difference 1.71 %, lowest channel agreement 95.8 %, largest length difference 0.25 cm (22 meshes); fitted length error median 2.0 %, volume error median 8.7 %; F1/F2/F3 median absolute error 9.9 / 8.0 / 2.5 %; lower pharynx (0.5–3 cm above the glottis), where MRI splits into ≥ 2 regions the model's count is within ±1 on 36/48 cuts; cut IoU median 0.67 (between the two speakers 0.71), aspect error (absolute log2 ratio) median 0.44 (between speakers 0.43); headline set median fitted d 0.438, sounds with fitted d ≤ 0.30: 2.
- **female**: port vs the research probe: worst median area difference 1.68 %, lowest channel agreement 95.1 %, largest length difference 0.25 cm (22 meshes); fitted length error median 0.8 %, volume error median 6.3 %; F1/F2/F3 median absolute error 9.5 / 4.1 / 4.1 %; lower pharynx (0.5–3 cm above the glottis), where MRI splits into ≥ 2 regions the model's count is within ±1 on 43/93 cuts; cut IoU median 0.70 (between the two speakers 0.71), aspect error (absolute log2 ratio) median 0.30 (between speakers 0.43); headline set median fitted d 0.539, sounds with fitted d ≤ 0.30: 3.

What the 1.5 mm limit costs: the same run without it (median fitted d 0.416 / 0.387, F1–F3
4.7 % / 3.9 %) and with a weaker limit on the cut's height (0.397 / 0.388) left 2 male and 1
female sound without a connected voxel path, so those sounds had no measured d. With the limit
every sound is measured, and the fits are about 0.05 worse.

The first build, for comparison (its run of the same pipeline, 2026-11-09): fitted d 0.683 /
0.732, identification 17 / 14 of 22, F1–F3 error 12.4 % / 7.7 %, slicer area error 5.9 % /
4.6 % with channel agreement 72 % / 65 %, default d 1.08 / 1.12, compute 11.2 ms.

### Per sound

German tense and lax vowels map to the engine's nearest symbol (/aː a/ → /a/, /ɛː ɛ/ → /ɛ/, long
vowels → short symbol). "Between speakers" is d between the two speakers' MRI for the same sound
after scaling areas to equal means. The F errors compare the fitted model's area function with the
MRI's through the same 1D chain-matrix solver; they measure the geometry. The engine's voice does
not use this geometry yet.

| Case | IPA | Default d | Fitted d | Between speakers (scaled) | F1 / F2 / F3 error, % | Constriction s/L, cm², channels: MRI → fitted | Nearest MRI sound |
|---|---|---:|---:|---:|---|---|---|
| s1-01-bahn-tense-a | /a/ | 1.118 | 0.321 | 0.64 | -1 / +1 / +8 | 0.49, 0.55, 1 → 0.51, 0.71, 1 | itself |
| s1-02-beet-tense-e | /e/ | 0.616 | 0.273 | 0.47 | +1 / +4 / +2 | 0.72, 0.53, 1 → 0.70, 0.58, 1 | itself |
| s1-03-tiere-tense-i | /i/ | 0.710 | 0.362 | 0.47 | +8 / +2 / -0 | 0.74, 0.32, 1 → 0.73, 0.37, 1 | itself |
| s1-04-boote-tense-o | /o/ | 0.855 | 0.478 | 0.78 | +18 / +14 / +3 | 0.96, 0.88, 1 → 0.58, 0.79, 1 | itself |
| s1-05-bude-tense-u | /u/ | 1.004 | 0.626 | 0.93 | +30 / +35 / +2 | 0.97, 0.50, 1 → 0.59, 0.74, 1 | itself |
| s1-06-laehmung-tense-ae | /ɛ/ | 0.675 | 0.436 | 0.64 | -10 / +2 / -1 | 0.07, 0.76, 1 → 0.06, 0.96, 1 | itself |
| s1-07-hoehle-tense-oe | /ø/ | 0.792 | 0.461 | 0.71 | +22 / +17 / +13 | 0.96, 0.66, 1 → 0.05, 0.96, 1 | itself |
| s1-08-guete-tense-y | /y/ | 1.441 | 0.657 | 0.77 | +34 / +23 / +12 | 0.96, 0.24, 1 → 0.74, 0.65, 1 | itself |
| s1-09-los-l | /l/ | 0.834 | 0.422 | 0.73 | +17 / +9 / +2 | 0.86, 1.04, 2 → 0.86, 0.90, 3 | itself |
| s1-10-fahrt-f | /f/ | 1.194 | 1.032 | 0.82 | +234 / +52 / +39 | 0.96, 0.85, 2 → 0.97, 0.00, 0 | s1-20-muetter-lax-y |
| s1-11-bass-s | /s/ | 0.954 | 0.579 | 0.73 | +41 / +16 / -2 | 0.90, 0.15, 1 → 0.89, 0.57, 3 | s1-22-ehe-schwa |
| s1-12-schoen-sh | /ʃ/ | 1.320 | 0.784 | 1.45 | +81 / +2 / -1 | 0.79, 0.10, 1 → 0.81, 0.71, 1 | s1-02-beet-tense-e |
| s1-13-ich-c | /ç/ | 1.015 | 0.394 | 0.91 | +7 / +4 / +6 | 0.82, 0.15, 1 → 0.82, 0.27, 1 | itself |
| s1-14-ach-x | /x/ | 1.539 | 0.454 | 0.56 | -14 / +1 / +5 | 0.42, 0.72, 1 → 0.39, 0.65, 1 | itself |
| s1-15-bass-lax-a | /a/ | 1.137 | 0.403 | 0.49 | -4 / +7 / +4 | 0.49, 0.56, 1 → 0.33, 0.74, 1 | itself |
| s1-16-bett-lax-ae | /ɛ/ | 0.756 | 0.457 | 0.56 | -6 / -1 / -3 | 0.03, 0.65, 1 → 0.06, 0.96, 1 | itself |
| s1-17-mit-lax-i | /ɪ/ | 0.835 | 0.440 | 0.77 | +8 / +7 / +1 | 0.65, 1.14, 1 → 0.05, 0.96, 1 | itself |
| s1-18-offen-lax-o | /ɔ/ | 0.541 | 0.345 | 0.97 | +7 / +8 / -0 | 0.44, 1.10, 1 → 0.05, 0.96, 1 | itself |
| s1-19-butter-lax-u | /ʊ/ | 1.006 | 0.616 | 0.83 | +28 / +21 / +1 | 0.53, 0.91, 1 → 0.57, 0.81, 1 | s1-04-boote-tense-o |
| s1-20-muetter-lax-y | /ʏ/ | 0.546 | 0.358 | 0.49 | +10 / +11 / +3 | 0.96, 0.86, 1 → 0.05, 0.96, 1 | itself |
| s1-21-goetter-lax-oe | /œ/ | 0.604 | 0.365 | 0.63 | +7 / +9 / +6 | 0.03, 1.30, 1 → 0.05, 0.96, 1 | itself |
| s1-22-ehe-schwa | /ə/ | 0.666 | 0.299 | 0.90 | -1 / +2 / -2 | 0.03, 1.15, 1 → 0.05, 0.96, 1 | itself |
| s2-01-bahn-tense-a | /a/ | 1.030 | 0.553 | 0.64 | -9 / +4 / +0 | 0.05, 0.39, 3 → 0.40, 0.66, 1 | itself |
| s2-02-beet-tense-e | /e/ | 0.792 | 0.338 | 0.47 | -2 / -1 / +2 | 0.71, 0.29, 1 → 0.68, 0.33, 1 | itself |
| s2-03-tiere-tense-i | /i/ | 0.743 | 0.289 | 0.47 | +13 / +2 / +2 | 0.81, 0.19, 1 → 0.75, 0.22, 1 | itself |
| s2-04-boote-tense-o | /o/ | 0.678 | 0.564 | 0.78 | +11 / +10 / -11 | 0.42, 0.41, 2 → 0.52, 0.33, 1 | itself |
| s2-05-bude-tense-u | /u/ | 0.663 | 0.443 | 0.93 | +18 / +19 / -4 | 0.97, 0.40, 3 → 0.55, 0.38, 1 | itself |
| s2-06-laehmung-tense-ae | /ɛ/ | 0.573 | 0.513 | 0.64 | -10 / -4 / +0 | 0.04, 0.25, 1 → 0.05, 0.79, 1 | s2-16-bett-lax-ae |
| s2-07-hoehle-tense-oe | /ø/ | 0.722 | 0.385 | 0.71 | +4 / +6 / +7 | 0.03, 0.40, 1 → 1.00, 0.57, 1 | itself |
| s2-08-guete-tense-y | /y/ | 0.921 | 0.364 | 0.77 | +4 / +6 / +4 | 0.97, 0.20, 1 → 1.00, 0.54, 1 | itself |
| s2-09-los-l | /l/ | 0.755 | 0.524 | 0.73 | +21 / +4 / +7 | 0.87, 0.35, 2 → 0.84, 0.29, 9 | itself |
| s2-10-fahrt-f | /f/ | 1.321 | 1.064 | 0.82 | -58 / -35 / +5 | 0.04, 0.23, 1 → 0.96, 0.00, 0 | itself |
| s2-11-bass-s | /s/ | 1.028 | 0.703 | 0.73 | +39 / +8 / +9 | 0.93, 0.10, 1 → 0.89, 0.60, 1 | s2-09-los-l |
| s2-12-schoen-sh | /ʃ/ | 1.370 | 0.770 | 1.43 | +29 / +8 / +3 | 0.86, 0.11, 1 → 0.88, 0.54, 1 | s2-13-ich-c |
| s2-13-ich-c | /ç/ | 1.088 | 0.318 | 0.91 | -5 / +3 / +1 | 0.89, 0.40, 1 → 0.91, 0.30, 1 | itself |
| s2-14-ach-x | /x/ | 1.272 | 0.577 | 0.56 | -11 / +3 / +1 | 0.04, 0.43, 2 → 0.37, 0.76, 1 | itself |
| s2-15-bass-lax-a | /a/ | 1.080 | 0.442 | 0.49 | -9 / +3 / -8 | 0.05, 0.38, 2 → 0.36, 0.43, 1 | itself |
| s2-16-bett-lax-ae | /ɛ/ | 0.404 | 0.306 | 0.56 | -1 / +1 / +4 | 0.05, 0.32, 1 → 0.05, 0.80, 2 | itself |
| s2-17-mit-lax-i | /ɪ/ | 0.439 | 0.240 | 0.77 | +5 / +5 / +4 | 0.05, 0.49, 2 → 0.05, 0.77, 1 | itself |
| s2-18-offen-lax-o | /ɔ/ | 0.808 | 0.491 | 0.97 | +22 / +6 / -8 | 0.40, 0.29, 1 → 0.38, 0.37, 1 | itself |
| s2-19-butter-lax-u | /ʊ/ | 0.974 | 0.407 | 0.83 | +17 / +4 / -12 | 0.46, 0.39, 2 → 0.55, 0.46, 1 | itself |
| s2-20-muetter-lax-y | /ʏ/ | 0.416 | 0.281 | 0.49 | +0 / +1 / -0 | 0.03, 0.42, 1 → 0.05, 0.78, 1 | itself |
| s2-21-goetter-lax-oe | /œ/ | 0.517 | 0.380 | 0.63 | +6 / +4 / +1 | 0.05, 0.35, 1 → 0.05, 0.77, 1 | itself |
| s2-22-ehe-schwa | /ə/ | 0.649 | 0.480 | 0.90 | +4 / +3 / +1 | 0.06, 0.18, 1 → 0.05, 0.79, 1 | s2-16-bett-lax-ae |


## 2. The tongue shapes and the side cavities (T-features, T-anatomy)

Checked by geometry on the presets (male template), with the lines of the plan; the same checks run
in `cargo test` (`the_presets_make_the_tongue_shapes_the_research_lists`).

| Preset | Feature | Measured | Line | Result |
|---|---|---|---|---|
| flat | no concavity along the tongue | largest surface concavity 0.000 cm | ≤ 0.05 cm | pass |
| ə | piriform fossa volume | 2.63 cm³ each | 2.1–2.9 cm³ (male; scaled by size³) [R12] | pass |
| ə | piriform fossa depth | 1.75 cm | 1.6–2.0 cm (male; scaled by size) [R12] | pass |
| ə | valleculae off the midline only | pocket outline points at z = 0: 0, at z = 0.5 cm: 48 | absent at z = 0, present at 0.5 cm | pass |
| s | narrow single channel | 0.114 cm², 1 channel(s) at s/L 0.90 | ≤ 0.20 cm², 1 channel, s/L 0.85–0.95 (DVTD 0.15/0.12 cm² at 0.90/0.93) | pass |
| s | grooved (concave) tongue behind the channel | groove index 0.54 cm | > 0 (concave lower outline) | pass |
| ʃ | constriction behind /s/ | 0.026 cm² at s/L 0.78 (/s/ at 0.90) | ≤ 0.15 cm², further back than /s/ (DVTD 0.10 cm² at 0.79–0.88) | pass |
| ʃ | long thin channel behind it | 6 cuts within 2.5 cm behind, under 0.6 cm high | ≥ 4 (≥ 1 cm of thin channel) | pass |
| ʃ | domed blade without a groove | largest surface concavity 0.00 cm over the blade | ≤ 0.05 cm | pass |
| l | two lateral channels at the tip contact | 2 channels at s/L 0.81, 2.00 cm apart | 2 channels, 2.0–2.8 cm apart (DVTD) | pass |
| l | channel areas follow sideAsym | right/left 0.43/0.40 cm² with sideAsym +0.15 | right > left when sideAsym > 0 | pass |
| funnel | groove narrows and deepens towards the tip | t 0.60: 1.26 wide, 0.24 deep; t 0.75: 1.00 wide, 0.28 deep; t 0.90: 0.74 wide, 0.33 deep | width falls and depth rises from t 0.6 to 0.9 | pass |
| cupped | margins raised above the midline | surface rise 0.39 cm | > 0.1 cm | pass |
| bunched | pharyngeal and palatal constrictions | 2.54 cm² at s/L 0.26, 0.00 cm² at s/L 0.56; 4.27 cm² between | both below 60 % of the area between them | pass |
| bunched | large front cavity | 10.4 cm³ in front of the palatal constriction | ≥ 10 cm³ | pass |
| retroflex | air under the raised tip (sublingual) | 3.83 cm of air below the tip in the midsagittal plane | ≥ 0.3 cm | pass |
| retroflex | palatal constriction at the curled tip | 0.61 cm² at s/L 0.79 | ≤ 0.8 cm² at s/L 0.7–0.92 | pass |

The presets are demonstration postures, tuned with `chimera tract3d tune` until their checks pass;
none is fitted to MRI, and the /ʃ/ preset narrows to 0.026 cm², tighter than DVTD's 0.10 cm². The female and child presets
pass 14 and 13 of 17: the adult lines do not scale with them
(/l/ channels 1.85 and 1.55 cm apart against 2.0–2.8 cm measured in two adults; /s/ narrows to
two channels in the female; the bunched /ɹ/ front cavity is smaller in a smaller mouth). They
are reported; the tests assert the male lines only.

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

Native release build, one male model, 65 laminae, 73 cuts: median 3.16 ms per compute over the 15 presets (50 calls each), slowest preset 5.99 ms; meshes 0.49 ms for 3304 vertices, measured while the
machine's load average was about 35 (other agents' work). Design target ≤ 5 ms per compute: the
median meets it, the slowest preset does not under that load. In the browser, the lab's call
across the JSON boundary (encode, wasm, decode, parse): median 12.9 ms with the meshes (p95 39.6 ms, 302 KB of JSON) and 4.4 ms for a cut alone (32 KB), over 20 calls in headless Chromium on the production build, same load (the first build: 41.7 ms and 1.3 MB per call) (target 16 ms).

## 5. Critique: where the model departs from MRI and why

Classes as in [validation-plan.md §9](validation-plan.md).

- **Anatomy, fixed per speaker: the larynx region.** The fit drives the piriform fossae to about the
  smallest the parameters allow (width 0.31–0.32 cm, bound 0.3; depth 1.00–1.06 cm, bound 1.0;
  both speakers) and the male hypopharynx to about its widest (half-width 2.38 cm, bound 2.4). In the DVTD cuts 0.5–3 cm above the glottis the fossae
  are smaller, or merged with the hypopharynx, than an ellipsoid of the published volume beside
  the laryngeal tube. The two references disagree (DVTD's smoothed segmentation, 1.2 × 1.2 ×
  1.8 mm voxels; the four speakers of the volumetric study); the defaults follow the published
  volumes, the fit follows DVTD. Where the MRI splits into separate regions there, the model's
  count is within ±1 on 36/48 (male) and 43/93 (female) cuts. In 14 of 44 fits the model's
  narrowest point is the laryngeal tube (about 0.95 cm² at the false folds, the engine's epilarynx
  profile), where the MRI's lies further up: the tube's profile is the engine's and stays unfitted.
- **Missing freedom: lip rounding.** The male's rounded vowels carry the largest F1/F2 errors
  (/u/ +30 / +35 %, /y/ +34 / +23 %, /ø/ +22 / +17 %, /ʊ/ +28 / +21 %, /o/ +18 / +14 %; the
  female's are smaller, up to +22 %). The MRI's rounded openings are small and round (male /y/:
  0.29 cm² in the cut 0.95 of the way to the lips); the model's opening is the slit between two
  lip outlines whose width has a floor (the engine's 1.1 cm at full rounding), and the fitted /y/
  keeps 1.29 cm² there.
- **Missing freedom: pharynx depth.** For front vowels the MRI oropharynx reaches 10–12 cm²; the
  model's depth there is the engine's midsagittal gap (bounded by its tongue-root range), and its
  width a single half-width per level, so it stays near 7–10 cm².
- **Reference artefact and lip geometry: /f/.** Both speakers' /f/ is the worst fit (d about 1). The
  DVTD authors note that smoothing widened the /f/ constriction of their meshes
  ([doi:10.1038/s41597-020-00597-w](https://doi.org/10.1038/s41597-020-00597-w)); the model's
  labiodental closure leaves the lip end almost shut in the voxels, so its 1D resonances are
  those of a nearly closed tube (the male F1 error of +234 % pairs a different resonance with
  the MRI's F1).
- **Resolution: sibilants.** Unconstrained, the fit narrowed the male /s/ and /ʃ/ and the female
  /ʃ/ to slits about 1 mm thick (fast d 0.27, 0.32, 0.44), which a pipeline at this resolution
  cannot follow; with the 1.5 mm limit
  the fitted constrictions are wider than the MRI's 0.15 / 0.10 cm². The model also makes sub-mm
  slivers between a tongue margin and the teeth that a smoothed segmentation does not have; the
  fast slicer drops regions under 0.01 cm².
- **Derivation.** The default shapes come from the engine's targets, which were calibrated to
  formants through its 1D area law, without any geometric reference: median default d 0.845 / 0.774.
  Fitting the eight `derive` constants to all 44 sounds moved the median default d only from
  0.828 to 0.826, with constants that differ between runs: what limits the
  defaults is the midline targets. The constants were left as they were.
- **Speaker variation.** The two DVTD speakers differ by a median scaled d of 0.73 for the same
  sound, and their cuts by IoU 0.71; the fitted model's cuts reach IoU 0.67 / 0.70
  against the speaker it was fitted to. The plan's fit lines (0.30 / 0.45) ask for a model closer to
  one speaker than two sounds of that speaker are to each other; this model meets the weaker standard
  of being closer to each speaker than the other speaker is.

Known limits of the references: one male and one female German speaker; sounds sustained for
14 s in the scanner; smoothed segmentations without the nasal cavity; the 1D reduction itself sits
6.5–8.6 % above the 3D prints' measured resonances (research.md §5). USC /ɹ/: unsegmented, 7 s,
unregistered here.

## 6. Not established in phase 1

- The fit lines of T-fit (median d ≤ 0.30, worst ≤ 0.45, 20/22 identified, resonances within
  5 % / 10 %) and T-default (median ≤ 0.71).
- Anatomy from the dental-cast scans: the arches and vault are fitted to the area functions
  instead; the casts were not measured.
- Registration of the USC planes, and any measured /ɹ/ feature.
- Descriptors for the sublingual and buccal spaces (they are geometry, they have no numbers yet).
- Any effect of the 3D geometry on the sound: the voice still uses the engine's 1D area law
  (phase 2). The operator's amendment changed the voice's acoustics themselves (§7, §8).

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
