# Model design: a parametric 3D vocal tract

*Phase 1 design, 2026-11-09, base 2e567e2.* The evidence behind each choice is in
[research.md](research.md) (sources `[Rn]`) and [reference-data.md](reference-data.md); how the
model is checked against MRI is in [validation-plan.md](validation-plan.md); the build order and
what is deferred are in [README.md](README.md).

## 0. As built (phase 1)

The sections below are the design. This section lists where the built model differs and why;
the code (`crates/chimera/src/anatomy/tract3d/`) is the reference.

- **Laminae.** 65 laminae, 1 mm apart (× body size), over ±3.2 cm; a symmetric shape computes
  33 and mirrors them. No edge bisection: at 1 mm the cuts agree with the model's own voxels to
  under 1 % (validation-results.md).
- **Lateral rules that cannot fold an outline.** The first build offset curved outlines by up to
  2 cm along their normals; offsets larger than the curvature radius folded the palate and the
  tongue into self-intersecting polygons, and parity filling turned the folds into air strips
  beside the teeth (the side strips and extra channels in that build's MRI comparison). Now
  (`parts.rs`):
  - the palate and the lips move only vertically: graphs stay graphs (the pharyngeal walls
    are a tissue polygon of their own since §0.3; at first the posterior wall moved
    horizontally);
  - the tongue's dorsal surface moves along fibres from itself to a core curve inside the tongue
    (depth `T(t)`: 1.2 cm at the root, 1.5 cm over the body, 0.9 at the blade, 0.45 at the tip):
    `p = c + n·T·λ`, `λ = E(w) + (side(t)·S(w) − groove(t)·G(z))/T`, clamped to [0, 3]. `E` is
    flat over the middle 70 % of the width and rounds over the margins (`edgeRound`); at the
    root the whole width is rounded (the root bulges into the pharynx). Raising (`side` > 0)
    spreads from a quarter of the width; lowering (`side` < 0) stays at the margins, so /l/
    keeps its midline contact;
  - the tip lobe (in front of the line from the blade's start to the frenulum) shortens towards
    that line where the tongue is narrower than the lamina's offset: an affine compression, so
    the polygon stays simple;
  - teeth, lips, cheeks, faucial pillars and lateral pharyngeal walls are separate tissue
    polygons, united by counting.
- **Dental arches.** Each arch's lingual edge has half-width `width·u^e` (`u` the depth fraction
  from the incisors, `e = 0.28 + 0.5·archTaper`), its buccal edge one tooth width (0.7 cm at the
  incisors, 1 cm at the molars) further out along the normal. In a lamina the tooth row is the
  stretch between the two edges, from the occlusal plane up (upper) or down (lower; carried by
  the jaw's rotation, read from the frame). Upper cusps overlap the lower ones by 1.2 mm, so a
  tooth row seals until the jaw opens. The rows rise into place 2.5–4.5 mm off the midline,
  where the engine's own incisors are. Lips and cheeks follow the arch back (the mouth's corners
  sit beside the canines); `buccalGap` is the vestibule between teeth and cheek (default 0: the
  cheeks rest on the teeth, as in most MRI).
- **Palate.** The vault drops at most to the gum line (occlusal plane + crown height), so the
  alveolar ridge, already near the gum line, barely drops.
- **Pharynx and larynx.** Rebuilt in §0.3 (a ring of walls, a free epiglottis, a lofted
  laryngeal tube, pouch-shaped fossae of the published size). The first build made a lamina
  beyond the pharynx's half-width all wall, rounded the posterior corners by the engine's own
  midline gap, and set an ellipsoid of 2.6 cm³ beside the tube for each fossa (the published
  pair's volume read as one fossa's).
- **Velum and uvula.** The uvula is a lobe `uvulaLength` cm long and `uvulaWidth` wide at the
  middle of the soft palate's free edge; beside it the edge recedes towards the root (the faucial
  arches), so a lamina meets the velum only where it is that wide.
