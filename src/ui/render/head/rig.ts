/**
 * Moves the head's skin with the articulators. Display only: the geometry that makes sound and
 * is cut is the engine's 3D tract; the skin follows two things it reports (`Tract3d::display`):
 *  - the jaw's turn about its pivot: each vertex turns by its share of it (MakeHuman's skeleton
 *    weights for the jaw and what hangs from it);
 *  - the lips' meeting line: the vertices where the skin's own lips meet are put on the engine's
 *    upper and lower lip edges, and the rest of each lip follows by its weight. The mouth's
 *    opening seen from outside is then the opening the cuts measure;
 *  - the loops of skin inside that line (the mesh's mouth is open at the fourth) are rolled in
 *    behind the lips' edges, towards the gums: left to their weights they hung in the opening
 *    (an /a/ open 2 cm showed a slit under 1 cm with a pink slab in it).
 * The mouth's corners move towards the midline as the lips round.
 *
 * One thing the skin does that the engine's lips do not: opened wide, the lips draw off the
 * teeth (`bared`). The engine's lip edges stay within a millimetre or two of the incisors' edges
 * whatever the opening; a mouth opened to sing or to yawn shows the crowns, and at its widest
 * the gum line. The cuts measure the engine's lips; the skin shows that much more of the teeth.
 */
import type { Tract3dDisplay } from "../../../wasm/types.ts";
import type { HeadMesh } from "./asset.ts";

export interface HeadBody {
  /** shares of the male, female and child shapes (they sum to 1) */
  weights: [number, number, number];
  /** the body's scale and its pharynx factor (the engine's template map) */
  k: number;
  ph: number;
}

const BINS = 24;
/** how many times the jaw's weights are averaged with their neighbours' */
const JAW_SPREAD = 10;

/** `w` after `passes` rounds of each vertex taking the mean of itself and its neighbours */
function spread(w: Float32Array, indices: Uint32Array, passes: number): Float32Array {
  const n = w.length;
  let cur = w;
  const count = new Uint16Array(n);
  for (const i of indices) count[i] += 2;
  for (let pass = 0; pass < passes; pass++) {
    const acc = new Float32Array(n);
    for (let t = 0; t < indices.length; t += 3) {
      const a = indices[t], b = indices[t + 1], c = indices[t + 2];
      acc[a] += cur[b] + cur[c];
      acc[b] += cur[c] + cur[a];
      acc[c] += cur[a] + cur[b];
    }
    const next = new Float32Array(n);
    for (let i = 0; i < n; i++) next[i] = count[i] ? 0.4 * cur[i] + 0.6 * (acc[i] / count[i]) : cur[i];
    cur = next;
  }
  return cur;
}

export class HeadRig {
  /** the rest pose in the body's own cm */
  readonly rest: Float32Array;
  /** the posed vertices (rewritten by `pose`) */
  readonly out: Float32Array;
  /** the mouth's corners at rest: half the distance between them */
  readonly corner: number;
  private jaw: Float32Array;
  private readonly lip: Float32Array;
  private readonly movers: Int32Array;
  /** the vertices inside the lips' line, and each one's share of the upper lip (0 lower … 1 upper) */
  private readonly inner: Int32Array;
  private readonly share: Float32Array;
  private readonly binsU = new Float64Array(3 * (BINS + 1));
  private readonly binsL = new Float64Array(3 * (BINS + 1));

