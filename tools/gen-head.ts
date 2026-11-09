/**
 * Bakes the head's skin mesh (`src/ui/render/head/head.generated.bin`) from the MakeHuman base
 * mesh and targets, which their authors released as CC0 (see docs/tract3d/head-asset.md).
 *
 *   bun tools/gen-head.ts <makehuman checkout> [--debug <dir>]
 *
 * `<makehuman checkout>` is a clone of github.com/makehumancommunity/makehuman (the folder that
 * holds `makehuman/data`), kept outside this repository. The output is deterministic for a given
 * checkout. `--debug` also writes each body's midline as SVG, in the engine's frame.
 *
 * What it does:
 *  1. reads `base.obj` (the `body` group) and builds three shapes from the macro targets: adult
 *     male, adult female and child, each with both eyes closed (the eye-closure targets);
 *  2. cuts the head off at an edge loop round the base of the neck and opens the mouth at an edge
 *     loop on the inside of the lips (the base mesh has a closed mouth bag; the mouth's inside is
 *     the engine's own 3D tract);
 *  3. turns each shape into the engine's frame (cm, x forward, y up, z to the speaker's right,
 *     the male template: the runtime applies the body's own map), scaled to a real head's size,
 *     tilted back, and with the neck moved forward so the engine's larynx lies inside it;
 *  4. adds a stand below the neck;
 *  5. keeps, per vertex, what the runtime needs to move the mouth: how much the vertex follows
 *     the jaw and each lip (from MakeHuman's default skeleton weights), and how far it is from
 *     the line where the lips meet (for the lips' colour).
 * The runtime (src/ui/render/head/) subdivides the mesh once and moves it with the articulators.
 */
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const args = process.argv.slice(2);
const MH = args[0];
if (!MH) throw new Error("usage: bun tools/gen-head.ts <makehuman checkout> [--debug <dir>]");
const debugDir = args.includes("--debug") ? args[args.indexOf("--debug") + 1] : null;
/** `chimera tract3d model ə out.json`, drawn under the male midline in the debug SVG */
const tractJson = args.includes("--tract") ? args[args.indexOf("--tract") + 1] : null;
const DATA = join(MH, "makehuman", "data");
const OUT = join(import.meta.dir, "..", "src", "ui", "render", "head", "head.generated.bin");

// ---------------------------------------------------------------- source mesh
const obj = readFileSync(join(DATA, "3dobjs", "base.obj"), "utf8");
const base: number[] = [];
const faces: number[][] = [];
{
  let group = "";
  for (const line of obj.split("\n")) {
    if (line.startsWith("v ")) {
      const p = line.split(/\s+/);
      base.push(+p[1], +p[2], +p[3]);
    } else if (line.startsWith("g ")) group = line.slice(2).trim();
    else if (line.startsWith("f ") && group === "body") faces.push(line.trim().split(/\s+/).slice(1).map((t) => parseInt(t, 10) - 1));
  }
}
const NV = base.length / 3;

function target(name: string): Map<number, [number, number, number]> {
  const out = new Map<number, [number, number, number]>();
  for (const line of readFileSync(join(DATA, "targets", `${name}.target`), "utf8").split("\n")) {
    if (!line || line.startsWith("#")) continue;
    const p = line.trim().split(/\s+/);
    if (p.length >= 4) out.set(+p[0], [+p[1], +p[2], +p[3]]);
  }
  return out;
}
function shape(parts: [string, number][]): Float64Array {
  const v = Float64Array.from(base);
  for (const [name, w] of parts) for (const [i, d] of target(name)) for (let c = 0; c < 3; c++) v[3 * i + c] += w * d[c];
  return v;
}
const EYES: [string, number][] = [
  ["expression/units/caucasian/eye-left-closure", 1],
  ["expression/units/caucasian/eye-right-closure", 1],
];
const macro = (sex: string, age: string, w: number): [string, number][] => [
  [`macrodetails/universal-${sex}-${age}-averagemuscle-averageweight`, w],
  [`macrodetails/caucasian-${sex}-${age}`, w / 3],
  [`macrodetails/asian-${sex}-${age}`, w / 3],
  [`macrodetails/african-${sex}-${age}`, w / 3],
];
/** the three bodies the engine blends between (MakeHuman dm, its axes) */
const BODIES: Record<string, Float64Array> = {
  male: shape([...macro("male", "young", 1), ...EYES]),
  female: shape([...macro("female", "young", 1), ...EYES]),
  child: shape([...macro("male", "child", 0.5), ...macro("female", "child", 0.5), ...EYES]),
};

