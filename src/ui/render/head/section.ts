/**
 * The cut face of the opened head: its midsagittal section, painted on a canvas that the scene
 * maps onto the plane of the cut. Display only.
 *
 * The air is the engine's: the 3D tract's lamina at the plane (`Tract3d::display`), drawn by the
 * rule the cuts use (the envelope, less the tissue polygons, plus the pockets inside tissue), so
 * the hole in the cut face is the air the cross-sections measure. The tongue, the soft palate
 * and the teeth are the engine's midline outlines. Around them, from the skin inwards: skin,
 * subcutaneous fat, muscle, and the skull with the brain, the cervical spine and cord, the hard
 * palate, the chin's bone, the hyoid and the laryngeal cartilages, placed by the template the
 * engine's tract is drawn in. Thicknesses are typical adult values from anatomy texts.
 */
import type { Tract3dDisplay, Tract3dPolygon } from "../../../wasm/types.ts";
import type { VoiceModel } from "../../analysis.ts";

type P = { x: number; y: number };

/** colours of the cut tissues */
export const CUT = {
  skin: "#e3b59c",
  fat: "#efd17c",
  muscle: "#a04442",
  deep: "#8c3d3c",
  bone: "#efe6d2",
  marrow: "#e2d2b4",
  cartilage: "#c4dcd6",
  brain: "#dcbcb6",
  brainFold: "#c79c98",
  cord: "#ecdcae",
  tongue: "#bc4a50",
  fibre: "#d4737a",
  mucosa: "#d4868a",
  enamel: "#f6f2e8",
  dentine: "#e8dcc0",
  hollow: "#3b1517",
  stand: "#cfa68f",
  line: "#5b2a28",
};

export interface SectionInput {
  model: VoiceModel;
  display: Tract3dDisplay;
  /** the head's outline in the plane (body cm): front chain (stand → chin → lower lip) and back chain (upper lip → crown → stand) */
  front: P[];
  back: P[];
  /** how many of the front chain's first points, and of the back chain's last, belong to the stand */
  standFront: number;
  standBack: number;
  larynx: number;
}

/** the canvas' extent in body cm, for a body of scale k */
export function sectionBounds(k: number): { x0: number; x1: number; y0: number; y1: number } {
  return { x0: -17 * k, x1: 11 * k, y0: -12.5 * k, y1: 27 * k };
}

export class SectionPainter {
  readonly canvas: HTMLCanvasElement;
  private readonly mask: HTMLCanvasElement;
  private readonly ctx: CanvasRenderingContext2D;
  private readonly mctx: CanvasRenderingContext2D;
  /** pixels per cm */
  readonly scale: number;
  constructor(readonly bounds: { x0: number; x1: number; y0: number; y1: number }, height = 1024) {
    this.scale = height / (bounds.y1 - bounds.y0);
    const w = Math.ceil((bounds.x1 - bounds.x0) * this.scale);
    this.canvas = document.createElement("canvas");
    this.mask = document.createElement("canvas");
    this.canvas.width = this.mask.width = w;
    this.canvas.height = this.mask.height = height;
    this.ctx = this.canvas.getContext("2d")!;
    this.mctx = this.mask.getContext("2d")!;
  }

