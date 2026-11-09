# Reference data: public vocal tract MRI, as observed

*Phase 1 research, 2026-11-09.* Every source below was opened on that date (pages and API answers
saved locally with their HTTP status and size). Source keys `[Rn]` refer to the list in
[research.md](research.md).

**Policy (operator decision, 2026-11-09).** Datasets are downloaded to a local folder outside the
repository, written here as `$TRACT3D_DATA`; git holds only published numbers with citations and
the code that reads `$TRACT3D_DATA`. Register-gated or non-redistributable data never enters git
or public text. Our own error measurements against a dataset are results and may be published
(with the dataset's citation); the dataset's own values are not copied into the repository.

## Summary

| Dataset | Licence | Access | 3D content | Sounds | Used for | Downloaded |
|---|---|---|---|---|---|---|
| Dresden Vocal Tract Dataset (DVTD) [R4] | figshare item CC0 1.0 (article CC BY 4.0) | open, 1.13 GB zip | segmented inner-surface meshes with teeth, FE meshes, raw MRI, dental casts, measured transfer functions | 22 German: 16 vowels, /f s ʃ ç x/, /l/; 1 male, 1 female | **primary quantitative reference** | yes, all |
| USC 75-Speaker Speech MRI Database [R22] | CC BY 4.0 | open, 676 GB total; 3D part one 92.6 GB zip | unsegmented 3D volumes, 1.5625 mm isotropic | 13 English vowels, 14 continuant consonants incl. /ɹ l s ʃ/, postures; 75 speakers × 2 repeats | /ɹ/ and English consonants, side-by-side images, speaker spread | 26 volumes + 48 snapshots (by range reads) |
| French static 3D MRI (10 speakers) [R3] | CC BY 4.0 (article); figshare collection | open | unsegmented 3D (0.69 × 0.76 × 1.2 mm, 36 slices) + teeth-locating postures | French vowels and consonants in vowel contexts, /l/ and /ʁ/ in 5 contexts, retroflex postures | not needed now | no |
| Manual segmentations of 53 of those volumes [R26] | not stated on the paper | figshare private link; scripted access refused (202 / 403) | segmentation masks | 21 French phonemes | would be a second segmented reference | no (could not reach) |
| Aalto Finnish vowels | CC BY-NC-SA 3.0 | open (25 MB and 883 MB) | DICOM + STL surface models | Finnish vowels, 2 male speakers | overlaps DVTD; non-commercial | no |
| USC Speech and Vocal Tract Morphology MRI Database (17 speakers) | research use | **behind a request form** | 3D volumes of vowels and continuants | English | gated: not used | no |
| Published area-function catalogues [R20, R6, R24, R5] | journal copyright | paywalled; author manuscript PDF refused scripted access (403) | tables of areas per section | English vowels (+ nasals, plosives, /ɝ l/ for the female set) | numbers would be git-safe with citation, **not obtained** | no |

## Details

### Dresden Vocal Tract Dataset (DVTD)

- **Where:** figshare item 11897187 "Dresden Vocal Tract Dataset (complete archive)", one file
  `dvtd.zip`, 1 129 467 424 bytes, md5 `02d159b107c9dd129744fc005e0f91d7` (matched after
  download). Mirror `vocaltractlab.de/dvtd.zip` (same length). Collection
  doi:10.6084/m9.figshare.c.4869732. The paper says its scripts are also on GitHub (that repository has no licence file).
- **Licence:** the figshare item says CC0 1.0; the paper is CC BY 4.0.
- **Acquisition [R4]:** Siemens 3 T, sagittal 3D VIBE, 1.2 × 1.2 × 1.8 mm, 14 s per sustained
  sound (the paper does not state the posture). Plaster dental casts scanned and merged into the MRI; volumes upsampled to
  0.25 mm; semi-automatic segmentation (ITK-SNAP), opened at the glottis (cutting plane through the
  vocal folds) and at the lips, manually smoothed; nasal cavity excluded.
- **Files per sound (`XX`):** `XX-mri/*.ima` raw MRI; `XX-inner-surface.stl` (open at glottis and
  lips, mm); `XX-inner-surface-closed.stl`; `XX-fem.msh` (Gmsh); printable models;
  `XX-vvtf-measured.txt` and `XX-vvtf-calculated.txt` (volume-velocity transfer function,
  0–10 kHz, 0.961 Hz steps); noise recordings at six flow powers; reference recordings of the
  speakers. Per speaker: `sN-mandible.stl`, `sN-maxilla.stl`.
- **Sounds:** /aː eː iː oː uː ɛː øː yː l f s ʃ ç x a ɛ ɪ ɔ ʊ ʏ œ ə/, labels `s1-01-bahn-tense-a`
  … `s2-22-ehe-schwa`. Speaker 1 male, speaker 2 female.
- **Planes:** full 3D; any plane can be cut.
- **Numbers usable in git:** the paper's own tables (FE vs measured resonances, recognition
  rates) with citation. Area functions are *not* published in it; they are derived here.
