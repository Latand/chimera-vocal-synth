# Design: sound from the 3D tract's waves

*Design stage, 2026-11-10, base revision 6a0602d. Companion documents:
[problems.md](problems.md) (what is wrong today, measured), [consonants.md](consonants.md)
(each consonant class: physics, parameters, natural ranges), [plan.md](plan.md) (the build
order with acceptance numbers). Evidence is outside git under
`$LLV_EVIDENCE_DIR/acoustics3d/design/`.*

## 0. The requirement

The operator, by voice message (the originals are kept in the task record):

> 2026-11-09, about 22:00 UTC (Ukrainian): «…щоб він пропрацював 3д правильно спочатку і
> швидко форми всі пофіксив, потім вже лише почне портувати на раст і щоб там рахувалося все
> від 3д хвиль, я афк, доробляй це повністю».
>
> In English: "…so that it works the 3D through properly first and quickly fixes all the
> shapes, and only then starts porting to Rust, and there everything is to be computed from
> 3D waves; I am away, finish this completely."

> 2026-11-10, about 05:40 UTC (Russian): «звук звучит всё ещё слишком
> синтезированно-роботически, если есть возможность улучшить каким-то образом за счёт… того,
> что он 3D… Реально звуковые волны двигаются по полости. Может быть, тогда будет менее
> синтетический звук… прям такой, максимально реалистичный, но при этом сгенерированный. Это
> нужно продумать. И звуки типа TPD, и другие шипящие, и не только шипящие, это нужно тоже
> продумать, потому что они, честно, иногда пропадают. Вот он говорит какую-то фразу, "Hello
> can you hear me"… и вот это "can"… почти первое… Буква, почти не слышно. Оно на них как-то
> очень неправильно делает».
>
> In English: "the sound is still too synthesised, too robotic; if there is a way to improve
> it through the fact that it is 3D… the sound waves really move through the cavity. Maybe
> then the sound will be less synthetic… as realistic as possible, and still generated. This
> has to be thought through. And sounds like T, P, D, and the other hissing ones, and others
> too, this has to be thought through as well, because, honestly, they sometimes vanish. It
> says some phrase, "Hello can you hear me"… and that "can"… almost the first… letter, you
> can hardly hear it. It does something very wrong on them."

> 2026-11-10, about 06:05 UTC: this is now the top priority; do it right away rather than
> polishing other things.

