/**
 * The lab's anatomical head around the engine's 3D tract: the skin and, beneath it, the
 * subcutaneous fat and the muscle layer as nested closed surfaces, with the skull and brain, the
 * mandible, the cervical vertebrae, the hyoid and the laryngeal cartilages. Display only: the
 * tract inside is the engine's (`chimera::anatomy::tract3d` meshes) and nothing here feeds the
 * sound.
 *
 * The skin is the face screen's head (profile.ts outlines at the current articulation, the
 * cross-section tables of anatomy3d.ts), so the jaw, the lips and the larynx move it. Each layer
 * is the skin pushed inwards along its cross-section's normal by a thickness that depends on
 * the region (scalp, forehead, nose, cheeks, lips, chin, neck front, sides and back), with the
 * midline features (nose, lips, chin) fading out with depth. Thicknesses are typical adult
 * values from anatomy texts (skin 1.5–2.5 mm; fat thin over the scalp, forehead and nose, thick in
 * the cheeks and neck; muscle thin in the face and scalp, thick at the masseter, the
 * sternocleidomastoid and the nuchal muscles behind the spine).
 *
 * Body cm: x forward, y up, z to the speaker's right, glottis at the origin.
 */
import type { HumanArtic } from "../../wasm/types.ts";
import type { VoiceModel } from "../analysis.ts";
import { buildProfile, jawAngle } from "./profile.ts";
import { BRAIN_WIDTH, FEATURE, PB, PLANE, Shape, WIDEST, WIDTH, interp, smooth, type Section } from "./anatomy3d.ts";
import type { Vec } from "./geom.ts";

export interface HeadPart {
  key: string;
  /** shaded surface colour, and the colour of its cut face */
  color: number;
  cap: number;
  positions: Float32Array;
  indices: Uint32Array;
  /** per-vertex colours (linear RGB), when the surface is not one colour */
  colors?: Float32Array;
  /** a closed surface: its cut face can be filled (stencil parity) */
  closed: boolean;
  /** the outermost skin (the one drawn when the head is shown whole) */
  skin?: boolean;
}

/** colours: skin, fat, muscle, bone, cartilage, brain */
export const TISSUE = {
  skin: 0xd8a48a,
  skinCut: 0xe7b9a1,
  fat: 0xf1d47c,
  muscle: 0xa8403e,
  bone: 0xeee5d1,
  cartilage: 0xbcd9d4,
  brain: 0xd9b7b2,
  /** the soft tissue beneath the muscle layer (deep muscles, glands, connective tissue) */
  deep: 0x6a3532,
  deepCut: 0x9a5a52,
  tongue: 0xb44c4a,
  mucosa: 0xd8898a,
  enamel: 0xf4f0e6,
};

/** a flat region of the midsagittal plane (z = 0), body cm */
export interface SectionRegion {
  color: number;
  poly: Vec[];
}

/** The midsagittal section the cut-open head shows on its cut face: the air of the tract, the
 *  nasal cavity and the trachea (holes in the layers' cut faces, so the tract behind shows), and
 *  the tract's own tissues there (tongue, soft and hard palate, teeth, epiglottis), from the
 *  engine's frame, which is the 3D model's z = 0 by construction. */
export function buildSection(model: VoiceModel, a: HumanArtic): { air: Vec[][]; tissue: SectionRegion[] } {
  const k = model.tract.scale;
  const f = model.frame(a);
  const prof = buildProfile(model, f, a);
  const g = f.glottis;
  const lumen: Vec[] = [];
  for (let y = g.y - 0.3 * k; y >= -4.5 * k; y -= 0.25 * k) lumen.push({ x: g.x + 0.1 * k + 0.04 * (g.y - y) - 0.72 * k, y });
  for (let y = -4.5 * k; y <= g.y - 0.3 * k; y += 0.25 * k) lumen.push({ x: g.x + 0.1 * k + 0.04 * (g.y - y) + 0.72 * k, y });
  const epi = f.epiglottis;
  const thick = (pts: Vec[], h: number): Vec[] => {
    const up: Vec[] = [], down: Vec[] = [];
    pts.forEach((p, i) => {
      const q = pts[Math.min(pts.length - 1, i + 1)], o = pts[Math.max(0, i - 1)];
      const tx = q.x - o.x, ty = q.y - o.y, l = Math.hypot(tx, ty) || 1;
      up.push({ x: p.x - (ty / l) * h, y: p.y + (tx / l) * h });
      down.push({ x: p.x + (ty / l) * h, y: p.y - (tx / l) * h });
    });
    return [...up, ...down.reverse()];
  };
  return {
    air: [[...f.outer, ...f.inner.slice().reverse()], prof.nasal, lumen],
    tissue: [
      { color: TISSUE.tongue, poly: f.tongue },
      { color: TISSUE.mucosa, poly: f.velum },
      { color: TISSUE.bone, poly: [...prof.palOral, ...prof.palNasal.slice().reverse()] },
      { color: TISSUE.enamel, poly: f.upperTeeth },
      { color: TISSUE.enamel, poly: f.lowerTeeth },
      { color: TISSUE.cartilage, poly: thick(epi.slice(0, 2), 0.09 * k) },
    ],
  };
}

