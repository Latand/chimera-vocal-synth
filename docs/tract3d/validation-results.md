# Validation results

*Measured 2026-11-10 on the model after the slice fix (model-design.md §0.3), by
`chimera tract3d validate` with 3000 fit iterations per sound and 1000 for each speaker's
anatomy (363 s on the shared machine, three jobs). How the comparison works:
[validation-plan.md](validation-plan.md); commands: [tools/tract3d](../../tools/tract3d/README.md).
Sections 3, 7 and 8 keep their own dates.*

**In short.** The parametric model reproduces the 44 MRI vocal tracts of the
[Dresden Vocal Tract Dataset](https://doi.org/10.1038/s41597-020-00597-w) (DVTD; two German
speakers, 22 sounds each) with a median area-function distance d of 0.445 (male) and 0.376
(female), below the distance between the two speakers themselves (median 0.73), and with the
first three resonances of the fitted area functions a median 2.1 % and 2.7 % from the MRI's
(same 1D solver both sides). Every one of the 44 fits is measured by the voxel pipeline, and
the fast slicer agrees with the model's own voxels (area within 1.1 % on every sound, channel
counts equal on a median 99 % of cuts). Where the MRI's lower pharynx splits into separate
regions, the model's count is now within one on 44 of 47 (male) and 86 of 93 (female) cuts
(35 of 47 and 45 of 93 before the slice fix). **Not met**: the plan's area-function lines for
a fit (median d ≤ 0.30, every sound ≤ 0.45), the identification (19 of 22 each against ≥ 20),
"every resonance within 10 %" (3 male and 4 female sounds are outside it), the lower-pharynx
line on every cut, the cut-shape measures at the level of the two speakers' own difference
(cut IoU 0.65 and 0.66 against 0.71; aspect error 0.51 against 0.43), and the default shapes
(the engine's own targets with `derive`: median d 0.887 / 0.743 against 0.71). The reasons,
cut by cut and sound by sound, are below.

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

Each speaker's anatomy (the pharyngeal ring's half-widths and outline, dental arches, palate
vault, the laryngeal tube's size, the epiglottis' curl, the aryepiglottic folds' height, the
piriform fossae, tongue width: 14 values) is fitted once on /ə aː iː uː s l/; then each
sound's 13 articulators, 11 tongue parameters, the lips' rounding and the pharynx's squeeze
are fitted. The objective is the area-function distance, the tract's length, the cuts'
proportions at six stations, a penalty for openings thinner than 1.5 mm (the reference
pipeline traces its centreline on a 1 mm grid; the MRI voxels are 1.2 mm), and the distance
of F1–F3 from the reference's (weight 0.8 per unit of log ratio: a 10 % error of one
resonance costs as much as 0.08 of d). A fit whose voxels have no connected path from glottis
to lips is backed off towards its starting posture until they have one; in this run one fit
(the male /ʃ/) was backed off half-way.

| Test (plan §5) | Line | Male | Female | Result |
|---|---|---:|---:|---|
| Fitted d, median of 22 | ≤ 0.30 | 0.445 | 0.376 | fail |
| Fitted d, worst sound | ≤ 0.45 | 1.050 | 1.020 | fail |
| Sounds with fitted d ≤ 0.45 | all 22 | 12/22 | 13/22 | fail |
| Measured by the voxel pipeline | 22 | 22/22 | 22/22 | pass |
| Identification (nearest MRI sound is the target) | ≥ 20/22 | 19/22 | 19/22 | fail |
| F1–F3 of fitted vs MRI area function, median absolute error | ≤ 5 % | 2.1 % | 2.7 % | pass |
| F1–F3, every value | ≤ 10 % | 243 % | 53 % | fail (3 and 4 sounds outside) |
| Lower pharynx: regions within ±1 where MRI splits | every such cut | 44/47 | 86/93 | fail |
| Cut IoU, median over stations and sounds | ≥ 0.71 (between the two speakers) | 0.65 | 0.66 | fail |
| Cut aspect error (absolute log2 ratio), median | ≤ 0.43 (between the two speakers) | 0.51 | 0.51 | fail |
| Default d (engine targets + derive), median | ≤ 0.71 | 0.887 | 0.743 | fail |
| Default d per sound ≤ max(1.0, between-speaker d) | all 22 | 19/22 | 18/22 | fail |
| Fast slicer vs the model's voxels, median area difference (T-consistency) | ≤ 3 % | 0.7 % | 0.7 % | pass |
| … worst sound | ≤ 3 % | 1.0 % | 1.1 % | pass |
| … channel count equal, median over sounds (lowest sound) | ≥ 95 % | 99 % (93 %) | 99 % (95 %) | pass |