// ---------------------------------------------------------------- topology helpers
const ekey = (a: number, b: number) => (a < b ? a * NV + b : b * NV + a);
const edgeFaces = new Map<number, number[]>();
const nbrs: Set<number>[] = Array.from({ length: NV }, () => new Set());
faces.forEach((f, fi) => {
  for (let i = 0; i < f.length; i++) {
    const a = f[i], b = f[(i + 1) % f.length];
    const k = ekey(a, b);
    (edgeFaces.get(k) ?? edgeFaces.set(k, []).get(k)!).push(fi);
    nbrs[a].add(b);
    nbrs[b].add(a);
  }
});
const onMid = (i: number) => Math.abs(base[3 * i]) < 1e-5;
/** the edge loop through `start` that leaves the midline towards +x (the mesh is all quads there) */
function loop(start: number): number[] {
  const first = [...nbrs[start]].find((w) => base[3 * w] > 1e-5 && !onMid(w));
  if (first === undefined) throw new Error(`no side neighbour at vertex ${start}`);
  const out = [start, first];
  let prev = start, cur = first;
  for (let n = 0; n < 1000; n++) {
    const inc = new Set(edgeFaces.get(ekey(prev, cur)));
    const next = [...nbrs[cur]].filter((w) => w !== prev && !edgeFaces.get(ekey(cur, w))!.some((f) => inc.has(f)));
    if (nbrs[cur].size !== 4 || next.length !== 1) throw new Error(`edge loop from ${start} breaks at ${cur}`);
    prev = cur;
    cur = next[0];
    if (cur === start) return out;
    out.push(cur);
  }
  throw new Error(`edge loop from ${start} does not close`);
}
/** the midline's vertices in order, from the front of the chest up over the face to the back */
const midChain = (() => {
  const mids = new Set<number>();
  for (const f of faces) for (const i of f) if (onMid(i) && base[3 * i + 1] > 5.0) mids.add(i);
  const adj = new Map<number, number[]>();
  for (const i of mids) adj.set(i, [...nbrs[i]].filter((w) => mids.has(w)));
  const start = [...mids].find((i) => adj.get(i)!.length === 1 && base[3 * i + 2] > 0)!;
  const out = [start];
  let prev = -1;
  for (;;) {
    const cur = out[out.length - 1];
    const next = adj.get(cur)!.find((w) => w !== prev);
    if (next === undefined) break;
    prev = cur;
    out.push(next);
  }
  return out;
})();

// Named vertices of the base mesh (its indices are stable: every target refers to them).
const V = {
  /** where the lips meet, upper and lower lip */
  stomionUpper: 474,
  stomionLower: 493,
  /** the loop on the inside of the lips where the mouth is opened (the fourth inwards) */
  rimUpper: 571,
  /** the loop round the base of the neck where the head is cut off */
  neckFront: 1522,
  noseTip: 297,
};
const lipLine = loop(V.stomionUpper);
/** the midline from the neck's cut up the throat to the chin */
const frontChain = midChain.slice(midChain.indexOf(V.neckFront), midChain.indexOf(V.neckFront) + 12);
const rim = loop(V.rimUpper);
const neck = loop(V.neckFront);
if (!lipLine.includes(V.stomionLower)) throw new Error("the lips' contact loop changed");