- **Regions.** A cut drops regions under 0.01 cm² (one-lamina slivers between a tongue margin
  and a tooth row, far below MRI resolution) from its area and channel count; the faded area
  keeps them for continuity.
- **Parameters.** 40 at first, 41 with `lipRound` (§0.2), 42 with `aryFoldHeight` (§0.3)
  (`params.rs`, one table for struct, JSON names, ranges and the lab's controls). `toothGap`
  was dropped (interdental gaps are not modelled). `derive` maps the 17 articulators as §5
  says, with ten constants since §0.3; fitting them against DVTD did not justify changing the
  first eight (validation-results.md).
- **Presets.** 16 postures: the engine's own targets for a / i / u / ə / s / ʃ / l / f / ç / x,
  plus flat, funnel, cupped, bunched and retroflex /ɹ/, and the mouth wide open (§0.3). /s/,
  /ʃ/, /l/ and retroflex /ɹ/ were tuned with `chimera tract3d tune` until their feature
  checks pass (T-features, male template).
- **Side cavities** reported: piriform fossae and valleculae (volume, depth, opening, where they
  attach). The sublingual and buccal spaces are geometry (they appear in cuts) but have no
  descriptor yet.
- **API.** `tract3d` returns, per call: the parameters in use and what `derive` gives, the
  centreline, the area function, every cut's summary, one cut with its rows (`want.cut`, 0–1),
  one plane (`want.plane`: coronal / axial / sagittal at `want.at`), the cavities, and the element
  meshes (fixed vertex counts per body; triangles for hidden-line fills plus line pairs).
- **Lab.** As §9, with the plane chooser offering a sagittal plane at any lateral offset instead
  of only the midsagittal one, an x-ray toggle (lines behind surfaces), and the 3D-only
  parameters shown as the values in use (derive + offset).
- **Faded area.** A cut's `weightedArea` weighs every air interval by its own distance from the
  band (`1 − smoothstep(band, band + 0.3 cm, distance)`). It used to take one weight per
  region, from the region's nearest interval, and a pocket beside the tongue that lost its last
  overlapping lamina (connection is yes/no) left the region at once: 0.86 cm² in one 0.02 mm step
  of the tongue body at /ə/. `area`, the number compared with MRI, keeps its region rule.

### 0.1 Operator amendment (2026-11-09): head, uvula, low frequencies, walls

The operator asked for a head that looks like a real one from outside with correct layers
beneath the skin, for the uvula ("the small second tongue") in shape and sound, for the low
frequencies that did not fit, and for the soft tissues' vibration. This lifts phase 1's
no-sound-change fence for the acoustics; the voice still does not read the 3D tract.

- **Head (lab).** A head with skin, subcutaneous fat, muscle, bones and cartilages round the
  tract, cut open with its layers on the cut face. That first head (a loft of the face screen's
  outline, `head3d.ts`) was replaced the next day: §0.2 describes the head as it is.
- **Uvula and epiglottis.** Both were in the 3D model (the uvula as the soft palate's middle lobe,
  `uvulaLength`, `uvulaWidth`; the epiglottis plate with the valleculae). In the voice the soft
  palate with the uvula is now a vibrating membrane (`Membrane`, 5 cm² × k², 0.9 g/cm² plus the
  uvula's 0.55 g) between the top of the pharynx and the nasopharynx: with the port shut, close
  vowels still drive the nasal cavity through it (`chimera tract3d walls`). The uvula's static
  share of the oropharynx stays the engine's midsagittal velum tip.
- **Low frequencies.** The engine's 1D acoustics, run on DVTD's MRI area functions, put F1 a
  median 5.5 % above the transfer functions DVTD measured on 3D prints of the same tracts, with
  F1 bandwidths at 0.65 of measured. Two physical terms the plane-wave tube lacked: the
  evanescent-mode inertance at abrupt area changes (the length correction of a step between
  tubes, Kergomard & Garcia 1987) and the thermal boundary-layer loss beside the viscous one
  (`AirwayNetwork::discontinuity`, `::thermal`; on in the human tract). Numbers:
  [validation-results.md §7](validation-results.md#7-low-frequencies-the-engines-acoustics-against-dvtds-measured-prints).
- **Walls.** The human tract's walls were one tissue everywhere (`WALL_SOFT`, its own resonance
  at 71 Hz, the whole perimeter yielding). Now per section and control rate: tongue, pharyngeal
  constrictors, laryngeal walls, cheeks and lips by region, the hard palate and the teeth rigid,
  stiffness at Hanna et al.'s measured ~20 Hz tissue resonance, masses for their 1–2 cm
  effective thickness (`physics::TISSUE_*`). Numbers and spectra:
  [validation-results.md §8](validation-results.md#8-the-walls-and-the-soft-palate-in-the-voice).
- **Calibration.** The vowel table was regenerated with the new acoustics (`chimera calibrate
  vowels all`) so the vowels keep their target formants; the consonant table with it. The 2D
  model's resampling counts each section's two ends as half samples, so a closure reaching a
  section's edge shuts it continuously (the recalibrated /u/ found a 0.5 cm² flicker there).

### 0.2 Integration (2026-11-10): one head and tract on every screen

The operator's next directive, restated: the 3D tract everywhere and not only in the lab; sound
in the lab (it could not be heard there); and realism inside and out, close to a reference
picture of a sculpted head with an open mouth. The same day the operator ordered the work: the
3D shapes first, then, in a lane of its own, the port that computes every human sound from the
3D geometry. So this stage changes what is seen and how the fits are judged; **the voice still
reads its 1D area law** (README, deferred).

- **The skin** (`src/ui/render/head/`, display only). A mesh baked from the MakeHuman project's
  CC0 base mesh ([head-asset.md](head-asset.md): source, licence, what the generator does):
  three shapes (adult male, adult female, child) blended by the body's weights, eyes closed,
  the mouth open at a loop inside the lips, on a stand. It is registered round the engine's
  tract (the lips' meeting point on the engine's closed lips, the larynx inside the neck) and
  takes the body's own template map, so it stays round the tract for any body. `rig.ts` moves
  it with two things the engine reports (`Tract3d::display`): each vertex turns by its share of
  the jaw's turn, and the line where the skin's lips meet is put on the engine's upper and
  lower lip edges, the rest of each lip following by its weight. The opening seen from outside
  is the opening the cuts measure.
- **One scene** (`scene.ts`) for the lab, every human mode's face screen and the duet page:
  the engine's element meshes shaded by material, the skin round them, and, opened, the cut
  face. The look is judged by eye against the operator's reference picture (a pale sculpted
  head on a stand, the open mouth dark inside): the skin has vertex colours (the lips' tint
  held to the vermilion, a little warmth on the cheeks, nose, ears and chin, shade in every
  crease by the mesh's own curvature), a wrapped red-tinted diffuse term and soft room light;
  a key light that travels with the camera casts shadows, so the chin shades the neck and the
  mouth is lit only where light reaches through the lips. See-through makes the skin
  translucent round the tract (the material is compiled again when it is toggled: compiled
  opaque it kept its alpha at 1).
- **The cut face** (`section.ts`) is painted on a canvas mapped onto the cut: its air is the 3D
  tract's own lamina at the plane, by the rule the cuts use (envelope, less the tissue
  polygons, plus the pockets inside tissue), with every 3D parameter. Round it, inside the
  skin's own midline outline: skin, fat and muscle bands, the skull with the brain, the
  cervical spine and cord, the nasal cavity, the hard palate, the chin's bone, the hyoid, the
  laryngeal cartilages and the trachea, and the engine's tongue, soft palate and teeth.
  Thicknesses are typical adult values from anatomy texts: the cut face is a drawing and
  measures nothing.
- **Engine, display** (`mesh.rs`, `parts.rs`). Every drawn surface comes from the helper the
  laminae are cut from: the fossae and valleculae are the laminae's ellipsoids, the pharyngeal
  wall is the lamina's own polyline (`wall_point`), the epiglottis `epiglottis_at`. Each arch's
  row is drawn as seven crowns a side (typical crown widths laid along the row), with the gum
  over the necks and the floor of the mouth inside the lower arch; the incisors' crowns are
  drawn half the row's thickness (the row reaches into the lip's polygon there). Triangles come
  in groups by material. `pose` computes the midline and the element rules only (no laminae, no
  cuts); `display(z)` gives the lips' meeting line, the jaw's pivot and turn, the lamina at z
  as named polygons and the engine's midline outlines.
- **Browser.** `tract3dView` runs `pose`, leaves all meshes' vertices in a float buffer
  (`chimera_floats_ptr`) and returns their layout and `display` as JSON; triangles, lines and
  groups come once per body (`topology: true`). The face screens call it only when the posture
  changed.
- **Lips.** `lipRound` (0…1, default 0): the opening's width × (1 − 0.65·lipRound), below the
  engine's 1.1 cm floor. 41 parameters now.
- **Cuts.** A cut's perimeter is its boundary's Euclidean length (it was a staircase, 28 % long
  on a circle).
- **Presets.** The bunched /ɹ/ is open (it shut the tract over 2 cm); its check asks for an
  open path and a low tip.
- **Lab.** Listening sits under the 3D view (Listen, Stop, Say with words or IPA) and a start
  prompt starts the voice; a consonant posture is said between two vowels. A sagittal plane is
  drawn in 3D at its lateral offset; whole planes are fitted in the CUT screen.
- **Face screens.** 3D is the default view of Play, Speak, Sing and Beatbox, opened along the
  midline ("Cut away" off shows the head whole); the flat midsagittal view keeps the handles.
- **Cost in the browser** (production build, headless Chromium, 2026-11-10, the machine's load
  average about 10–14; `tools/tract3d/browser-bench.ts`). `tract3dView`: median 1.3 ms, p95
  1.4 ms (16.5 kB of JSON and 29 643 floats). The lab's `tract3d` call with the meshes: 7.6 ms
  (857 kB of JSON; 9 881 vertices with the single crowns and gums, three times the reviewed
  head's), 2.2 ms for a cut alone; target 16 ms. A screen's whole update for a posture (the
  view call, the skin's rig, the normals) took 5–7 ms in a test page. The shadow pass's cost on
  a phone's GPU is not measured; a screen renders only when its posture or camera changes.
  The audio thread's load in `tools/e2e.ts` was 42 % and 41 % (desktop and phone viewport) on
  one run and 50 % and 60 % on another under the heavier load; the reviewed head measured 41 %
  and 42 %. The voice's code is unchanged in this stage.

### 0.3 Slice fix (2026-11-10): the lower tract's shapes, the head's look

The slice-critic judged the integrated head from its own frames: the cuts of the pharynx and
larynx were blocky polygons, the same in every sound, with no epiglottis in the lumen and a
pharynx 3.0 cm wide in every posture; the head read as a mannequin's bust with a slit of a
mouth; /u/ had spread lips; the cut face read as icons. What answers each (the engine's sound
is untouched; `git log` has the reasons and what was ruled out):

- **The pharynx is a ring of walls** (`Parts::ring_at`): one tissue polygon per lamina, the
  posterior wall on the midline, coming forward round the sides until, at the ring's
  half-width (`pharynxWidth*`), it has reached the back teeth (the thyroid's plates low in the
  throat). Its outline is a superellipse (`pharynxShape`, 2 an ellipse; squarer in the
  hypopharynx, which lies flat on the spine). The air ends sideways where the ring meets what
  stands in front of it: the tongue's root, the epiglottis and its folds, the pillars. So the
  lumen's width follows its depth (/i/ is wider than /a/ at the same level without any
  parameter saying so), and its sides are where two curved surfaces meet. `lateralSqueeze`
  draws the ring's sides in; `derive` sets it for a low, retracted tongue.
- **The tongue's back** is convex across its whole width wherever it faces the pharynx (the
  grooves beside it run forward as horns); what faces the palate stays flat across the middle.
- **The epiglottis is a leaf**: its laryngeal surface curled back at the sides
  (`epiglottisCurl`), highest on the midline and lowest at its edges, free above the
  valleculae. Beyond its edges the front wall rises again into the folds that run to the
  pharynx's sides; the saddle at each edge is where a vallecula opens into the pharynx beside
  the leaf. In a cut through its free part the leaf crosses the lumen as a crescent.
- **The valleculae** are the air between the leaf's lingual surface and the tongue's root (the
  engine's own root curve, moved with the tongue's surface above it), `valleculaDepth` deep
  below the leaf's tip, with the median glossoepiglottic fold a ridge between them (55 % of
  their depth; the two communicate over it). They were two ellipsoids inside tongue tissue.
