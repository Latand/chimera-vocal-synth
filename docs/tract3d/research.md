# Research: the human vocal tract in 3D

*Phase 1 research, 2026-11-09. Base revision 2e567e2.* What the anatomy and the literature say
about each part of the tract the 3D model needs, what the public MRI data showed when opened, and
what the engine does today. The datasets themselves are described in
[reference-data.md](reference-data.md); the model in [model-design.md](model-design.md).

**How to read the claims.** Every external fact carries a source key `[Rn]` (list at the end).
Most sources were read as abstracts; a claim that needed the full text says so. *Observed* marks
something measured here on downloaded data, with the script that measured it (the scripts live in
the local data folder, outside the repository; see reference-data.md). *Unverified* marks a claim
that was not confirmed from a primary source; nothing unverified is used as a number in the design.

## 1. What the engine does today

Read from the code at 2e567e2:

- **Geometry is a midsagittal section.** `crates/chimera/src/anatomy/human.rs` computes an outer
  wall (posterior pharyngeal wall, velum, hard palate, upper incisors, upper lip) and an inner wall
  (epiglottis, tongue root, tongue body circle, blade, tip, floor of mouth, lower incisors, lower
  lip) from 17 articulators (`HumanArtic`: jaw, tongue body x/y, tip x/y, tip curl, root, lip
  aperture, protrusion, width, labiodental, velum, larynx, epilarynx, ventricular, lateral,
  dorsum).
- **Area is a power law of one distance.** A semipolar grid (12 pharyngeal horizontals, 31 oral
  rays, 6 front verticals) measures the midsagittal gap `d`; each region turns it into area with
  `A = α·d^β` (pharynx 2.0/1.4, oral 2.6/1.4, front 2.0/1.4, ahead-of-tip 1.6/1.4), the lips with
  an elliptical slit. The epilarynx is a separate parametric profile (`epilarynx_area`).
- **Lateral shape is a patch.** `lateral` (for /l/) only puts a floor under the area near the tip
  (`max(A, lateral·0.35·k²)`); there is no groove articulator (the architecture document mentions
  one; the code has none).
- **The airway** (`voice/human_airway.rs`) resamples the area function into two uniform FDTD tubes
  (pharynx, oral), with a nasal branch and sinus, and a fixed piriform side tube (2 cm, 0.9 cm²).
- **The 3D view is decorative.** `src/ui/render/anatomy3d.ts` sweeps the midsagittal outlines
  sideways with fixed width tables (`TONGUE_W`, `TONGUE_H`) and draws the airway as ellipses whose
  area is the engine's area at each grid line. Nothing acoustic comes from it.
- **State reaches the UI** through the worklet's telemetry (the 17 articulators, the tube areas,
  pressures; `src/worklet/protocol.ts`) and through main-thread JSON calls into the same Rust code
  (`frame`, `apply`, `transfer` in `crates/chimera-wasm/src/api.rs`, wrapped by `src/wasm/engine.ts`
  and `src/ui/analysis.ts`).

So the tract's cross-sectional shape, which the operator wants to control, does not exist as
geometry today: one midsagittal distance stands for every shape.

## 2. Anatomy by part

### 2.1 Pharynx, side branches and larynx

- **Piriform fossae (sinuses).** Paired side branches at the bottom of the pharynx, lateral to the
  laryngeal vestibule. Volumetric MRI of four speakers gave 2.1–2.9 cm³ each and 1.6–2.0 cm depth;
  they put spectral troughs at 4–5 kHz and also lower the formants below them [R12]. 3D-printed
  tracts confirm the 4–5 kHz trough [R13]; area-function work found they lower formants
  significantly [R6]; 3D FDTD on MRI geometry found their effect the largest of the side cavities
  [R10].
  *Observed:* in the DVTD meshes, cuts perpendicular to the centreline 0.75–1.75 cm above the
  glottis split into **three** air regions (the epilarynx tube in the middle and one fossa on each
  side) for the female speaker in /s/, /ʃ/, /i/, /a/, /u/, /l/; the male speaker shows two regions
  at 1.25–2.75 cm (`dvtd_slices.py`, evidence 05).