// ---------------------------------------------------------------- the head's faces
const barrier = new Set<number>();
for (const l of [rim, neck]) l.forEach((a, i) => barrier.add(ekey(a, l[(i + 1) % l.length])));
const keep = new Uint8Array(faces.length);
{
  const seed = faces.findIndex((f) => f.includes(V.noseTip));
  const stack = [seed];
  keep[seed] = 1;
  while (stack.length) {
    const f = faces[stack.pop()!];
    for (let i = 0; i < f.length; i++) {
      const k = ekey(f[i], f[(i + 1) % f.length]);
      if (barrier.has(k)) continue;
      for (const g of edgeFaces.get(k)!) if (!keep[g]) (keep[g] = 1), stack.push(g);
    }
  }
}
const headFaces = faces.filter((_, i) => keep[i]);
const remap = new Int32Array(NV).fill(-1);
const kept: number[] = [];
for (const f of headFaces) for (const i of f) if (remap[i] < 0) (remap[i] = kept.length), kept.push(i);
const n0 = kept.length;

// ---------------------------------------------------------------- per-vertex rig data
const skel = JSON.parse(readFileSync(join(DATA, "rigs", "default.mhskel"), "utf8")) as { bones: Record<string, { parent: string | null }> };
const weights = (JSON.parse(readFileSync(join(DATA, "rigs", "default_weights.mhw"), "utf8")) as { weights: Record<string, [number, number][]> }).weights;
const under = (bone: string, root: string): boolean => {
  for (let b: string | null = bone; b; b = skel.bones[b]?.parent ?? null) if (b === root) return true;
  return false;
};
function boneWeight(pred: (bone: string) => boolean): Float64Array {
  const w = new Float64Array(NV);
  for (const [bone, list] of Object.entries(weights)) if (pred(bone)) for (const [i, x] of list) w[i] += x;
  return w;
}
// the lower lip's bones hang from `special04` (a child of the jaw), the upper lip's from `special01`
const wJaw = boneWeight((b) => under(b, "jaw"));
const wLower = boneWeight((b) => under(b, "special04"));
const wUpper = boneWeight((b) => under(b, "special01"));
// rings of the lips counted from the line where they meet: 0 on it, growing outwards over the
// vermilion onto the skin, negative inwards
const ring = new Float64Array(NV).fill(99);
{
  // along the midline the chain runs chin → lower lip → into the mouth and out → upper lip → nose;
  // each loop round the mouth passes through both lips
  const up = midChain.indexOf(V.stomionUpper);
  for (let r = -4; r <= 7; r++) {
    let l: number[];
    try {
      l = loop(midChain[up + r]);
    } catch {
      break; // further out the loops leave the mouth (the nose's base, the chin)
    }
    for (const i of l) if (Math.abs(r) < Math.abs(ring[i])) ring[i] = r;
  }
}

// ---------------------------------------------------------------- into the engine's frame
/**
 * MakeHuman: dm, x to the figure's left, y up, z forward. Engine: cm, x forward, y up, z to the
 * speaker's right. The registration, in the sagittal plane, per body:
 *  - a scale that gives the skull a real adult's breadth, the same template breadth for the three
 *    bodies (the runtime scales the head with the body, as the engine scales the tract);
 *  - a backward tilt about the mouth, which brings the throat forward under the jaw;
 *  - the point where the lips meet goes to the engine's closed lips;
 *  - below the jaw, a forward shift of the neck that puts its front SKIN_FRONT cm in front of the
 *    glottis: the engine's larynx then lies inside the neck, as far under the skin as a real one.
 */
const BREADTH = 14.8; // cm, the skull's breadth in the male template
const TILT = (8 * Math.PI) / 180;
/** the engine's closed lips (template cm): x of the lips' edge at rest, the lips' line */
const LIPS = { x: 6.95, y: 7.85 };
/** the neck's front at the level of the glottis (y = 0), cm in front of it */
const SKIN_FRONT = 2.15;
/** the shift is complete below this height and fades out up to the jaw line */
const NECK_FULL = 1.4, NECK_NONE = 4.4;
const smooth = (a: number, b: number, x: number) => {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
};