- **The laryngeal tube** is a loft of elliptical sections whose areas are the engine's
  epilarynx profile with its steps rounded (two passes of ¼-½-¼ over 1 mm samples), times
  `epilarynxWidth` / 0.6 (a speaker's tube against the voice's). Each section is as long front
  to back as the larynx leaves room and as wide as its area asks: the ventricle bulges
  sideways, the ventricular folds draw in, the vestibule widens to its rim; the tube leans back
  to where the engine's inner wall begins, and a narrowed tube (twang) shuts from behind. The
  centreline runs up the tube's own axis.
- **Each piriform fossa** is the lower half of an ellipsoid whose equator lies at the
  hypopharynx's floor: a pouch widest at its opening, `piriformDepth` deep, beside the tube's
  rim beyond the aryepiglottic fold, between the ring behind it and the front wall. By default
  1.27 cm³ and 1.8 cm deep; published 1.0–1.4 cm³ each and 1.6–2.0 cm [R12].
- **The aryepiglottic folds** are ridges on the hypopharynx's floor between the laryngeal
  inlet and each fossa (`aryFoldHeight`): how high the three lower channels stay apart differs
  between speakers by about a centimetre.
- **The faucial pillars** begin where they leave the tongue's sides and have a rounded face
  and foot (they were a block over every height, a flat front for the lower pharynx).
  **The cheeks** stand in between the tooth rows beyond the mouth's corners when the jaw is
  down.