- **male**: fitted length error median 2.6 %, volume error median 6.2 %; F1/F2/F3 median
  absolute error 1.6 / 2.0 / 2.4 %; headline set median fitted d 0.429, sounds with fitted
  d ≤ 0.30: 1.
- **female**: fitted length error median 1.6 %, volume error median 3.6 %; F1/F2/F3 median
  absolute error 2.0 / 2.4 / 3.2 %; headline set median fitted d 0.466, sounds with fitted
  d ≤ 0.30: 5.

**One run is one sample.** The fit searches 26 values per sound and 14 per speaker (Nelder–Mead
with restarts) and ends in different places. Six runs of the command were made while the
geometry changed in this stage; over the last three, whose geometry differed only in details
(where the lips' rounding begins, their corner exponent, the cheeks, default values), the
medians ranged: fitted d
0.43–0.45 (male) and 0.34–0.42 (female); cut IoU 0.65–0.67 and 0.66–0.68; aspect error
0.44–0.51 and 0.40–0.52; lower pharynx 44–45 of 47 and 86–89 of 93; default d 0.88–0.91 and
0.74–0.80. Differences of that size between two runs are the search; the table is the last
run on the final geometry.

### What the slice fix changed, and what it bought

The slice-critic judged the integrated head (commit d5241e0) from its own frames: cuts of the
pharynx and larynx that were blocky polygons of the same shape in every sound, no epiglottis
in the lumen, a pharynx 3.0 cm wide in every posture, default shapes far from MRI. The
geometry that answers it is model-design.md §0.3. The same command on both:

| | Integrated head | After the slice fix |
|---|---|---|
| Fitted d, median (male / female) | 0.453 / 0.447 | 0.445 / 0.376 |
| Sounds with fitted d ≤ 0.45 | 10 / 11 | 12 / 13 |
| Identified | 20 / 16 | 19 / 19 |
| Sounds with a resonance over 10 % off | 5 / 4 | 3 / 4 |
| Lower pharynx: regions within ±1 where MRI splits | 35/47, 45/93 | 44/47, 86/93 |
| Cut IoU, median | 0.66 / 0.71 | 0.65 / 0.66 |
| Cut aspect error, median | 0.47 / 0.36 | 0.51 / 0.51 |
| Default d, median | 0.868 / 0.801 | 0.887 / 0.743 |
| Channel agreement with the model's voxels, median | 99 / 98 % | 99 / 99 % |

- **The lower pharynx's channels.** The fossae are pouches beside the laryngeal tube with the
  aryepiglottic folds between; how high the folds stand is fitted per speaker (0.58 cm for
  the male, 0.35 cm for the female). The nine sounds that still miss do so on one or two
  cuts: three male sounds on one cut each near the top of the range the line covers (the
  MRI still splits where the model has merged into one region or, once, has two more than
  it), and six female sounds whose MRI splits one cut lower than the model's fossae begin
  (the fit ended with the fossae 1.0 cm deep, their bound).
- **The pharynx's cuts.** At the stations in the pharynx (0.3 and 0.45 of the tract's length)
  the male model's cuts are as close to the speaker as the other speaker's are, or closer
  (cut IoU 0.68 and 0.78 against 0.65 between the speakers; aspect error 0.52 and 0.23
  against 0.58 and 0.71). The female model's are as close at 0.3 (IoU 0.65, aspect error
  0.43) and less close at 0.45 (IoU 0.55; aspect error 0.70 against 0.71).
- **Where the cut shape lost.** At 0.15 of the length (the hypopharynx's floor) the male
  model's cut is 2.2 times too wide for its depth (aspect error 1.14 against 0.33 between
  the speakers; IoU 0.49 against 0.72; female 0.63 and 0.65): the laryngeal tube's rim is as
  wide as the engine's area asks in the room the engine's larynx leaves front to back, and
  the fossae stand beyond it, so the three channels spread further sideways than the MRI's.
  At 0.9 (the teeth and lips) IoU is 0.55 and 0.58 against 0.74: the tooth rows are blocks
  with flat faces and the incisors' rows rise in steps beside the midline (critique, below).
  The pharynx's lumen also ends sideways in thin tips where the ring meets the tongue or the
  epiglottis at a shallow angle; a smoothed segmentation has no such tips, and they widen
  the model's cuts (0.6 of the length: aspect error 0.58 and 0.62 against 0.41).