- **Observed coordinate frame:** each mesh is in its own scanner frame (x lateral; the other two
  axes and origin differ between sounds), so the reference pipeline finds the glottis and lip
  openings from the open mesh's two boundary loops.
- **What was computed locally** (`dvtd_slices.py`, all 44 meshes, ~5 s each): area function every
  2.5 mm along a centreline, cut width/height, channel count, volume. Tract length from the probe:
  male 17.25–21.5 cm, female 13.75–16.25 cm.

### USC 75-Speaker Speech MRI Database

- **Where:** figshare item 13725546. Files (listing read through a reader proxy because figshare's
  API answered 403 to this host over IPv6): `dataset.zip` 570.7 GB, `dataset_3d_only.zip`
  92 592 570 427 bytes, `dataset_t2w_only.zip` 2.5 GB, `dataset_2drt_video_only.zip` 2.6 GB,
  `example_for_sub001.zip` 7.4 GB, `metafile_public_20210129.json`, `Subjects.xlsx`,
  `Stimuli.pptx`. Companion code on GitHub, BSD-3-Clause (linked from the dataset page).
- **Licence:** CC BY 4.0 (DataCite record for doi:10.6084/m9.figshare.13725546).
- **Access:** open. The download host works over IPv4 and redirects to signed storage URLs that
  accept HTTP range requests, so single members of the 92.6 GB zip can be read without the rest:
  the central directory (10 114 members) cost 5 requests and 1.3 MB.
- **3D acquisition [R22]:** 1.5 T, accelerated 3D gradient echo, 1.5625 mm isotropic,
  FOV 250 × 250 × 125 mm, matrix 160 × 160 × 80, 7 s per sustained sound, two repeats.
  *Observed array layout:* `recon` complex 160 × 160 × 80, axis 0 superior–inferior, axis 1
  anterior–posterior, axis 2 left–right.
- **3D stimuli [R22, Table 4]:** vowels in /bVt/ (beet, bit, bait, bet, bat, pot, but, bought,
  boat, boot, put, bird, abbot); consonants in /aCa/ (f, v, θ, ð, s, z, ʃ, ʒ, h, m, n, ŋ, l, ɹ);
  postures (breathe, clench, tongue out, yawn, tip, hold).
- **Formats:** `subXYZ/3d/recon/*_recon.mat` (MATLAB v5), `subXYZ/3d/snapshot/*.png`
  (midsagittal, 160 × 160).
- **Numbers usable in git:** none published as area functions; images only with attribution.
- **Downloaded:** `Subjects.xlsx`, `Stimuli.pptx`, the metafile (md5s match the listing); 26
  volumes: two native-English speakers (one male, one female) × {ara, ala, asa, asha, beet, pot,
  boot, bird, abbot, clench}, and /ɹ/ (ara) for six more native-English speakers; midsagittal
  snapshots of /ɹ/ for all 48 English-L1 speakers. Speaker IDs are in the local data README.
- **Caveat:** teeth are dark like air, so a threshold segmentation of the oral cavity counts teeth
  as air. Used here for images and simple measures; area functions come from DVTD.

### French static 3D MRI

