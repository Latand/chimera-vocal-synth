# The head's skin mesh: source and licence

The head drawn round the 3D tract (`src/ui/render/head/`) is display only. Its skin is one
generated file, `src/ui/render/head/head.generated.bin` (142 kB), baked by
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

1. Builds the three shapes with both eyes closed.
2. Cuts the head off at an edge loop round the base of the neck and opens the mouth at the
   fourth edge loop inside the lips (the base mesh has a closed mouth bag; the inside of the
   mouth is the engine's 3D tract).
3. Puts each shape in the engine's frame for the male template (cm, x forward, y up, z to the
   speaker's right): scaled to a skull 14.8 cm across (the same template breadth for the three
   bodies; the runtime scales the head with the body, as the engine scales the tract), tilted
   back 8° about the mouth, the point where the lips meet on the engine's closed lips, and the
   neck below the jaw moved forward until its front is 2.15 cm in front of the glottis (1.0,
   1.5 and 1.6 cm for the three shapes), so the engine's larynx lies inside the neck as deep
   under the skin as a real one.
4. Adds the stand: eight rings below the neck's cut and an underside.
5. Stores, per vertex, the three positions (hundredths of a millimetre), how far the vertex
   follows the jaw, the upper lip and the lower lip (the skeleton's weights), and the lips'
   colour (by its ring from the line where the lips meet); and the loops the runtime needs:
   that line, the mouth's open rim, and the midline (the outline of the midsagittal section).

The runtime (`asset.ts`) subdivides the 4 401 vertices once (Catmull–Clark, 17 600 vertices)
and carries every per-vertex quantity through the same weights. `tests/head.test.ts` holds the
mesh whole and symmetric, each body's head round its own tract, and the skin on the engine's
jaw and lips.
