/**
 * Moves the head's skin with the articulators. Display only: the geometry that makes sound and
 * is cut is the engine's 3D tract; the skin follows two things it reports (`Tract3d::display`):
 *  - the jaw's turn about its pivot: each vertex turns by its share of it (MakeHuman's skeleton
 *    weights for the jaw and what hangs from it);
 *  - the lips' meeting line: the vertices where the skin's own lips meet are put on the engine's
 *    upper and lower lip edges, and the rest of each lip follows by its weight. The mouth's
 *    opening seen from outside is then the opening the cuts measure.
 * The mouth's corners move towards the midline as the lips round.
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

export class HeadRig {
  /** the rest pose in the body's own cm */
  readonly rest: Float32Array;
  /** the posed vertices (rewritten by `pose`) */
  readonly out: Float32Array;
  /** the mouth's corners at rest: half the distance between them */
  readonly corner: number;
  private readonly jaw: Float32Array;
  private readonly lip: Float32Array;
  private readonly movers: Int32Array;
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
      this.jaw[i] = Math.sin((Math.PI / 2) * Math.min(1, mesh.jaw[i] / 0.84));
      this.lip[i] = Math.min(1, mesh.upper[i] + mesh.lower[i]);
      if (this.jaw[i] > 0 || this.lip[i] > 0) movers.push(i);
    }
    this.movers = Int32Array.from(movers);
    let corner = 0;
    for (const i of mesh.lipLine.upper) corner = Math.max(corner, Math.abs(this.rest[3 * i + 2]));
    this.corner = corner;
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
    table(mesh.lipLine.upper, mesh.upper, d.lips.upper, this.binsU);
    table(mesh.lipLine.lower, mesh.lower, d.lips.lower, this.binsL);
    const { binsU, binsL } = this;
    for (const i of this.movers) {
      const u = mesh.upper[i], l = mesh.lower[i];
      if (u === 0 && l === 0) continue;
      const f = Math.min(BINS, (Math.abs(out[3 * i + 2]) / reach) * BINS);
      const b = Math.min(BINS - 1, Math.floor(f)), t = f - b;
      out[3 * i] += u * (binsU[3 * b] * (1 - t) + binsU[3 * b + 3] * t) + l * (binsL[3 * b] * (1 - t) + binsL[3 * b + 3] * t);
      out[3 * i + 1] += u * (binsU[3 * b + 1] * (1 - t) + binsU[3 * b + 4] * t) + l * (binsL[3 * b + 1] * (1 - t) + binsL[3 * b + 4] * t);
    }
    return out;
  }
}

/** a lip edge at lateral offset az (the engine samples it from the midline out) */
function edge(zs: number[], pts: { x: number; y: number }[], az: number): { x: number; y: number } {
  const n = zs.length;
  if (az >= zs[n - 1]) return pts[n - 1];
  const step = zs[1] - zs[0];
  const i = Math.min(n - 2, Math.floor(az / step)), t = az / step - i;
  return { x: pts[i].x + (pts[i + 1].x - pts[i].x) * t, y: pts[i].y + (pts[i + 1].y - pts[i].y) * t };
}
