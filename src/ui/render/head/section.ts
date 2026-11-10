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
  brainFold: "#b98d8a",
  white: "#efe3d8",
  cord: "#ecdcae",
  tongue: "#b5484e",
  fibre: "#c8656b",
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
      // the brain's medial face. Its parts are placed in the box of the cranial cavity (u from
      // the occiput to the forehead, v from the skull's base to the vault), so they follow the
      // body's own skull.
      let bx0 = Infinity, bx1 = -Infinity, by0 = Infinity, by1 = -Infinity;
      for (const p of brain) {
        bx0 = Math.min(bx0, p.x);
        bx1 = Math.max(bx1, p.x);
        by0 = Math.min(by0, p.y);
        by1 = Math.max(by1, p.y);
      }
      const W = bx1 - bx0, H = by1 - by0;
      const Q = (u: number, v: number): P => ({ x: bx0 + u * W, y: by0 + v * H });
      const line = (pts: [number, number][], color: string, cm: number) => stroke(smoothOpen(pts.map(([u, v]) => Q(u, v))), color, cm);
      const blob = (pts: [number, number][], color: string) => fill(smoothClosed(pts.map(([u, v]) => Q(u, v))), color);
      // the cortex's ribbon follows the vault: sulci run in from the surface, each its own depth
      // and lean (a fixed sequence: the drawing is the same every frame)
      const rim = offsetIn(vault, 0.12 * k);
      let run = 0;
      for (let q = 3; q + 3 < rim.length; q++) {
        run += Math.hypot(rim[q].x - rim[q - 1].x, rim[q].y - rim[q - 1].y);
        if (run < 1.15 * k) continue;
        run = 0;
        const a = rim[q - 2], b = rim[q + 2];
        const tx = b.x - a.x, ty = b.y - a.y, l = Math.hypot(tx, ty) || 1;
        const h = Math.abs(Math.sin(q * 12.9898) * 43758.5453) % 1;
        const depth = (0.55 + 0.75 * h) * k, lean = (h - 0.5) * 0.9 * k;
        const p0 = rim[q], p2 = { x: p0.x - (ty / l) * depth + (tx / l) * lean, y: p0.y + (tx / l) * depth + (ty / l) * lean };
        const p1 = { x: (p0.x + p2.x) / 2 + (tx / l) * 0.3 * lean, y: (p0.y + p2.y) / 2 + (ty / l) * 0.3 * lean };
        stroke(smoothOpen([p0, p1, p2]), CUT.brainFold, 0.07 * k);
      }
      // the corpus callosum, the fornix under it, the cingulate sulcus over it
      line([[0.3, 0.5], [0.33, 0.6], [0.45, 0.66], [0.58, 0.64], [0.66, 0.55], [0.64, 0.47]], CUT.white, 0.5 * k);
      line([[0.34, 0.5], [0.42, 0.55], [0.52, 0.52], [0.56, 0.44]], CUT.white, 0.16 * k);
      line([[0.22, 0.52], [0.28, 0.69], [0.45, 0.78], [0.62, 0.75], [0.74, 0.6], [0.73, 0.44]], CUT.brainFold, 0.07 * k);
      // the parieto-occipital and calcarine sulci of the occipital lobe
      line([[0.2, 0.8], [0.25, 0.66], [0.27, 0.54]], CUT.brainFold, 0.07 * k);
      line([[0.06, 0.52], [0.16, 0.5], [0.27, 0.54]], CUT.brainFold, 0.07 * k);
      // the thalamus, the midbrain, the pons and the medulla down to the foramen magnum
      blob([[0.44, 0.5], [0.51, 0.52], [0.55, 0.47], [0.51, 0.41], [0.44, 0.42]], CUT.white);
      const stem = [B(-3.3, 15.2), B(-3.5, 14.3), B(-3.2, 13.5), B(-3.6, 12.6), B(-4.2, 11.9), B(-4.6, 11.2)];
      stroke(smoothOpen(stem), CUT.cord, 1.25 * k);
      fill(smoothClosed([B(-3.3, 14.1), B(-2.4, 13.7), B(-2.3, 12.9), B(-3.1, 12.5), B(-3.7, 13.1)]), CUT.cord);
      // the cerebellum: its white tree and the folia round it
      // (it lies in the back of the skull's base, under the cerebrum)
      const cbx = B(-8.3, 0).x;
      const cb = { x: cbx, y: baseAt(cbx) + 1.95 * k };
      const leaf = (a: number, r: number): P => ({ x: cb.x + Math.cos(a) * r * 2.6 * k, y: cb.y + Math.sin(a) * r * 1.75 * k });
      fill(smoothClosed([0, 1, 2, 3, 4, 5, 6, 7, 8, 9].map((q) => leaf((q / 10) * 2 * Math.PI, 1))), "#d3b0ab");
      const root = leaf(0.15, 0.82);
      for (let q = 0; q < 9; q++) {
        const a = 0.75 + (q / 8) * 4.2;
        const tip = leaf(a, 0.9), mid = leaf(a - 0.25, 0.42);
        stroke(smoothOpen([root, mid, tip]), CUT.white, (0.2 - 0.008 * q) * k);
        for (const side of [-1, 1]) stroke([leaf(a + side * 0.09, 0.66), leaf(a + side * 0.2, 0.86)], CUT.white, 0.08 * k);
      }
      for (let q = 0; q < 16; q++) {
        const a = 0.55 + (q / 15) * 4.6;
        stroke([leaf(a, 1.0), leaf(a + 0.03, 0.8)], CUT.brainFold, 0.06 * k);
      }
      ctx.restore();
    }
    // cervical spine, on the curve of the neck: the cord in its canal, each vertebra's body with
    // its cortex and marrow, the discs between, the arches and spinous processes behind; the
    // axis carries the dens, with the atlas' two arches round it
    const wallX = (y: number) => -1.2 - 0.025 * y;
    const lordosis = (y: number) => 0.35 * Math.sin(((y + 6) / 17) * Math.PI);
    const bodyX = (y: number) => wallX(y) - 2.2 + lordosis(y);
    stroke(smoothOpen([T(bodyX(-7.5) - 0.75, -7.5), T(bodyX(0) - 0.75, 0), T(bodyX(6) - 0.75, 6), T(bodyX(10.4) - 0.7, 10.4), B(-4.6, 11.3)]), CUT.cord, 0.85 * k);
    for (let i = 0; i < 9; i++) {
      const y0 = -6.15 + i * 1.75, top = i === 8;
      const x0 = bodyX(y0 + 0.7);
      // (a body is waisted: its front and back are hollow, its end plates slightly domed)
      const h = top ? 1.55 : 1.38;
      const bodyPts: [number, number][] = [[0.04, 0], [0.8, -0.04], [1.56, 0.02], [1.5, 0.5 * h], [1.6, h], [0.8, h + 0.05], [0.02, h - 0.02], [0.12, 0.5 * h]];
      fill(smoothClosed(bodyPts.map(([dx, dy]) => T(x0 + dx, y0 + dy))), CUT.bone);
      fill(smoothClosed(bodyPts.map(([dx, dy]) => T(x0 + 0.8 + (dx - 0.8) * 0.78, y0 + h / 2 + (dy - h / 2) * 0.76))), CUT.marrow);
      if (!top) fill(smoothClosed([[0.06, h + 0.06], [0.8, h + 0.12], [1.56, h + 0.08], [1.58, h + 0.32], [0.8, h + 0.3], [0.04, h + 0.34]].map(([dx, dy]) => T(x0 + dx, y0 + dy))), CUT.cartilage);
      // the arch behind the canal and its spinous process, longer at the bottom of the neck
      const long = i <= 2 ? 1.25 : i === 7 ? 1.15 : 0.9;
      fill(smoothClosed([[-1.5, 1.05], [-1.9, 1.12], [-2.2 - long, 0.5], [-2.3 - long, 0.18], [-1.95, 0.42], [-1.55, 0.5]].map(([dx, dy]) => T(x0 + dx, y0 + dy))), CUT.bone);
      if (top) {
        // the dens, and the atlas' anterior and posterior arches
        fill(smoothClosed([[0.5, h - 0.1], [0.42, h + 0.75], [0.72, h + 1.35], [1.06, h + 0.8], [1.02, h - 0.1]].map(([dx, dy]) => T(x0 + dx, y0 + dy))), CUT.bone);
        fill(smoothClosed([[1.3, h + 0.55], [1.25, h + 1.0], [1.55, h + 1.12], [1.72, h + 0.78], [1.58, h + 0.5]].map(([dx, dy]) => T(x0 + dx, y0 + dy))), CUT.bone);
        fill(smoothClosed([[-1.6, h + 0.9], [-2.3, h + 1.0], [-2.75, h + 0.75], [-2.3, h + 0.55], [-1.65, h + 0.6]].map(([dx, dy]) => T(x0 + dx, y0 + dy))), CUT.bone);
      }
    }
    // nasal cavity with the turbinates, the hard palate under it
    const nasal = outline("nasalCavity");
    const nasalRoof = [B(0.3, 11.2), B(1.1, 10.25), B(2.7, 10.75), B(4.6, 10.85), B(6.4, 10.65), B(7.6, 10.85), B(8.3, 11.3), B(7.9, 12.3), B(6.7, 13.4), B(4.4, 13.9), B(1.8, 13.6), B(0.2, 12.6)];
    fill(smoothClosed(nasalRoof), CUT.hollow);
    // the sphenoid sinus behind the nasal cavity and the frontal sinus over its root
    fill(smoothClosed([B(-0.3, 13.3), B(0.9, 13.9), B(2.0, 13.6), B(1.9, 12.7), B(0.6, 12.4), B(-0.3, 12.7)]), CUT.hollow);
    fill(smoothClosed([B(5.9, 15.4), B(6.5, 16.3), B(6.2, 17.2), B(5.6, 16.6), B(5.5, 15.7)]), CUT.hollow);
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
    // the hyoid's body, the fat pad between it and the epiglottis, the thyroid cartilage's
    // front, the cricoid's arch in front of the airway and its plate behind with an arytenoid on
    // it, and the trachea running down and a little back with its rings in its front wall and
    // the gullet behind it
    const lar = 0.6 * input.larynx * k;
    const hy = { x: T(1.7, 0).x, y: T(0, 3.1).y + lar };
    const G = (dx: number, dy: number): P => ({ x: g.x + dx * k, y: g.y + dy * k });
    fill(smoothClosed([{ x: hy.x - 0.4 * k, y: hy.y - 0.5 * k }, G(1.25, 1.95), G(0.75, 2.0), G(0.5, 2.5), { x: hy.x - 0.75 * k, y: hy.y + 0.1 * k }]), CUT.fat);
    fill(smoothClosed([{ x: hy.x - 0.3 * k, y: hy.y + 0.3 * k }, { x: hy.x + 0.25 * k, y: hy.y + 0.2 * k }, { x: hy.x + 0.4 * k, y: hy.y - 0.15 * k }, { x: hy.x, y: hy.y - 0.32 * k }, { x: hy.x - 0.4 * k, y: hy.y - 0.1 * k }]), CUT.bone);
    // the airway below the glottis: the cone under the folds, then the trachea
    const lean = (dy: number) => 0.045 * dy;
    const depth = bounds.y0 / k - g.y / k;
    const ahead: P[] = [], behind: P[] = [];
    for (let q = 0; q <= 12; q++) {
      const dy = (depth * q) / 12;
      const wide = 0.3 + 0.48 * Math.min(1, -dy / 1.4) ** 0.7;
      ahead.push(G(0.12 + wide + lean(dy), dy));
      behind.push(G(0.12 - wide + lean(dy), dy));
    }
    fill([...ahead, ...behind.reverse()], CUT.hollow);
    behind.reverse();
    stroke(smoothOpen(behind.map((p) => ({ x: p.x - 0.42 * k, y: p.y })).slice(2)), CUT.hollow, 0.09 * k);
    // the thyroid cartilage: from its notch down the front of the larynx
    fill(smoothClosed([G(1.0, 1.75), G(1.42, 1.5), G(1.7, 0.6), G(1.62, -0.35), G(1.3, -0.55), G(1.32, 0.3), G(1.2, 1.1)]), CUT.cartilage);
    // the cricoid: its arch in front, its tall plate behind, an arytenoid on the plate
    fill(smoothClosed([G(1.12, -0.75), G(1.5, -0.85), G(1.55, -1.3), G(1.2, -1.4), G(1.0, -1.1)]), CUT.cartilage);
    fill(smoothClosed([G(-0.82, 0.55), G(-0.5, 0.2), G(-0.55, -1.2), G(-0.85, -1.5), G(-1.08, -0.9), G(-1.05, 0.1)]), CUT.cartilage);
    fill(smoothClosed([G(-0.8, 0.6), G(-0.48, 1.15), G(-0.42, 0.6), G(-0.55, 0.35)]), CUT.cartilage);
    for (let q = 0; q < 6; q++) {
      const dy = -1.95 - q * 0.95;
      if (g.y + (dy - 0.6) * k < bounds.y0) break;
      const c = G(1.12 + lean(dy), dy - 0.25);
      ctx.beginPath();
      ctx.ellipse(X(c.x), Y(c.y), 0.19 * k * S, 0.3 * k * S, 0, 0, Math.PI * 2);
      ctx.fillStyle = CUT.cartilage;
      ctx.fill();
    }
    // the vocal folds at the glottis, the ventricular folds over them
    fill(smoothClosed([G(-0.62, 0.02), G(-0.1, 0.13), G(-0.6, 0.32)]), CUT.mucosa);
    fill(smoothClosed([G(0.72, 0.02), G(0.12, 0.13), G(0.7, 0.32)]), CUT.mucosa);

    // ---- the tract's own parts on the midline (the engine's outlines)
    const tongue = outline("tongue");
    if (tongue.length > 2) {
      // the muscles under the tongue from the chin's inside back to the hyoid
      fill(smoothClosed([J(6.0, 5.2), J(4.6, 5.0), { x: hy.x + 0.3 * k, y: hy.y + 0.25 * k }, { x: hy.x + 0.35 * k, y: hy.y - 0.3 * k }, J(4.4, 4.45), J(6.1, 4.75)]), CUT.muscle);
      fill(tongue, CUT.tongue);
      // a muscle mass: the genioglossus' fibres curve up from the chin's inside into the body,
      // faintly, and the fibres that run along the tongue lie in a band under its surface
      const origin = J(6.0, 5.5);
      ctx.save();
      path(ctx, tongue);
      ctx.clip();
      ctx.globalAlpha = 0.34;
      const dorsal = tongue.slice(2, tongue.length - 8);
      for (let i = 1; i < dorsal.length; i += 2) {
        const end = dorsal[i];
        const mx = (origin.x + end.x) / 2, my = (origin.y + end.y) / 2;
        const dx = end.x - origin.x, dy = end.y - origin.y, l = Math.hypot(dx, dy) || 1;
        // (each fibre bows towards the tongue's back, more the longer it is; it stops short of the surface)
        const bow = 0.12 * l;
        const stop = { x: end.x - (dx / l) * 0.45 * k, y: end.y - (dy / l) * 0.45 * k };
        stroke(smoothOpen([origin, { x: mx - (dy / l) * bow, y: my + (dx / l) * bow }, stop]), CUT.fibre, 0.045 * k);
      }
      ctx.globalAlpha = 0.5;
      stroke(offsetIn(dorsal, -0.28 * k), CUT.fibre, 0.2 * k);
      ctx.globalAlpha = 1;
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
/** an open Catmull–Rom curve through the points */
function smoothOpen(pts: P[], per = 8): P[] {
  const n = pts.length, out: P[] = [];
  if (n < 3) return pts;
  for (let i = 0; i + 1 < n; i++) {
    const p0 = pts[Math.max(0, i - 1)], p1 = pts[i], p2 = pts[i + 1], p3 = pts[Math.min(n - 1, i + 2)];
    for (let s = 0; s < per; s++) {
      const t = s / per, t2 = t * t, t3 = t2 * t;
      out.push({
        x: 0.5 * (2 * p1.x + (p2.x - p0.x) * t + (2 * p0.x - 5 * p1.x + 4 * p2.x - p3.x) * t2 + (3 * p1.x - p0.x - 3 * p2.x + p3.x) * t3),
        y: 0.5 * (2 * p1.y + (p2.y - p0.y) * t + (2 * p0.y - 5 * p1.y + 4 * p2.y - p3.y) * t2 + (3 * p1.y - p0.y - 3 * p2.y + p3.y) * t3),
      });
    }
  }
  out.push(pts[n - 1]);
  return out;
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