An earlier decision (2026-11-09, the operator's form): a real-time path from the 3D geometry
in the engine, plus an offline 3D wave solver as the reference and a "precise render" mode.

The lane's acceptance, from the task: (A) every human mode's voice is computed from the 3D
geometry, directly or through a real-time model derived from the 3D solver whose accuracy
against it is measured; the solver exists as the reference and as a precise render; the
sound is less synthetic, shown by before/after renders. (B) Consonants are reliably audible
and correct, "can" first, measured against natural ranges. (C) The speech judge's word error
rate improves; singing does not regress. (D) Real time stays real time in wasm; the engine's
contract holds; goldens are re-blessed; creatures keep their models. (E) The project's checks
pass.

## 1. The decision on one screen

1. **One physics at two resolutions.** The engine already solves pressure in volumes and flow
   between them (`airway/network.rs`). The reference solver applies the same equations to the
   air of the 3D tract cut into cells of 2 mm, each cell with its true air volume and each
   face with its true open area. The real-time voice applies them to the same air cut into
   about forty sections and a handful of side branches. Both read one geometry
   (`anatomy/tract3d`), use one glottis, one set of noise sources, one planner.
2. **The reference solver** (new module `chimera::acoustics3d`) gives a posture's transfer
   function and impulse response, and, with moving walls, renders whole phrases: the
   **precise render** (`chimera render precise`). It is checked against analytic resonators
   and against the transfer functions measured on 3D prints of real tracts (DVTD, already on
   the validation machine).
3. **The real-time voice** takes each section's area, perimeter and tissue, its side branches,
   and each jet's exit and obstacle from the 3D cuts, in place of the midline area law. A
   posture's 3D geometry costs 2.4 ms, so it is computed in slices spread over control steps
   and refreshed 50–100 times a second, with the midline model carrying each section between
   refreshes. Its transfer functions are compared with the solver's on a fixed set of
   postures; the limits are in plan.md.
4. **Direct 3D in real time is ruled out by measurement** (§3): the tract's cells alone need
   1.6–3.1 P-cores at 2 mm before anything else is counted, and 2 mm without sub-cell
   geometry gets close vowels wrong by 20 %.
5. **The consonants are fixed in the sources and the planner** (consonants.md): the air
   supply, one turbulence law for every jet, where the jet is and what it hits (from the 3D
   cuts), release speed by articulator, voice onset time by context. These changes are heard
   in both resolutions.

## 2. Where the sound is computed today

A gesture score moves 17 articulators; `anatomy/human.rs` turns them into midline outlines
and an area function by a law A = α·d^β per region; `voice/human_airway.rs` feeds 18
pharyngeal and 18 oral sections (male), a trachea, a nasal branch with one sinus, one
piriform tube and the lips' and nostrils' radiation loads; `airway/network.rs` steps the
76 nodes and 75 branches at 96 kHz; three band-limited noise sources sit at the glottis, at
the narrowest section and at the teeth. The 3D tract (`anatomy/tract3d`) is drawn and
measured and does not reach the voice (`docs/tract3d/README.md`).

## 3. What was measured for this design

Machine: Intel i9-12900K, one pinned P-core thread, other work running (load average 7–14);
best of several runs. Logs: `cost/`, `fdtd3d/`.

### 3.1 Costs

| What | Measured |
|---|---|
| Today's voice, male, 2× oversampling, native | 18.0–20.4 % of a core |
| The same in wasm (Bun, JavaScriptCore; Node 22, V8), after a warm-up | 30.8–31.1 %; 32.2–32.6 % |
| Of that: the midline geometry per control step | 3.3 % of a core (44 µs per step, 750 steps a second) |
| Of that: the wave step | 16.6 % of a core: 1.73 µs for 76 nodes and 75 branches, about 23 ns per node and branch |
| The 3D tract's geometry, one posture (65 laminae, 73 cuts) | 2.4 ms median, 4.2 ms worst |
| Voxels of one posture from the laminae | 1.9–2.3 ms at 2 mm, 13–16 ms at 1 mm |
| A 3D wave step, structured box with wall masks, 32-bit floats, native (AVX2) | 700–1150 million cell-steps a second (166 000–198 000 cells); 115–125 million with 1.4 million cells and five runs sharing the memory bus |
| The same kernel in wasm with simd128 | 760 million (V8), 765 million (JavaScriptCore) |
| The same kernel in wasm without SIMD | 180 million (V8), 248 million (JavaScriptCore) |
| The tract's air at 2 mm (male) | 5 700–8 800 cells (45–71 cm³), stepped at 306 kHz |
| A box round the tract with 4 cm of free air in front, 2 mm | 166 000–198 000 cells: 44–58 times slower than real time |
| The tract's cells alone at the box kernel's speed, 2 mm | 1.6–3.1 times a core |
| 1 mm | 16 times the 2 mm cost (8 times the cells, twice the rate) |

The prototype kernel (`scripts/fdtd3d`) is a plain staggered-grid FDTD. On a closed-open
tube of 17 cm its first four resonances are within 0.3 % of the analytic values at 2 mm and
at 1 mm, and the next four within 1.5 % (the analytic end correction is the low-frequency
one).

### 3.2 A first look at the 3D tract's response

The prototype was run on postures of the project's own 3D tract (male): the laminae's air as
voxels, rigid walls, a flow pulse at the glottis, the flow through the lip plane, free air in
front of a flat face. The lips of the model are slabs with no front surface, so the opening
in front of the lip line is this stage's own construction; the numbers below are a first
look. Beside it: the engine's 1D chain on the 3D area function (rigid walls, the engine's
losses and radiation load), and today's voice for the same articulation.