  constructor(readonly mesh: HeadMesh, readonly body: HeadBody) {
    const n = mesh.n;
    const { k, ph } = body;
    this.rest = new Float32Array(3 * n);
    for (let b = 0; b < 3; b++) {
      const w = body.weights[b], src = mesh.bodies[b];
      if (w === 0) continue;
      for (let i = 0; i < 3 * n; i++) this.rest[i] += w * src[i];
    }
    for (let i = 0; i < n; i++) {
      const y = this.rest[3 * i + 1];
      this.rest[3 * i] *= k;
      this.rest[3 * i + 1] = (y < 7 ? 7 - (7 - y) * ph : y) * k;
      this.rest[3 * i + 2] *= k;
    }
    this.out = Float32Array.from(this.rest);
    // the skin's chin follows the bone under it fully (the skeleton shares a fifth of it with the head)
    this.jaw = new Float32Array(n);
    this.lip = new Float32Array(n);
    const movers: number[] = [];
    for (let i = 0; i < n; i++) {
      // (the throat's skin stays over the larynx: the skeleton lets the jaw pull the neck's
      // front back with it, which uncovered the engine's piriform fossae when the mouth opened)
      const yt = this.rest[3 * i + 1] / k;
      const throat = Math.min(1, Math.max(0, (yt - 2.2) / 1.8));
      this.jaw[i] = Math.sin((Math.PI / 2) * Math.min(1, mesh.jaw[i] / 0.84)) * throat * throat * (3 - 2 * throat);
      this.lip[i] = Math.min(1, mesh.upper[i] + mesh.lower[i]);
    }
    // the skin stretches between the jaw and the skull: the weights spread over their
    // neighbours (the skeleton's fell from all to nothing within two edges along the jaw's
    // line, and the open jaw drew a seam from the ear to the chin)
    this.jaw = spread(this.jaw, mesh.indices, JAW_SPREAD);
    for (let i = 0; i < n; i++) if (this.jaw[i] > 1e-4 || this.lip[i] > 0) movers.push(i);
    this.movers = Int32Array.from(movers);
    let corner = 0, meet = 0;
    for (const i of mesh.lipLine.upper) {
      corner = Math.max(corner, Math.abs(this.rest[3 * i + 2]));
      meet += this.rest[3 * i + 1] / mesh.lipLine.upper.length;
    }
    this.corner = corner;
    const inner: number[] = [];
    this.share = new Float32Array(n);
    for (let i = 0; i < n; i++) {
      if (mesh.inside[i] <= 0) continue;
      inner.push(i);
      const u = mesh.upper[i], l = mesh.lower[i];
      // (a vertex neither lip's bones hold belongs to the lip it lies in at rest)
      this.share[i] = u + l > 0.05 ? u / (u + l) : this.rest[3 * i + 1] >= meet ? 1 : 0;
    }
    this.inner = Int32Array.from(inner);
  }

  /**
   * The skin for a posture: `d` from the engine's view of it, `lipWidth` the articulator
   * (0 rounded … 1 spread).
   */
  pose(d: Tract3dDisplay, lipWidth: number): Float32Array {
    const { mesh, rest, out, jaw, lip } = this;
    const k = this.body.k;
    // the corners: at their rest width with spread lips, drawn in as the lips round, and never
    // inside the opening itself
    const wide = Math.max(d.lips.halfWidth + 0.3 * k, (1.4 + 1.1 * Math.min(1, Math.max(0, lipWidth))) * k);
    const narrow = Math.min(1.04, wide / this.corner);
    const c = Math.cos(d.jaw.angle), s = Math.sin(d.jaw.angle);
    const px = d.jaw.pivot.x, py = d.jaw.pivot.y;
    for (const i of this.movers) {
      let x = rest[3 * i], y = rest[3 * i + 1];
      const z = rest[3 * i + 2] * (1 - lip[i] * (1 - narrow));
      const w = jaw[i];
      if (w > 0) {
        // a share w of the jaw's turn: the point between its place and its turned place
        const dx = x - px, dy = y - py;
        x += w * (px + dx * c - dy * s - x);
        y += w * (py + dx * s + dy * c - y);
      }
      out[3 * i] = x;
      out[3 * i + 1] = y;
      out[3 * i + 2] = z;
    }
    // how far each lip's line is from the engine's lip edge, by lateral offset
    const reach = this.corner * 1.05;
    const table = (line: Int32Array, weight: Float32Array, target: { x: number; y: number }[], bins: Float64Array) => {
      bins.fill(0);
      for (const i of line) {
        const az = Math.abs(out[3 * i + 2]);
        const t = edge(d.lips.z, target, az);
        const w = Math.max(0.5, weight[i]);
        const b = Math.min(BINS, Math.round((az / reach) * BINS));
        bins[3 * b] += (t.x - out[3 * i]) / w;
        bins[3 * b + 1] += (t.y - out[3 * i + 1]) / w;
        bins[3 * b + 2] += 1;
      }
      // empty bins take their neighbours' values
      let last = -1;
      for (let b = 0; b <= BINS; b++) {
        if (bins[3 * b + 2] === 0) continue;
        bins[3 * b] /= bins[3 * b + 2];
        bins[3 * b + 1] /= bins[3 * b + 2];
        if (last < 0) for (let q = 0; q < b; q++) (bins[3 * q] = bins[3 * b]), (bins[3 * q + 1] = bins[3 * b + 1]);
        else
          for (let q = last + 1; q < b; q++) {
            const f = (q - last) / (b - last);
            bins[3 * q] = bins[3 * last] + (bins[3 * b] - bins[3 * last]) * f;
            bins[3 * q + 1] = bins[3 * last + 1] + (bins[3 * b + 1] - bins[3 * last + 1]) * f;
          }
        last = b;
      }
      for (let q = last + 1; q <= BINS; q++) (bins[3 * q] = bins[3 * last]), (bins[3 * q + 1] = bins[3 * last + 1]);
    };
    const lips = bared(d, k);
    table(mesh.lipLine.upper, mesh.upper, lips.upper, this.binsU);
    table(mesh.lipLine.lower, mesh.lower, lips.lower, this.binsL);
    const { binsU, binsL } = this;
    for (const i of this.movers) {
      const u = mesh.upper[i], l = mesh.lower[i];
      if (u === 0 && l === 0) continue;
      const f = Math.min(BINS, (Math.abs(out[3 * i + 2]) / reach) * BINS);
      const b = Math.min(BINS - 1, Math.floor(f)), t = f - b;
      out[3 * i] += u * (binsU[3 * b] * (1 - t) + binsU[3 * b + 3] * t) + l * (binsL[3 * b] * (1 - t) + binsL[3 * b + 3] * t);
      out[3 * i + 1] += u * (binsU[3 * b + 1] * (1 - t) + binsU[3 * b + 4] * t) + l * (binsL[3 * b + 1] * (1 - t) + binsL[3 * b + 4] * t);
    }
    // inside the lips' line: each loop a step further behind its lip's edge and towards its gum
    // (the upper lip's up, the lower lip's down), so the opening is clear from edge to edge
    for (const i of this.inner) {
      const az = Math.abs(out[3 * i + 2]);
      const depth = mesh.inside[i], share = this.share[i];
      const eu = edge(d.lips.z, lips.upper, az), el = edge(d.lips.z, lips.lower, az);
      const back = 0.85 * k * depth, away = 0.4 * k * depth;
      out[3 * i] = (el.x - back) * (1 - share) + (eu.x - back) * share;
      out[3 * i + 1] = (el.y - away) * (1 - share) + (eu.y + away) * share;
    }
    return out;
  }
}

