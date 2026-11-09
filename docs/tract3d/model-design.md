# Model design: a parametric 3D vocal tract

*Phase 1 design, 2026-11-09, base 2e567e2.* The evidence behind each choice is in
[research.md](research.md) (sources `[Rn]`) and [reference-data.md](reference-data.md); how the
model is checked against MRI is in [validation-plan.md](validation-plan.md); the build order and
what is deferred are in [README.md](README.md).

## 1. Decisions in one screen

1. **Geometry lives in the Rust engine**, a new module `chimera::anatomy::tract3d`, reached by
   the browser through the existing JSON call interface. The UI only draws what it returns. Phase
   2's acoustics call the same module, so there is one source of truth.
2. **The midline is the engine's.** The 3D model does not re-derive the midsagittal shape: it takes
   `HumanVocalTract::compute`'s outlines (tongue, velum, palate, lips, teeth, epiglottis,
   pharyngeal wall) and adds the third dimension as *lateral profiles* per element. At z = 0 the 3D
   model equals today's 2D model by construction, and every existing gesture drives the 3D shape.
3. **The 3D solid is a stack of parasagittal sections ("laminae").** For a lateral offset z, each
   element yields closed tissue polygons in the sagittal plane at z. Air is what no tissue
   polygon covers. Every cut the requirement asks for (perpendicular to the centreline, coronal,
   axial) contains the lateral axis, so a cut is a set of 1D air intervals, one line per lamina,
   found with the same ray–polyline intersection the engine already uses (`anatomy::geom`).
4. **The tongue is a parametric surface** with five stations along its length (root, back,
   dorsum, blade, tip), each with a *side elevation* (cupping or lowering, after the published
   3D model's TS1–TS4 [R16]) and a *groove depth*, plus groove width, taper, left/right asymmetry,
   tip width and edge rounding. Groove, funnel, cupping, lateral lowering (/l/), bunched and
   retroflex /ɹ/ and flat are combinations of these plus the engine's midline articulators.
5. **Side cavities are geometry**: piriform fossae and valleculae appear in the laminae as pockets,
   the epiglottis as a tissue plate, the sublingual cavity under a curled tip, the buccal
   vestibule beside the teeth. A cut reports every air region in its band, with a channel count,
   like the MRI reference does.
6. **Phase 1 changes no sound.** Nothing in `voice/` or the worklet reads the new module. The lab
   plays the existing engine for the matching 17-articulator posture, and says so on screen.

### Why not the alternatives

- *Geometry in TypeScript/three.js.* Phase 2 needs the same geometry on the audio side, in Rust.
  Two copies would drift (the repository already pays that cost for the gesture records).
- *Surface meshes plus a general mesh–plane cutter.* Needs polygon booleans for the tissue union,
  is slow and fragile at contacts, and gives no cheap path to real-time areas.
- *Implicit fields sampled on a grid per cut.* Simple, but every sample needs a nearest-point
  search on every element (estimated tens of milliseconds per frame); kept only for the
  validation voxelizer, where speed does not matter (§6.4).
- *A biomechanical tongue (FEM).* Excluded by the operator's decision.

## 2. Frame and units

Head frame of the engine: centimetres, x forward, y up, glottis at the origin for a neutral
larynx, plus **z lateral, positive to the speaker's right** (the side the 3D view's default camera
faces, as in `src/ui/render/anatomy3d.ts`). Body scaling (`HumanBody.scale`, `pharynx`) applies to
everything through the same `p()` template map the 2D model uses; lateral sizes scale with
`scale` only.

## 3. Representation: laminae

```
HumanArtic (17) ──► HumanVocalTract::compute ──► HumanFrame (midsagittal outlines, grid, contacts)
                                                       │
ShapeParams (3D-only) ──► derive(artic) + overrides ───┤
                                                       ▼
                 elements: tongue, palate, velum, pharynx, larynx, lips, teeth+jaw, cheeks
                                                       │ lamina(z) → closed tissue polygons
                                                       ▼
          laminae z_k ──► cuts (perpendicular / coronal / axial) ──► slices, area function
                                                       │
                                                       └──► element meshes (display only)
```