| Posture | Resonances, Hz: 3D at 1 mm | 1D chain on the 3D areas | Today's voice |
|---|---|---|---|
| /a/ | 686, [1133], 1358, [1748], 2598, 3820 | 702, 1341, 2635, 4015 | 718, 1449, 2552, 3882 |
| /i/ | 259, 2244, 2472, 2864, 3521 | 293, 2440, 2944, 3927 | 336, 2237, 2843, 3994 |
| /u/ | 233, 762, 1237, 1328, 2282 | 290, 965, 2423, 3858 | 378, 997, 2247, 3757 |

Plots: `fdtd3d/tf-h0.1-a-i-u.png`, `fdtd3d/tf-h0.2-a-i.png`, `fdtd3d/tf-a-width.png`. Four
findings decide parts of this design:

1. **A staircase grid is wrong where the tract is narrow.** The /i/'s first resonance is
   230 Hz at 2 mm, 243 Hz at 1.5 mm, 259 Hz at 1 mm; a straight line through the last two
   reaches 291 Hz at zero spacing, where the 1D chain has 293 Hz. Against that limit the
   grids are −21 %, −17 % and −12 % off. The /u/'s lip opening (0.21 cm²) is 21 faces at
   1 mm. Consonant constrictions are narrower still. The reference solver therefore carries
   each cell's true air volume and each face's true open area (§5.2); whole voxels of 0.5 mm
   would still be about 6 % off at 256 times the cost of 2 mm.
2. **The 3D geometry has resonators that an area function hides.** The /a/ has an extra
   pole–zero pair at 1.1–1.8 kHz (bracketed above). Clipping the tract to ±1.5 cm of the
   midline removes it (resonances 715, 1443, 2566, 3468 Hz at 2 mm); at ±2 cm it is back. It
   is the air beside the tongue, between the teeth and the cheeks. Published 3D computations
   on MRI tracts find only very small dips below 2.5 kHz from the spaces between the teeth
   (`docs/tract3d/research.md`, R10), so the model's side spaces are probably too large or
   too open; only the validated solver, run on the MRI meshes and on the model, can say
   (plan.md step 2).
3. **Above 4 kHz the 3D response is another kind of curve.** It has zeros 20–40 dB deep (/a/:
   near 6, 8 and 9 kHz; /i/: 3.7, 5.9, 6.8 kHz) and a falling level, where the tube model
   has a regular row of peaks. This is where published 1D and 3D computations part too
   (R18).
4. **Below 3 kHz the tube model on the 3D areas is close for the open vowel**: the 1 mm 3D
   run of /a/ has its F1, F2 and F3 at −2 %, +1 % and −1 % from the tube's, and today's
   voice sits 2–8 % from both. For the close vowels the comparison waits for a solver that
   resolves their constrictions.

## 4. The options

| Option | What it is | Measured or known cost | Verdict |
|---|---|---|---|
| A. 3D waves in real time | the solver in the audio thread | tract cells alone 1.6–3.1 cores at 2 mm natively; the geometry refresh (4–5 ms a posture) on top; one thread in the worklet | no: out by an order of magnitude, and 2 mm is too coarse without sub-cell geometry |
| B. A coarse 3D grid in real time (4–5 mm, sub-cell geometry) | about 1 000 cells at 150 kHz | 1.5·10⁸ cell-steps a second: affordable; the geometry refresh is the larger cost | no for now: a 4 mm cell is wider than every consonant constriction, so the constrictions would have to be modelled as 1D elements inside it; that is option D with more machinery |
| C. The 1D tube on the 3D area function only | the phase-2 outline of `docs/tract3d` | today's cost plus the geometry | part of the answer: §3.2 finding 4. Alone it hides the side resonators and has no account of what it misses |
| D. **A 1D network derived from the 3D geometry and checked against the 3D solver** | C plus side branches from the 3D cuts, per-section perimeter and tissue, jets and obstacles from the 3D shape, corrections fitted to the solver where the comparison demands them | today's cost plus 15–25 % in the wave step (12–20 more nodes) plus the sliced geometry | **chosen for real time** |
| E. Transverse modes in real time (multimodal, or sections split into side-by-side channels) | brings the first cross modes (6–8 kHz in a symmetric tract) | each split section costs two or three sections; the wave step today is 23 ns per element | deferred: decide from the measured gap between D and the solver above 4 kHz, and by ear from the precise render (§9) |
| F. Precomputed 3D responses per posture | tables or a fitted map from posture to a filter | 59 dimensions of posture; a filter has no closures, bursts or flow | no for the voice. The solver's responses are used offline, to check and to fit D |
| G. **The 3D solver offline** | 2 mm cells with true volumes and openings | §6: a 2 s phrase in tens of seconds on this machine's cores | **chosen as the reference and the precise render** |