/**
 * The lips' line the skin follows: the engine's, with the lips drawn off the teeth as the mouth
 * opens wide. From an opening of 1.1 cm the upper lip uncovers the upper incisors' crowns, at
 * 2.4 cm and more all 10 mm of them and 1.5 mm of the gum above (at 9 mm the gum stayed under
 * the lip, and a mouth opened that wide shows it), and the lower lip 4.5 mm of the lower ones;
 * most on the midline, nothing at the corners. `k` is the body's scale.
 */
export function bared(d: Tract3dDisplay, k: number): { upper: { x: number; y: number }[]; lower: { x: number; y: number }[] } {
  const { upper, lower } = d.lips;
  const open = upper[0].y - lower[0].y;
  const t = Math.min(1, Math.max(0, (open / k - 1.1) / 1.3));
  const wide = t * t * (3 - 2 * t);
  const tooth = (name: string) => d.outlines.find((o) => o.name === name)?.points[2];
  const up = tooth("upperTeeth"), low = tooth("lowerTeeth");
  if (wide <= 0 || !up || !low || open <= 0) return { upper, lower };
  const lift = Math.max(0, up.y + 1.15 * k * wide - upper[0].y);
  const drop = Math.max(0, lower[0].y - (low.y - 0.45 * k * wide));
  const share = upper.map((u, i) => Math.pow(Math.max(0, (u.y - lower[i].y) / open), 0.6));
  return {
    upper: upper.map((u, i) => ({ x: u.x, y: u.y + lift * share[i] })),
    lower: lower.map((l, i) => ({ x: l.x, y: l.y - drop * share[i] })),
  };
}

/** a lip edge at lateral offset az (the engine samples it from the midline out) */
function edge(zs: number[], pts: { x: number; y: number }[], az: number): { x: number; y: number } {
  const n = zs.length;
  if (az >= zs[n - 1]) return pts[n - 1];
  const step = zs[1] - zs[0];
  const i = Math.min(n - 2, Math.floor(az / step)), t = az / step - i;
  return { x: pts[i].x + (pts[i + 1].x - pts[i].x) * t, y: pts[i].y + (pts[i + 1].y - pts[i].y) * t };
}