- figshare collection 5270387 (doi:10.6084/m9.figshare.c.5270387); article CC BY 4.0 [R3].
- 10 native French speakers; static 3D VIBE, 0.69 × 0.76 mm in plane, 1.2 mm slices, 36 slices,
  7 s; DICOM in `PXX/DCM_3D/<phoneme>`; consonants /l/, /ʁ/, /p/, /t/ … in vowel contexts
  (folders `li la lu ly …`), retroflex postures (`retro`, `deep`), teeth postures (UP, DOWN,
  CONTACT).
- Not downloaded: unsegmented like USC, French phonology, and the per-item licences on figshare
  were not checked.

### Segmentations of the French volumes

- Paper: arXiv 2501.06229 [R26]; data statement points to a figshare *private* link and code on
  GitHub (the repository the paper names has no licence file).
- The share page answered 202 with an empty body over IPv4 and 403 through a reader proxy;
  the licence is not stated in the paper. Not used.

### Aalto Finnish vowels

- `speech.math.aalto.fi/data.html`: "licensed under a Creative Commons
  Attribution-NonCommercial-ShareAlike 3.0 Unported License"; subject 1 ~25 MB, subject 2
  ~883 MB; DICOM, sound, surface models of vowels.
- Not downloaded: Finnish vowels only (covered by DVTD), and the non-commercial term does not fit
  an MIT project's outputs.

### USC 17-speaker morphology database

- `sail.usc.edu/span/morphdb/`: "To access/download the database please begin by filling out this
  form." Gated: not used.

### Published area functions

- Male speaker catalogue (18 shapes) [R20], the same speaker re-imaged in 2002 [R24], a female
  speaker (10 vowels, /ɝ/, /l/) [R6], two male speakers' point vowels [R5].
- Tried: Europe PMC full-text (500 for the 2008 paper, an author manuscript outside the open-access subset), Europe PMC PDF renderer (403 over IPv6 and IPv4), PubMed Central (reCAPTCHA page),
  the lab's publication page (lists the papers, no tables). **No area-function numbers were
  obtained.** They stay out of the design; a later stage may add them by hand from the journal
  articles if access is available, citing table and page.

## Published numbers that may go into git (with citation)

| Number | Value | Source |
|---|---|---|
| Piriform fossa volume | 2.1–2.9 cm³ each (4 speakers) | [R12] |
| Piriform fossa depth | 1.6–2.0 cm | [R12] |
| Piriform trough frequency | 4–5 kHz | [R12, R13] |
| 3D-MRI morphology repeat variation | about 8 % | [R19] |
| Shape/bending matters above | 4–5 kHz | [R18] |
| Interdental-space dips | very small, below 2.5 kHz | [R10] |
| Formant discrimination threshold | ~14 Hz below 800 Hz; ~1.5 % in the F2 region (trained listeners) | [R23] |
| 3D model tongue side elevation range | −1.4…1.4 cm at four points | [R16] |
| DVTD voxel size and scan time | 1.2 × 1.2 × 1.8 mm, 14 s | [R4] |
| USC voxel size and scan time | 1.5625 mm isotropic, 7 s | [R22] |

## Numbers derived here (results, local data)

Made with the scripts in `$TRACT3D_DATA/_scripts` on the CC0 DVTD meshes. They are this
project's measurements, used to set thresholds in [validation-plan.md](validation-plan.md):

- Distance between area functions, `d` = RMS over normalised length of log2(A₁/A₂):
  nearest *other* sound of the same speaker: median 0.59 (male) / 0.59 (female), smallest
  0.31 / 0.35; same sound across the two speakers: median 0.99 raw, 0.71 after scaling areas by
  the speakers' mean-area ratio (range 0.43–1.45).
- 1D resonances of the cut area functions against the measured transfer-function peaks of the
  3D prints: median +6.7 % F1, +6.5 % F2, +8.6 % F3 (n = 44; 3 F2 and 5 F3 values were
  peak-pairing misses above 25 %).
- Constrictions: /s/ 0.15 cm² (male), 0.12 cm² (female); /ʃ/ 0.10 cm² both; /i/ 0.32 / 0.20 cm²;
  /l/: two lateral channels at the alveolar contact in both speakers.
