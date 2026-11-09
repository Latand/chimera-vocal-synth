# tract3d tools: the 3D model against MRI

The geometry is the engine's (`crates/chimera/src/anatomy/tract3d/`). The CLI reads the MRI
references from a local folder outside the repository (`$TRACT3D_DATA`, see
[reference-data.md](../../docs/tract3d/reference-data.md)) and writes everything to an output
folder you name, also outside the repository. `cargo test` needs no data.

```sh
cargo build --release -p chimera-cli
CLI=target/release/chimera
OUT=/somewhere/outside/the/repo

# the model alone: a drawing of laminae, cuts, planes and the area function; JSON of a posture
"$CLI" tract3d sheet s "$OUT/s.svg"
"$CLI" tract3d model retroflex "$OUT/retroflex.json"
"$CLI" tract3d cut l 14.0 "$OUT/l-cut.svg"        # one cut, large, coloured by tissue
"$CLI" tract3d presets "$OUT/presets"             # every preset's drawing + the feature checks
"$CLI" tract3d tune s                             # move a preset until its feature checks pass
"$CLI" tract3d benchmark                          # compute and mesh time

# against the Dresden Vocal Tract Dataset (DVTD, CC0 meshes)
"$CLI" tract3d reference "$TRACT3D_DATA/dvtd/extracted/subject-1/s1-22-ehe-schwa" "$OUT/ref"
"$CLI" tract3d validate "$TRACT3D_DATA/dvtd" "$OUT/dvtd" --iterations 3000 --anatomy-iterations 600 --jobs 3
"$CLI" tract3d fit-derive "$OUT/dvtd"             # fits derive()'s constants; prints them
bun tools/tract3d/capture.ts "$OUT/dvtd"          # SVG figures → PNG

# against the USC 75-speaker set (CC BY 4.0 volumes; unsegmented)
bun tools/tract3d/usc-compare.ts "$CLI" "$TRACT3D_DATA/usc75/recon" "$OUT/usc"
bun tools/tract3d/capture.ts "$OUT/usc"

# the lab's geometry call in the browser (against `bun run preview`)
bun tools/tract3d/browser-bench.ts http://127.0.0.1:4173/ "$OUT/browser.json"
```

## What `validate` does

1. **Reference pipeline**, the same code for MRI and model: voxels at 0.5 mm, the two openings of
   the open mesh (glottis lower), a centreline through the lumen weighted towards its middle and
   pinned to the midsagittal plane, cuts every 2.5 mm perpendicular to it (0.25 mm pixels), and the
   air regions entering a band ±6 mm across and ±30 mm sideways. Where the research stage's probe
   output is kept (`dvtd/derived/`), each sound reports the port's difference from it.
2. **Anatomy, once per speaker** (male body for speaker 1, female for speaker 2): pharynx widths
   and corner shape, the arches, the palate's vault, the laryngeal tube, the piriform fossae and
   the tongue's width are fitted to /ə aː iː uː s l/ together, with those sounds' articulation
   refitted around them twice.
3. **Every sound**: the *default* posture (the engine's own targets for the nearest symbol, the
   3D shape from `derive`) and a *fitted* posture (13 articulators and 11 tongue parameters,
   Nelder–Mead with two restarts). The objective is the area-function distance d plus the tract
   length and cut proportions. Both are then measured by the reference pipeline on the model's
   own voxels.
4. **The fast slicer's own check** (T-consistency): the model's voxels cut along the fast
   slicer's centreline, cut by cut.
5. **Per sound**: d (default, fitted), length and volume, F1–F3 of both area functions through the
   engine's chain-matrix code, the constriction, the lower pharynx's channel counts, cut shape
   (IoU after equal-area scaling, aspect), the nearest reference (identification), and the
   distance between the two speakers for the same sound.

Outputs: `table.json`, `table.md`, `anatomy-*.json`, and per sound `NN-name.svg` (MRI cuts above
model cuts at the same scale, the area functions, both midsagittal planes),
`-reference.json`, `-fit.json`, `-fit-voxel.json`, `-posture.json`. These contain values derived
from the datasets: they stay outside git. Only this project's error measurements go into
[validation-results.md](../../docs/tract3d/validation-results.md).

An exit status of zero means the run finished; acceptance is in the numbers.