- **Valleculae.** Pockets between the tongue root and the epiglottis. 3D FDTD found their spectral
  dips small and vowel-dependent [R10]; a 3D model study reports resonances and antiresonances in
  the 3–5 kHz region from both valleculae and piriform sinuses (record summary only, *unverified*
  in full text) [R14].
  *Observed:* in the male /ʃ/ the cut at 5.2 cm shows the epiglottis as a crescent of tissue
  inside the oropharynx, with air on both sides of it (evidence 06).
- **Epilarynx / laryngeal cavity.** The cavity above the folds acts as a quarter-wave resonator
  that generates one formant of the whole tract (the fourth, for one male speaker's Japanese
  vowels) [R11]. Its narrowing relative to the pharynx was the classic explanation of the singer's
  formant (title read; abstract not available, *unverified* detail) [R15].
- **Pharynx shape.** Cross-sectional area is not a single function of the midsagittal width: the
  1991 MRI study fitted area-versus-width relations per region exactly because of that [R5]. The
  pharynx is the region where speakers differ most for fricatives [R9 summary, *unverified* in
  full text].
  *Observed:* the oropharynx cut of male /a/ at 4.8 cm is an ellipse about 1 cm² with a midline
  constriction; male /i/ widens the oropharynx to 9–12 cm² (tongue root forward) (evidence 01–03).

### 2.2 Velum

The engine's velum is a flap with one degree of freedom (angle from the port area). A 3D
articulatory model used two velum parameters (shape and opening) because velum height varied
across vowels with the port closed [R16]. Lateral pharyngeal walls also move in velopharyngeal
closure (*unverified*: no primary source read for this).

### 2.3 Hard palate

Palate shape varies between speakers: with a flat palate the area changes more for a given tongue
movement than with a dome, and EPG speakers with flat palates showed less tongue-height
variability [R17]. The palate's vault (its depth and width) is therefore a per-speaker anatomy
parameter.
*Observed:* in the DVTD oral cuts the upper boundary is an arch and the air under it a dome-shaped
region (evidence 04, 06). The dataset includes the two speakers' dental-cast scans
(`s1-maxilla.stl`, `s2-maxilla.stl`), from which vault depth and arch width can be measured
(not measured in this stage).

### 2.4 Teeth and jaw

Teeth are invisible in MRI; the DVTD authors merged 3D scans of plaster dental casts into the
MRI before segmenting, because teeth matter acoustically [R4]. The French dataset added three
static postures (tongue on upper incisors, on lower incisors, incisors in contact) to locate teeth
[R3]. 3D FDTD found the interdental spaces cause only very small dips below 2.5 kHz [R10]. For
sibilants the teeth are the obstacle the jet hits (the engine already models a teeth noise
source).

### 2.5 Cheeks and the buccal space

No primary source on buccal-space acoustics was read (*unverified*). *Observed:* the male DVTD
segmentations are large (volume 52–123 cm³, against 22–64 cm³ for the female speaker,
`dvtd_slices.py`), and the paper says the segmentation extends slightly in front of the mouth
[R4]; how much of the difference is buccal space (between teeth and cheeks) was not separated.
The model treats the cheeks as the lateral bound of the oral cavity with an adjustable vestibule
gap, and the validation reports volume separately so this shows up.

### 2.6 Lips

The engine models the lip opening as an elliptical slit (aperture × width). DVTD cuts at the lips
show a wide flat opening for spread vowels and a rounded one for /u/ (female /u/: 3 separate
regions in the cut 0.25 cm behind the lip opening, 3.65 cm across, where the plane passes the
lip corners; `dvtd_slices.py`). No lip-shape measurement study was read (*unverified*).

## 3. The tongue: grooves, funnels, laterals, rhotics

