/**
 * One cut through the 3D tract, drawn to scale on a centimetre grid: a perpendicular cut, a
 * coronal or axial plane (rows of air per lamina, joined row to row so the outline is smooth), or
 * a sagittal plane at any lateral offset (rows of air per height, joined the same way, on the
 * same grid: it was the one plane drawn without it, as a stack of bars). The main region is
 * amber, separate side regions blue.
 */
import { useRef } from "react";
import { usePhosphor } from "../../usePhosphor.ts";
import { beam, label, seg } from "../../render/vector.ts";
import { V } from "../../colors.ts";
import type { Tract3dCut, Tract3dPlane } from "../../../wasm/engine.ts";

export type CutDrawing = { kind: "cut"; cut: Tract3dCut; axes: [string, string] } | { kind: "sagittal"; plane: Extract<Tract3dPlane, { kind: "sagittal" }> };

/** Fill and outline a cut's regions; `map` takes (lateral z, along u) in cm to the canvas. */
function drawCut(ctx: CanvasRenderingContext2D, cut: Tract3dCut, map: (z: number, a: number) => [number, number]) {
  const rows = cut.rows;
  const fills = cut.regions.map(() => new Path2D());
  const edges = cut.regions.map(() => new Path2D());
  const overlap = (a: { lo: number; hi: number }, b: { lo: number; hi: number }) => a.lo <= b.hi && b.lo <= a.hi;
  for (let k = 0; k < rows.length; k++) {
    const r = rows[k];
    const dz = k + 1 < rows.length ? rows[k + 1].z - r.z : r.z - rows[k - 1].z;
    for (const q of r.intervals) {
      const f = fills[q.component], e = edges[q.component];
      if (!f) continue;
      const next = rows[k + 1]?.intervals.find((p) => p.component === q.component && overlap(p, q));
      const prev = rows[k - 1]?.intervals.find((p) => p.component === q.component && overlap(p, q));
      const quad = (pts: [number, number][]) => {
        f.moveTo(...pts[0]);
        for (const p of pts.slice(1)) f.lineTo(...p);
        f.closePath();
      };
      if (next) {
        quad([map(r.z, q.lo), map(r.z, q.hi), map(r.z + dz, next.hi), map(r.z + dz, next.lo)]);
        e.moveTo(...map(r.z, q.lo));
        e.lineTo(...map(r.z + dz, next.lo));
        e.moveTo(...map(r.z, q.hi));
        e.lineTo(...map(r.z + dz, next.hi));
      } else {
        quad([map(r.z, q.lo), map(r.z, q.hi), map(r.z + dz / 2, q.hi), map(r.z + dz / 2, q.lo)]);
        e.moveTo(...map(r.z + dz / 2, q.lo));
        e.lineTo(...map(r.z + dz / 2, q.hi));
      }
      if (!prev) {
        quad([map(r.z - dz / 2, q.lo), map(r.z - dz / 2, q.hi), map(r.z, q.hi), map(r.z, q.lo)]);
        e.moveTo(...map(r.z - dz / 2, q.lo));
        e.lineTo(...map(r.z - dz / 2, q.hi));
      }
    }
  }
  cut.regions.forEach((reg, i) => {
    const color = reg.tag === "main" ? V.amber : V.blue;
    ctx.save();
    ctx.globalAlpha = 0.28;
    ctx.fillStyle = color;
    ctx.fill(fills[i], "nonzero");
    ctx.restore();
    beam(ctx, edges[i], color, 1.4, { hot: reg.tag === "main" });
  });
}