interface Registered {
  /** engine-frame positions of the kept vertices (template cm) */
  pos: Float64Array;
  scale: number;
  shift: number;
}
function register(mh: Float64Array): Registered {
  // the skull's breadth: the widest pair of points well above the ears
  const sy = mh[3 * V.stomionUpper + 1];
  let wide = 0;
  for (const i of kept) if (mh[3 * i + 1] > sy + 0.9) wide = Math.max(wide, Math.abs(mh[3 * i]));
  const s = BREADTH / (2 * wide);
  const ax = mh[3 * V.stomionUpper + 2], ay = sy;
  const c = Math.cos(TILT), sn = Math.sin(TILT);
  const pos = new Float64Array(3 * n0);
  kept.forEach((i, j) => {
    const dx = (mh[3 * i + 2] - ax) * s, dy = (mh[3 * i + 1] - ay) * s;
    pos[3 * j] = LIPS.x + dx * c - dy * sn;
    pos[3 * j + 1] = LIPS.y + dx * sn + dy * c;
    pos[3 * j + 2] = -mh[3 * i] * s;
  });
  // the neck's front on the midline at the glottis' height (the lowest point of the front, if the
  // cut lies above it), then the shift that puts it in place
  const chain = frontChain.map((i) => remap[i]);
  let front = pos[3 * chain[0]];
  for (let k = 1; k < chain.length; k++) {
    const y0 = pos[3 * chain[k - 1] + 1], y1 = pos[3 * chain[k] + 1];
    if (y0 <= 0 && y1 > 0) front = pos[3 * chain[k - 1]] + ((pos[3 * chain[k]] - pos[3 * chain[k - 1]]) * (0 - y0)) / (y1 - y0);
  }
  const shift = SKIN_FRONT - front;
  for (let j = 0; j < n0; j++) pos[3 * j] += shift * (1 - smooth(NECK_FULL, NECK_NONE, pos[3 * j + 1]));
  return { pos, scale: s, shift };
}

// ---------------------------------------------------------------- the stand
/**
 * Rings below the neck's cut: down a short collar, out in a fillet to a disc, down its rim and in
 * to the centre of its underside. Every ring has the cut loop's vertex count, so the stand's
 * faces are quads but for the underside's fan.
 */
const STAND = [
  // [drop below the collar's top (cm), growth of the radius (cm), blend towards a circle]
  [0.5, 0.0, 0.15],
  [1.1, 0.25, 0.4],
  [1.5, 1.0, 0.7],
  [1.7, 2.1, 0.9],
  [1.85, 2.9, 1.0],
  [2.5, 3.0, 1.0],
  [3.1, 2.9, 1.0],
  [3.2, 2.2, 1.0],
] as const;
const neckKept = neck.map((i) => remap[i]);
if (neckKept.some((i) => i < 0)) throw new Error("the neck loop left the head");
const nStand = STAND.length * neck.length + 1;
function stand(pos: Float64Array): Float64Array {
  const out = new Float64Array(3 * nStand);
  const m = neck.length;
  let cx = 0, cz = 0, top = Infinity, r = 0;
  for (const j of neckKept) {
    cx += pos[3 * j] / m;
    cz += pos[3 * j + 2] / m;
    top = Math.min(top, pos[3 * j + 1]);
  }
  for (const j of neckKept) r += Math.hypot(pos[3 * j] - cx, pos[3 * j + 2] - cz) / m;
  STAND.forEach(([drop, grow, round], k) => {
    neckKept.forEach((j, i) => {
      const dx = pos[3 * j] - cx, dz = pos[3 * j + 2] - cz, d = Math.hypot(dx, dz) || 1;
      const rad = d * (1 - round) + r * round + grow;
      out[3 * (k * m + i)] = cx + (dx / d) * rad;
      out[3 * (k * m + i) + 1] = top - drop;
      out[3 * (k * m + i) + 2] = cz + (dz / d) * rad;
    });
  });
  out[3 * (nStand - 1)] = cx;
  out[3 * (nStand - 1) + 1] = top - STAND[STAND.length - 1][0];
  out[3 * (nStand - 1) + 2] = cz;
  return out;
}