## 5. The design

### 5.1 One physics, two resolutions

The engine's network is already a finite-volume method: a node is a volume of air with
pressure p and compliance V/ρc², a branch carries volume flow U through an inertance, and
the leapfrog step exchanges them (`airway/network.rs`, header). It handles what speech needs
and plain wave solvers lack: steady flow, closures (a branch below a threshold area passes
nothing), walls that move (−dV/dt as a source), walls that yield (a mass, spring and damper
per wall area), kinetic losses at constrictions, noise sources in series with a branch,
radiation loads. A chain of such nodes is the 1D tube. A Cartesian grid of them, with
V = φ·h³ per cell and L = ρ·h/(a·h²) per face (φ the cell's air fraction, a the face's open
fraction), is a conformal 3D solver.

Stability is the same kind of rule in both. On a grid a node's compliance may not fall
below T²·Σ(1/L)/2 (T the time step, the sum over the node's branches); the engine applies a
floor of the same form at its junctions, with the constant 2.4 that suits its branched
chains. On the grid this bounds a cut cell's volume fraction from below by the sum of its
faces' openings times (cT/h)²/2, which a whole cell meets exactly at the usual limit
cT/h = 1/√3. A cell under the bound is given the bound (it is then slightly too compliant,
a known and small error) or is merged with its neighbour.

What the two resolutions share, by construction: the geometry (`Tract3d`), the glottis and
its port, the lungs, trachea and nose (1D branches in both), the wall tissues, the noise
laws, the planner and the control loop of `HumanVoice`.

### 5.2 The reference solver: `chimera::acoustics3d`

- **Cells.** A grid of spacing h (default 2 mm; 1 mm and 3 mm for convergence and drafts)
  over the tract's bounding box. From the laminae: each cell's air fraction and each face's
  open fraction, by integrating the laminae's air intervals over the cell (the intervals are
  exact along x; the other two directions are sampled at the laminae's 1 mm). Only cells
  with air are stored (a list with six neighbour indices), so the cost follows the air.
- **Walls.** A cell's wall area is its closed face area. Each wall cell carries the engine's
  tissue (mass, resistance, stiffness per area) by the tissue polygon that closes it: tongue,
  pharyngeal wall, cheek, lip, velum; teeth and hard palate rigid. This is the same wall
  model as the 1D voice, with the true wall area and tissue (the item `docs/tract3d` left for
  this lane).
- **Outside.** Free air in front of the mouth on a spherical head (radius by body size), the
  lips' own shape from the model's lip line, an absorbing layer 3–4 cm out. The output is
  the pressure at a point on the mouth's axis (and the lip flow, to compare with the 1D
  model). The model's lips have no front surface today; the solver needs one small addition
  to `tract3d` for it (§5.7).
- **Ends that stay 1D.** The glottis is the engine's port between the trachea's last node
  and the cells at the glottis. The velopharyngeal port joins the cells under the velum to
  the 1D nasal branch. The soft palate's membrane couples the same two places.
- **Sources.** Flow at the glottis through the port (the engine's parametric glottis,
  unchanged). Noise as pressure in series with faces: at the jet's exit and on the faces of
  the obstacle it meets (consonants.md §3).
- **Motion.** Each control step the fractions are recomputed and ramped; a cell's −dV/dt is
  a flow source; a face under the closure threshold passes nothing; the engine's rules for a
  closing cell (the squeezed air goes to open neighbours) carry over.
