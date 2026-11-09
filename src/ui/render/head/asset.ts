/**
 * The head's skin mesh: reads `head.generated.bin` (tools/gen-head.ts) and subdivides it once
 * (Catmull–Clark), carrying every per-vertex quantity through the same weights. Display only.
 *
 * The mesh is in the engine's frame for the male template (cm, x forward, y up, z to the
 * speaker's right); `rig.ts` blends the three bodies and applies the body's own map.
 */

export interface HeadMesh {
  /** vertices after subdivision */
  n: number;
  /** rest positions per body, template cm (male, female, child) */
  bodies: Float32Array[];
  /** 0…1 per vertex: how far it follows the jaw, the upper lip, the lower lip; the lips' colour */
  jaw: Float32Array;
  upper: Float32Array;
  lower: Float32Array;
  tint: Float32Array;
  /** triangles, outward-facing */
  indices: Uint32Array;
  /** the stand's vertices (they follow nothing and keep one colour) */
  stand: Uint8Array;
  /** where the lips meet, split: each lip's vertices with their side (−1 left … +1 right) */
  lipLine: { upper: Int32Array; lower: Int32Array };
  /** the open rim inside the lips (a closed loop) */
  rim: Int32Array;
  /** the midline: neck front → chin → lower lip's rim; upper lip's rim → nose → crown → neck back */
  midFront: Int32Array;
  midBack: Int32Array;
}

interface Header {
  version: number;
  vertices: number;
  skin: number;
  bodies: string[];
  polygons: number;
  loops: { lipLine: number[]; rim: number[]; midFront: number[]; midBack: number[] };
}

interface Raw {
  n: number;
  skin: number;
  bodies: Float32Array[];
  attrs: Float32Array[];
  polys: number[][];
  loops: Header["loops"];
}

export function readHead(bytes: ArrayBuffer): Raw {
  const dv = new DataView(bytes);
  if (dv.getUint32(0, true) !== 0x44414548) throw new Error("not a head mesh");
  const hl = dv.getUint32(4, true);
  const h = JSON.parse(new TextDecoder().decode(new Uint8Array(bytes, 8, hl))) as Header;
  if (h.version !== 1) throw new Error(`head mesh version ${h.version}`);
  let o = 8 + hl + ((4 - (hl % 4)) % 4);
  const take = (len: number) => {
    const s = bytes.slice(o, o + len);
    o += len + ((4 - (len % 4)) % 4);
    return s;
  };
  const n = h.vertices;
  const bodies = h.bodies.map(() => {
    const q = new Int16Array(take(n * 3 * 2));
    const f = new Float32Array(n * 3);
    for (let i = 0; i < f.length; i++) f[i] = q[i] / 100;
    return f;
  });
  const attrs = [0, 1, 2, 3].map(() => {
    const q = new Uint8Array(take(n));
    const f = new Float32Array(n);
    for (let i = 0; i < n; i++) f[i] = q[i] / 255;
    return f;
  });
  const pd = new Uint16Array(take(bytes.byteLength - o));
  const polys: number[][] = [];
  for (let i = 0; polys.length < h.polygons; ) {
    const k = pd[i++];
    polys.push(Array.from(pd.subarray(i, i + k)));
    i += k;
  }
  return { n, skin: h.skin, bodies, attrs, polys, loops: h.loops };
}

/** One Catmull–Clark step as weights over the old vertices (compressed rows). */
interface Subdivision {
  n: number;
  start: Int32Array;
  index: Int32Array;
  weight: Float32Array;
  quads: number[][];
  /** new index of an old edge's point */
  edge(a: number, b: number): number;
}