- **Ruled out for the tips:** a squarer ring (an outline exponent of 3 and more, so the walls
  turn forward later and meet the tongue more steeply), as a seventh run: male cut IoU 0.61
  and aspect error 0.55, female 0.67 and 0.41, lower pharynx 43/47 and 82/93. No gain beyond
  the search's own spread, and the male's cuts lost; the exponent's range stays 2 to 4.
- **The female fit** gained most (median d 0.447 → 0.376, five sounds under 0.30, three more
  identified): her pharynx is narrow for its depth and her three lower channels lie where the
  model's now are.
- **The defaults** did not come nearer the line (next section).

### The default shapes, sound by sound

The default posture of a sound is the engine's own target for the nearest symbol with
`derive`'s 3D shape and the speaker's fitted anatomy. The per-sound table gives, for each, the
default model's area against the MRI's over four stretches of the tract (geometric mean of the
ratio: larynx to 0.2 of the length, pharynx to 0.55, mouth to 0.9, lips). What it shows:

- **Open and mid front vowels** (/a/, /ɛ/, /e/): the pharynx is 1.2 to 2.2 times the MRI's
  (the male /ɛ/ the most), the mouth 1.7 to 2.1 times for /e/ and the female /i/. The engine's
  targets for these keep a deeper pharynx than either speaker has.
- **Back rounded vowels** (/o/, /ɔ/): the pharynx is a quarter to two fifths of the MRI's for
  the male /o/ and /ɔ/ and the female /ɔ/ (default d 1.41, 1.53, 1.20, the three vowels over
  their line). The engine's targets draw the tongue's root far back for them, and `derive`
  draws the side walls in on top of that.
- **Close rounded vowels of the male speaker** (/u/, /y/, /ʊ/, /ø/): nearly every stretch is
  0.4 to 0.9 of the MRI's: his rounded vowels are larger than the engine's almost everywhere.