- **Losses.** Viscous and thermal wall losses per wall area at the engine's reference
  frequency, as in the 1D voice.
- **Uses.** (1) *Response*: a posture's transfer function and impulse response
  (`chimera acoustics3d response`). (2) *Check*: against analytic resonators, grid
  convergence, and the measured prints (`chimera acoustics3d validate`). (3) *Fit and
  compare*: the real-time airway against it per posture (`chimera acoustics3d compare`).
  (4) *Precise render* (§5.3).

### 5.3 The precise render

`HumanVoice` gets a second airway behind the interface the first one already has: the
pharynx and the mouth as the solver's cells, everything else as today. The planner, the
articulators, the glottis and the limiter are the same code, so a precise render differs
from the real-time one in the waves alone. `chimera render precise <out> "text" [body]
[--grid mm]` writes the WAV; `--compare` renders the real-time voice beside it and prints
the band levels of the difference. The whole network steps at the grid's rate (306 kHz at
2 mm), the 1D parts included, and the output is decimated to 48 kHz.

An action in the app (a worker running the same code in wasm, with a progress bar and the
result played back) is the last step of the plan and is taken only if the measured time
allows (§6).

### 5.4 The real-time voice from the 3D geometry

`HumanAirway` keeps its tubes and its step. What feeds it changes (new module
`acoustics3d::reduce`, read by `voice/human_airway.rs`):

1. **Sections from 3D cuts.** At the centre of each of the tube's sections: the cut's area
   (main channel), its narrowest sub-section, its perimeter split by tissue, and the regions
   beside the main channel. The area sets the node; the perimeter and tissue set the wall
   loss and the yielding wall (in place of fixed perimeter factors and regional shares).
2. **Two rates.** The 3D geometry of a posture is computed in slices over K control steps
   (a few laminae per step, the cuts at the end), so no control step carries more than its
   share; K is chosen so a refresh lands every 10–20 ms. Between refreshes the midline model
   (which runs every control step, 44 µs) carries each section: the section's area is the
   midline law's area times the ratio the last refresh measured, and the ratio is ramped
   from one refresh to the next. Where the midline closes, the area closes with it, so
   closures and releases keep their control-step timing; where the 3D cut has air and the
   midline has none (a groove, the sides of an /l/), the 3D area stands. A test compares the
   areas the voice used with the 3D cuts computed at every control step, over the training
   sentences (plan.md).
3. **Side branches from the cuts' side regions.** The piriform fossae (two, from their
   volume, neck and depth), the valleculae, the space under a raised tongue tip, the pocket
   above the tongue behind an /l/ seal with its two side channels as a parallel path, and
   the space beside the tongue. Each is a short tube or a cavity on a neck (the network has
   both), sized at each refresh, shut when the region is absent. Whether the space beside
   the tongue is kept as a resonator or folded into its section's volume is decided by the
   solver on the MRI meshes (§3.2, finding 2).
4. **Jets.** For each narrow place: the channel's exit (the most anterior narrow cut), the
   jet's width and height, the first surface along its axis and its distance, the cavity in
   front. The noise sources of consonants.md take these in place of "narrowest section" and
   the teeth's distance window. The junction of the two tubes is a section like any other.
5. **Corrections, only where the comparison demands them.** The network already adds the
   mass of abrupt area changes. After items 1–3 the builder measures the remaining gap to
   the solver (§5.5). A gap that is systematic gets a physical term with its constant fitted
   to the solver over the posture set and written to a generated table
   (`acoustics3d.generated.json`, by `chimera calibrate acoustics3d`): candidates are an end
   correction for the lips' true shape and a length correction for the bend. No filter is
   applied to the output (the hill-climb rule against output shaping stands).
6. **The 17 articulators stay the gesture record.** `Tract3d::derive` already turns them
   into the 3D shape (a groove under a raised tip, lowered sides for laterals). If a
   consonant's acceptance line is missed and the solver shows the tuned preset's shape would
   pass, the missing shape becomes a control key with the record change in both
   `chimera-wasm/src/lib.rs` and `src/wasm/wire.ts` and a `LAYOUT_VERSION` bump. Until then the
   record stays as it is.