- **Lamina.** For a lateral offset z the solver asks each element for its tissue polygons in the
  sagittal plane at that z (closed, in the x–y plane, cm). Air in the lamina = the head box minus
  the union of tissue polygons. Contact is overlap: a tongue polygon pressed into the palate
  closes the air there, as in the 2D model ("if the tongue crosses the wall … closed").
- **How an element makes its lamina polygon.** Each surface that faces the air is its midsagittal
  contour `C(t)` (from `HumanFrame`) moved along its in-plane normal `N(t)` by a lateral profile
  `h(t, z)`; the element is present only where `|z| < W(t)` (its half-width). Walls that close
  the tract laterally (pharyngeal walls, dental arch, cheeks) are profiles whose offset grows
  until the walls meet beyond the local half-width, so a lamina outside the airway contains no
  air.
- **Spacing.** Base spacing Δz = 0.2 cm over |z| ≤ 3.2 cm. Where air appears or vanishes between
  two neighbouring laminae, the cut bisects in z (up to 3 levels, 0.025 cm) to place the edge.
  Symmetric parameters compute one side and mirror it; any non-zero asymmetry parameter turns
  mirroring off.
- **Why laminae fit this tract.** The cuts all contain the lateral axis; the midline stays the
  engine's; tissue islands and several air intervals per line (a curled tip with air under it,
  the epiglottis inside the oropharynx, two /l/ channels) come out of the interval logic without
  special cases.
- **Limit.** Features that turn sideways (a surface whose normal points along z, such as the
  front of the lateral pharyngeal wall) are approximated by the lamina spacing plus edge
  bisection. Validation measures what this costs.

## 4. Elements and parameters

Each element is a Rust type with `lamina(z, &mut PolygonSet)`, `mesh(&mut Mesh)` and a parameter
spec (name, unit, min, max, default, which articulators drive it, one-line doc). The specs are
the single source for the lab's controls (served by `tract3dSpecs`). "Start" values marked
*(fit)* are first guesses that the build stage replaces with values fitted to the DVTD data
(validation-plan.md §5); they are not anatomical claims.

Template half-widths already in the code (`TONGUE_W` in `anatomy3d.ts`: 1.5 cm at the root, up to
2.45 cm across the body, 0.55 cm at the tip) are the starting tongue widths; that table moves into
Rust and the TS copy is deleted when the 3D view switches to the engine's meshes.

### 4.1 Tongue

Midline from the engine: root Bézier, body arc, blade, tip, underside, floor (`f.tongue`). Arc
parameter `t` from the vallecula point (0) over the dorsum to the tip apex (1). Stations:
root 0.0, back 0.3, dorsum 0.55, blade 0.8, tip 0.95; values between stations by monotone cubic
interpolation (no overshoot, C¹).

Cross-section at `t`, lateral position `w = z / W(t)` (−1…1), as an offset of the dorsal surface
along `N(t)`:

```
h(t, w) = side(t) · S(|w|) · (1 + asym · sign(w)) − groove(t) · G(w / gw(t)) − thick(t) · (1 − E(|w|))
```

`N(t)` points out of the tongue towards the roof, so positive `h` raises the surface. `S` rises
from 0 for |w| ≤ 0.25 to 1 at |w| = 0.8 (smoothstep) and stays 1 to the edge; `G` is a raised
cosine of width `gw`, 1 at the midline; `E(|w|) = (1 − |w|^p)^(1/p)` rounds the margin down to
the ventral surface (`p` = edge rounding; `thick(t)` is the tongue's dorsal–ventral thickness
there). The ventral surface narrows with the same `W(t)`. At z = 0 only the groove term acts, so
with no groove the midline is exactly the engine's contour.

| Parameter | Unit | Range | Start | Meaning |
|---|---|---|---|---|
| `sideRoot … sideTip` (5) | cm | −1.4 … 1.4 [R16] | derived (§5) | margin elevation relative to the midline; + cups/braces, − lowers (lateral channels) |
| `grooveRoot … grooveTip` (5) | cm | 0 … 0.8 | derived | depth of the central groove |
| `grooveWidth` | cm | 0.3 … 2.0 | 0.8 *(fit)* | groove width at half depth, at the blade |
| `grooveTaper` | — | −1 … 1 | 0 | groove width change towards the tip (+ narrows: a funnel) |
| `sideAsym` | — | −1 … 1 | 0 | right/left difference of the side elevation (unequal /l/ channels, observed in both DVTD speakers) |
| `tongueWidth` | ×template | 0.8 … 1.2 | 1 | scales `W(t)` |
| `tipWidth` | cm | 0.3 … 1.5 | 0.55 (template) | half-width at the tip: apical (narrow) vs laminal (broad) |
| `edgeRound` | — | 1.5 … 4 | 2 | `p` of the margin: rounded vs square cross-section |