- **Four shape classes.** 3D tongue surfaces from coronal ultrasound for English vowels and
  consonants fell into four classes: *front raising, complete groove, back raising, two-point
  displacement*; the first three include vowels and consonants, the last only consonants. EPG
  showed three contact patterns (bilateral, cross-sectional, both); vowels used only bilateral
  [R7]. Which sounds fall in which class needs the full text (*unverified*).
- **Sibilants (groove, funnel).** A summary attributed to the 1995 fricative MRI study says the
  anterior tongue body is concave (grooved) for alveolar stridents and flat or slightly convex
  (domed) for postalveolars, with a concave posterior body for all fricatives (*unverified*
  attribution; the 1994 conference version, read as an abstract, says the concave /s z/ tongue
  makes the area function more abrupt behind the constriction) [R8, R9].
  *Observed (DVTD):* /s/ narrows to a single small channel, 0.15 cm² (male, 0.75 × 0.28 cm) and
  0.12 cm² (female, 0.90 × 0.15 cm) at 90–93 % of the tract length; /ʃ/ narrows to 0.10 cm² at
  79–88 %, behind a long thin channel arched under the palate dome (evidence 04–06). The groove
  must therefore produce a narrow channel *and* control its width/height ratio.
- **Laterals.** MRI of /l/ (light and dark) in four English speakers: alveolar contact with inward
  lateral compression of the tongue and convex cross-sections of the posterior body; flow
  channels along the sides, of somewhat different areas, talker-dependent; dark /l/ has a smaller
  pharynx (root retraction, back raising) [R1].
  *Observed (DVTD /l/):* both speakers show **two lateral channels** about 2–2.8 cm apart across
  the contact: male 0.69 and 0.51 cm², female 0.41 and 0.16 cm² (`plane_peek.py`, evidence 07–08).
  The asymmetry the paper reports is visible in both.