- **`derive`** has ten constants: the lips' rounding from the engine's protrusion and spread
  (above the resting protrusion), and the pharynx's squeeze for a low retracted tongue, joined
  the eight. Opened lips part in a rounded opening (the corner exponent rises with the
  aperture).
- **Display.** Every drawn surface still comes from the helpers the laminae are cut from
  (the tube's sections, the fossae's pouches, the valleculae's floor and the root in front of
  it, the leaf with its thickness, the ring); the test holds the laminae's own polygons on the
  same helpers. The drawn ring stops at 93 % of its half-width (beyond, it runs forward inside
  the tongue's sides and the throat's walls). Not drawn: the aryepiglottic folds' ridges and
  the pillars (they are in the cuts).
- **The head** ([head-asset.md](head-asset.md)): a sculpt from the same CC0 detail targets, a
  higher cut through the neck, a stand with a bevelled edge; in the rig the loops of skin
  inside the lips' line are rolled in behind the lips' edges, the throat's skin stays over the
  larynx when the jaw turns, and a mouth opened beyond 1.1 cm draws its lips off the teeth
  (`bared`; the one place where the skin leaves the engine's lips, which stay within a
  millimetre or two of the incisors' edges at any opening). Inside a whole head the tract's
  surfaces darken with their depth behind the lips; the skin has a soft highlight and the lips
  a gloss.
- **The cut face** (`section.ts`): the brain's medial face (sulci, corpus callosum, thalamus,
  brain stem, the cerebellum's white tree), the cervical spine on the neck's curve with the
  dens between the atlas' arches, the sphenoid and frontal sinuses, the hyoid, the thyroid's
  front, the cricoid's arch and plate with an arytenoid, the trachea with its rings and the
  gullet behind it, and a tongue of muscle with faint curved fibres. Still a drawing that
  measures nothing; the air is the lamina's.
- **Lab.** A fitted plane in the CUT screen stays clear of its axis label and scale bar. A
  sixteenth posture, "mouth wide open": the jaw and lips at the far end of their ranges, the
  tongue low and flat (a posture to look at; the voice has no such target).
- **Cost** (native release, 65 laminae, 73 cuts, the machine under other load): median
  1.8 ms per compute over the 16 presets, slowest 3.1 ms; the display meshes 0.8 ms for 9 972
  vertices. The ring's polygon and the pockets cost 0.3 ms over §0.2's 1.5 ms.

### 0.4 Second slice fix (2026-11-10): the cuts' forms, the elements as organs, the head beside the picture

The slice-critic's second round, from its own frames and its own DVTD run: every cut had a
straight floor or a right-angle step, and against MRI the pharynx's cuts were upside down
(MRI: flat and wide on the posterior wall, the belly towards the tongue; the model: a ring
that bulged at the back over a tongue root flat across); the elements drawn alone were plates
and open shells; the head still read as a mannequin. What answers each (the engine's sound is
untouched; `git log` has the reasons and what was ruled out). Where this section and §0.3 or
§4 disagree, this section is the model as built.

- **A tube of soft walls** (`Parts::closing`). From the larynx to the back teeth the lumen
  ends sideways in a taper: at lateral offset z a share `closing(|z| / W)` of the gap between
  the tube's two walls is taken up, nothing on the midline and all of it from the tube's
  half-width W on (`pharynxWidthHypo`, `-Oro`, `-Naso` by height; `lateralSqueeze` takes up to
  half of it). The taper's outline is a superellipse whose exponent is `pharynxShape` (1.3
  pointed in front, 2 half an ellipse, 4 a rounded rectangle). Which wall carries the taper
  differs along the tube:
  - **In the pharynx the posterior wall is flat** with round corners (the ring,
    `Parts::ring_at`, a fixed outline: flat over two thirds of its width) **and the front wall
    comes back to it**: the epiglottis and the folds beside it (`Parts::epiglottis_at`), and
    the tongue's root and back. Each dorsal point of the tongue looks along its normal at
    what it faces; where that is the posterior wall its sides come out to it
    (`Parts::dclose`).
  - **Under the soft palate the roof comes down to the tongue**: the faucial arches
    (`Parts::varch`, straight down so the roof stays a graph over x), and the hard palate's
    edge behind the back teeth comes down with the soft palate's root. The two pillar blocks
    of §0.3 are gone. The tongue stays level across there.
  - **Under the hard palate the tongue is a loaf**, convex across its whole width
    (`Parts::dloaf`; the middle 70 % drawn flat gave every oral cut a straight floor), from
    1.5 cm in front of the palate's back edge on and in front of the last lower molar.