- **/x/**: the pharynx 2.7 to 3.4 times the MRI's and the mouth a third to two fifths of it (default d
  1.56 and 1.31, over the line for both speakers): the engine's velar closure is tighter
  and further forward than the sustained /x/ of either speaker.
- **/ç/, /ʃ/, /s/, /f/**: the mouth 0.4 to 0.5 of the MRI's for /ç/ and 2.6 to 2.8 times for
  /ʃ/; the lips 2.5 to 3 times for /s/ and a fifth to a quarter for /f/ (the engine's
  labiodental closure against the MRI's opening beside the teeth). The female /s/ and /ç/
  are over their line (1.10 and 1.19).

`derive`'s ten constants were fitted to the median default d over all 44 sounds (the fast
slicer, the fitted anatomies of an earlier run): it moved from 0.823 to 0.783, with most
constants at their bounds; fitting only the two new ones (the lips' rounding and the pharynx's
squeeze) gave 0.787 with the squeeze at 1.17 and the rounding at 0.21. The values in the code
are a rounding of 1.0 and a squeeze of 0.9, and the eight older constants as they were. The
rounding stays at 1.0 because the fitted tense rounded vowels of both speakers end between
0.76 and 1.0 (the female's lax ones between 0.06 and 0.47; the area distance alone prefers a
wide slit at the lips, the resonances do not); the squeeze is between what the fitted open
vowels and /x/ use (0.5 to 0.9) and what the median asks for.
What keeps the defaults from the line is the engine's midline targets, calibrated to formants
through its 1D area law without any geometric reference; they move when the sound is computed
from this geometry.

**Still outside the 10 % resonance line**, with what the run's own files show of each:

- both /f/ (+243 % and −53 % on F1): the fitted area function ends in a closure (its last cuts
  hold no air) where the MRI has an opening beside the teeth; the male fit's lowest resonance
  is missing from its curve, so the first peak found is the next one.
- both /ʃ/ (+91 % and +20 %): the fitted channels are 5.2 and 1.4 times the MRI's at their
  narrowest (the 1.5 mm limit; the male fit was backed off half-way to keep a connected path).
- the male /y/ (F3 +10 %), the female /ɔ/ (F1 +14 %) and /ʊ/ (F3 −13 %): not explained.

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

| Case | IPA | Default d | Default area against MRI by stretch: larynx / pharynx / mouth / lips (×) | Fitted d | Between speakers (scaled) | Nearest reference | F1/F2/F3 error % (fitted) | Constriction, model against MRI: place (Δ s/L), area (×), channels | Mean cut IoU |
|---|---|---:|---|---:|---:|---|---|---|---:|
| s1-01-bahn-tense-a | a | 0.704 | 1.16 / 1.69 / 1.32 / 1.56 | 0.411 | 0.64 | itself | -0 / 3 / 1 | -0.14, ×1.46, equal | 0.59 |
| s1-02-beet-tense-e | e | 0.704 | 0.99 / 1.31 / 1.70 / 1.28 | 0.313 | 0.47 | itself | -2 / 2 / -0 | -0.03, ×0.96, equal | 0.61 |
| s1-03-tiere-tense-i | i | 0.626 | 0.72 / 0.76 / 0.70 / 0.82 | 0.305 | 0.47 | itself | -2 / 2 / -1 | +0.02, ×1.35, equal | 0.61 |
| s1-04-boote-tense-o | o | 1.413 | 0.58 / 0.29 / 0.85 / 0.67 | 0.528 | 0.78 | itself | 5 / 9 / 6 | +0.04, ×0.56, equal | 0.71 |
| s1-05-bude-tense-u | u | 1.001 | 0.68 / 0.62 / 0.61 / 0.67 | 0.714 | 0.93 | itself | -2 / 1 / 4 | +0.03, ×0.51, equal | 0.58 |
| s1-06-laehmung-tense-ae | ɛ | 0.849 | 1.61 / 1.92 / 1.65 / 1.16 | 0.444 | 0.64 | itself | 0 / 1 / -1 | -0.01, ×2.14, +2 | 0.60 |
| s1-07-hoehle-tense-oe | ø | 0.690 | 0.79 / 0.75 / 0.65 / 0.71 | 0.484 | 0.71 | itself | 1 / 2 / 10 | +0.02, ×1.29, equal | 0.67 |
| s1-08-guete-tense-y | y | 1.133 | 0.56 / 0.62 / 0.42 / 1.10 | 0.674 | 0.77 | itself | 1 / 2 / 10 | +0.04, ×1.38, equal | 0.66 |
| s1-09-los-l | l | 0.704 | 0.72 / 0.67 / 1.00 / 1.08 | 0.356 | 0.73 | itself | 2 / 9 / 2 | +0.01, ×1.03, -1 | 0.54 |
| s1-10-fahrt-f | f | 1.194 | 1.06 / 0.83 / 0.94 / 0.18 | 1.050 | 0.82 | s1-20-muetter-lax-y | 243 / 45 / 39 | +0.01, ×0.00, -2 | 0.71 |
| s1-11-bass-s | s | 0.936 | 1.08 / 0.90 / 0.95 / 2.95 | 0.457 | 0.73 | itself | -0 / 7 / 2 | -0.01, ×0.41, +1 | 0.57 |
| s1-12-schoen-sh | ʃ | 1.347 | 1.05 / 0.68 / 2.80 / 0.39 | 0.935 | 1.45 | s1-22-ehe-schwa | 91 / -8 / -2 | +0.02, ×5.20, +1 | 0.58 |
| s1-13-ich-c | ç | 1.004 | 1.10 / 1.07 / 0.54 / 0.80 | 0.343 | 0.91 | itself | -2 / 2 / 2 | +0.01, ×1.18, equal | 0.62 |
| s1-14-ach-x | x | 1.558 | 1.78 / 3.41 / 0.32 / 0.57 | 0.447 | 0.56 | itself | 1 / 2 / 3 | -0.01, ×0.93, equal | 0.59 |
| s1-15-bass-lax-a | a | 0.732 | 1.48 / 1.83 / 0.93 / 1.50 | 0.554 | 0.49 | itself | 0 / 1 / 3 | -0.13, ×1.23, equal | 0.54 |
| s1-16-bett-lax-ae | ɛ | 0.926 | 1.59 / 2.15 / 1.53 / 1.24 | 0.422 | 0.56 | itself | -0 / 0 / -4 | +0.59, ×2.23, equal | 0.66 |
| s1-17-mit-lax-i | ɪ | 0.810 | 0.82 / 1.32 / 0.65 / 0.88 | 0.416 | 0.77 | itself | -0 / 0 / 2 | -0.06, ×0.88, equal | 0.62 |
| s1-18-offen-lax-o | ɔ | 1.528 | 0.61 / 0.25 / 0.55 / 1.16 | 0.470 | 0.97 | itself | 3 / 4 / 2 | +0.01, ×1.08, equal | 0.63 |
| s1-19-butter-lax-u | ʊ | 1.058 | 0.53 / 0.87 / 0.43 / 0.59 | 0.671 | 0.83 | s1-04-boote-tense-o | 5 / 9 / 5 | +0.06, ×0.58, equal | 0.65 |
| s1-20-muetter-lax-y | ʏ | 0.488 | 0.85 / 0.94 / 0.82 / 1.40 | 0.410 | 0.49 | itself | 1 / 2 / 2 | +0.04, ×0.81, equal | 0.67 |
| s1-21-goetter-lax-oe | œ | 0.590 | 0.89 / 1.24 / 0.74 / 0.77 | 0.398 | 0.63 | itself | 2 / 2 / 4 | +0.97, ×0.99, equal | 0.67 |
| s1-22-ehe-schwa | ə | 0.652 | 1.24 / 1.10 / 1.65 / 1.06 | 0.253 | 0.90 | itself | 2 / 2 / -0 | +0.64, ×1.33, equal | 0.51 |
| s2-01-bahn-tense-a | a | 0.519 | 1.10 / 1.25 / 1.01 / 1.01 | 0.455 | 0.64 | s2-15-bass-lax-a | 1 / 2 / -1 | +0.00, ×1.76, -2 | 0.47 |
| s2-02-beet-tense-e | e | 0.806 | 1.05 / 1.21 / 1.96 / 1.67 | 0.306 | 0.47 | itself | -2 / 2 / 1 | -0.04, ×1.14, equal | 0.64 |
| s2-03-tiere-tense-i | i | 0.773 | 1.17 / 1.12 / 2.06 / 1.56 | 0.246 | 0.47 | itself | -2 / 1 / 3 | -0.07, ×0.97, equal | 0.69 |
| s2-04-boote-tense-o | o | 0.701 | 0.77 / 1.21 / 0.86 / 0.63 | 0.551 | 0.78 | itself | 4 / 4 / -6 | +0.10, ×0.88, -1 | 0.68 |
| s2-05-bude-tense-u | u | 0.687 | 0.81 / 1.09 / 0.66 / 0.70 | 0.394 | 0.93 | itself | 2 / 4 / 0 | -0.41, ×0.93, -2 | 0.63 |
| s2-06-laehmung-tense-ae | ɛ | 0.621 | 1.57 / 1.51 / 1.08 / 0.85 | 0.404 | 0.64 | s2-16-bett-lax-ae | 0 / 3 / 5 | +0.04, ×2.76, +2 | 0.62 |
| s2-07-hoehle-tense-oe | ø | 0.713 | 1.16 / 1.27 / 0.58 / 1.22 | 0.318 | 0.71 | itself | 2 / 0 / 2 | +0.97, ×1.16, equal | 0.71 |
| s2-08-guete-tense-y | y | 0.797 | 1.09 / 1.02 / 0.47 / 1.41 | 0.297 | 0.77 | itself | 1 / -1 / 2 | +0.03, ×1.72, equal | 0.68 |
| s2-09-los-l | l | 0.816 | 0.94 / 0.95 / 0.83 / 1.77 | 0.519 | 0.73 | itself | 5 / 3 / 7 | -0.02, ×1.62, equal | 0.55 |
| s2-10-fahrt-f | f | 1.406 | 1.22 / 2.52 / 1.89 / 0.27 | 1.020 | 0.82 | itself | -53 / -23 / -4 | +0.93, ×0.00, -1 | 0.63 |
| s2-11-bass-s | s | 1.096 | 1.28 / 1.56 / 0.94 / 2.47 | 0.480 | 0.73 | itself | 7 / 2 / 8 | -0.02, ×1.14, equal | 0.59 |
| s2-12-schoen-sh | ʃ | 1.376 | 1.45 / 1.85 / 2.59 / 1.16 | 0.687 | 1.43 | itself | 20 / 3 / 4 | -0.02, ×1.42, +1 | 0.62 |
| s2-13-ich-c | ç | 1.188 | 1.17 / 1.59 / 0.42 / 1.45 | 0.343 | 0.91 | itself | -1 / 1 / -0 | +0.00, ×0.54, equal | 0.64 |
| s2-14-ach-x | x | 1.307 | 1.50 / 2.72 / 0.43 / 0.79 | 0.357 | 0.56 | itself | 4 / 5 / 5 | +0.02, ×1.60, -1 | 0.54 |
| s2-15-bass-lax-a | a | 0.433 | 1.06 / 1.47 / 0.93 / 1.22 | 0.327 | 0.49 | itself | 1 / 5 / 5 | +0.31, ×1.16, -1 | 0.49 |
| s2-16-bett-lax-ae | ɛ | 0.479 | 1.00 / 1.44 / 1.15 / 1.34 | 0.229 | 0.56 | itself | 3 / 3 / 3 | +0.00, ×2.20, equal | 0.64 |
| s2-17-mit-lax-i | ɪ | 0.463 | 0.81 / 1.10 / 0.81 / 0.83 | 0.279 | 0.77 | itself | 1 / 2 / 1 | +0.00, ×1.36, -1 | 0.67 |
| s2-18-offen-lax-o | ɔ | 1.203 | 0.44 / 0.40 / 0.58 / 0.61 | 0.636 | 0.97 | itself | 14 / 3 / 5 | +0.01, ×0.89, equal | 0.59 |
| s2-19-butter-lax-u | ʊ | 1.061 | 0.78 / 1.52 / 0.58 / 0.27 | 0.518 | 0.83 | itself | 7 / 2 / -13 | +0.08, ×0.88, -1 | 0.69 |
| s2-20-muetter-lax-y | ʏ | 0.386 | 1.11 / 1.04 / 1.22 / 1.26 | 0.236 | 0.49 | itself | 1 / 3 / 3 | +0.02, ×1.58, equal | 0.71 |
| s2-21-goetter-lax-oe | œ | 0.545 | 0.97 / 1.48 / 0.94 / 0.60 | 0.329 | 0.63 | itself | 2 / 2 / 3 | +0.00, ×1.98, equal | 0.67 |
| s2-22-ehe-schwa | ə | 0.702 | 1.26 / 1.86 / 0.98 / 1.34 | 0.476 | 0.90 | s2-16-bett-lax-ae | 1 / 0 / 1 | -0.02, ×3.63, equal | 0.65 |


## 2. The tongue shapes and the side cavities (T-features, T-anatomy)

Checked by geometry on the presets (male template), with the lines of the plan; the same checks run
in `cargo test` (`the_presets_make_the_tongue_shapes_the_research_lists`).

| Preset | Feature | Measured | Line | Result |
|---|---|---|---|---|
| flat | no concavity along the tongue | largest surface concavity 0.000 cm | ≤ 0.05 cm | pass |
| ə | piriform fossa volume | 1.27 cm³ each | 1.0–1.4 cm³ each (male; scaled by size³) [R12] | pass |
| ə | piriform fossa depth | 1.79 cm | 1.6–2.0 cm (male; scaled by size) [R12] | pass |
| ə | valleculae beside the median fold | 0.80 cm deep at z = 0.5 cm, 0.36 cm over the fold | ≥ 0.5 cm (scaled by size) beside the fold, the fold at most 60 % of that | pass |
| s | narrow single channel | 0.115 cm², 1 channel(s) at s/L 0.90 | ≤ 0.20 cm², 1 channel, s/L 0.85–0.95 (encloses both DVTD speakers' /s/) | pass |
| s | grooved (concave) tongue behind the channel | groove index 0.54 cm | > 0 (concave lower outline) | pass |
| ʃ | constriction behind /s/ | 0.062 cm² at s/L 0.77 (/s/ at 0.90) | ≤ 0.15 cm², further back than /s/ (as in both DVTD speakers) | pass |
| ʃ | long thin channel behind it | 6 cuts within 2.5 cm behind, under 0.6 cm high | ≥ 4 (≥ 1 cm of thin channel) | pass |
| ʃ | domed blade without a groove | largest surface concavity 0.00 cm over the blade | ≤ 0.05 cm | pass |
| l | two lateral channels at the tip contact | 2 channels at s/L 0.81, 2.00 cm apart | 2 channels, 2.0–2.8 cm apart (encloses both DVTD speakers' /l/) | pass |
| l | channel areas follow sideAsym | right/left 0.43/0.40 cm² with sideAsym +0.15 | right > left when sideAsym > 0 | pass |
| funnel | groove narrows and deepens towards the tip | t 0.60: 1.26 wide, 0.24 deep; t 0.75: 0.99 wide, 0.28 deep; t 0.90: 0.74 wide, 0.34 deep | width falls and depth rises from t 0.6 to 0.9 | pass |
| cupped | margins raised above the midline | surface rise 0.40 cm | > 0.1 cm | pass |
| bunched | pharyngeal and palatal constrictions | 1.29 cm² at s/L 0.25, 0.39 cm² at s/L 0.58; 2.25 cm² between | both below 60 % of the area between them | pass |
| bunched | open along its whole length | narrowest cut 0.39 cm², 0 cut(s) without air | every cut ≥ 0.15 cm² (an approximant, no closure) | pass |
| bunched | large front cavity | 14.0 cm³ in front of the palatal constriction | ≥ 10 cm³ | pass |
| bunched | tip low | tip 0.95 cm below the upper incisors' edge | ≥ 0.2 cm below | pass |
| retroflex | air under the raised tip (sublingual) | 3.83 cm of air below the tip in the midsagittal plane | ≥ 0.3 cm | pass |
| retroflex | palatal constriction at the curled tip | 0.61 cm² at s/L 0.79 | ≤ 0.8 cm² at s/L 0.7–0.92 | pass |

The presets are demonstration postures, tuned with `chimera tract3d tune` until their checks pass;
none is fitted to MRI (the /ʃ/ preset's channel is tighter than either speaker's). The bunched
/ɹ/ is open along its whole length since the review (its narrowest cut 0.39 cm², where the
reviewed preset had nine cuts without air), and the check refuses a closure; its root is drawn
further back since the slice fix, for its pharyngeal constriction in the ring. The female and
child presets do not pass every check: the adult lines do not scale with them (the /l/
channels' distance, /s/ narrowing to more than one channel, the child's smaller front cavity).
They are reported by `chimera tract3d presets --body`; the tests assert the male lines only.

Piriform fossae: 1.27 cm³ each and 1.79 cm deep in the default male model, measured from the
model's own pockets, within the 1.0–1.4 cm³ for each cavity and 1.6–2.0 cm of four speakers'
volumetric MRI ([doi:10.1121/1.417990](https://doi.org/10.1121/1.417990); the abstract gives
the pair's 2.1–2.9 cm³, the report version of the paper each cavity's: research.md §2.1). The
first build set 2.63 cm³ for each, the pair's figure, and the MRI fits drove the fossae to the
parameters' bounds. Valleculae: 0.80 cm deep beside the median glossoepiglottic fold, which
stands in them as a ridge (0.36 cm deep over it).

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
bunched speakers show (two constrictions with the tract open between and through them, tip low, front cavity of 14.6 cm³, root drawn back); the
retroflex preset what the tip-up speakers show (air under the tip, the constriction at the
curled tip). Not established: where each speaker's constriction lies (needs registration of the
incisors and palate), the coronal outline of the tongue body in /ɹ/, and any area.

## 4. Cost

Native release build, one male model, 65 laminae, 73 cuts: median 1.90 ms per compute over the
16 presets (50 calls each), slowest preset 3.14 ms; the display meshes 0.86 ms for 9972
vertices, measured while the machine's load average was about 6. Design target ≤ 5 ms per
compute: met. (Before the slice fix: 1.51 ms and 2.58 ms; the ring's polygon, the fossae's
and valleculae's outlines and the folds are the difference.) In the browser every screen asks
the engine for the posture's view each time the posture changes (`tract3dView`: positions as
a float buffer, without JSON); its timing is in
[model-design.md §0.2](model-design.md#02-integration-2026-11-10-one-head-and-tract-on-every-screen)
and was not measured again.

## 5. Critique: where the model departs from MRI and why

Classes as in [validation-plan.md §9](validation-plan.md).

- **Anatomy, fixed per speaker: the larynx region.** With the fossae at the published size
  for one cavity (they were built at the pair's) the two references agree better, and the
  model's region count is within one of the MRI's on 44/47 and 86/93 of the cuts where the
  MRI splits. The fit still ends with small fossae: the male's 0.30 cm wide (its bound) and
  1.52 cm deep, the female's 0.74 cm wide and 1.00 cm deep (its bound); the male's
  hypopharyngeal ring is fitted 2.78 cm in half-width and the laryngeal tube at 1.35 times
  the voice's area (0.77 times for the female). What the fit cannot move: how long the
  laryngeal tube is (the engine's, shared with the voice; the male speaker's three channels
  stay apart higher than it reaches, which the folds' height only partly answers), and how
  much room the larynx has front to back (the tube's rim is therefore wide, and the cut at
  the hypopharynx's floor too wide for its depth: §1).
- **Shape: thin tips, tooth blocks, the lip end.** The pharynx's lumen ends sideways where
  the ring meets the tongue's root or the epiglottis; at a shallow angle that is a tip a
  millimetre or two thick, which no smoothed 1.2 mm segmentation shows. In the mouth the tooth
  rows are blocks with flat occlusal and side faces, the incisors' rows rise into place in
  steps 2.5–4.5 mm off the midline, and the lips' tube ends on a plane just in front of the
  lips: the cuts at the teeth and lips keep straight edges and notches (cut IoU 0.55 and 0.58
  at 0.9 of the length against 0.74 between the speakers). These three are the next things
  to model if the cut shape is to reach the two speakers' own difference.
- **Lip rounding.** `lipRound` and the resonance term keep the rounded vowels' F1 within 10 %
  in 15 of 16 cases (the female /ɔ/ is at +14 %); `derive` now sets the rounding from the
  engine's own lip articulators, so the presets and every live gesture round the 3D lips.
- **A freedom since the slice fix: the pharynx's width.** The lumen's width now follows its
  depth (the ring closes on the tongue), and each sound's fit may draw the ring's sides in
  (`lateralSqueeze`: fitted near 0 for the male's close vowels, 0.5 to 0.9 for the open
  vowels and /x/). The model's depth in the oropharynx is still the engine's midsagittal gap, bounded
  by its tongue-root range.
- **Reference artefact and lip geometry: /f/.** Both speakers' /f/ is the worst fit (d 1.05 and
  1.02). The DVTD authors note that smoothing widened the /f/ constriction of their meshes
  ([doi:10.1038/s41597-020-00597-w](https://doi.org/10.1038/s41597-020-00597-w)); the model's
  labiodental closure leaves the lip end shut in the voxels (its last cuts hold no air), so its
  1D resonances are those of a closed tube.
- **Resolution: sibilants.** Unconstrained, an earlier fit narrowed the male /s/ and /ʃ/ and the
  female /ʃ/ to slits about 1 mm thick, which a pipeline at this resolution cannot follow; with
  the 1.5 mm limit (now on the true hydraulic thickness: the perimeter was a staircase 28 % long
  on a circle) the fitted channels are wider than the MRI's (the male /ʃ/ 5.2 times at its
  narrowest in this run). The model also makes sub-mm slivers between a tongue margin and the teeth that a
  smoothed segmentation does not have; both pipelines drop regions under 0.01 cm².
- **Derivation.** The default shapes come from the engine's targets, which were calibrated to
  formants through its 1D area law, without any geometric reference: median default d
  0.887 / 0.743, itemised per sound and per stretch of the tract in §1. Fitting `derive`'s
  constants moves the median by four hundredths at most, with constants at their bounds: what
  limits the defaults is the midline targets; they move with the sound's port.
- **Speaker variation.** The two DVTD speakers differ by a median scaled d of 0.73 for the same
  sound, and their cuts by IoU 0.71; the fitted model's cuts reach IoU 0.65 / 0.66 against the
  speaker it was fitted to (at the pharynx's stations as much as the other speaker's, or
  more: §1). The plan's fit lines (0.30 / 0.45) ask for a model closer to one speaker than two
  sounds of that speaker are to each other; in area function this model meets the weaker
  standard of being closer to each speaker than the other speaker is.

Known limits of the references: one male and one female German speaker; sounds sustained for
14 s in the scanner; smoothed segmentations without the nasal cavity; the 1D reduction itself sits
6.5–8.6 % above the 3D prints' measured resonances (research.md §5). USC /ɹ/: unsegmented, 7 s,
unregistered here.

## 6. Not established

- The area-function lines of T-fit (median d ≤ 0.30, worst ≤ 0.45), the identification (19 of
  22 each against ≥ 20), "every resonance within 10 %" (7 of 44 sounds are outside it), the
  lower-pharynx line on every cut (9 sounds miss it on one or two cuts), the cut shape at the
  two speakers' own difference, and T-default (median ≤ 0.71).
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