- **Rhotics.** MRI of English /ɹ/ in four speakers: three cavities from two constrictions (oral and
  pharyngeal), a large volume in front of the oral constriction, a convex anterior and concave
  posterior tongue body [R2]. Retroflex and bunched /ɹ/ give similar F1–F3 but different F4/F5
  spacing, explained by whether the long back cavity acts as a half- or quarter-wave resonator
  [R27]. *Observed (USC, 8 speakers' sustained /ɹ/):* the midsagittal images differ visibly: some
  speakers raise the tip and blade over an air pocket under the tongue, others bunch the dorsum
  with the tip low (evidence 09). The split into "retroflex" and "bunched" here is a visual
  judgement on 1.56 mm images; nothing was measured to classify them.
- **Vowels.** Mostly bilateral contact (sides braced against the upper molars) for front vowels
  [R7]. *Observed:* the DVTD /i/ constriction is a wide flat slit (male 1.45 × 0.25 cm,
  0.32 cm²), i.e. a midline-only area law cannot know its width.
- **Precedent for parameters.** A published 3D geometric vocal tract model controls the tongue
  with tongue-body, tip, blade and root positions plus four *tongue side elevation* parameters
  (TS1–TS4, −1.4…1.4 cm) at equally spaced points along the tongue contour, interpolated between
  them, and three *minimal area* parameters for the tongue-back, tip and lip regions; its velum has
  two parameters (shape, opening) [R16]. That model's source is GPL-3.0, so this project takes the
  idea and writes its own code.

## 4. Which details matter for the sound

| Detail | Effect | Source |
|---|---|---|
| Cross-section shape and tract bending | weak below 4–5 kHz; above, realistic cross-sections needed for higher modes | [R18] |
| Piriform fossae | troughs at 4–5 kHz, lowers lower formants; largest side-cavity effect | [R10, R12, R13, R6] |
| Valleculae | small dips, vary with the vowel | [R10] |
| Interdental spaces | very small dips below 2.5 kHz | [R10] |
| Laryngeal cavity | generates one formant (F4 in one speaker) | [R11] |
| Constriction area of fricatives | decides the noise spectrum; over-smoothed constrictions hurt /f/ recognition in 3D-printed tracts | [R4] |
| Back cavity of /ɹ/ | sets F4/F5 spacing | [R27] |
| Groove of /s/ | abrupter area function behind the constriction | [R9] |

Consequence for phase 2: the 1D FDTD fed with correct *areas* (including side branches and
multi-channel sums) captures most of what matters below about 4–5 kHz; the shape itself matters
above that and for the noise sources, which is the job of the offline 3D reference solver.

## 5. Caveats of MRI as ground truth

- **Speaker variability.** *Observed (DVTD):* the same sound in the two speakers differs more than
  two different sounds of one speaker: median distance 0.71 (size-normalised) against a
  nearest-other-sound median of 0.59, in log2 area-ratio RMS (`af_stats.py`; see
  validation-plan.md). A generic model cannot be judged against one speaker as if that speaker
  were "the" human.
- **Reproducibility.** Repeated 3D MRI of the same sustained task varied by about 8 % overall
  [R19].
- **Imaged shapes may be centralised** relative to natural speech [R6, R20].
- **Posture.** The papers read here do not state the speakers' posture in the scanner; a study
  compared upright and supine shapes (bibliographic record only, findings *unverified*) [R21].
- **Sustained sounds only.** All 3D sets are sustained sounds (7–14 s per volume
  [R3, R4, R22]).
- **The 1D reduction is itself approximate.** *Observed:* resonances of a 1D chain-matrix model of
  the area functions cut here sit above the peaks DVTD measured on 3D prints of the same
  geometry, median +6.7 % (F1), +6.5 % (F2), +8.6 % (F3), n = 44 (`af_formants.py`; 8 F2/F3 cases
  were peak-pairing misses). So model and reference must go through the same slicer and the same
  1D solver before they are compared.

## 6. Gaps between the engine and the research

1. No groove, cupping or lateral lowering: needed for /s/, /ʃ/, /l/, /ɹ/.
2. No cross-section width: /i/ (wide slit) and /u/ get their area only through α·d^β.
3. Valleculae and piriform pockets are absent from the geometry (the piriform is a fixed side tube
   in the acoustics; the vallecula pocket is deliberately removed from the inner wall for ray
   continuity).
4. Two lateral channels of /l/ are one area floor.
5. Palate vault and dental arch are fixed and absent laterally.
6. The 3D view shows none of the above.

## Sources

Keys used above. "Abstract" means only the abstract (or a database summary) was read.

| Key | Title, venue, year | Identifier | Read |
|---|---|---|---|
| R1 | Toward articulatory-acoustic models for liquid approximants based on MRI and EPG data. Part I. The laterals. J. Acoust. Soc. Am. 1997 | doi:10.1121/1.418030 | abstract |
| R2 | … Part II. The rhotics. J. Acoust. Soc. Am. 1997 | doi:10.1121/1.417972 | abstract |
| R3 | Multimodal dataset of real-time 2D and static 3D MRI of healthy French speakers. Sci. Data 8:258, 2021 (CC BY 4.0) | doi:10.1038/s41597-021-01041-3 | full text |
| R4 | Printable 3D vocal tract shapes from MRI data and their acoustic and aerodynamic properties (Dresden Vocal Tract Dataset). Sci. Data 7:255, 2020 (CC BY 4.0) | doi:10.1038/s41597-020-00597-w | full text |
| R5 | Analysis of vocal tract shape and dimensions using magnetic resonance imaging: vowels. J. Acoust. Soc. Am. 1991 | doi:10.1121/1.401949 | abstract |
| R6 | Vocal tract area functions for an adult female speaker based on volumetric imaging. J. Acoust. Soc. Am. 1998 | doi:10.1121/1.423298 | abstract |
| R7 | Three-dimensional tongue surface shapes of English consonants and vowels. J. Acoust. Soc. Am. 1996 | doi:10.1121/1.414969 | abstract |
| R8 | An articulatory study of fricative consonants using magnetic resonance imaging. J. Acoust. Soc. Am. 98(3), 1995 | bibliographic record and a summary | summary |
| R9 | Same authors' conference version, ICSLP 1994 | ISCA Archive, ICSLP 1994 proceedings | abstract |
| R10 | Acoustic analysis of the vocal tract during vowel production by finite-difference time-domain method. J. Acoust. Soc. Am. 2010 | doi:10.1121/1.3502470 | abstract |
| R11 | Acoustic roles of the laryngeal cavity in vocal tract resonance. J. Acoust. Soc. Am. 2006 | doi:10.1121/1.2261270 | abstract |
| R12 | Acoustic characteristics of the piriform fossa in models and humans. J. Acoust. Soc. Am. 1997 | doi:10.1121/1.417990 | abstract |
| R13 | A new method to explore the spectral impact of the piriform fossae on the singing voice. PLoS ONE 2014 (open access) | doi:10.1371/journal.pone.0102680 | abstract |
| R14 | Modeling the influence of piriform sinuses and valleculae on the vocal tract resonances and antiresonances. Acta Acustica united with Acustica 2015 | research-database summary | summary |
| R15 | Articulatory interpretation of the "singing formant". J. Acoust. Soc. Am. 1974 | doi:10.1121/1.1914609 | title |
| R16 | Modeling consonant-vowel coarticulation for articulatory speech synthesis. PLoS ONE 2013 (CC BY) | doi:10.1371/journal.pone.0060603 | full text |
| R17 | On the relationship between palate shape and articulatory behavior. J. Acoust. Soc. Am. 2009 | doi:10.1121/1.3125313 | abstract |
| R18 | Influence of vocal tract geometry simplifications on the numerical simulation of vowel sounds. J. Acoust. Soc. Am. 2016 | doi:10.1121/1.4962488 | abstract |
| R19 | Three-dimensional vocal tract morphology based on multiple magnetic resonance images is highly reproducible during sustained phonation. J. Voice 2017 | doi:10.1016/j.jvoice.2016.11.009 | abstract |
| R20 | Vocal tract area functions from magnetic resonance imaging. J. Acoust. Soc. Am. 1996 | doi:10.1121/1.415960 | abstract |
| R21 | A comparison of upright and supine vocal tract shapes with an open-type MRI scanner, Acoust. Sci. Tech. 26(5), 465–468, 2005 (exact title not confirmed) | researcher publication list | record only |
| R22 | A multispeaker dataset of raw and reconstructed speech production real-time MRI video and 3D volumetric images. Sci. Data 8:187, 2021 (CC BY 4.0) | doi:10.1038/s41597-021-00976-x | full text |
| R23 | Formant-frequency discrimination for isolated English vowels. J. Acoust. Soc. Am. 1994 | doi:10.1121/1.410024 | abstract |
| R24 | Comparison of MRI-based vocal tract area functions obtained from the same speaker in 1994 and 2002. J. Acoust. Soc. Am. 2008 | doi:10.1121/1.2805683 | abstract |
| R25 | Morphology and development of the human vocal tract: a study using magnetic resonance imaging. J. Acoust. Soc. Am. 1999 | doi:10.1121/1.427148 | abstract |
| R27 | A magnetic resonance imaging-based articulatory and acoustic study of "retroflex" and "bunched" American English /r/. J. Acoust. Soc. Am. 2008 | doi:10.1121/1.2902168 | abstract |
| R26 | Open-source manually annotated vocal tract database for automatic segmentation from 3D MRI using deep learning. J. Voice 2025 (arXiv 2501.06229) | doi:10.1016/j.jvoice.2025.02.026 | full text (arXiv) |

Not read at all (cited elsewhere, kept out of the design): the 3D linear tongue/lips/face model
(J. Phonetics 2002), the MRI+EMA+EPG 3D tongue model (Speech Commun. 2003), the cross-sectional
tongue shape study of [s], [ʃ], [l] (J. Phonetics 1992), the /ɹ/ taxonomy from MRI and ultrasound
(2004). Their bibliographic records were found; their content was not opened.