- **The tongue's front is round** seen from above: it keeps the blade's width and comes in to
  `tipWidth` over the tip lobe as a quarter of an ellipse, so at rest it fills the lower arch.
  It narrowed as a wedge from 1.75 to 0.55 cm, and the air beside it down to the floor of the
  mouth was the "legs" under every cut of an open mouth. A lateral (`lateral` = 1) draws the
  front in to the wedge again: the channels of /l/ pass beside it.
- **The uvula is a lobe** of round section (`Parts::uvula_outline`, tissue of its own): it
  hangs from the soft palate's oral surface just in front of its tip, straight down when the
  soft palate is raised against the wall, along the soft palate when that hangs itself, and is
  squashed where the tongue's back comes up to it. `uvulaLength` and `uvulaWidth` size it.
- **Tooth rows are round in section**: from the gum line on the lingual side over the biting
  surface, and at the row's back end (`Parts::crown_round`); the buccal face stays upright,
  with the cheek and the lips on it. A row is as thin at the incisors as the engine's own
  incisors (3 mm, 1 cm at the molars) and runs through to the midline: at 7 mm the incisors'
  rows were left out within 2.5 mm of the midline, a square slot in every cut through the
  front teeth. Rows under 4 mm apart hold no air.
- **The cheek** lies on the upper row's buccal face above the mouth and on the lower row's
  below it, where the jaw has carried that, and bulges in between the rows (a blunt nose) up
  to half a centimetre inside the mouth's corner.