Midline articulators that keep their meaning: `tbX`, `tbY`, `ttX`, `ttY`, `ttCurl` (retroflex
curl, makes the sublingual cavity in the midline), `root`, `dorsum` (click hollowing; keeps the
sides sealed, which a click's sealed pocket needs).

Shapes the research lists, as parameter sets (the lab's presets; numbers are starting points
until fitted):

| Shape | Midline | 3D parameters |
|---|---|---|
| flat | neutral | all sides 0, no groove |
| grooved (/s/) | tip near the alveolar ridge | `grooveBlade` ≈ 0.3, `grooveWidth` ≈ 0.6, sides of back/dorsum + (braced) |
| funnel | as grooved | groove rising from back to blade, `grooveTaper` > 0 |
| cupped / domed (/ʃ/) | blade raised, tip behind ridge | sides + along back…blade, small groove |
| lateral lowering (/l/) | tip contact | `sideBlade`, `sideTip` −0.6 … −1.0, `sideDorsum` slightly −, `sideAsym` ≠ 0 allowed |
| bunched /ɹ/ | dorsum high, tip low, root back | sides + at back/dorsum (braced), groove at root/back (concave posterior), blade convex (side −) [R2] |
| retroflex /ɹ/ | tip up and curled (`ttCurl`), root back | narrow tip (`tipWidth` small), blade convex, sublingual cavity from the curl |

### 4.2 Hard palate

Midline: `HumanVocalTract.palate` (fixed landmarks). Laterally the roof drops towards the dental
arch: offset `V(t) · (|z| / Wp(t))^q` for `|z| ≤ Wp(t)`, closing into the alveolar process and the
upper teeth beyond.

| Parameter | Unit | Range | Start | Meaning |
|---|---|---|---|---|
| `vaultDepth` | cm | 0.5 … 2.5 | 1.4 *(fit)* | how far the roof drops from the midline to the arch at the molars |
| `archWidth` | cm | 1.4 … 2.6 | 1.9 *(fit)* | inner half-width of the upper arch at the molars |
| `vaultShape` | — | 1.5 … 6 | 2.5 *(fit)* | `q`: dome (low) vs flat with steep sides (high) [R17] |
| `archTaper` | — | 0 … 1 | 0.6 *(fit)* | how the arch narrows towards the incisors |

The DVTD dental-cast scans (`s1-maxilla.stl`, `s2-maxilla.stl`) give these per speaker; the fitted
values become the two anatomy presets used in validation.

### 4.3 Velum and uvula

Midline: the engine's velum flap (angle from `velum`, the port area). 3D: width tapering from the
palate's width at the root to the uvula; the flap's lateral edges meet the lateral pharyngeal
walls (palatopharyngeal arches). The velopharyngeal port is the gap left between the velum's
posterior surface and the pharyngeal walls; phase 1 reports its area and shape at the port but
does not model the nasal cavity in 3D.

| Parameter | Unit | Range | Start | Meaning |
|---|---|---|---|---|
| `velumWidth` | cm | 1.5 … 3.5 | 2.6 *(fit)* | width at the root |
| `uvulaLength` | cm | 0.5 … 1.5 | 1.0 *(fit)* | uvula length (part of the flap's tip) |
| `uvulaWidth` | cm | 0.4 … 1.2 | 0.8 *(fit)* | |

### 4.4 Pharynx walls

Midline: the posterior pharyngeal wall (`phar_wall`). Laterally the cross-section is a
superellipse whose half-width varies with height; the posterior wall moves forward and the
lateral walls inward until they meet beyond the half-width.

| Parameter | Unit | Range | Start | Meaning |
|---|---|---|---|---|
| `pharynxWidthNaso` | cm | 0.8 … 2.2 | 1.5 *(fit)* | half-width, nasopharynx |
| `pharynxWidthOro` | cm | 0.8 … 2.4 | 1.6 *(fit)* | half-width, oropharynx (observed full widths 2.5–4.4 cm in DVTD cuts) |
| `pharynxWidthHypo` | cm | 0.8 … 2.4 | 1.5 *(fit)* | half-width, hypopharynx above the piriform fossae |
| `pharynxShape` | — | 1.5 … 4 | 2.2 | superellipse exponent |
| `lateralSqueeze` | — | 0 … 1 | 0 | medial movement of the lateral walls (pharyngeal constriction, sphincter) |

Driven by: `root` (the anterior wall is the tongue root), `larynx` (lower end moves), `tbX`
(through the root).

### 4.5 Larynx, epilarynx and the side cavities

Built from the engine's epilarynx (`epilarynx`, `ventricular`, larynx height) so that at default
parameters the tube's cross-sectional area reproduces `epilarynx_area` (the 2D model's own
profile) at every height.

| Part | Geometry | Parameters (start) |
|---|---|---|
| Glottis, ventricular folds, ventricle | elliptical tube profile from the glottis up; ventricle a short widening; false folds a narrowing | from `ventricular`, `larynx` |
| Epilarynx tube | between the epiglottis' laryngeal surface and the arytenoids, walled by the aryepiglottic folds | `epilarynxWidth` 0.6 cm *(fit)*; narrowing from `epilarynx` (twang) |
| Epiglottis | a curved plate (tissue island), 0.25 cm thick, crescent in cross-section | `epiglottisWidth` 1.0 cm half-width *(fit)*, `epiglottisCurl` 0.4 *(fit)* |
| Piriform fossae (L, R) | blind pockets lateral to the aryepiglottic folds, from the hypopharynx down towards the glottis | `piriformDepth` 1.8 cm (published 1.6–2.0 [R12]), `piriformWidth` 0.6 cm *(fit)*; reported volume must fall in 2.1–2.9 cm³ [R12] |
| Valleculae (L, R) | pockets between the tongue root and the epiglottis' lingual surface, split at z = 0 by the median fold | `valleculaDepth` 0.8 cm *(fit)*, `valleculaWidth` 0.7 cm *(fit)* |

Because the median glossoepiglottic fold sits at z = 0, the midsagittal lamina has no vallecula
air, which is what the 2D model's inner wall already assumes; the pockets appear only off the
midline.

### 4.6 Lips

Midline: the engine's upper and lower lip outlines (`lipAp`, `lipProt`, `labiodental`). The
opening's width is the engine's `lip_w` (from `lipWidth` and the body's lip width); 3D adds the
shape of the opening and of the lip tube between the incisors and the opening.

| Parameter | Unit | Range | Start | Meaning |
|---|---|---|---|---|
| `lipCornerShape` | — | 1 … 4 | 2 | opening outline exponent: lens (spread) ↔ ellipse (rounded); default follows `lipWidth` |
| `lipThickness` | cm | 0.6 … 1.6 | 1.0 *(fit)* | lip depth front to back |

### 4.7 Teeth and jaw

Rigid dental arches: the upper fixed to the skull, the lower carried by the jaw's rotation about
the condyle (the 2D model's `j()` map). Arch shape from width, depth and taper; crowns from the
per-tooth table already in `anatomy3d.ts` (`UPPER_TEETH`, `LOWER_TEETH`, which also moves to
Rust). The incisors' midsagittal positions are the engine's landmarks, so overjet and overbite
stay what the 2D model has.

| Parameter | Unit | Range | Start | Meaning |
|---|---|---|---|---|
| `lowerArchWidth` | cm | 1.3 … 2.5 | 1.8 *(fit)* | inner half-width of the lower arch at the molars |
| `toothGap` | cm | 0 … 0.15 | 0.05 | interdental gaps (small acoustic effect [R10]; matters for sibilant jets) |

Driven by `jaw` (rotation), `labiodental` (lower lip to upper incisors).

### 4.8 Cheeks and buccal space

Lateral bound of the oral cavity outside the dental arches.

| Parameter | Unit | Range | Start | Meaning |
|---|---|---|---|---|
| `buccalGap` | cm | 0 … 0.6 | 0.1 *(fit)* | air between the teeth and the cheeks (the vestibule) |
| `cheekPuff` | — | 0 … 1 | 0 | puffed cheeks (beatbox, trumpet embouchure) |

Driven by `jaw` (the lateral gap between upper and lower teeth opens into the vestibule) and
`lipWidth` (spread lips pull the cheeks in).

## 5. How the 17 articulators map onto the model

Final 3D parameter = `derive(artic)` + offset (lab control or preset). `derive` uses only the
articulators, so speech, singing and beatbox gestures drive the 3D shape without any change to
gesture records or plans.

| Articulator | Midline (from `HumanFrame`) | Also derives |
|---|---|---|
| `jaw` | mandible rotation, lower teeth, floor, lower lip | lateral teeth gap into the vestibule |
| `tbX`, `tbY` | tongue body circle | `sideBack`, `sideDorsum` + with tongue height near the palate (bracing of front-raising vowels [R7]) |
| `ttX`, `ttY` | tip | `grooveBlade`, `grooveTip` with tip-to-ridge proximity (`contact_tip`): sibilant groove |
| `ttCurl` | curl, sublingual cavity | `tipWidth` narrows; `sideBlade` − (convex anterior) |
| `root` | tongue root, vallecula position | `sideRoot` + with retraction (concave posterior body [R2]) |
| `lipAp`, `lipProt`, `lipWidth`, `labiodental` | lips | `lipCornerShape` from `lipWidth`; cheek tension |
| `velum` | velum angle, port | — |
| `larynx` | glottis height, lower pharynx | piriform and vallecula move with it |
| `epilarynx`, `ventricular` | (parametric today) | epilarynx tube and false-fold geometry |
| `lateral` | tip contact unchanged | `sideBlade`, `sideTip` −(0.8 *(fit)*)·lateral, `sideDorsum` −0.3·that; replaces the area floor hack in 3D |
| `dorsum` | dorsum hollowing | `sideDorsum` + while hollowing (lateral seal) |

The derivation constants (`*(fit)*`) are fitted once against DVTD in the build stage
(validation-plan.md, test T-default) and then fixed in code with a comment naming the fit run.

## 6. Cuts, area function and side cavities

### 6.1 Centreline

From the midsagittal lamina: the glottis point, then the engine's grid midpoints
(`f.grid[i].mid`, which the 2D model keeps continuous in every articulator), Laplacian-smoothed
with fixed end points (fixed iteration count), resampled every 0.25 cm (`s` from the glottis to
the lip opening). Tangent `t(s)`; across-direction `u(s)` = `t` turned 90° in the sagittal plane
towards the outer wall; lateral `ẑ`. If the radius of curvature falls below the local half-height
(neighbouring cuts would cross inside the airway), the slice is flagged `fold` in debug output.

### 6.2 A cut

`cut(origin, u)` intersects the plane spanned by `u` and `ẑ` through `origin`:

1. For each lamina z_k: the line `origin + a·u` (a ∈ [−4, 4] cm) crossed with that lamina's
   tissue polygons → air intervals (crossings sorted; inside-any-tissue by parity per polygon).
2. Edge refinement in z where air starts or ends between laminae (§3).
3. Connected components in the (a, z) plane (intervals overlapping between neighbouring laminae
   connect).
4. Keep the components that enter the band |a| ≤ 0.6 cm, |z| ≤ 3.0 cm — the same band the MRI
   reference uses, so model and reference count the same air (a cut that crosses the tract twice
   at a bend meets the other part outside the band; both /l/ channels sit inside it). For the
   area function the voice will use (phase 2), a component's weight fades from 1 inside the band
   to 0 at 0.3 cm outside it, so a region drifting across the band edge changes the area
   smoothly; validation uses the hard rule, as the reference does, and reports both.
5. Outputs per cut: area (cm², trapezoid over z of interval lengths), perimeter, channel count,
   lateral extent, extent along `u`, the outline polygon(s) in (a, z), and a tag per component:
   `main` (contains or is nearest the band centre), else `left`/`right` side regions labelled by
   context (`piriform` below the epiglottis tip, `vallecula` between root and epiglottis,
   `lateral` beside a tip contact, `sublingual` under a curled tip, `buccal` outside the arch).

### 6.3 Planes and the area function

- **Perpendicular cuts** every 0.25 cm along `s` → the **area function**: arrays `s`, `area`,
  `perimeter`, `channels`, `width`, `height`, plus totals (length, volume) and the minimum area
  and its position (searched over s ≥ 0.5 cm: the first cut lies on the glottal plane).
- **Midsagittal plane**: the z = 0 lamina's polygons (exactly the 2D model's outlines).
- **Coronal plane** at x = x₀ and **axial plane** at y = y₀: the same `cut` with `u` = ŷ or x̂
  through a point on the midline.
- **Side cavities as descriptors** (for phase 2, §10): for each piriform fossa, vallecula,
  sublingual and buccal region: attachment `s`, volume, neck area, depth.

### 6.4 Voxelization (for validation and phase 2's 3D solver)

`air(x, y, z)` is answered by the nearest laminae (interpolating interval end points in z), so the
model can be voxelized at any spacing. The validation runs the *reference* pipeline (§3 of
validation-plan.md: voxels → centreline → cuts) on these voxels too, so the model and the MRI
meshes are measured by identical code, and the fast cut of §6.2 is checked against it.

## 7. Behaviour the code must keep

- **Continuity.** Areas change smoothly with every parameter and articulator: no lamina switches
  topology without a ramp, no station jumps, no band in/out flips without fading (the 2D model's
  history in `human.rs` comments lists the traps: rays switching surfaces, pockets flipping
  crossings, sample points popping). A sweep test like `tests/geometry_continuity.rs` covers the
  3D area function.
- **Allocation-free compute.** `Tract3d::compute` (laminae, cuts, area function; not meshes)
  reuses its buffers after the first call, like `HumanVocalTract::compute`, so phase 2 can call it
  from the voice. Transcendentals only through `chimera::math` (engine contract,
  `docs/rust-engine.md`).
- **No effect on the voice.** No file under `voice/`, `airway/`, `control/`, `phonetics/` or the
  worklet changes; goldens are not re-blessed.
- **Cost.** Target, measured in release on this machine with the default 17 laminae and 72 cuts:
  `compute` ≤ 5 ms native, the `tract3d` JSON call with meshes ≤ 16 ms in the browser. Report
  the measured numbers and the load; if missed, reduce laminae or cut count before optimising.

## 8. API

### Rust (`crates/chimera/src/anatomy/tract3d/`)

```rust
pub struct ShapeParams { /* §4, serde camelCase, defaults = neutral offsets */ }
pub struct ParamSpec { pub key: &'static str, pub element: &'static str, pub unit: &'static str,
                       pub min: f64, pub max: f64, pub default: f64, pub doc: &'static str }
pub struct Tract3d { /* HumanVocalTract, elements, laminae buffers */ }
impl Tract3d {
    pub fn new(body: HumanBody) -> Self;
    pub fn specs() -> &'static [ParamSpec];
    pub fn derive(&self, a: &HumanArtic) -> ShapeParams;             // §5
    pub fn compute(&mut self, a: &HumanArtic, offsets: &ShapeParams, out: &mut Tract3dFrame);
    pub fn cut(&self, origin: Vec2, u: Vec2, out: &mut Cut);         // any plane containing ẑ
    pub fn meshes(&self, out: &mut ElementMeshes);                    // display only, may allocate
    pub fn air(&self, x: f64, y: f64, z: f64) -> bool;                // voxelization
}
pub struct Tract3dFrame { pub centreline: Vec<Vec3>, pub slices: Vec<Cut>,
                          pub area_fn: AreaFunction, pub cavities: Vec<SideCavity>, … }
```

Module files: `mod.rs`, `params.rs`, `lamina.rs`, `cut.rs`, `mesh.rs`, and one file per element.

### Browser

- `crates/chimera-wasm/src/api.rs` gains JSON calls: `tract3dSpecs` (the parameter specs),
  `tract3d` (`{model, artic, shape, want: {meshes, slices, planes: [...]}}` → element meshes as
  flat position arrays plus index arrays with fixed vertex counts per element, the centreline,
  the cuts and the area function), reusing the page's `Model` (same body as the voice).
- `src/wasm/engine.ts` and `src/wasm/types.ts` get the typed wrappers. No change to the C ABI,
  the gesture records or `LAYOUT_VERSION`.
- Meshes keep a fixed vertex count per element and body, so the lab updates positions in place
  (no geometry rebuild per frame).

## 9. The lab mode

A sixth mode, **Lab**, beside Play, Speak, Sing, Bestiary and Beatbox (`Mode` in
`src/ui/store.ts`, the top bar's segmented control) and reachable as `?mode=lab` the way
`src/ui/mount.tsx` handles `?mode=speak`. Existing modes are untouched. It follows the wall-of-
screens layout and the vector-display look (`docs/ui-redesign.md`): lines on black, blue
structure, amber sound, four control colours.

| Screen | Shows | Controls |
|---|---|---|
| **TRACT** (tall, left) | the 3D model: each element as line grids in its own colour; the current cut drawn in the model | drag to turn, wheel/pinch to zoom; element chips: show/hide, solo; *cut away* (hide z > 0 to look inside); *explode* (pull elements apart) |
| **CUT** | the current cut's outline(s) with area, channel count, width × height; plane chooser: perpendicular / midsagittal / coronal / axial | position encoder (s, or x₀ / y₀) |
| **AREA** | the 3D area function (amber), the engine's current 1D area function (dim) on the same axes, cut marker, channel markers; length, volume, minimum area and where | — |
| **SHAPE** | encoders for the 3D parameters, paged by element (pages of four, the existing `Encoder`); presets: flat, /a i u ə/, /s/, /ʃ/, /l/, bunched /ɹ/, retroflex /ɹ/ | preset chips set both the 17 articulators and the 3D offsets |
| **SOUND** | the engine's spectrum for the posture (existing transfer function view); the line "Sound: current engine. Shape-only parameters are heard from phase 2." | *Hold* (sustain the voice on this posture), *Say* (a short IPA string through the speech planner, e.g. "asa"), *Follow* (the model follows the live voice's articulators from telemetry) |

Articulator encoders for the 17 midline articulators reuse the Play mode's pages (MOUTH, THROAT,
LARYNX). Lab state (offsets, cut, visibility, follow) lives in the store; the voice gets only the
17 articulators through `setArtic`, as Play mode does.

Rendering reuses `beams3d.ts` and the orbit handling of `AnatomyView3D.tsx`; the old decorative
airway ellipses are not used in the lab. Whether the face screen's 3D view should later draw the
engine's meshes in place of its own sweep is left for a follow-up (README, deferred).

## 10. Phase 2, outline

**Real-time acoustics from the 3D geometry.**
- `HumanAirway` gets a 3D path: each control step (or every n-th, with area interpolation, if the
  measured cost demands) it calls `Tract3d::compute`, splits the main area function at the
  velopharyngeal junction into the pharynx and oral tubes, sets per-section perimeter (wall
  losses, currently one constant per tube), and drives side branches from the cavity
  descriptors: piriform L/R (replacing today's fixed 2 cm × 0.9 cm² tube), valleculae, the
  sublingual cavity of /ɹ/, and the buccal space.
- Noise sources take the constriction and the jet's obstacle from the 3D geometry (groove exit,
  teeth position), replacing the midline estimate.
- The 3D-only parameters become articulators: `HumanArtic` grows, gesture records change in both
  `crates/chimera-wasm/src/lib.rs` and `src/wasm/wire.ts` with a `LAYOUT_VERSION` bump, phoneme
  inventory targets gain the new keys (groove for sibilants, sides for /l/, …).
- The calibrated vowel and consonant targets are re-run (`chimera calibrate`), goldens re-blessed,
  and the hill-climb judges decide whether the sound improved (`docs/hill-climb/README.md`).

**Offline reference and "precise render".**
- A 3D acoustic solver (FDTD on the voxelized air, 1–2 mm grid, the approach validated against 3D
  prints up to 10 kHz in [R10]) computes the transfer function of a posture; DVTD's measured
  transfer functions of its 3D prints validate the solver on the DVTD meshes first.
- The 1D path is compared with it per posture (where they differ above 4–5 kHz is expected
  [R18]); a "precise render" mode renders sustained sounds through the 3D impulse response.

**Hooks phase 1 leaves.** `Tract3d::compute` allocation-free with perimeter and cavity
descriptors; `air()` and the CLI voxelizer; the DVTD reference pipeline and metrics; the lab's
AREA screen, which already shows the 3D-versus-engine difference phase 2 will close.