// ---------------------------------------------------------------- assemble
const N = n0 + nStand;
const polys: number[][] = headFaces.map((f) => f.map((i) => remap[i]));
{
  const m = neck.length;
  // which way round the cut loop the head's faces run decides the stand's winding
  const a = neckKept[0], b = neckKept[1];
  const forward = polys.some((f) => f.some((v, i) => v === b && f[(i + 1) % f.length] === a));
  const quad = (p: number, q: number, r: number, s: number) => polys.push(forward ? [p, q, r, s] : [s, r, q, p]);
  for (let k = 0; k < STAND.length; k++) {
    for (let i = 0; i < m; i++) {
      const j = (i + 1) % m;
      const up = (x: number) => (k === 0 ? neckKept[x] : n0 + (k - 1) * m + x);
      const down = (x: number) => n0 + k * m + x;
      quad(up(i), up(j), down(j), down(i));
    }
  }
  const last = n0 + (STAND.length - 1) * m;
  for (let i = 0; i < m; i++) {
    const j = (i + 1) % m;
    polys.push(forward ? [last + i, last + j, N - 1] : [N - 1, last + j, last + i]);
  }
}
const bodies: Record<string, Float32Array> = {};
const info: Record<string, { scale: number; shift: number }> = {};
for (const [name, mh] of Object.entries(BODIES)) {
  const reg = register(mh);
  const all = new Float32Array(3 * N);
  all.set(reg.pos);
  all.set(stand(reg.pos), 3 * n0);
  bodies[name] = all;
  info[name] = { scale: reg.scale, shift: reg.shift };
}
const attr = (src: Float64Array, max = 1): Uint8Array => {
  const out = new Uint8Array(N);
  kept.forEach((i, j) => (out[j] = Math.round(255 * Math.min(1, Math.max(0, src[i] / max)))));
  return out;
};
// the lips' colour: full over the vermilion (rings 0–4 and the inside), gone by ring 6
const tint = new Float64Array(NV);
for (let i = 0; i < NV; i++) tint[i] = ring[i] > 50 ? 0 : 1 - smooth(4, 6, ring[i]);
const loops = {
  /** where the lips meet: upper lip's vertices then the lower lip's, each from the midline outwards to both corners */
  lipLine: lipLine.map((i) => remap[i]),
  rim: rim.map((i) => remap[i]),
  /** the midline: from under the stand up the neck's front and over the chin to the lower lip's rim; from the upper lip's rim over the head and down the neck's back to under the stand */
  midFront: [] as number[],
  midBack: [] as number[],
};
{
  const chain = midChain.map((i) => remap[i]);
  const first = chain.findIndex((i) => i >= 0);
  const gap = chain.findIndex((i, k) => k > first && i < 0);
  const second = chain.findIndex((i, k) => k > gap && i >= 0);
  let end = chain.length;
  for (let k = second; k < chain.length; k++) if (chain[k] < 0) (end = k), (k = chain.length);
  // each chain continues down the stand under it to the centre of its underside, so the two
  // chains with the lips' chord between them outline the whole midsagittal section
  const m = neck.length;
  const under = (v: number) => {
    const i = neckKept.indexOf(v);
    if (i < 0) throw new Error("a midline chain does not end on the neck's cut");
    return [...STAND.map((_, k) => n0 + k * m + i), N - 1];
  };
  const front = chain.slice(first, gap), back = chain.slice(second, end);
  loops.midFront = [...under(front[0]).reverse(), ...front];
  loops.midBack = [...back, ...under(back[back.length - 1])];
}