- **The piriform fossae** are 1.03 cm³ each by default (0.85 of the lumen's depth there as
  their front-to-back radius, 0.55 cm to either side); `piriformDepth` goes down to 0.1 cm
  and under 0.2 cm there are none (one of the two DVTD speakers has next to none below the
  hypopharynx's floor).
- **Presets.** With a round tongue front the air of /s/, /ʃ/ and the retroflex /ɹ/ no longer
  passes beside the tip, and all three presets closed. /s/ has a deeper groove, /ʃ/ and the
  retroflex a lower tip, and their checks refuse a closure (0.05, 0.03 and 0.15 cm² at
  least).
- **Display** (`mesh.rs`, from the same helpers): the tongue is a closed body (its dorsal
  surface over its own width, the tip lobe's underside, a belly from margin to margin through
  the core, the root's end); the soft palate has its thickness (the oral surface the laminae
  hold, the nasal one the engine's outline), its arches and the uvula's lobe, and lies
  against the posterior wall when raised; the pharynx is a tube: the ring from the laryngeal
  inlet up over the nasopharynx's roof (the engine's whole wall; no air of the oral tract lies
  above the port), with its side walls in front of the ring's corners (the hypopharynx's
  front wall beside the leaf, and the tongue's surface beyond the tongue's own body); the
  hypopharynx's floor is one surface with the aryepiglottic folds on it and the fossae in it
  (`Parts::floor_height`), and a web runs from the tube to each fossa below it; the
  epiglottis is a leaf on a stalk a third as wide (`Parts::epiglottis_foot`); the cheeks turn
  in over the gums and draw in behind the last molar; each crown follows the arch (straight
  blocks of each tooth's own thickness stepped out from tooth to tooth); each arch's gum is
  one surface across the midline. What bounds no air and is display only: the belly, the web,
  the soft palate's nasal surface, the single crowns and the gums.
- **The head** ([head-asset.md](head-asset.md)): a skull shorter behind the ears, a brow,
  cheekbones and a chin that stand forward, fuller lips, the cheeks a little raised under eyes
  pressed shut, lids over an eyeball, a neck no wider than 4.9 cm to either side that flares
  into a wide low disc with bevelled edges; the jaw's weights spread over their neighbours
  (the seam from ear to chin); skin of uneven tone with warmer cheeks, nose, ears and lids and
  a softer, wider highlight; behind the lips' line the lining's colour; a pink tongue, and
  light that reaches 1.6 cm into the mouth before it fades; the cheeks' inner surfaces drawn
  inside a whole head (they are the mouth's side walls).
- **Lab.** The sagittal plane is drawn on the 1 cm grid with a joined outline, like the other
  three planes. The along-tract readout says "on the bend (this cut overlaps the next)".
  `chimera tract3d cut <preset> coronal:<x>` and `axial:<y>` draw a whole plane with every
  tissue polygon's span under the air.
- **The fit** weighs the cuts' proportions three times as much as it did (0.15 for a cut
  twice too wide at all six stations; at 0.05 it all but ignored them).
- **Cost.** In [validation-results.md §4](validation-results.md#4-cost).

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
| `pharynxWidthOro` | cm | 0.8 … 2.4 | 1.6 *(fit)* | half-width, oropharynx |
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