  paint(input: SectionInput): void {
    const { ctx, mctx, canvas, bounds, scale: S } = this;
    const { model, display: d } = input;
    const k = model.tract.scale;
    const ph = model.bodyParams.pharynx;
    const X = (x: number) => (x - bounds.x0) * S;
    const Y = (y: number) => (bounds.y1 - y) * S;
    const path = (c: CanvasRenderingContext2D, pts: P[], close = true) => {
      c.beginPath();
      pts.forEach((p, i) => (i ? c.lineTo(X(p.x), Y(p.y)) : c.moveTo(X(p.x), Y(p.y))));
      if (close) c.closePath();
    };
    const fill = (pts: P[], color: string) => {
      if (pts.length < 3) return;
      path(ctx, pts);
      ctx.fillStyle = color;
      ctx.fill("evenodd");
    };
    const stroke = (pts: P[], color: string, cm: number, close = false) => {
      path(ctx, pts, close);
      ctx.strokeStyle = color;
      ctx.lineWidth = cm * S;
      ctx.stroke();
    };
    /** template → body: the tract's own map, and the face's (no oral compression) */
    const T = (x: number, y: number): P => model.tract.P(x, y);
    const B = (x: number, y: number): P => ({ x: x * k, y: (y < 7 ? 7 - (7 - y) * ph : y) * k });
    const jc = Math.cos(d.jaw.angle), js = Math.sin(d.jaw.angle);
    const J = (x: number, y: number): P => {
      const p = T(x, y);
      const dx = p.x - d.jaw.pivot.x, dy = p.y - d.jaw.pivot.y;
      return { x: d.jaw.pivot.x + dx * jc - dy * js, y: d.jaw.pivot.y + dx * js + dy * jc };
    };
    const outline = (name: string) => d.outlines.find((o) => o.name === name)?.points ?? [];
    const g = d.glottis;

    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.globalCompositeOperation = "source-over";
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.lineJoin = ctx.lineCap = "round";
    const front = input.front, back = input.back;
    const head = [...front, ...back];
    ctx.save();
    path(ctx, head);
    ctx.clip();

    // ---- deep tissue, then what lies in it
    ctx.fillStyle = CUT.deep;
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    // the muscle layer under the fat: a band inside the outline, thick at the nape
    stroke(front.slice(input.standFront), CUT.muscle, 2 * 1.3 * k);
    stroke(back.slice(0, back.length - input.standBack), CUT.muscle, 2 * 1.6 * k);

    // the skull: the outline above the base of the skull, moved in by the scalp and the bone
    const skin = back.slice(0, back.length - input.standBack);
    const baseT: [number, number][] = [[6.6, 15.6], [5.0, 14.6], [3.4, 14.0], [0.8, 13.4], [-2.2, 12.5], [-4.4, 11.2], [-6.4, 11.0], [-9.2, 11.6], [-12.4, 13.4]];
    const base = baseT.map(([x, y]) => B(x, y));
    const baseAt = (x: number) => {
      if (x >= base[0].x) return base[0].y;
      for (let i = 1; i < base.length; i++) if (x >= base[i].x) return base[i].y + ((base[i - 1].y - base[i].y) * (x - base[i].x)) / (base[i - 1].x - base[i].x);
      return base[base.length - 1].y;
    };
    const inset = offsetIn(skin, 1.25 * k);
    const vault = inset.filter((p) => p.y > baseAt(p.x) + 0.2 * k);
    if (vault.length > 4) {
      // closed along the base of the skull, from the back to the front
      const brain = [...vault, ...base.slice().reverse().filter((p) => p.x > vault[vault.length - 1].x && p.x < vault[0].x)];
      stroke(brain, CUT.bone, 2 * 0.75 * k, true);
      fill(brain, CUT.brain);
      ctx.save();
      path(ctx, brain);
      ctx.clip();
      // folds of the cortex, the cerebellum and the brain stem
      for (const f of offsetBands(vault, [0.7 * k, 1.6 * k])) stroke(f, CUT.brainFold, 0.09 * k);
      const cb = B(-8.4, 13.9);
      for (let r = 0.5; r < 2.6; r += 0.42) {
        ctx.beginPath();
        ctx.ellipse(X(cb.x), Y(cb.y), r * 1.25 * k * S, r * 0.8 * k * S, 0.25, 0, Math.PI * 2);
        ctx.strokeStyle = CUT.brainFold;
        ctx.lineWidth = 0.08 * k * S;
        ctx.stroke();
      }
      stroke([B(-3.6, 14.4), B(-4.2, 12.8), B(-4.7, 11.2)], CUT.cord, 1.1 * k);
      ctx.restore();
    }
    // cervical spine: bodies with discs between, the cord behind them, the spinous processes
    const wallX = (y: number) => -1.2 - 0.025 * y;
    stroke([T(-4.9, -7.5), T(-4.9, 10.6), B(-4.6, 11.4)], CUT.cord, 0.85 * k);
    for (let i = 0; i < 10; i++) {
      const y0 = -6.15 + i * 1.75;
      if (y0 > 10.5) break;
      const x0 = wallX(y0) - 2.25;
      fill(roundRect(T(x0, y0 + 1.4), T(x0 + 1.6, y0), 0.3 * k), CUT.bone);
      fill(roundRect(T(x0 + 0.25, y0 + 1.15), T(x0 + 1.35, y0 + 0.25), 0.2 * k), CUT.marrow);
      fill(roundRect(T(x0 + 0.1, y0 + 1.72), T(x0 + 1.5, y0 + 1.45), 0.1 * k), CUT.cartilage);
      fill([T(x0 - 1.75, y0 + 0.95), T(x0 - 3.0, y0 + 0.35), T(x0 - 3.1, y0 + 0.05), T(x0 - 1.75, y0 + 0.45)], CUT.bone);
    }
    // nasal cavity with the turbinates, the hard palate under it
    const nasal = outline("nasalCavity");
    const nasalRoof = [B(0.3, 11.2), B(1.1, 10.25), B(2.7, 10.75), B(4.6, 10.85), B(6.4, 10.65), B(7.6, 10.85), B(8.3, 11.3), B(7.9, 12.3), B(6.7, 13.4), B(4.4, 13.9), B(1.8, 13.6), B(0.2, 12.6)];
    fill(smoothClosed(nasalRoof), CUT.hollow);
    // the nasopharynx behind the soft palate, down to where the engine's pharynx begins (the
    // soft palate, drawn over it below, shuts or opens the way)
    fill(smoothClosed([T(-1.32, 9.0), T(-1.4, 11.3), B(0.4, 12.4), B(1.5, 10.6), T(0.9, 9.5), T(-0.4, 9.0)]), CUT.hollow);
    for (const [cx, cy, rx, ry] of [[3.0, 11.5, 2.0, 0.3], [3.5, 12.45, 1.8, 0.28], [4.0, 13.25, 1.35, 0.2]] as const) {
      const c = B(cx, cy);
      ctx.beginPath();
      ctx.ellipse(X(c.x), Y(c.y), rx * k * S, ry * k * S, 0.04, 0, Math.PI * 2);
      ctx.fillStyle = CUT.mucosa;
      ctx.fill();
    }
    if (nasal.length > 2) stroke(nasal, CUT.mucosa, 0.12 * k);
    const pal: [number, number][] = [[1.0, 9.6], [1.8, 9.92], [2.7, 10.18], [3.6, 10.3], [4.4, 10.25], [5.1, 10.05], [5.7, 9.72], [6.15, 9.36], [6.55, 9.15], [7.0, 9.6]];
    const palOral = pal.map(([x, y]) => T(x, y));
    fill([...palOral, ...palOral.slice().reverse().map((p) => ({ x: p.x, y: p.y + 0.6 * k }))], CUT.bone);
    // the upper jaw's bone over the incisors' roots
    fill([T(6.55, 9.15), T(7.0, 9.6), T(7.5, 9.5), T(7.55, 8.7), T(7.2, 8.45), T(6.9, 8.6)], CUT.bone);
    // the chin's bone, carried by the jaw
    fill(smoothClosed([J(6.5, 6.5), J(7.15, 6.7), J(7.5, 6.0), J(7.55, 5.2), J(7.2, 4.7), J(6.5, 4.6), J(6.15, 5.2), J(6.2, 6.0)]), CUT.bone);
    fill(smoothClosed([J(6.6, 6.1), J(7.1, 6.2), J(7.25, 5.4), J(7.0, 4.95), J(6.55, 4.95), J(6.45, 5.5)]), CUT.marrow);
    // the hyoid, the thyroid and cricoid cartilages, the trachea with its rings
    const hy = { x: T(1.6, 0).x, y: T(0, 2.9).y + 0.6 * input.larynx * k };
    ctx.beginPath();
    ctx.ellipse(X(hy.x), Y(hy.y), 0.5 * k * S, 0.28 * k * S, -0.3, 0, Math.PI * 2);
    ctx.fillStyle = CUT.bone;
    ctx.fill();
    fill([{ x: g.x - 0.72 * k, y: g.y }, { x: g.x + 0.82 * k, y: g.y }, { x: g.x + 1.05 * k, y: bounds.y0 }, { x: g.x - 0.85 * k, y: bounds.y0 }], CUT.hollow);
    fill([{ x: g.x + 0.95 * k, y: g.y - 1.5 * k }, { x: g.x + 1.6 * k, y: g.y - 0.7 * k }, { x: g.x + 1.75 * k, y: g.y + 1.0 * k }, { x: g.x + 1.4 * k, y: g.y + 1.8 * k }, { x: g.x + 1.15 * k, y: g.y + 1.1 * k }, { x: g.x + 1.25 * k, y: g.y - 0.4 * k }], CUT.cartilage);
    fill(roundRect({ x: g.x - 1.25 * k, y: g.y - 0.2 * k }, { x: g.x - 0.8 * k, y: g.y - 1.5 * k }, 0.15 * k), CUT.cartilage);
    for (let i = 0; i < 6; i++) {
      const y = g.y - (1.9 + i * 0.95) * k;
      fill(roundRect({ x: g.x + 0.9 * k + 0.035 * (g.y - y), y }, { x: g.x + 1.2 * k + 0.035 * (g.y - y), y: y - 0.55 * k }, 0.12 * k), CUT.cartilage);
    }
    // the vocal folds at the glottis
    fill([{ x: g.x - 0.6 * k, y: g.y + 0.02 * k }, { x: g.x - 0.08 * k, y: g.y + 0.14 * k }, { x: g.x - 0.6 * k, y: g.y + 0.34 * k }], CUT.mucosa);
    fill([{ x: g.x + 0.7 * k, y: g.y + 0.02 * k }, { x: g.x + 0.1 * k, y: g.y + 0.14 * k }, { x: g.x + 0.7 * k, y: g.y + 0.34 * k }], CUT.mucosa);

    // ---- the tract's own parts on the midline (the engine's outlines)
    const tongue = outline("tongue");
    if (tongue.length > 2) {
      fill(tongue, CUT.tongue);
      // the fan of the genioglossus from the chin's inside to the dorsum
      const origin = J(6.0, 5.5);
      ctx.save();
      path(ctx, tongue);
      ctx.clip();
      for (let i = 2; i < tongue.length - 8; i += 3) stroke([origin, tongue[i]], CUT.fibre, 0.05 * k);
      ctx.restore();
    }
    fill(outline("velum"), CUT.mucosa);
    const epi = d.tissue.find((t) => t.name === "epiglottis");
    if (epi) fill(epi.points, CUT.cartilage);
    for (const name of ["upperTeeth", "lowerTeeth"]) {
      const t = outline(name);
      fill(t, CUT.enamel);
      if (t.length > 2) stroke(t, CUT.dentine, 0.06 * k, true);
    }

    // ---- fat and skin along the outline (no fat over the vermilion: the lips are muscle)
    const frontSkin = front.slice(input.standFront);
    stroke(frontSkin, CUT.fat, 2 * 0.55 * k);
    stroke(skin, CUT.fat, 2 * 0.55 * k);
    stroke(frontSkin, CUT.skin, 2 * 0.2 * k);
    stroke(skin, CUT.skin, 2 * 0.2 * k);
    // the stand: one material
    const stand = [...front.slice(0, input.standFront + 1), ...back.slice(back.length - input.standBack - 1)];
    fill(stand, CUT.stand);
    ctx.restore();

    // ---- the air, by the lamina's rule, taken out of the painting
    mctx.setTransform(1, 0, 0, 1, 0, 0);
    mctx.globalCompositeOperation = "source-over";
    mctx.clearRect(0, 0, canvas.width, canvas.height);
    mctx.fillStyle = "#fff";
    const poly = (p: Tract3dPolygon) => {
      path(mctx, p.points);
      mctx.fill("evenodd");
    };
    if (d.air[0]) poly(d.air[0]);
    mctx.globalCompositeOperation = "destination-out";
    d.tissue.forEach(poly);
    mctx.globalCompositeOperation = "source-over";
    d.air.slice(1).forEach(poly);
    // a line of mucosa round the air, then the air itself out
    ctx.save();
    path(ctx, head);
    ctx.clip();
    ctx.shadowColor = CUT.mucosa;
    ctx.shadowBlur = 0.12 * k * S;
    ctx.globalCompositeOperation = "source-atop";
    ctx.drawImage(this.mask, 0, 0);
    ctx.restore();
    ctx.globalCompositeOperation = "destination-out";
    ctx.drawImage(this.mask, 0, 0);
    ctx.globalCompositeOperation = "source-over";
  }
}