export function LabCutView({ drawing, size }: { drawing: CutDrawing | null; size: number }) {
  const d = useRef(drawing);
  d.current = drawing;
  const span = useRef(size);
  span.current = size;
  const ref = usePhosphor(({ ctx, w, h }) => {
    const dr = d.current;
    if (!dr) return;
    // a fixed scale for each kind of view, so the shape's size reads directly; a cut that would
    // leave the screen at that scale (a whole coronal or axial plane: its origin is not its
    // middle) is centred on its own extent and, if still too large, drawn smaller. The scale bar
    // always shows the scale in use.
    let sc = Math.min(w, h * 1.4) / span.current;
    const cx = w / 2;
    let cy = h / 2;
    if (dr.kind === "cut") {
      let lo = Infinity, hi = -Infinity, wide = 0;
      for (const r of dr.cut.rows) {
        for (const q of r.intervals) {
          lo = Math.min(lo, q.lo);
          hi = Math.max(hi, q.hi);
          wide = Math.max(wide, Math.abs(r.z) + 0.1);
        }
      }
      // (the fitted cut stays clear of the axis label above it and of the scale bar below: a
      // plane that reached the top edge lay over its own "↑ up" label)
      const top = 24, bottom = 30;
      if (Number.isFinite(lo) && (hi * sc > h / 2 - top || -lo * sc > h / 2 - bottom || wide * sc > w / 2 - 12)) {
        sc = Math.min(sc, (w - 24) / (2 * wide), (h - top - bottom) / Math.max(1e-6, hi - lo));
        cy = (top + h - bottom) / 2 + ((lo + hi) / 2) * sc;
      }
    }
    // grid: 1 cm from the cut's origin (cuts); a sagittal plane is fitted to the screen instead
    const nx = Math.ceil(w / sc / 2) + 1, ny = Math.ceil(h / sc) + 1;
    if (dr.kind === "cut") {
      for (let i = -nx; i <= nx; i++) seg(ctx, cx + i * sc, 0, cx + i * sc, h, V.frame, i === 0 ? 1 : 0.6, { alpha: i === 0 ? 1 : 0.7 });
      for (let i = -ny; i <= ny; i++) seg(ctx, 0, cy + i * sc, w, cy + i * sc, V.frame, i === 0 ? 1 : 0.6, { alpha: i === 0 ? 1 : 0.7 });
    }
    let bar = sc;
    if (dr.kind === "cut") {
      drawCut(ctx, dr.cut, (z, a) => [cx + z * sc, cy - a * sc]);
      label(ctx, dr.axes[1], cx + 4, 14, { size: 9, color: V.blueMid, ls: 0.12 });
      label(ctx, dr.axes[0], w - 6, Math.min(h - 26, Math.max(14, cy - 6)), { size: 9, color: V.blueMid, align: "right", ls: 0.12 });
    } else {
      const rows = dr.plane.rows;
      if (rows.length) {
        // a sagittal plane spans the whole tract: fit it, glottis low left of centre
        const xs = rows.flatMap((r) => r.x);
        const x0 = Math.min(...xs), x1 = Math.max(...xs);
        const y0 = rows[0].y, y1 = rows[rows.length - 1].y;
        const s2 = Math.min((w - 24) / (x1 - x0), (h - 24) / (y1 - y0 + dr.plane.step));
        const ox = (w - (x1 - x0) * s2) / 2 - x0 * s2, oy = h - 12 + y0 * s2;
        bar = s2;
        // the grid: 1 cm, from the body's own origin (the glottis' height at rest, x = 0)
        for (let i = Math.floor(-ox / s2); i <= Math.ceil((w - ox) / s2); i++) seg(ctx, ox + i * s2, 0, ox + i * s2, h, V.frame, i === 0 ? 1 : 0.6, { alpha: i === 0 ? 1 : 0.7 });
        for (let i = Math.floor((oy - h) / s2); i <= Math.ceil(oy / s2); i++) seg(ctx, 0, oy - i * s2, w, oy - i * s2, V.frame, i === 0 ? 1 : 0.6, { alpha: i === 0 ? 1 : 0.7 });
        // each stretch of air joins the stretch it overlaps in the row above it
        const at = (x: number, y: number): [number, number] => [ox + x * s2, oy - y * s2];
        const fill = new Path2D(), edge = new Path2D();
        const half = dr.plane.step / 2;
        const quad = (pts: [number, number][]) => {
          fill.moveTo(...pts[0]);
          for (const q of pts.slice(1)) fill.lineTo(...q);
          fill.closePath();
        };
        const pairs = (r: { x: number[] }) => {
          const out: [number, number][] = [];
          for (let j = 0; j + 1 < r.x.length; j += 2) out.push([r.x[j], r.x[j + 1]]);
          return out;
        };
        rows.forEach((r, k) => {
          const above = rows[k + 1] ? pairs(rows[k + 1]) : [], below = rows[k - 1] ? pairs(rows[k - 1]) : [];
          for (const [lo, hi] of pairs(r)) {
            const next = above.find(([a, b]) => a <= hi && lo <= b);
            if (next) {
              quad([at(lo, r.y), at(hi, r.y), at(next[1], rows[k + 1].y), at(next[0], rows[k + 1].y)]);
              edge.moveTo(...at(lo, r.y));
              edge.lineTo(...at(next[0], rows[k + 1].y));
              edge.moveTo(...at(hi, r.y));
              edge.lineTo(...at(next[1], rows[k + 1].y));
            } else {
              quad([at(lo, r.y), at(hi, r.y), at(hi, r.y + half), at(lo, r.y + half)]);
              edge.moveTo(...at(lo, r.y + half));
              edge.lineTo(...at(hi, r.y + half));
            }
            if (!below.some(([a, b]) => a <= hi && lo <= b)) {
              quad([at(lo, r.y - half), at(hi, r.y - half), at(hi, r.y), at(lo, r.y)]);
              edge.moveTo(...at(lo, r.y - half));
              edge.lineTo(...at(hi, r.y - half));
            }
          }
        });
        ctx.save();
        ctx.globalAlpha = 0.28;
        ctx.fillStyle = V.amber;
        ctx.fill(fill, "nonzero");
        ctx.restore();
        beam(ctx, edge, V.amber, 1.4, { hot: true });
        label(ctx, "↑ up", ox + 4 > 60 ? 8 : ox + 4, 14, { size: 9, color: V.blueMid, ls: 0.12 });
        label(ctx, "front →", w - 6, h - 6, { size: 9, color: V.blueMid, align: "right", ls: 0.12 });
      }
    }
    label(ctx, "1 cm", 8, h - 8, { size: 9, color: V.blueMid, ls: 0.1 });
    seg(ctx, 8, h - 20, 8 + bar, h - 20, V.blue, 1.2);
  });
  return <canvas ref={ref} className="lab-cut" role="img" aria-label="Cross-section of the airway, to scale on a 1 cm grid" />;
}