7. **Recalibration.** The vowel and consonant tables are regenerated against the new airway
   (`chimera calibrate`), the goldens re-blessed.

### 5.5 How the accuracy is measured

Three comparisons, each a CLI command that writes a table (errors only for anything derived
from the MRI data, as `docs/tract3d` does):

1. **Solver against physics**: a closed-open tube, a two-cavity resonator with a narrow neck
   (the case a staircase fails), a tube with a side branch; resonances and the zero against
   chain-matrix values; convergence between 2 mm and 1 mm on /i/, /u/ and /s/.
2. **Solver against measurement**: the 44 DVTD meshes with rigid walls at the prints' speed
   of sound, against the transfer functions measured on the prints (the 1D chain's error on
   the same data is in `docs/tract3d/validation-results.md` §7: F1 +4.7 %, F2 +4.8 %, curve
   2.5 dB RMS up to 1 kHz).
3. **Real-time airway against the solver**: the 16 presets and 24 postures sampled from the
   training sentences (vowel midpoints, fricative and nasal midpoints), both bodies:
   resonance errors, the curve's level difference in third-octave bands to 4 kHz and,
   reported, to 10 kHz. And **the model against the MRI meshes through the solver** for the
   sounds DVTD has, which is the acoustic check of the 3D shapes that phase 1 could not
   make.

### 5.6 Sources and the planner

Described in consonants.md: the lungs' termination, one turbulence law for the glottal jet,
free jets and jets on obstacles, release speed by articulator, voice onset by context,
voiced obstruents' pressure, the thump before a phrase-initial closure. They are
independent of the resolution and come first in the plan, because they are what the
operator hears first.

### 5.7 Contract, creatures, goldens, files

- **Engine contract** (`docs/rust-engine.md`): the real-time path allocates nothing while
  rendering (the reduce module's buffers are built with the airway; a body switch may
  allocate, as now); transcendental math through `chimera::math`; block-size independence
  holds because the geometry slices are counted in control steps at absolute sample
  positions. The existing tests for each stand, and the allocation test's scenario gains
  speech through the 3D path. The solver is offline code under the same math rule.
- **Creatures** keep the plain network: every new term in `airway/network.rs` sits behind a
  switch that only the human airway sets, as `discontinuity` and `thermal` do, and the
  creatures' goldens stay bit-identical (they cannot be re-blessed).
- **Goldens**: human voice audio, plans, the airway's transfer functions and the geometry
  snapshots re-bless (`CHIMERA_BLESS=1 cargo test --workspace --no-fail-fast`, twice).
- **Files.** New: `crates/chimera/src/acoustics3d/` (`mod.rs`, `grid.rs`, `solver.rs`,
  `response.rs`, `airway3d.rs`, `reduce.rs`, `jets.rs`), CLI commands under
  `crates/chimera-cli/src/acoustics3d/`, tests `crates/chimera/tests/acoustics3d.rs`, the
  measuring tools of problems.md brought into the repository (`chimera diagnose consonants`,
  `tools/asr/wordtest.py`, `tools/asr/transcribe.py`: new files, the judges untouched).
  Changed: `voice/human_airway.rs`, `voice/human_voice.rs`, `airway/network.rs` (behind
  switches), `phonetics/planner.rs`, `phonetics/inventory.rs`, `sources/glottis.rs`.
  Additive only in `anatomy/tract3d/` (it has another owner): cuts at given stations with
  perimeter by tissue and side-region descriptors, the tissue at a point, the lips' front
  surface, a sliced `compute`. Nothing in `src/ui/` is needed for the real-time path; the
  precise render's app action adds one worker and one button and is the plan's last step.

## 6. Expected costs