const bump = (u: number) => (Math.abs(u) < 1 ? (1 - u * u) ** 2 : 0);
const NB = 26; // points on the back half of a half-ring
const NF = 38; // on the front half (dense towards the midline: nose, lips, chin)

/** half a cross-section, back midline → front midline (x, z ≥ 0), the midline feature scaled */
function halfCurve(s: Section, feature: number): number[] {
  const out: number[] = [];
  for (let i = 0; i < NB; i++) {
    const th = Math.PI - (i / NB) * (Math.PI / 2);
    out.push(s.xw - (s.xw - s.Bk) * Math.pow(Math.abs(Math.cos(th)), 2 / PB), s.W * Math.pow(Math.sin(th), 2 / PB));
  }
  for (let j = 0; j <= NF; j++) {
    const ph = (Math.PI / 2) * Math.pow(1 - j / NF, 1.5);
    const z = s.W * Math.pow(Math.sin(ph), 2 / s.pf);
    out.push(s.xw + (s.Fs - s.xw) * Math.pow(Math.cos(ph), 2 / s.pf) + feature * s.E * bump(z / s.wf), z);
  }
  return out;
}

/** a half-curve moved inwards (in its plane) by depth(x, z) at each point */
function inset(half: number[], depth: (x: number, z: number) => number, cap: number): number[] {
  const n = half.length / 2;
  const out = new Array<number>(half.length);
  // centroid of the full (mirrored) ring lies on z = 0
  let cx = 0;
  for (let i = 0; i < n; i++) cx += half[2 * i];
  cx /= n;
  for (let i = 0; i < n; i++) {
    const x = half[2 * i], z = half[2 * i + 1];
    const a = Math.max(0, i - 1), b = Math.min(n - 1, i + 1);
    let tx = half[2 * b] - half[2 * a], tz = half[2 * b + 1] - half[2 * a + 1];
    // at the midline ends the mirrored neighbour makes the tangent lateral
    if (i === 0 || i === n - 1) [tx, tz] = [0, i === 0 ? 1 : -1];
    let nx = -tz, nz = tx;
    const len = Math.hypot(nx, nz) || 1;
    nx /= len;
    nz /= len;
    if (nx * (cx - x) + nz * -z < 0) [nx, nz] = [-nx, -nz];
    const d = Math.min(depth(x, z), cap);
    out[2 * i] = x + nx * d;
    out[2 * i + 1] = Math.max(0, z + nz * d);
  }
  return out;
}

/** closed surface through rings of equal size (flat x,y,z loops), fanned shut at both ends */
class Loft {
  pos: number[] = [];
  idx: number[] = [];
  private rings: number[] = [];
  private size = 0;
  ring(points: number[]): void {
    const n = points.length / 3;
    if (this.size && n !== this.size) throw new Error("ring sizes differ");
    this.size = n;
    this.rings.push(this.pos.length / 3);
    this.pos.push(...points);
  }
  /** the surface, outward-facing, optionally leaving out quads `drop(ring, i)` */
  build(drop?: (r: number, i: number) => boolean): { positions: Float32Array; indices: Uint32Array } {
    this.pos = this.pos.slice();
    this.idx = [];
    const n = this.size, R = this.rings.length;
    for (let r = 0; r + 1 < R; r++) {
      const a = this.rings[r], b = this.rings[r + 1];
      for (let i = 0; i < n; i++) {
        if (drop?.(r, i)) continue;
        const j = (i + 1) % n;
        this.idx.push(a + i, a + j, b + i, b + i, a + j, b + j);
      }
    }
    {
      for (const [r, flip] of [[0, true], [R - 1, false]] as const) {
        const base = this.rings[r];
        let x = 0, y = 0, z = 0;
        for (let i = 0; i < n; i++) {
          x += this.pos[3 * (base + i)];
          y += this.pos[3 * (base + i) + 1];
          z += this.pos[3 * (base + i) + 2];
        }
        const c = this.pos.length / 3;
        this.pos.push(x / n, y / n, z / n);
        for (let i = 0; i < n; i++) {
          const j = (i + 1) % n;
          this.idx.push(...(flip ? [c, base + j, base + i] : [c, base + i, base + j]));
        }
      }
    }
    return { positions: new Float32Array(this.pos), indices: new Uint32Array(this.idx) };
  }
}