/** a polyline moved to its left by d (towards the inside of a head outline that runs face → crown → nape) */
function offsetIn(pts: P[], d: number): P[] {
  const out: P[] = [];
  for (let i = 0; i < pts.length; i++) {
    const a = pts[Math.max(0, i - 2)], b = pts[Math.min(pts.length - 1, i + 2)];
    const tx = b.x - a.x, ty = b.y - a.y, l = Math.hypot(tx, ty) || 1;
    out.push({ x: pts[i].x - (ty / l) * d, y: pts[i].y + (tx / l) * d });
  }
  return out;
}
function offsetBands(pts: P[], ds: number[]): P[][] {
  return ds.map((d) => offsetIn(pts, d));
}

function roundRect(p0: P, p1: P, r: number): P[] {
  const x0 = Math.min(p0.x, p1.x), x1 = Math.max(p0.x, p1.x), y0 = Math.min(p0.y, p1.y), y1 = Math.max(p0.y, p1.y);
  const rr = Math.min(r, (x1 - x0) / 2, (y1 - y0) / 2);
  const o: P[] = [];
  const corner = (cx: number, cy: number, a0: number) => {
    for (let i = 0; i <= 5; i++) {
      const an = a0 + (i / 5) * (Math.PI / 2);
      o.push({ x: cx + Math.cos(an) * rr, y: cy + Math.sin(an) * rr });
    }
  };
  corner(x1 - rr, y1 - rr, 0);
  corner(x0 + rr, y1 - rr, Math.PI / 2);
  corner(x0 + rr, y0 + rr, Math.PI);
  corner(x1 - rr, y0 + rr, (3 * Math.PI) / 2);
  return o;
}

/** a closed Catmull–Rom curve through the points */
function smoothClosed(pts: P[], per = 6): P[] {
  const n = pts.length, out: P[] = [];
  for (let i = 0; i < n; i++) {
    const p0 = pts[(i + n - 1) % n], p1 = pts[i], p2 = pts[(i + 1) % n], p3 = pts[(i + 2) % n];
    for (let s = 0; s < per; s++) {
      const t = s / per, t2 = t * t, t3 = t2 * t;
      out.push({
        x: 0.5 * (2 * p1.x + (p2.x - p0.x) * t + (2 * p0.x - 5 * p1.x + 4 * p2.x - p3.x) * t2 + (3 * p1.x - p0.x - 3 * p2.x + p3.x) * t3),
        y: 0.5 * (2 * p1.y + (p2.y - p0.y) * t + (2 * p0.y - 5 * p1.y + 4 * p2.y - p3.y) * t2 + (3 * p1.y - p0.y - 3 * p2.y + p3.y) * t3),
      });
    }
  }
  return out;
}