| What | Expectation | Basis |
|---|---|---|
| Real-time wave step | +15–25 % (12–20 more nodes on 76) | 23 ns per element, measured |
| Real-time geometry | the sliced 3D refresh must fit in 2–4 % of a core natively: 0.2–0.4 ms of 3D work per 10 ms | today's 2.4 ms a posture is 6–12 times that; the sections need 36 cuts where the full frame makes 73, a symmetric posture needs 33 laminae and could do with every second one; to be measured, with the refresh rate as the fallback lever |
| Real-time total | native 18–20 % → 24–28 %; wasm 31–33 % → 40–46 % | sum of the above |
| Guard | wasm load at most 1.25 times the base, both measured back to back on one machine (`tools/bench-wasm.ts`), and no 128-sample block above half its duration | plan.md |
| If the guard is missed | the human wave step gets a fast path for branches with no kinetic loss and control-rate coefficients (creatures untouched); `docs/rust-engine.md` lists it as open | 23 ns per element against about 1 ns for a plain cell-step |
| Reference solver, a posture's response at 2 mm | 60 ms of sound for about 8 000 tract cells and a compact outside region (about 20 000 cells in all): 3.7·10⁸ cell-steps, 1–3 s | the box kernel does 10⁹ a second; an air-only list kernel is assumed 3–8 times slower per cell-step, to be measured |
| Precise render at 2 mm | 6·10⁹ cell-steps per second of sound: 20–50 times real time on one core, 3–8 times on eight; plus the cells' fractions per control step | the same |
| Precise render in the browser | one thread: 1–2 minutes for a 2 s phrase at 2 mm, 10–25 s at 3 mm | wasm with simd128 ran the box kernel at native speed; without SIMD it was three to four times slower |

## 7. Risks and the rules that settle them

| Risk | Rule |
|---|---|
| The solver cannot be trusted as a reference | it is not used to judge anything until it passes the analytic cases, the convergence check and the prints' check (plan.md step 2) |
| The model's side spaces are not real | solver on the MRI meshes against solver on the fitted model, same sounds: a pole–zero pair present in the model and absent on the mesh is listed for the geometry's owner; until the shape changes, the real-time airway folds that space into its section's volume |
| The two-rate geometry mistimes a release or clicks | the area-trajectory test of §5.4; if it fails at every K that fits the budget, the constriction's own sections are refreshed every control step from a small set of laminae |
| Moving cells in the precise render click or blow up | the engine's closure and squeeze rules, the stability floor, a test that renders the battery and checks for steps in the output; if a release cannot be made clean in the time, the precise render ships for voiced passages and says so (it remains the reference for transfer functions) |
| Real-time cost | §6: the guard, then the fast path |
| The speech judge's noise (±2 points) hides the gain | the acoustic lines of consonants.md §9 and the word test decide the consonants; the judge must be at least not worse and is expected better by more than its noise |
| Singing regresses (louder sibilants raised note MCD in the hill climb) | the singer comparison runs at every step that changes sound; `docs/hill-climb` and its judges are not edited; a guard that fails is reported with its number |

## 8. Validation against the requirement

| The operator's words | Where the design answers |
|---|---|
| "everything is to be computed from 3D waves" | the precise render computes the mouth and pharynx's waves in 3D (§5.3); the real-time voice is the same physics reduced, fed by the 3D geometry and checked against the 3D solver (§5.4, §5.5); direct 3D in real time is ruled out by measurement (§3.1) |
| "the sound waves really move through the cavity… less synthetic… as realistic as possible, and still generated" | 3D response with its side branches and zeros (§3.2); walls by true area and tissue; breath noise in the voice (consonants.md §4); vowels after stops voiced again; before/after renders, spectrograms and a written comparison (plan.md step 8) |
| "T, P, D… the hissing ones, and others… sometimes vanish" | problems.md §4–5 (measured causes), consonants.md (the model and the acceptance lines for stops, fricatives, sibilants, nasals, liquids) |
| "'can'… you can hardly hear it" | problems.md §2; acceptance lines K1–K4 |
| "think it through" | problems.md, the prototype and cost measurements of §3, the options of §4 |
| top priority, right away | plan.md puts the audible consonant fixes first |

## 9. Deferred — not currently justified