/** a full ring at height y from a half-curve: right side back → front, then the left side back */
function fullRing(half: number[], y: number): number[] {
  const n = half.length / 2;
  const pts: number[] = [];
  for (let i = 0; i < n; i++) pts.push(half[2 * i], y, half[2 * i + 1]);
  for (let i = n - 2; i > 0; i--) pts.push(half[2 * i], y, -half[2 * i + 1]);
  return pts;
}

/** a tube along a polyline (closed by end fans), elliptical section with half-axes (a: in the
 *  curve's plane normal, b: across) per point */
function tube(path: [number, number, number][], ra: (i: number) => number, rb: (i: number) => number, segs = 18): { positions: Float32Array; indices: Uint32Array } {
  const L = new Loft();
  for (let i = 0; i < path.length; i++) {
    const p = path[i], q = path[Math.min(path.length - 1, i + 1)], o = path[Math.max(0, i - 1)];
    let t = [q[0] - o[0], q[1] - o[1], q[2] - o[2]];
    const tl = Math.hypot(...t) || 1;
    t = t.map((v) => v / tl);
    // a frame: u ⟂ t towards "up" (y), v = t × u
    const up = Math.abs(t[1]) > 0.9 ? [1, 0, 0] : [0, 1, 0];
    const d = up[0] * t[0] + up[1] * t[1] + up[2] * t[2];
    let u = [up[0] - d * t[0], up[1] - d * t[1], up[2] - d * t[2]];
    const ul = Math.hypot(...u) || 1;
    u = u.map((v) => v / ul);
    const v = [t[1] * u[2] - t[2] * u[1], t[2] * u[0] - t[0] * u[2], t[0] * u[1] - t[1] * u[0]];
    const pts: number[] = [];
    for (let k = 0; k < segs; k++) {
      const an = (k / segs) * Math.PI * 2, c = Math.cos(an) * ra(i), s = Math.sin(an) * rb(i);
      pts.push(p[0] + u[0] * c + v[0] * s, p[1] + u[1] * c + v[1] * s, p[2] + u[2] * c + v[2] * s);
    }
    L.ring(pts);
  }
  return L.build();
}

/** a 2D closed outline (x, y) extruded between z0 and z1 */
function prism(outline: Vec[], z0: number, z1: number): { positions: Float32Array; indices: Uint32Array } {
  const L = new Loft();
  const pts = outline.slice(0, outline.length - (outline[0].x === outline[outline.length - 1].x && outline[0].y === outline[outline.length - 1].y ? 1 : 0));
  const steps = 6;
  for (let s = 0; s <= steps; s++) {
    const z = z0 + ((z1 - z0) * s) / steps;
    L.ring(pts.flatMap((p) => [p.x, p.y, z]));
  }
  return L.build();
}

/** two closed surfaces as one mesh: by parity, the solid between them (a tube's wall) */
function merge(a: { positions: Float32Array; indices: Uint32Array }, b: { positions: Float32Array; indices: Uint32Array }) {
  const n = a.positions.length / 3;
  const positions = new Float32Array(a.positions.length + b.positions.length);
  positions.set(a.positions);
  positions.set(b.positions, a.positions.length);
  const indices = new Uint32Array(a.indices.length + b.indices.length);
  indices.set(a.indices);
  // the inner surface faces inwards (towards the hollow)
  for (let i = 0; i < b.indices.length; i += 3) {
    indices[a.indices.length + i] = b.indices[i] + n;
    indices[a.indices.length + i + 1] = b.indices[i + 2] + n;
    indices[a.indices.length + i + 2] = b.indices[i + 1] + n;
  }
  return { positions, indices };
}

