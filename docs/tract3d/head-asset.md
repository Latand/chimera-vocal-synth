# The head's skin mesh: source and licence

The head drawn round the 3D tract (`src/ui/render/head/`) is display only. Its skin is one
generated file, `src/ui/render/head/head.generated.bin` (151 kB), baked by
`tools/gen-head.ts` from assets of the **MakeHuman** project. Like every `*.generated.*` file
here it is regenerated, never edited.

## Source

- Repository: <https://github.com/makehumancommunity/makehuman>, commit
  `a8bc2d54ff0ac92e78ff71431b1023eda42bf482` (2024-06-26).
- Files read (all under `makehuman/data/`):
  - `3dobjs/base.obj`: the base mesh (its `body` group);
  - `targets/macrodetails/universal-{male,female}-{young,child}-averagemuscle-averageweight.target`
    and `targets/macrodetails/{caucasian,asian,african}-{male,female}-{young,child}.target`:
    the adult male, adult female and child shapes (the three origins in equal parts);
  - `targets/expression/units/caucasian/eye-{left,right}-closure.target`: the closed eyes;
    `eye-{left,right}-slit.target` and `nose-{left,right}-elevation.target` of the same
    folder, at three tenths: the cheeks a little raised under eyes pressed shut;
  - seventeen detail targets under `targets/head`, `forehead`, `cheek`, `eyebrows`, `chin`,
    `mouth` and `neck` (`head-round`, `head-scale-vert-decr`, `head-scale-horiz-incr`,
    `head-back-scale-depth-decr`, `forehead-temple-incr`, `{l,r}-cheek-bones-incr`,
    `{l,r}-cheek-volume-decr`, `eyebrows-trans-forward`, `chin-prominent-incr`,
    `chin-bones-incr`, `mouth-{upperlip,lowerlip}-volume-incr`, `neck-double-decr`,
    `neck-scale-horiz-decr`, `neck-scale-depth-decr`): the sculpt every body shares;
  - `rigs/default.mhskel` and `rigs/default_weights.mhw`: which vertices follow the jaw and
    each lip.

## Licence

The MakeHuman project releases its assets (the base mesh and proxies, targets and modifiers,
textures, poses and expressions) under **CC0 1.0 Universal**: `LICENSE.md` section C of that
repository, with the full text in its `LICENSE.ASSETS.md`. Each file read carries the same
statement in its header ("This asset was explicitly released as CC0 in september 2020"; the
skeleton and weight files have `"license": "CC0"`). CC0 asks for no attribution; the source is
recorded here anyway. MakeHuman's program code is AGPL and none of it is used or copied: the
generator is this project's own and reads only the data files above.

Nothing else in the head comes from outside: the stand, the colours, the rig
(`src/ui/render/head/rig.ts`), the cut face's painting (`section.ts`) and the shading
(`scene.ts`, on three.js, MIT) are this project's code.

## What the generator does

`bun tools/gen-head.ts <makehuman checkout> [--debug <dir>] [--tract <model.json>]`; the
checkout stays outside this repository. The output is deterministic for a given checkout.

1. Builds the three shapes with both eyes closed and the shared sculpt: a rounder skull,
   shorter behind the ears, with fuller temples; a brow, cheekbones and a chin that stand
   forward; fuller lips; a firm line under the jaw; a slim neck. (The macro shapes alone gave
   a long cranium with a small flat face on its front, a receding chin and a neck as wide as
   the jaw; set beside the reference picture the head read as a mannequin's bust.) The
   weights were chosen by eye against that picture.
2. Cuts the head off at an edge loop round the neck, two loops above its base (before it
   widens into the shoulders), and opens the mouth at the fourth edge loop inside the lips
   (the base mesh has a closed mouth bag; the inside of the mouth is the engine's 3D tract).
3. Puts each shape in the engine's frame for the male template (cm, x forward, y up, z to the
   speaker's right): scaled to a skull 14.8 cm across (the same template breadth for the three
   bodies; the runtime scales the head with the body, as the engine scales the tract), tilted
   back 8° about the mouth, the point where the lips meet on the engine's closed lips, and the
   neck below the jaw moved forward until its front is 1.95 cm in front of the glottis (1.0,
   1.8 and 1.6 cm for the three shapes), so the engine's larynx lies inside the neck as deep
   under the skin as a real one.
   Two steps of the generator's own follow, in that frame. The closed lids are brought out
   over an eyeball: every lid vertex within 1.9 cm of an eye's middle that lies inside a
   sphere of 1.3 cm radius centred 0.9 cm behind the closed lids is moved onto it (the base
   mesh's lids close over an empty socket and read as a slit cut in the face). And the neck
   is a slim column below the jaw: no wider than 4.9 cm to either side from 1.5 cm above the
   glottis down, sideways only (the base mesh widens towards the shoulders there, which on a
   stand read as a collar; front to back the neck keeps the room the engine's larynx and the
   spine need).
4. Adds the stand: twelve rings below the neck's cut. The neck runs on down, flares out in a
   hollow curve into a wide low disc (3.3 cm beyond the neck all round), and the disc has a
   bevelled upper edge, an upright rim, a bevelled lower edge and a flat underside.
5. Stores, per vertex, the three positions (hundredths of a millimetre, rounded away from zero
   so both sides of the head keep the same size), how far the vertex follows the jaw, the
   upper lip and the lower lip (the skeleton's weights), the lips' colour (by its ring from
   the line where the lips meet) and how far inside that line the vertex lies; and the loops
   the runtime needs: that line, the mouth's open rim, and the midline (the outline of the
   midsagittal section). The file's version is 2.

The runtime (`asset.ts`) subdivides the 4 527 vertices once (Catmull–Clark) and carries every
per-vertex quantity through the same weights. `rig.ts` then moves the skin: the jaw's turn
(the throat's skin stays over the larynx; the skeleton's jaw weights are spread over their
neighbours ten times, because they fell from all to nothing within two edges along the jaw's
line and the open jaw drew a seam from the ear to the chin), the lips' line on the engine's lip edges, the
loops inside that line rolled in behind the edges, and, with the mouth opened beyond 1.1 cm,
the lips drawn off the teeth (`bared`: the engine's lips stay within a millimetre or two of
the incisors' edges at any opening; a mouth opened wide shows the crowns). `tests/head.test.ts`
holds the mesh whole and symmetric, each body's head round its own tract, the skin on the
engine's jaw and lips in speech, the teeth shown when it opens wide, and nothing of the skin
hanging in the opening.