// ---------------------------------------------------------------- write
const header = {
  version: 1,
  source: "MakeHuman base mesh, macro targets, eye-closure targets and default skeleton weights (CC0 1.0)",
  vertices: N,
  skin: n0,
  bodies: Object.keys(bodies),
  polygons: polys.length,
  loops,
  registration: info,
  tilt: TILT,
};
const head = new TextEncoder().encode(JSON.stringify(header));
const polyData: number[] = [];
for (const p of polys) polyData.push(p.length, ...p);
const sections: Uint8Array[] = [
  ...Object.values(bodies).map((b) => {
    // millimetre hundredths in 16 bits: ±327 cm
    const q = new Int16Array(b.length);
    for (let i = 0; i < b.length; i++) q[i] = Math.round(b[i] * 100);
    return new Uint8Array(q.buffer);
  }),
  attr(wJaw),
  attr(wUpper),
  attr(wLower),
  attr(tint),
  new Uint8Array(Uint16Array.from(polyData).buffer),
];
const total = 8 + head.length + ((4 - (head.length % 4)) % 4) + sections.reduce((s, x) => s + x.length + ((4 - (x.length % 4)) % 4), 0);
const out = new Uint8Array(total);
const dv = new DataView(out.buffer);
dv.setUint32(0, 0x44414548, true); // "HEAD"
dv.setUint32(4, head.length, true);
let o = 8;
const put = (x: Uint8Array) => {
  out.set(x, o);
  o += x.length + ((4 - (x.length % 4)) % 4);
};
put(head);
sections.forEach(put);
mkdirSync(join(OUT, ".."), { recursive: true });
writeFileSync(OUT, out);
console.log(`${OUT}: ${N} vertices (${n0} skin + ${nStand} stand), ${polys.length} polygons, ${out.length} bytes`);
for (const [name, r] of Object.entries(info)) console.log(`  ${name}: ${r.scale.toFixed(3)} cm per dm, neck shifted ${r.shift.toFixed(2)} cm forward`);

if (debugDir) {
  mkdirSync(debugDir, { recursive: true });
  let tract = "";
  if (tractJson) {
    const t = JSON.parse(readFileSync(tractJson, "utf8")) as { midsagittal: [string, { x: number; y: number }[]][] };
    for (const [part, poly] of t.midsagittal) {
      const d = poly.map((q) => `${(q.x * 20 + 260).toFixed(1)},${(520 - q.y * 20).toFixed(1)}`).join(" ");
      tract += `<polygon points="${d}" fill="${part === "envelope" || part === "tube" ? "#9cf" : "#fc9"}" fill-opacity="0.35" stroke="#888" stroke-width="0.5"/>`;
    }
  }
  for (const [name, b] of Object.entries(bodies)) {
    const pts = (ids: number[]) => ids.map((i) => `${(b[3 * i] * 20 + 260).toFixed(1)},${(520 - b[3 * i + 1] * 20).toFixed(1)}`).join(" ");
    const grid: string[] = [];
    for (let g = -10; g <= 12; g += 2) grid.push(`<line x1="${g * 20 + 260}" y1="0" x2="${g * 20 + 260}" y2="640" stroke="#ddd"/><text x="${g * 20 + 262}" y="12" font-size="9">${g}</text>`);
    for (let g = -4; g <= 24; g += 2) grid.push(`<line x1="0" y1="${520 - g * 20}" x2="520" y2="${520 - g * 20}" stroke="#ddd"/><text x="2" y="${518 - g * 20}" font-size="9">${g}</text>`);
    writeFileSync(
      join(debugDir, `midline-${name}.svg`),
      `<svg xmlns="http://www.w3.org/2000/svg" width="520" height="640" style="background:#fff">${grid.join("")}${name === "male" ? tract : ""}<polyline points="${pts(loops.midFront)}" fill="none" stroke="#c33" stroke-width="1.5"/><polyline points="${pts(loops.midBack)}" fill="none" stroke="#36c" stroke-width="1.5"/><circle cx="260" cy="520" r="3" fill="#000"/></svg>`,
    );
  }
}
