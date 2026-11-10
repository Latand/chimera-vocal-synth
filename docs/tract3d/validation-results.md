# Validation results

*Measured 2026-11-10 on the model after the second slice fix (model-design.md §0.4), by
`chimera tract3d validate` with 3000 fit iterations per sound and 1000 for each speaker's
anatomy (395 s on the shared machine, three jobs). How the comparison works:
[validation-plan.md](validation-plan.md); commands: [tools/tract3d](../../tools/tract3d/README.md).
Sections 3, 7 and 8 keep their own dates.*

**In short.** The parametric model reproduces the 44 MRI vocal tracts of the
[Dresden Vocal Tract Dataset](https://doi.org/10.1038/s41597-020-00597-w) (DVTD; two German
speakers, 22 sounds each) with a median area-function distance d of 0.457 (male) and 0.416
(female), below the distance between the two speakers themselves (median 0.73), and with the
first three resonances of the fitted area functions a median 2.1 % and 1.3 % from the MRI's
(same 1D solver both sides). Every one of the 44 fits is measured by the voxel pipeline, and
the fast slicer agrees with the model's own voxels (area within 0.8 % on every sound, channel
counts equal on a median 99 % and 97 % of cuts). The cuts' shape is now as close to each
speaker as the other speaker's cuts are: median cut IoU 0.71 and 0.72 against 0.71 between
the speakers (the male's 0.706 against 0.708, the same to two places and a hair under it),
aspect error 0.34 and 0.33 against 0.43 (they were 0.65 and 0.66, 0.51 and 0.51). The price
was area: the fitted d rose from 0.445 and 0.376, and one sound fewer of the male speaker
and two fewer of the female are identified. **Not met**: the plan's area-function lines for
a fit (median d ≤ 0.30, every sound ≤ 0.45), the identification (18 and 17 of 22 against
≥ 20), "every resonance within 10 %" (2 male and 4 female sounds are outside it), the
lower-pharynx line on every cut (42 of 47 and 87 of 93), the cut shape at two of the six
stations (the hypopharynx's floor of the male speaker and the front teeth of both), and the
default shapes (the engine's own targets with `derive`: median d 0.908 / 0.751 against
0.71). The reasons, cut by cut and sound by sound, are below.

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

Each speaker's anatomy (the pharyngeal tube's half-widths and its taper's outline, dental
arches, palate vault, the laryngeal tube's size, the epiglottis' curl, the aryepiglottic
folds' height, the piriform fossae, tongue width: 14 values) is fitted once on
/ə aː iː uː s l/; then each sound's 13 articulators, 11 tongue parameters, the lips' rounding
and the pharynx's squeeze are fitted. The objective is the area-function distance, the
tract's length, the cuts' proportions at six stations (weight 0.15 for a cut twice too wide
at all six; it was 0.05, at which the fit all but ignored them), a penalty for openings
thinner than 1.5 mm (the reference pipeline traces its centreline on a 1 mm grid; the MRI
voxels are 1.2 mm), and the distance of F1–F3 from the reference's (weight 0.8 per unit of
log ratio: a 10 % error of one resonance costs as much as 0.08 of d). A fit whose voxels have
no connected path from glottis to lips is backed off towards its starting posture until they
have one; in this run none was.

| Test (plan §5) | Line | Male | Female | Result |
|---|---|---:|---:|---|
| Fitted d, median of 22 | ≤ 0.30 | 0.457 | 0.416 | fail |
| Fitted d, worst sound | ≤ 0.45 | 0.914 | 0.927 | fail |
| Sounds with fitted d ≤ 0.45 | all 22 | 9/22 | 12/22 | fail |
| Measured by the voxel pipeline | 22 | 22/22 | 22/22 | pass |
| Identification (nearest MRI sound is the target) | ≥ 20/22 | 18/22 | 17/22 | fail |
| F1–F3 of fitted vs MRI area function, median absolute error | ≤ 5 % | 2.1 % | 1.3 % | pass |
| F1–F3, every value | ≤ 10 % | 27 % | 40 % | fail (2 and 4 sounds outside) |
| Lower pharynx: regions within ±1 where MRI splits | every such cut | 42/47 | 87/93 | fail |
| Cut IoU, median over stations and sounds | ≥ 0.71 (between the two speakers) | 0.71 | 0.72 | at the line (male 0.706 against 0.708) and pass |
| Cut aspect error (absolute log2 ratio), median | ≤ 0.43 (between the two speakers) | 0.34 | 0.33 | pass |
| Default d (engine targets + derive), median | ≤ 0.71 | 0.908 | 0.751 | fail |
| Default d per sound ≤ max(1.0, between-speaker d) | all 22 | 19/22 | 19/22 | fail |
| Fast slicer vs the model's voxels, median area difference (T-consistency) | ≤ 3 % | 0.5 % | 0.6 % | pass |
| … worst sound | ≤ 3 % | 0.7 % | 0.8 % | pass |
| … channel count equal, median over sounds (lowest sound) | ≥ 95 % | 99 % (95 %) | 97 % (92 %) | pass |

- **male**: fitted length error median 3.5 %, volume error median 7.3 %; F1/F2/F3 median
  absolute error 1.2 / 2.8 / 2.5 %; headline set median fitted d 0.448, sounds with fitted
  d ≤ 0.30: none.
- **female**: fitted length error median 1.6 %, volume error median 3.0 %; F1/F2/F3 median
  absolute error 1.3 / 1.5 / 1.4 %; headline set median fitted d 0.495, sounds with fitted
  d ≤ 0.30: 5.

**The search is repeatable and touchy.** Two runs of one binary give the same table to the
last digit (checked in this round). A small change of geometry gives another: the fit
searches 26 values per sound and 14 per speaker (Nelder–Mead with restarts), each speaker's
anatomy is fitted first, and where that search ends moves all 22 of the speaker's sounds.
Over the four geometries measured in this round (they differ in the tooth rows, the cheek
between them and the fossae's lowest depth; the first was fitted with the old weight on the
cuts' proportions) the medians ranged: fitted d 0.46–0.51 (male)
and 0.35–0.47 (female); cut IoU 0.67–0.71 and 0.69–0.72; aspect error 0.29–0.42 and
0.32–0.37; lower pharynx 41–45 of 47 and 87–93 of 93; default d 0.86–0.91 and 0.74–0.78.
The table is the run on the final geometry.

### What the second slice fix changed, and what it cost

The slice-critic's second round judged the cuts' forms from its own run: flat floors,
right-angle steps, and a pharynx upside down against MRI (the MRI's cuts lie flat and wide on
the posterior wall with their belly towards the tongue; the model's ring bulged at the back
over a tongue root flat across). The geometry that answers it is model-design.md §0.4. The
same command on both:

| | After the first slice fix | After the second |
|---|---|---|
| Cut IoU, median (male / female; 0.71 between the speakers) | 0.65 / 0.66 | 0.71 / 0.72 |
| Cut aspect error, median (0.43 between the speakers) | 0.51 / 0.51 | 0.34 / 0.33 |
| Fitted d, median | 0.445 / 0.376 | 0.457 / 0.416 |
| Sounds with fitted d ≤ 0.45 | 12 / 13 | 9 / 12 |
| Identified | 19 / 19 | 18 / 17 |
| F1–F3 error of the fits, median | 2.1 / 2.7 % | 2.1 / 1.3 % |
| Sounds with a resonance over 10 % off | 3 / 4 | 2 / 4 |
| Lower pharynx: regions within ±1 where MRI splits | 44/47, 86/93 | 42/47, 87/93 |
| Default d, median | 0.887 / 0.743 | 0.908 / 0.751 |
| Channel agreement with the model's voxels, median | 99 / 99 % | 99 / 97 % |

The cuts station by station (median over the 22 sounds; in brackets the same measure between
the two speakers' own cuts):

| Station (share of the length) | Male: cut IoU, before → after | Male: aspect error | Female: cut IoU | Female: aspect error |
|---|---|---|---|---|
| 0.15, the hypopharynx's floor | 0.49 → 0.53 (0.72) | 1.14 → 1.06 (0.33) | 0.65 → 0.77 (0.72) | 0.63 → 0.52 (0.33) |
| 0.3, the lower pharynx | 0.68 → 0.75 (0.65) | 0.52 → 0.22 (0.58) | 0.65 → 0.79 (0.65) | 0.43 → 0.20 (0.58) |
| 0.45, the upper pharynx | 0.78 → 0.77 (0.65) | 0.23 → 0.25 (0.71) | 0.55 → 0.69 (0.65) | 0.70 → 0.49 (0.71) |
| 0.6, under the soft palate | 0.68 → 0.72 (0.71) | 0.58 → 0.34 (0.41) | 0.68 → 0.77 (0.72) | 0.62 → 0.14 (0.41) |
| 0.75, the mouth | 0.68 → 0.75 (0.71) | 0.35 → 0.24 (0.30) | 0.69 → 0.72 (0.71) | 0.30 → 0.26 (0.30) |
| 0.9, the front teeth and lips | 0.55 → 0.48 (0.74) | 0.54 → 0.58 (0.42) | 0.58 → 0.59 (0.74) | 0.49 → 0.45 (0.42) |

- **From the lower pharynx to the mouth** (0.3 to 0.75) the model's cuts are now as close to
  each speaker as the other speaker's are, or closer, in both measures and for both
  speakers: the flat posterior wall with the taper on the tongue's side, the arches under
  the soft palate, the loaf of a tongue under the hard palate.
- **The hypopharynx's floor of the male speaker** (0.15) is still twice too wide for its
  depth. His lower pharynx is deeper front to back than the engine's larynx and epiglottis
  leave room for, so his anatomy's fit takes the width instead (the tube's half-width there
  is fitted at 2.71 cm, the laryngeal tube at one and a half times the voice's area) and
  stands shallow fossae far out to the sides (0.74 cm deep; the default is 1.8). The depth is
  the midline's, which is the engine's.
- **The front teeth and lips** (0.9) did not gain, and the male's cuts lost there. The cut
  runs along the tooth rows' faces and keeps their straight edges, and the lips' tube still
  ends on a plane just in front of the lips.
- **What it cost in area.** The fitted d is higher by 0.01 (male) and 0.04 (female), and
  three fewer sounds are identified. Two things changed together and this round did not
  separate them: the fit weighs the cuts' proportions three times as much, and a lumen that
  ends sideways in a taper has less freedom to trade width for depth than a ring that
  closed on a flat tongue. The anatomy fit's own objective (mean over its six sounds) ended
  at 0.59 (male) and 0.53 (female).
- **Ruled out in this round's runs**: a lowest fossa depth of 0.4 cm in place of 0.1, tried
  on the round's third geometry (the same table to the last digit; in the final run too,
  neither speaker's fit went below 0.4 cm).

### What the first slice fix changed, and what it bought

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
  And, as an eighth run, walls that touch where they come within 2 mm of the tongue or the
  epiglottis off the midline (blunt ends in place of the tips): cut IoU 0.64 and 0.66, aspect
  error 0.45 and 0.48, fitted d 0.467 and 0.400. Inside the spread as well; not adopted.
- **The female fit** gained most (median d 0.447 → 0.376, five sounds under 0.30, three more
  identified): her pharynx is narrow for its depth and her three lower channels lie where the
  model's now are.
- **The defaults** did not come nearer the line (next section).

### The default shapes, sound by sound

The default posture of a sound is the engine's own target for the nearest symbol with
`derive`'s 3D shape and the speaker's fitted anatomy. The per-sound table gives, for each, the
default model's area against the MRI's over four stretches of the tract (geometric mean of the
ratio: larynx to 0.2 of the length, pharynx to 0.55, mouth to 0.9, lips). What it shows:

- **Open and mid front vowels** (/a/, /ɛ/, /e/): the pharynx is 1.1 to 2.3 times the MRI's
  (both speakers' lax /a/ the most), the mouth 2.0 to 2.4 times for /e/ and the female /i/.
  The engine's targets for these keep a deeper pharynx than either speaker has.
- **Back rounded vowels** (/o/, /ɔ/): the male speaker's pharynx is two fifths to a half of
  the MRI's (default d 1.29 and 1.06; his /o/ is over its line). The engine's targets draw
  the tongue's root far back for them, and `derive` draws the side walls in on top of that.
- **Close rounded vowels of the male speaker** (/u/, /y/, /ʊ/, /ø/): nearly every stretch is
  0.45 to 0.85 of the MRI's (his /y/ is over its line at 1.18): his rounded vowels are larger
  than the engine's almost everywhere.
- **/x/**: the pharynx 2.5 to 3.2 times the MRI's and the mouth a third to under a half of
  it (default d 1.45 and 1.21, over the line for both speakers): the engine's velar closure is
  tighter and further forward than the sustained /x/ of either speaker.
- **/ç/, /ʃ/, /s/, /f/**: the mouth 0.5 to 0.65 of the MRI's for /ç/ and 2.4 to 2.6 times
  for /ʃ/; the lips 2.8 to 3.2 times for /s/ and a quarter to a third for /f/ (the engine's
  labiodental closure against the MRI's opening beside the teeth). The female /s/ is over
  its line (1.04), and so is her /ʊ/ (1.21: the lips a fifth of the MRI's).

`derive`'s ten constants were fitted on the first slice fix's geometry, to the median default
d over all 44 sounds (the fast slicer, the fitted anatomies of an earlier run): it moved from
0.823 to 0.783, with most constants at their bounds; fitting only the lips' rounding and the
pharynx's squeeze gave 0.787 with the squeeze at 1.17 and the rounding at 0.21. The values in
the code are a rounding of 1.0 and a squeeze of 0.9, and the eight older constants as they
were. They were not fitted again on this geometry, where the squeeze takes up to half the
tube's width (it took 70 % of the ring's): the median default d moved by two hundredths at
most (0.887 → 0.908, 0.743 → 0.751).
What keeps the defaults from the line is the engine's midline targets, calibrated to formants
through its 1D area law without any geometric reference; they move when the sound is computed
from this geometry.

**Still outside the 10 % resonance line**, with what the run's own files show of each:

- both /f/ (−27 % and −40 % on F1): the fitted area function ends in a closure (its last cuts
  hold no air) where the MRI has an opening beside the teeth.
- the female /ʃ/ (F1 +12 %): her fit is larger than the MRI everywhere behind the lips (1.3
  times in the larynx and the pharynx, 1.5 times in the mouth), its narrowest cut is in three
  channels where the MRI's is in one, and it is the second worst of her 22 (d 0.84).
- the male /ø/ (F3 +14 %): the fitted tract is 9 % shorter than the MRI's, and its lip end
  holds 0.4 of the MRI's area.
- the female /o/ (F3 −10 %) and /ʊ/ (F1 +16 %, F3 −11 %): the fit's narrowest cut lies a
  tenth and a twelfth of the length in front of the MRI's and is one channel where the MRI's
  is two. (Read from the cuts, without a test: the MRI's two channels pass either side of
  the uvula, and the fit closes on the palate in front of its own.)

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
| s1-01-bahn-tense-a | a | 0.836 | 1.13 / 2.09 / 1.39 / 1.61 | 0.534 | 0.64 | s1-14-ach-x | -0 / 1 / 1 | -0.12, ×1.59, equal | 0.62 |
| s1-02-beet-tense-e | e | 0.808 | 0.93 / 1.18 / 2.02 / 1.40 | 0.371 | 0.47 | itself | -0 / 2 / -1 | -0.01, ×1.11, equal | 0.64 |
| s1-03-tiere-tense-i | i | 0.669 | 0.68 / 0.68 / 0.84 / 0.98 | 0.351 | 0.47 | itself | -1 / 3 / -2 | +0.02, ×1.24, equal | 0.68 |
| s1-04-boote-tense-o | o | 1.294 | 0.57 / 0.39 / 0.90 / 0.45 | 0.770 | 0.78 | itself | -3 / -1 / 5 | -0.04, ×0.27, +1 | 0.68 |
| s1-05-bude-tense-u | u | 1.105 | 0.63 / 0.59 / 0.65 / 0.64 | 0.914 | 0.93 | itself | -1 / 4 / 4 | +0.03, ×0.41, equal | 0.66 |
| s1-06-laehmung-tense-ae | ɛ | 0.875 | 1.50 / 1.73 / 1.93 / 1.26 | 0.444 | 0.64 | itself | -0 / 0 / -5 | +0.56, ×1.85, equal | 0.65 |
| s1-07-hoehle-tense-oe | ø | 0.749 | 0.73 / 0.67 / 0.75 / 0.51 | 0.703 | 0.71 | itself | -6 / -1 / 14 | -0.06, ×0.27, equal | 0.73 |
| s1-08-guete-tense-y | y | 1.185 | 0.52 / 0.53 / 0.49 / 0.86 | 0.837 | 0.77 | itself | -0 / 6 / 9 | -0.05, ×0.78, +2 | 0.68 |
| s1-09-los-l | l | 0.793 | 0.67 / 0.59 / 1.19 / 1.04 | 0.439 | 0.73 | itself | 6 / 5 / 3 | +0.03, ×0.36, equal | 0.57 |
| s1-10-fahrt-f | f | 1.032 | 0.98 / 0.74 / 1.07 / 0.35 | 0.854 | 0.82 | s1-20-muetter-lax-y | -27 / -15 / -2 | +0.04, ×0.00, -2 | 0.73 |
| s1-11-bass-s | s | 0.970 | 1.00 / 0.79 / 0.91 / 3.15 | 0.457 | 0.73 | itself | -2 / 5 / -0 | -0.00, ×0.37, +1 | 0.62 |
| s1-12-schoen-sh | ʃ | 1.339 | 0.97 / 0.60 / 2.62 / 0.28 | 0.456 | 1.45 | itself | 5 / 9 / 0 | +0.03, ×1.04, equal | 0.70 |
| s1-13-ich-c | ç | 0.973 | 1.02 / 0.93 / 0.65 / 0.85 | 0.324 | 0.91 | itself | -1 / 2 / 3 | +0.00, ×1.21, equal | 0.68 |
| s1-14-ach-x | x | 1.451 | 1.65 / 3.16 / 0.35 / 0.54 | 0.415 | 0.56 | itself | -2 / 1 / -3 | -0.04, ×1.07, equal | 0.68 |
| s1-15-bass-lax-a | a | 0.859 | 1.45 / 2.27 / 0.98 / 1.55 | 0.606 | 0.49 | s1-14-ach-x | 1 / 4 / 1 | -0.12, ×1.14, equal | 0.58 |
| s1-16-bett-lax-ae | ɛ | 0.941 | 1.48 / 1.93 / 1.78 / 1.34 | 0.503 | 0.56 | itself | 1 / 1 / -2 | +0.57, ×2.21, equal | 0.71 |
| s1-17-mit-lax-i | ɪ | 0.696 | 0.76 / 1.17 / 0.76 / 1.04 | 0.447 | 0.77 | itself | 1 / 3 / 2 | -0.02, ×0.97, equal | 0.64 |
| s1-18-offen-lax-o | ɔ | 1.060 | 0.62 / 0.46 / 0.62 / 1.20 | 0.657 | 0.97 | itself | 0 / 4 / 2 | +0.48, ×0.80, +1 | 0.59 |
| s1-19-butter-lax-u | ʊ | 1.120 | 0.50 / 0.85 / 0.46 / 0.54 | 0.905 | 0.83 | s1-04-boote-tense-o | 6 / 9 / 2 | +0.47, ×0.44, equal | 0.58 |
| s1-20-muetter-lax-y | ʏ | 0.559 | 0.79 / 0.82 / 0.92 / 1.40 | 0.451 | 0.49 | itself | 1 / 4 / 4 | -0.03, ×0.32, equal | 0.66 |
| s1-21-goetter-lax-oe | œ | 0.565 | 0.84 / 1.09 / 0.81 / 0.61 | 0.437 | 0.63 | itself | 2 / 2 / 6 | +0.97, ×1.04, equal | 0.69 |
| s1-22-ehe-schwa | ə | 0.718 | 1.16 / 0.96 / 1.85 / 1.15 | 0.352 | 0.90 | itself | -1 / 0 / -4 | +0.88, ×1.22, +2 | 0.66 |
| s2-01-bahn-tense-a | a | 0.651 | 1.07 / 1.74 / 1.06 / 1.03 | 0.483 | 0.64 | s2-15-bass-lax-a | -1 / 4 / -6 | +0.00, ×1.85, -2 | 0.57 |
| s2-02-beet-tense-e | e | 0.913 | 0.92 / 1.08 / 2.30 / 1.82 | 0.301 | 0.47 | itself | -0 / -0 / 3 | +0.00, ×1.28, equal | 0.68 |
| s2-03-tiere-tense-i | i | 0.951 | 1.05 / 0.98 / 2.42 / 1.85 | 0.272 | 0.47 | itself | -1 / -0 / -1 | -0.05, ×0.89, equal | 0.72 |
| s2-04-boote-tense-o | o | 0.795 | 0.70 / 1.32 / 0.82 / 0.64 | 0.643 | 0.78 | itself | 2 / -1 / -10 | +0.10, ×0.70, -1 | 0.75 |
| s2-05-bude-tense-u | u | 0.745 | 0.72 / 1.09 / 0.69 / 0.71 | 0.508 | 0.93 | itself | 3 / 8 / 1 | -0.41, ×0.58, -2 | 0.74 |
| s2-06-laehmung-tense-ae | ɛ | 0.586 | 1.40 / 1.40 / 1.27 / 0.96 | 0.419 | 0.64 | s2-16-bett-lax-ae | -5 / 1 / 1 | +0.02, ×2.89, equal | 0.72 |
| s2-07-hoehle-tense-oe | ø | 0.624 | 1.02 / 1.12 / 0.66 / 0.99 | 0.414 | 0.71 | itself | -2 / -0 / 1 | +0.90, ×1.38, equal | 0.74 |
| s2-08-guete-tense-y | y | 0.756 | 0.98 / 0.89 / 0.53 / 1.04 | 0.347 | 0.77 | itself | 2 / 3 / -0 | -0.03, ×0.81, +2 | 0.76 |
| s2-09-los-l | l | 0.711 | 0.83 / 0.86 / 1.03 / 1.74 | 0.324 | 0.73 | itself | 1 / 2 / 1 | +0.01, ×0.56, +1 | 0.63 |
| s2-10-fahrt-f | f | 1.391 | 1.09 / 2.22 / 2.13 / 0.26 | 0.927 | 0.82 | itself | -40 / -19 / -2 | +0.95, ×0.00, -1 | 0.61 |
| s2-11-bass-s | s | 1.038 | 1.14 / 1.38 / 0.91 / 2.77 | 0.573 | 0.73 | s2-09-los-l | 1 / 1 / 1 | -0.03, ×0.34, +1 | 0.67 |
| s2-12-schoen-sh | ʃ | 1.297 | 1.29 / 1.63 / 2.43 / 0.83 | 0.837 | 1.43 | itself | 12 / 1 / -2 | -0.03, ×1.07, +2 | 0.74 |
| s2-13-ich-c | ç | 1.033 | 1.04 / 1.35 / 0.50 / 1.57 | 0.289 | 0.91 | itself | -0 / -1 / -1 | +0.02, ×1.02, +2 | 0.69 |
| s2-14-ach-x | x | 1.208 | 1.34 / 2.52 / 0.45 / 0.71 | 0.379 | 0.56 | itself | 1 / 4 / 3 | +0.02, ×1.69, -1 | 0.68 |
| s2-15-bass-lax-a | a | 0.691 | 1.03 / 2.04 / 0.97 / 1.25 | 0.304 | 0.49 | itself | -0 / 3 / -2 | +0.29, ×1.17, -1 | 0.54 |
| s2-16-bett-lax-ae | ɛ | 0.551 | 0.89 / 1.33 / 1.36 / 1.52 | 0.280 | 0.56 | itself | 1 / 1 / 3 | +0.00, ×2.20, equal | 0.75 |
| s2-17-mit-lax-i | ɪ | 0.459 | 0.73 / 0.98 / 0.91 / 0.99 | 0.287 | 0.77 | itself | 1 / 4 / -1 | +0.00, ×1.41, -1 | 0.67 |
| s2-18-offen-lax-o | ɔ | 0.870 | 0.45 / 0.81 / 0.63 / 0.59 | 0.619 | 0.97 | itself | 1 / 2 / -2 | -0.02, ×0.66, equal | 0.73 |
| s2-19-butter-lax-u | ʊ | 1.210 | 0.70 / 1.45 / 0.61 / 0.20 | 0.599 | 0.83 | itself | 16 / 8 / -11 | +0.08, ×0.90, -1 | 0.71 |
| s2-20-muetter-lax-y | ʏ | 0.465 | 0.99 / 0.92 / 1.37 / 1.25 | 0.243 | 0.49 | itself | 1 / 1 / 0 | +0.02, ×1.72, equal | 0.78 |
| s2-21-goetter-lax-oe | œ | 0.644 | 0.89 / 1.40 / 1.04 / 0.52 | 0.476 | 0.63 | s2-16-bett-lax-ae | -0 / -2 / 4 | -0.00, ×2.10, equal | 0.75 |
| s2-22-ehe-schwa | ə | 0.664 | 1.12 / 1.66 / 1.11 / 1.16 | 0.537 | 0.90 | s2-16-bett-lax-ae | 1 / 1 / 1 | -0.02, ×3.91, equal | 0.68 |


## 2. The tongue shapes and the side cavities (T-features, T-anatomy)

Checked by geometry on the presets (male template), with the lines of the plan; the same checks run
in `cargo test` (`the_presets_make_the_tongue_shapes_the_research_lists`).

| Preset | Feature | Measured | Line | Result |
|---|---|---|---|---|
| flat | no concavity along the tongue | largest surface concavity 0.003 cm | ≤ 0.05 cm | pass |
| ə | piriform fossa volume | 1.03 cm³ each | 1.0–1.4 cm³ each (male; scaled by size³) [R12] | pass |
| ə | piriform fossa depth | 1.80 cm | 1.6–2.0 cm (male; scaled by size) [R12] | pass |
| ə | valleculae beside the median fold | 0.80 cm deep at z = 0.5 cm, 0.36 cm over the fold | ≥ 0.5 cm (scaled by size) beside the fold, the fold at most 60 % of that | pass |
| s | narrow single channel | 0.057 cm², 1 channel(s) at s/L 0.90 | 0.05–0.20 cm² (a fricative: narrow and open), 1 channel, s/L 0.85–0.95 (encloses both DVTD speakers' /s/) | pass |
| s | grooved (concave) tongue behind the channel | groove index 0.59 cm | > 0 (concave lower outline) | pass |
| ʃ | constriction behind /s/ | 0.054 cm² at s/L 0.77 (/s/ at 0.90) | 0.03–0.15 cm² (narrow and open), further back than /s/ (as in both DVTD speakers) | pass |
| ʃ | long thin channel behind it | 5 cuts within 2.5 cm behind, under 0.6 cm high | ≥ 4 (≥ 1 cm of thin channel) | pass |
| ʃ | domed blade without a groove | largest surface concavity -0.00 cm over the blade | ≤ 0.05 cm | pass |
| l | two lateral channels at the tip contact | 2 channels at s/L 0.82, 2.00 cm apart | 2 channels, 2.0–2.8 cm apart (encloses both DVTD speakers' /l/) | pass |
| l | channel areas follow sideAsym | right/left 0.36/0.35 cm² with sideAsym +0.15 | right > left when sideAsym > 0 | pass |
| funnel | groove narrows and deepens towards the tip | t 0.60: 1.26 wide, 0.12 deep; t 0.75: 0.99 wide, 0.14 deep; t 0.90: 0.74 wide, 0.24 deep | width falls and depth rises from t 0.6 to 0.9 | pass |
| cupped | margins raised above the midline | surface rise 0.23 cm | > 0.1 cm | pass |
| bunched | pharyngeal and palatal constrictions | 1.58 cm² at s/L 0.25, 0.38 cm² at s/L 0.61; 2.86 cm² between | both below 60 % of the area between them | pass |
| bunched | open along its whole length | narrowest cut 0.38 cm², 0 cut(s) without air | every cut ≥ 0.15 cm² (an approximant, no closure) | pass |
| bunched | large front cavity | 13.2 cm³ in front of the palatal constriction | ≥ 10 cm³ | pass |
| bunched | tip low | tip 0.95 cm below the upper incisors' edge | ≥ 0.2 cm below | pass |
| retroflex | air under the raised tip (sublingual) | 3.64 cm of air below the tip in the midsagittal plane | ≥ 0.3 cm | pass |
| retroflex | palatal constriction at the curled tip | 0.24 cm² at s/L 0.78 | 0.15–0.8 cm² at s/L 0.7–0.92 (an approximant: no closure) | pass |

The presets are demonstration postures, tuned with `chimera tract3d tune` until their checks pass;
none is fitted to MRI (the /ʃ/ preset's channel is tighter than either speaker's). The bunched
/ɹ/ is open along its whole length since the review (its narrowest cut 0.38 cm², where the
reviewed preset had nine cuts without air), and the check refuses a closure; its root is drawn
further back since the first slice fix, for its pharyngeal constriction. Since the second,
the tongue's front is round and fills the lower arch, so the air of /s/, /ʃ/ and the
retroflex /ɹ/ no longer passes beside the tip (it did, unnoticed: the three presets closed
when the wedge went). /s/ has a deeper groove, /ʃ/ and the retroflex a lower tip, and their
checks now refuse a closure too (at least 0.05, 0.03 and 0.15 cm²). The female and
child presets do not pass every check: the adult lines do not scale with them (the /l/
channels' distance, /s/ narrowing to more than one channel, the child's smaller front cavity).
They are reported by `chimera tract3d presets --body`; the tests assert the male lines only.

Piriform fossae: 1.03 cm³ each and 1.80 cm deep in the default male model, measured from the
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
bunched speakers show (two constrictions with the tract open between and through them, tip low, front cavity of 13.2 cm³, root drawn back); the
retroflex preset what the tip-up speakers show (air under the tip, the constriction at the
curled tip). Not established: where each speaker's constriction lies (needs registration of the
incisors and palate), the coronal outline of the tongue body in /ɹ/, and any area.

## 4. Cost

Native release build, one male model, 65 laminae, 73 cuts, three runs each while the
machine's load average was about 10: median 2.2 to 2.3 ms per compute over the 16 presets
(50 calls each), slowest preset 3.8 to 4.1 ms; the display meshes 1.73 to 1.76 ms for 11 732
vertices. The head before this round, in the same minutes: 1.95 to 2.07 ms and 3.4 ms, the
meshes 0.98 ms for 9 972 vertices. Design target ≤ 5 ms per compute: met. The meshes cost
nearly twice what they did: a closed tongue, a soft palate with its thickness and arches,
the tube's side walls and the hypopharynx's floor are new surfaces, and each point of the
tongue and of the ring costs two powers more. (They cost 4.2 ms before the floor's reach
was searched once a spoke, each tooth's frame tabulated and the tongue's powers computed
only where their rule acts.)

In the browser every screen asks the engine for the posture's view each time the posture
changes (`tract3dView`: positions as a float buffer, without JSON). Headless Chromium on
the same machine, production build, 20 calls (`tools/tract3d/browser-bench.ts`): the view
call a median 2.5 ms, 3.1 ms at the 95th percentile (1.3 ms at the integration,
[model-design.md §0.2](model-design.md#02-integration-2026-11-10-one-head-and-tract-on-every-screen));
the lab's JSON call with cuts 2.9 ms, with meshes 11.2 ms (target ≤ 16 ms). Not measured: a
phone's processor and graphics.

## 5. Critique: where the model departs from MRI and why

Classes as in [validation-plan.md §9](validation-plan.md).

- **Anatomy, fixed per speaker: the larynx region.** The model's region count is within one
  of the MRI's on 42/47 and 87/93 of the cuts where the MRI splits (four male and six female
  sounds miss it, on one cut each but for the male /e/ on two). The two speakers' fits end
  far apart: the male's fossae 0.74 cm deep and 0.95 cm to either side, his hypopharynx
  2.71 cm in half-width, his laryngeal tube at one and a half times the voice's area and his
  aryepiglottic folds almost flat (0.08 cm); the female's fossae 1.17 cm deep, her
  hypopharynx 2.26 cm and her tube at 0.8 times. What the fit cannot move: how long the
  laryngeal tube is (the engine's, shared with the voice), and how much room the larynx and
  the epiglottis leave front to back. The male speaker's lower pharynx is deeper than that
  room, his fit takes the width instead, and the cut at his hypopharynx's floor stays twice
  too wide for its depth (§1: cut IoU 0.53 against 0.72 between the speakers, aspect error
  1.06 against 0.33).
- **Shape: the front teeth, the lip end.** From the lower pharynx to the mouth the cuts now
  end sideways in a taper and are as close to each speaker as the other speaker's (§1). At
  0.9 of the length they are where they were (cut IoU 0.48 and 0.59 against 0.74, aspect
  error 0.58 and 0.45 against 0.42): the cut runs along the tooth rows' faces, whose lingual
  and buccal sides are upright in it, and the lips' tube ends on a plane just in front of the
  lips. A curved lip end and crowns with their own lingual curve are the next things to model
  there.
- **Lip rounding.** `lipRound` and the resonance term keep the rounded vowels' F1 within 10 %
  in 15 of 16 cases (the female /ʊ/ is at +16 %); `derive` now sets the rounding from the
  engine's own lip articulators, so the presets and every live gesture round the 3D lips.
- **The pharynx's width.** The tube's half-width is each speaker's (fitted per level) and
  the lumen keeps most of it at any depth (the taper closes it at the tube's side), so a
  retracted tongue leaves a wide flat lens, as the MRI shows; each sound's fit may still draw
  the sides in (`lateralSqueeze`, up to half the width). The model's depth in the pharynx is
  still the engine's midsagittal gap, bounded by its tongue-root range and by where its
  epiglottis stands.
- **Reference artefact and lip geometry: /f/.** Both speakers' /f/ is among the worst fits (d 0.85
  and 0.93; the male /u/ and /ʊ/ are at 0.91). The DVTD authors note that smoothing widened the /f/ constriction of their meshes
  ([doi:10.1038/s41597-020-00597-w](https://doi.org/10.1038/s41597-020-00597-w)); the model's
  labiodental closure leaves the lip end shut in the voxels (its last cuts hold no air), so its
  1D resonances are those of a closed tube.
- **Resolution: sibilants.** Unconstrained, an earlier fit narrowed the male /s/ and /ʃ/ and the
  female /ʃ/ to slits about 1 mm thick, which a pipeline at this resolution cannot follow; with
  the 1.5 mm limit (now on the true hydraulic thickness: the perimeter was a staircase 28 % long
  on a circle) the fitted /ʃ/ channels are within a tenth of the MRI's at their narrowest in
  this run (1.04 and 1.07 times), and the fitted /s/ channels a third of it (0.37 and 0.34
  times: with a round tongue front the air of /s/ passes through the groove alone). The model
  also makes sub-mm slivers between a tongue margin and the teeth that a smoothed
  segmentation does not have; both pipelines drop regions under 0.01 cm².
- **Derivation.** The default shapes come from the engine's targets, which were calibrated to
  formants through its 1D area law, without any geometric reference: median default d
  0.908 / 0.751, itemised per sound and per stretch of the tract in §1. Fitting `derive`'s
  constants moves the median by four hundredths at most, with constants at their bounds: what
  limits the defaults is the midline targets; they move with the sound's port.
- **Speaker variation.** The two DVTD speakers differ by a median scaled d of 0.73 for the same
  sound, and their cuts by IoU 0.71 and an aspect error of 0.43; the fitted model's cuts reach
  IoU 0.71 / 0.72 and an aspect error of 0.34 / 0.33 against the speaker it was fitted to. The
  plan's fit lines (0.30 / 0.45) ask for a model closer to one speaker than two sounds of that
  speaker are to each other; in area function and now in cut shape this model meets the weaker
  standard of being as close to each speaker as the other speaker is.

Known limits of the references: one male and one female German speaker; sounds sustained for
14 s in the scanner; smoothed segmentations without the nasal cavity; the 1D reduction itself sits
6.5–8.6 % above the 3D prints' measured resonances (research.md §5). USC /ɹ/: unsegmented, 7 s,
unregistered here.

## 6. Not established

- The area-function lines of T-fit (median d ≤ 0.30, worst ≤ 0.45), the identification (18
  and 17 of 22 against ≥ 20), "every resonance within 10 %" (6 of 44 sounds are outside it),
  the lower-pharynx line on every cut (10 sounds miss it on one or two cuts), the cut shape
  at the hypopharynx's floor of the male speaker and at the front teeth of both, and
  T-default (median ≤ 0.71).
- Which of the two changes cost the fitted d its 0.01 and 0.04: the heavier weight on the
  cuts' proportions or the tapering lumen (no run took them apart).
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