function subdivide(polys: number[][], n: number): Subdivision {
  const key = (a: number, b: number) => (a < b ? a * n + b : b * n + a);
  const edges = new Map<number, { id: number; a: number; b: number; faces: number[] }>();
  const vertFaces: number[][] = Array.from({ length: n }, () => []);
  const vertEdges: number[][] = Array.from({ length: n }, () => []);
  polys.forEach((p, f) => {
    for (let i = 0; i < p.length; i++) {
      const a = p[i], b = p[(i + 1) % p.length];
      vertFaces[a].push(f);
      const k = key(a, b);
      let e = edges.get(k);
      if (!e) {
        e = { id: edges.size, a, b, faces: [] };
        edges.set(k, e);
        vertEdges[a].push(k);
        vertEdges[b].push(k);
      }
      e.faces.push(f);
    }
  });
  const nf = polys.length;
  const total = n + nf + edges.size;
  const rows: Map<number, number>[] = Array.from({ length: total }, () => new Map());
  const add = (row: Map<number, number>, i: number, w: number) => row.set(i, (row.get(i) ?? 0) + w);
  const addRow = (row: Map<number, number>, from: Map<number, number>, w: number) => {
    for (const [i, x] of from) add(row, i, x * w);
  };
  // face points
  polys.forEach((p, f) => {
    for (const i of p) add(rows[n + f], i, 1 / p.length);
  });
  // edge points
  for (const e of edges.values()) {
    const row = rows[n + nf + e.id];
    if (e.faces.length === 2) {
      add(row, e.a, 0.25);
      add(row, e.b, 0.25);
      addRow(row, rows[n + e.faces[0]], 0.25);
      addRow(row, rows[n + e.faces[1]], 0.25);
    } else {
      add(row, e.a, 0.5);
      add(row, e.b, 0.5);
    }
  }
  // vertex points
  for (let v = 0; v < n; v++) {
    const row = rows[v];
    const es = vertEdges[v].map((k) => edges.get(k)!);
    const border = es.filter((e) => e.faces.length === 1);
    if (border.length >= 2) {
      add(row, v, 0.75);
      for (const e of border.slice(0, 2)) add(row, e.a === v ? e.b : e.a, 0.125);
    } else if (es.length === 0) {
      add(row, v, 1);
    } else {
      const m = es.length;
      add(row, v, (m - 3) / m);
      for (const f of vertFaces[v]) addRow(row, rows[n + f], 1 / (m * vertFaces[v].length));
      for (const e of es) {
        add(row, v, 1 / (m * m));
        add(row, e.a === v ? e.b : e.a, 1 / (m * m));
      }
    }
  }
  let count = 0;
  for (const r of rows) count += r.size;
  const start = new Int32Array(total + 1), index = new Int32Array(count), weight = new Float32Array(count);
  let o = 0;
  rows.forEach((r, i) => {
    start[i] = o;
    for (const [j, w] of r) {
      index[o] = j;
      weight[o++] = w;
    }
  });
  start[total] = o;
  const edge = (a: number, b: number) => n + nf + edges.get(key(a, b))!.id;
  const quads: number[][] = [];
  polys.forEach((p, f) => {
    for (let i = 0; i < p.length; i++) {
      const prev = p[(i + p.length - 1) % p.length], cur = p[i], next = p[(i + 1) % p.length];
      quads.push([cur, edge(cur, next), n + f, edge(prev, cur)]);
    }
  });
  return { n: total, start, index, weight, quads, edge };
}

function apply(s: Subdivision, src: Float32Array, width: number): Float32Array {
  const out = new Float32Array(s.n * width);
  for (let i = 0; i < s.n; i++) {
    for (let k = s.start[i]; k < s.start[i + 1]; k++) {
      const j = s.index[k], w = s.weight[k];
      for (let c = 0; c < width; c++) out[i * width + c] += w * src[j * width + c];
    }
  }
  return out;
}

export function buildHead(bytes: ArrayBuffer): HeadMesh {
  const raw = readHead(bytes);
  const s = subdivide(raw.polys, raw.n);
  const bodies = raw.bodies.map((b) => apply(s, b, 3));
  const [jaw, upper, lower, tint] = raw.attrs.map((a) => apply(s, a, 1));
  const isStand = new Float32Array(raw.n);
  for (let i = raw.skin; i < raw.n; i++) isStand[i] = 1;
  const standF = apply(s, isStand, 1);
  const stand = new Uint8Array(s.n);
  for (let i = 0; i < s.n; i++) stand[i] = standF[i] > 0.999 ? 1 : 0;
  const indices = new Uint32Array(s.quads.length * 6);
  s.quads.forEach((q, i) => indices.set([q[0], q[1], q[2], q[0], q[2], q[3]], i * 6));
  const closed = (l: number[]) => l.flatMap((a, i) => [a, s.edge(a, l[(i + 1) % l.length])]);
  const open = (l: number[]) => l.flatMap((a, i) => (i + 1 < l.length ? [a, s.edge(a, l[i + 1])] : [a]));
  // the lips' line: the upper lip's vertices follow the upper lip more than the lower
  const line = closed(raw.loops.lipLine);
  const up = line.filter((i) => upper[i] >= lower[i]), low = line.filter((i) => upper[i] < lower[i]);
  return {
    n: s.n,
    bodies,
    jaw,
    upper,
    lower,
    tint,
    indices,
    stand,
    lipLine: { upper: Int32Array.from(up), lower: Int32Array.from(low) },
    rim: Int32Array.from(closed(raw.loops.rim)),
    midFront: Int32Array.from(open(raw.loops.midFront)),
    midBack: Int32Array.from(open(raw.loops.midBack)),
  };
}

let loading: Promise<HeadMesh> | null = null;
/** The head mesh, fetched and built once. */
export function loadHead(): Promise<HeadMesh> {
  loading ??= import("./head.generated.bin?url")
    .then((m) => fetch(m.default))
    .then((r) => {
      if (!r.ok) throw new Error(`head mesh: ${r.status}`);
      return r.arrayBuffer();
    })
    .then(buildHead);
  return loading;
}