/** The head's parts for posture `a` (body cm). */
export function buildHead(model: VoiceModel, a: HumanArtic): HeadPart[] {
  const t = model.tract;
  const k = t.scale;
  const ph = model.bodyParams.pharynx;
  const unBy = (y: number) => (y / k >= 7 ? y / k : 7 - (7 - y / k) / ph);
  const unByInv = (Y: number) => (Y >= 7 ? Y : 7 - (7 - Y) * ph) * k;
  const frame = model.frame(a);
  const prof = buildProfile(model, frame, a);
  const skin = [...prof.skinLower, ...prof.skinUpper];
  const head = new Shape(skin, 0.06 * k, (y, F, Bk) => {
    const Y = unBy(y);
    const plane = Y >= 3.4 && Y <= 16.1 ? interp(PLANE, Y) * k : Infinity;
    const Fs = Math.min(F, plane);
    // the front half rounder than the face screen's (an ellipse narrows the face towards its
    // front, where a squarer section read as a box from the front)
    const pf = 2.0 - 0.15 * smooth(4, 6, Y) * (1 - smooth(15, 17, Y));
    return { F, Bk, Fs, W: interp(WIDTH, Y) * k, xw: Bk + (Fs - Bk) * interp(WIDEST, Y), E: F - Fs, wf: interp(FEATURE, Y) * k, pf };
  });
  const lipTop = frame.upperLip[frame.upperLip.length - 1];
  const lipBottom = frame.lowerLip[frame.lowerLip.length - 1];
  // the mouth's half-width: about 2.5 cm spread, 1.2 cm rounded (template), as the lips' patch
  const mouthHalf = (1.2 + 1.4 * Math.max(0, Math.min(1, a.lipWidth))) * k;

  // ---- layer thicknesses (template cm) by region; X, Y, Z template, Z = |z|
  const fat = (X: number, Y: number, Z: number) => {
    // over the larynx the skin lies on thin strap muscles (the prominence is felt through it)
    const neck = (1 - smooth(3.0, 5.0, Y)) * (1 - 0.7 * smooth(0.5, 2.0, X));
    const face = smooth(3.5, 5.0, Y) * (1 - smooth(15.5, 17, Y)) * smooth(1.0, 3.5, X);
    const cheek = face * smooth(1.2, 2.4, Z) * smooth(5.5, 7.5, Y) * (1 - smooth(11.5, 13, Y));
    const nose = smooth(10.4, 11.0, Y) * (1 - smooth(14.5, 15.2, Y)) * (1 - smooth(0.9, 1.6, Z)) * smooth(5.5, 7.0, X);
    const back = 1 - smooth(-3.5, -1.5, X);
    return 0.35 + 0.35 * neck + 0.25 * neck * back + 0.75 * cheek - 0.25 * nose + 0.15 * face * (1 - cheek);
  };
  const muscle = (X: number, Y: number, Z: number) => {
    const neck = (1 - smooth(3.5, 5.5, Y)) * (1 - 0.6 * smooth(0.5, 2.0, X) * (1 - smooth(1.5, 3.0, Z)));
    const nuchal = neck * (1 - smooth(-4.0, -1.5, X)) * (1 - smooth(9, 11, Y));
    const scm = neck * smooth(2.0, 3.5, Z);
    const masseter = smooth(4.5, 5.5, Y) * (1 - smooth(9.5, 10.5, Y)) * smooth(3.4, 4.4, Z) * smooth(-2.8, -1.8, X) * (1 - smooth(2.0, 3.2, X));
    const temporal = smooth(11.5, 12.5, Y) * (1 - smooth(17.5, 19, Y)) * smooth(4.0, 5.0, Z) * smooth(-4.5, -3, X) * (1 - smooth(3.5, 5, X));
    const lips = smooth(6.0, 6.8, Y) * (1 - smooth(9.0, 9.8, Y)) * (1 - smooth(1.6, 2.6, Z)) * smooth(5.0, 6.5, X);
    const scalp = smooth(15, 17, Y);
    return Math.max(0.12, 0.45 * (1 - scalp) + 0.6 * neck + 3.0 * nuchal + 0.9 * scm + 1.1 * masseter + 0.9 * temporal + 0.35 * lips - 0.3 * scalp);
  };
  const SKIN = 0.2;
  const depths = [
    (_X: number, _Y: number, _Z: number) => 0,
    () => SKIN,
    (X: number, Y: number, Z: number) => SKIN + fat(X, Y, Z),
    (X: number, Y: number, Z: number) => SKIN + fat(X, Y, Z) + muscle(X, Y, Z),
  ];
  const layerKeys = ["skin", "fat", "muscle", "deep"] as const;
  const caps = [TISSUE.skinCut, TISSUE.fat, TISSUE.muscle, TISSUE.deepCut];
  const stacks: number[][][] = depths.map(() => []);
  const ys: number[] = [];
  /** rings beside the open mouth: kept unsmoothed so the rim stays where it was put */
  const near: boolean[] = [];
  const mouthRing: boolean[][] = [];
  const mouthMid = 0.5 * (lipTop.y + lipBottom.y), mouthH = 0.5 * Math.max(0, lipTop.y - lipBottom.y);
  const y0 = head.y0 + 0.15 * k, y1 = head.y1 - 0.12 * k, dy = 0.14 * k;
  let rings = 0;
  for (let y = y0; y <= y1; y += dy) {
    const s = head.at(y);
    if (!s || s.W < 0.25 * k) continue;
    const Y = unBy(y);
    // the face's relief over the cross-sections: brow ridge, eye sockets, cheekbones (template cm,
    // along the surface's normal), front half only
    const relief = (x: number, z: number) => {
      const front = smooth(s.xw, s.xw + 0.5 * (s.Fs - s.xw), x);
      const Z = Math.abs(z) / k;
      const g = (a: number, b: number) => Math.exp(-(a * a + b * b));
      const brow = 0.32 * Math.exp(-(((Y - 16.4) / 0.55) ** 2)) * (1 - smooth(4.0, 5.2, Z));
      const socket = -0.3 * g((Y - 15.35) / 0.8, (Z - 3.15) / 1.15);
      const cheekbone = 0.28 * g((Y - 13.7) / 0.85, (Z - 4.5) / 1.1);
      const temple = -0.2 * g((Y - 16.0) / 1.2, (Z - 6.2) / 0.8);
      return front * (brow + socket + cheekbone + temple) * k;
    };
    const outer = inset(halfCurve(s, 1), (x, z) => -relief(x, z), Infinity);
    // the mouth: a lens between the lips' edges; the first point outside it is moved onto its
    // edge (a smooth rim instead of whole quads), and the rim rolls back into the mouth
    const vm = mouthH > 0 ? (y - mouthMid) / mouthH : 2;
    if (Math.abs(vm) < 1) {
      const zb = mouthHalf * Math.sqrt(1 - vm * vm);
      const n = outer.length / 2;
      for (let i = NB; i < n; i++) {
        const z = outer[2 * i + 1];
        if (z >= zb && (i === n - 1 || outer[2 * i + 3] < zb)) {
          outer[2 * i + 1] = zb;
          outer[2 * i] -= 0.25 * k * Math.sqrt(1 - vm * vm);
          break;
        }
      }
    }
    depths.forEach((depth, li) => {
      const curve = li === 0 ? outer : inset(halfCurve(s, Math.max(0, 1 - depth(0, Y, 0) / 0.6)), (x, z) => depth(x / k, Y, Math.abs(z) / k) * k, 0.42 * s.W);
      stacks[li].push(curve);
    });
    ys.push(y);
    near.push(mouthH > 0 && Math.abs((y - mouthMid) / mouthH) < 1.4);
    // the mouth: quads of the outer skin between the lips' edges, near the midline, in front
    const ring = fullRing(outer, y);
    const inMouth: boolean[] = [];
    for (let i = 0; i < ring.length / 3; i++) {
      const x = ring[3 * i], z = ring[3 * i + 2];
      // a lens between the lips' edges, widest at their middle
      const v = mouthH > 0 ? (y - mouthMid) / mouthH : 2;
      inMouth.push(Math.abs(v) < 1 && Math.abs(z) < mouthHalf * Math.sqrt(1 - v * v) && x > s.xw + 0.5 * (s.Fs - s.xw));
    }
    mouthRing.push(inMouth);
    rings++;
  }
  const parts: HeadPart[] = [];
  if (rings < 2) return [];
  // the cross-section tables are piecewise linear in height: smooth each point's path up the head
  // (a few passes of neighbour averaging) so the shading shows no bands at the tables' knots
  const lofts = stacks.map((stack) => {
    let cur = stack;
    for (let pass = 0; pass < 4; pass++) {
      cur = cur.map((c, r) => (r === 0 || r === cur.length - 1 || near[r] ? c : c.map((v, i) => 0.25 * cur[r - 1][i] + 0.5 * v + 0.25 * cur[r + 1][i])));
    }
    const L = new Loft();
    cur.forEach((c, r) => L.ring(fullRing(c, ys[r])));
    return L;
  });
  // the visible skin: the outer surface with the mouth open
  const visible = lofts[0].build((r, i) => (mouthRing[r][i] && mouthRing[r + 1]?.[i]) || false);
  // colour: the lips' vermilion round the mouth, a little warmth over the cheeks and the nose
  const lin = (hex: number) => new Float32Array([((hex >> 16) & 255) / 255, ((hex >> 8) & 255) / 255, (hex & 255) / 255].map((c) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4)));
  const cSkin = lin(TISSUE.skin), cLip = lin(0xb4625e), cWarm = lin(0xd88f7c);
  const colors = new Float32Array(visible.positions.length);
  for (let i = 0; i < visible.positions.length / 3; i++) {
    const x = visible.positions[3 * i], y = visible.positions[3 * i + 1], z = visible.positions[3 * i + 2];
    const s = head.at(y);
    const front = s ? smooth(s.xw + 0.55 * (s.Fs - s.xw), s.xw + 0.8 * (s.Fs - s.xw), x) : 0;
    const Z = Math.abs(z);
    // the vermilion: a wider lens round the opening (closed lips: round the line between them)
    const ev = (y - mouthMid) / (mouthH + 0.62 * k), ez = Z / (mouthHalf + 0.3 * k);
    const lipBand = 1 - smooth(0.8, 1.05, Math.hypot(ev, ez));
    const Y = unBy(y);
    const warm = 0.35 * Math.exp(-(((Y - 12.8) / 1.3) ** 2 + ((Z / k - 4.0) / 1.4) ** 2)) + 0.25 * Math.exp(-(((Y - 12.2) / 1.2) ** 2 + (Z / k / 0.9) ** 2)) * front;
    const l = Math.min(1, lipBand) * front;
    for (let c = 0; c < 3; c++) colors[3 * i + c] = cSkin[c] * (1 - l - warm * (1 - l)) + cLip[c] * l + cWarm[c] * warm * (1 - l);
  }
  parts.push({ key: "skin-visible", color: TISSUE.skin, cap: TISSUE.skinCut, ...visible, colors, closed: false, skin: true });
  // the layers as nested closed surfaces (their cut faces stack outermost first); the innermost
  // ("deep", the inner face of the muscle layer) is only seen from inside, when the head is cut
  lofts.forEach((L, li) => parts.push({ key: layerKeys[li], color: [TISSUE.skin, TISSUE.fat, TISSUE.muscle, TISSUE.deep][li], cap: caps[li], ...L.build(), closed: true }));

  // ---- skull vault and brain: the brain's cross-sections, the skull 0.7 cm outside them
  const brainShape = new Shape(prof.brain, 0.08 * k, (y, F, Bk) => ({ F, Bk, Fs: F, W: 0.92 * interp(BRAIN_WIDTH, unBy(y)) * k, xw: Bk + (F - Bk) * 0.48, E: 0, wf: 1, pf: 2.3 }));
  const brain = new Loft();
  const skull = new Loft();
  for (let y = brainShape.y0 + 0.1 * k; y <= brainShape.y1 - 0.08 * k; y += 0.15 * k) {
    const s = brainShape.at(y);
    if (!s || s.W < 0.3 * k) continue;
    const c = halfCurve(s, 0);
    brain.ring(fullRing(c, y));
    skull.ring(fullRing(inset(c, () => -0.55 * k, 10), y));
  }
  parts.push({ key: "skull", color: TISSUE.bone, cap: TISSUE.bone, ...skull.build(), closed: true });
  parts.push({ key: "brain", color: TISSUE.brain, cap: TISSUE.brain, ...brain.build(), closed: true });

  // ---- ears: a rim (the helix) round a shallow bowl, on each side behind the jaw's joint
  for (const side of [1, -1]) {
    const ear = new Loft();
    const cy = unByInv(12.4), cx = -3.9 * k;
    const sAt = head.at(cy);
    const zs = sAt ? sAt.W * 0.97 : 6 * k;
    for (let r = 0; r <= 10; r++) {
      const u = r / 10; // from the skin outwards
      const ring: number[] = [];
      for (let i = 0; i < 28; i++) {
        const an = (i / 28) * Math.PI * 2;
        // a tall oval, narrower at the lobe, leaning back
        const h = 3.2 * k * (an > Math.PI ? 0.5 + 0.12 * Math.cos(an) : 0.5), w = 1.0 * k * (an > Math.PI ? 0.8 : 1);
        let ex = Math.cos(an) * w, ey = Math.sin(an) * h;
        const lean = 0.25;
        [ex, ey] = [ex * Math.cos(lean) - ey * Math.sin(lean), ex * Math.sin(lean) + ey * Math.cos(lean)];
        // the rim stands out and curls; the middle (the concha) stays near the head
        const rim = Math.sin(Math.PI * Math.min(1, u * 1.1));
        const shrink = 1 - 0.75 * u * u;
        ring.push(cx + ex * shrink, cy + ey * shrink, side * (zs - 0.25 * k + (0.2 + 1.1 * rim) * k * (1 - 0.5 * u)));
      }
      ear.ring(side > 0 ? ring : ring.reverse());
    }
    parts.push({ key: `ear${side}`, color: TISSUE.skin, cap: TISSUE.skinCut, ...ear.build(), closed: false, skin: true });
  }

  // ---- eyes, closed: the lid's curve over the eyeball and the line of the lashes
  for (const side of [1, -1]) {
    const c = prof.eyeC;
    const s = head.at(c.y);
    if (!s) continue;
    const zc = 3.15 * k;
    // the face's surface at the eye, set back into the socket
    let xs = s.xw;
    for (let i = 0; i <= 40; i++) {
      const x = s.xw + ((s.Fs - s.xw) * i) / 40;
      const u = (x - s.xw) / Math.max(1e-6, s.Fs - s.xw);
      const zz = s.W * Math.pow(Math.max(0, 1 - Math.pow(u, s.pf)), 1 / s.pf);
      if (zz >= zc) xs = x;
    }
    const lid = new Loft();
    const R = 1.15 * k;
    for (let r = 0; r <= 12; r++) {
      const v = -1 + (2 * r) / 12; // bottom → top of the lid
      const ring: number[] = [];
      for (let i = 0; i < 24; i++) {
        const an = (i / 24) * Math.PI * 2;
        const h = Math.sqrt(Math.max(0, 1 - v * v));
        // an almond: wide across, short up and down, bulging forward over the eyeball
        const dz = Math.cos(an) * 1.25 * k * h, dy = v * 0.62 * k, dx = (0.25 + 0.3 * Math.sin(an) * 0) * k;
        ring.push(xs - 0.3 * k + dx + R * h * 0.5, c.y + dy, side * (zc + dz * 0.95));
      }
      lid.ring(side > 0 ? ring : ring.reverse());
    }
    parts.push({ key: `lid${side}`, color: TISSUE.skin, cap: TISSUE.skinCut, ...lid.build(), closed: false, skin: true });
    const lash: [number, number, number][] = [];
    for (let i = 0; i <= 16; i++) {
      const u = -1 + (2 * i) / 16;
      lash.push([xs - 0.3 * k + 0.25 * k + R * 0.5 * Math.sqrt(Math.max(0, 1 - u * u)) + 0.03 * k, c.y - 0.05 * k - 0.12 * k * (1 - u * u), side * (zc + u * 1.19 * k)]);
    }
    parts.push({ key: `lash${side}`, color: 0x2a1a16, cap: 0x2a1a16, ...tube(lash, () => 0.045 * k, () => 0.03 * k, 8), closed: false });
  }

  // ---- mandible: a bar along the U of the body and up the rami to the condyles, turned by the jaw
  const condyle = t.P(-2.5, 10.5);
  const phi = jawAngle(model, a.jaw);
  const J = (x: number, y: number, z: number): [number, number, number] => {
    const p = t.P(x, y);
    const c = Math.cos(phi), s = Math.sin(phi);
    return [condyle.x + (p.x - condyle.x) * c - (p.y - condyle.y) * s, condyle.y + (p.x - condyle.x) * s + (p.y - condyle.y) * c, z * k];
  };
  const MANDIBLE: [number, number, number, number, number][] = [
    // x, y, z (template), vertical half-height, lateral half-thickness
    [-2.45, 10.2, 4.9, 0.45, 0.4],
    [-2.2, 9.0, 4.85, 0.9, 0.3],
    [-2.0, 7.6, 4.75, 1.2, 0.3],
    [-1.4, 6.2, 4.6, 1.0, 0.35],
    [0.2, 5.0, 4.3, 1.1, 0.45],
    [2.0, 4.6, 3.8, 1.15, 0.5],
    [3.8, 4.4, 3.0, 1.2, 0.5],
    [5.3, 4.3, 1.9, 1.25, 0.55],
    [6.3, 4.5, 0.9, 1.3, 0.6],
    [6.6, 4.6, 0.0, 1.35, 0.6],
  ];
  const mpath = [...MANDIBLE.map(([x, y, z]) => J(x, y, z)), ...MANDIBLE.slice(0, -1).reverse().map(([x, y, z]) => J(x, y, -z))];
  const mrad = [...MANDIBLE.map((m) => m[3]), ...MANDIBLE.slice(0, -1).reverse().map((m) => m[3])];
  const mthk = [...MANDIBLE.map((m) => m[4]), ...MANDIBLE.slice(0, -1).reverse().map((m) => m[4])];
  parts.push({ key: "mandible", color: TISSUE.bone, cap: TISSUE.bone, ...tube(mpath, (i) => mrad[i] * k, (i) => mthk[i] * k), closed: true });

  // ---- cervical vertebrae (bodies), hyoid, thyroid cartilage, cricoid and trachea
  prof.vertebrae.forEach((v, i) => parts.push({ key: `vertebra${i}`, color: TISSUE.bone, cap: TISSUE.bone, ...prism(v, -1.3 * k, 1.3 * k), closed: true }));
  const hy = prof.hyoid.reduce((m, p) => ({ x: m.x + p.x / prof.hyoid.length, y: m.y + p.y / prof.hyoid.length }), { x: 0, y: 0 });
  const hpath: [number, number, number][] = [];
  for (let i = 0; i <= 16; i++) {
    const an = -Math.PI / 2 + (i / 16) * Math.PI;
    hpath.push([hy.x - 1.6 * k * (1 - Math.cos(an)), hy.y + 0.15 * k * Math.abs(Math.sin(an)), 1.9 * k * Math.sin(an)]);
  }
  parts.push({ key: "hyoid", color: TISSUE.bone, cap: TISSUE.bone, ...tube(hpath, () => 0.28 * k, () => 0.22 * k, 12), closed: true });
  const g = frame.glottis;
  const shield = new Loft();
  for (let y = g.y - 1.4 * k; y <= g.y + 1.9 * k; y += 0.12 * k) {
    const u = (y - (g.y - 1.4 * k)) / (3.3 * k);
    // the laryngeal prominence in front (highest at its notch), the laminae going back at ~90°
    const xf = g.x + (1.2 + 0.7 * Math.sin(Math.PI * Math.min(1, u * 1.15))) * k;
    const back = 2.0 * k, wide = 2.1 * k, th = 0.22 * k;
    const half: number[] = [];
    for (let i = 0; i <= 10; i++) half.push(xf - (back * i) / 10, (wide * i) / 10);
    const inner = half.map((v, i) => (i % 2 === 0 ? v - th : Math.max(0, v - th * 0.7)));
    const ring: number[] = [];
    for (let i = 0; i <= 10; i++) ring.push(half[2 * i], y, half[2 * i + 1]);
    for (let i = 10; i >= 0; i--) ring.push(inner[2 * i], y, inner[2 * i + 1]);
    for (let i = 1; i < 10; i++) ring.push(inner[2 * i], y, -inner[2 * i + 1]);
    for (let i = 10; i >= 1; i--) ring.push(half[2 * i], y, -half[2 * i + 1]);
    shield.ring(ring);
  }
  parts.push({ key: "thyroid", color: TISSUE.cartilage, cap: TISSUE.cartilage, ...shield.build(), closed: true });
  const trachea: [number, number, number][] = [];
  for (let y = g.y - 0.3 * k; y >= -4.5 * k; y -= 0.25 * k) trachea.push([g.x + 0.1 * k + 0.04 * (g.y - y), y, 0]);
  parts.push({ key: "trachea", color: TISSUE.cartilage, cap: TISSUE.cartilage, ...merge(tube(trachea, () => 0.95 * k, () => 1.05 * k, 24), tube(trachea, () => 0.72 * k, () => 0.8 * k, 24)), closed: true });
  return parts;
}