- **Transverse modes in real time** (sections split into channels, or a multimodal model).
  Trigger: the measured gap between the real-time airway and the solver above 4 kHz, heard
  in a precise-against-real-time comparison that the operator judges worth the cost.
- **A coarse 3D grid in real time** (option B). Trigger: the same, if channels do not close
  the gap.
- **The nose and sinuses in 3D.** No 3D nasal geometry exists; the 1D branch stays in both
  resolutions.
- **The subglottal airway in 3D**, and self-oscillating folds: the parametric glottis stays.
- **New articulator keys for 3D shapes** (groove, side elevation): only under the rule of
  §5.4 item 6.
- **Intonation and rhythm.** 29 % of voiced time sits on a flat pitch and stressed vowels
  are 1.5 times the unstressed ones (problems.md §6). Both make speech sound mechanical;
  neither is acoustics or consonants, and the requirement asks for the improvement through
  the 3D sound. Worth a lane of its own.
- **The long-term spectrum's dip at 1–2 kHz** (problems.md §6): the glottal source's shape,
  a hill-climb lead; re-measure after this lane, since the 3D path and the walls move it.
- **Fixing the 3D shapes** where the acoustic check finds them wrong (side spaces, the lips'
  front): reported to the geometry's owner; this lane adds interfaces only.
- **A natural-speech corpus run through the measuring scripts** (aligned recordings), beyond
  the one reference voice of plan.md step 1.
- **GPU or multi-threaded wasm for the precise render in the app.**
- **Radiation directivity and the listener's position** beyond one point on the mouth's axis.

## 10. Not established by this stage

- The solver with sub-cell geometry was not built; the prototype is a whole-voxel FDTD with
  rigid walls and a flat face, good for cost and for a first look.
- The speed of an air-only list kernel (the cost table assumes 3–8 times the box kernel's
  time per cell-step).
- The real-time geometry's sliced cost (the budget in §6 is a requirement on it).
- Whether the side resonators of §3.2 are in real tracts.
- The wasm numbers are from Bun and Node on this machine; a browser's worklet was not timed.

## References

- R10, R18 and the DVTD data: `docs/tract3d/research.md` and `reference-data.md`.
- Arnela, Dabbaghchian, Blandin, Guasch, Engwall, Van Hirtum & Pelorson (2016). Influence of
  vocal tract geometry simplifications on the numerical simulation of vowel sounds. *J.
  Acoust. Soc. Am.* 140, 1707–1718 (R18).
- Bilbao, Hamilton, Botts & Savioja (2016). Finite volume time domain room acoustics
  simulation under general impedance boundary conditions. *IEEE/ACM Trans. Audio Speech
  Lang. Process.* 24, 161–173.
- Birkholz, Kürbis, Stone, Häsner, Blandin & Fleischer (2020). Printable 3D vocal tract
  shapes from MRI data and their acoustic and aerodynamic properties. *Sci. Data* 7, 255.
- Blandin, Arnela, Laboissière, Pelorson, Guasch, Van Hirtum & Laval (2015). Effects of
  higher order propagation modes in vocal tract like geometries. *J. Acoust. Soc. Am.* 137,
  832–843.
- Dang & Honda (1997). Acoustic characteristics of the piriform fossa in models and humans.
  *J. Acoust. Soc. Am.* 101, 456–465.
- Flanagan (1972). *Speech Analysis, Synthesis and Perception.* Springer.
- Maeda (1982). A digital simulation method of the vocal-tract system. *Speech Commun.* 1,
  199–229.
- Mohapatra, Zappi & Fels (2019). An extended two-dimensional vocal tract model for fast
  acoustic simulation of single-axis symmetric three-dimensional tubes. *Proc. Interspeech.*
- Sondhi (1986). Resonances of a bent vocal tract. *J. Acoust. Soc. Am.* 79, 1113–1116.
- Takemoto, Mokhtari & Kitamura (2010). Acoustic analysis of the vocal tract during vowel
  production by finite-difference time-domain method. *J. Acoust. Soc. Am.* 128, 3724–3738
  (R10).
