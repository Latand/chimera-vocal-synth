/**
 * Area along the centreline: the 3D model's (amber) against what the engine's voice uses now
 * (its 1D area law, blue), with cuts of several separate channels marked and the current cut in
 * pink. Clicking or dragging on the plot moves the cut.
 */
import { useRef } from "react";
import { usePhosphor } from "../../usePhosphor.ts";
import { beamPts, dot, label, seg, type Pt } from "../../render/vector.ts";
import { V } from "../../colors.ts";
import type { Tract3dFrame } from "../../../wasm/engine.ts";

const PAD = { left: 34, right: 12, top: 14, bottom: 22 };

export function LabAreaView(props: { frame: Tract3dFrame; engine: { s: number[]; area: number[] }; cut: number; onCut: (fraction: number) => void }) {
  const p = useRef(props);
  p.current = props;
  const ref = usePhosphor(({ ctx, w, h }) => {
    const { frame, engine, cut } = p.current;
    const a = frame.areaFn;
    const max = Math.max(2, ...a.area, ...engine.area) * 1.05;
    const left = PAD.left, right = w - PAD.right, top = PAD.top, bot = h - PAD.bottom;
    const L = Math.max(a.length, engine.s[engine.s.length - 1] ?? 0);
    const x = (s: number) => left + (s / L) * (right - left);
    const y = (v: number) => bot - (v / max) * (bot - top);
    for (let v = 2; v < max; v += 2) {
      seg(ctx, left, y(v), right, y(v), V.frame, 0.8, { dash: [2, 4] });
      label(ctx, String(v), left - 6, y(v) + 3, { size: 8, color: V.blueDim, align: "right", ls: 0.05 });
    }
    for (let s = 2; s < L; s += 2) seg(ctx, x(s), bot, x(s), bot + 3, V.blueDim, 1);
    seg(ctx, left, bot, right, bot, V.blueDim, 1);
    seg(ctx, left, top, left, bot, V.blueDim, 1);
    label(ctx, "GLOTTIS", left, h - 6, { size: 8, color: V.blueMid, ls: 0.14 });
    label(ctx, "LIPS", right, h - 6, { size: 8, color: V.blueMid, align: "right", ls: 0.14 });
    label(ctx, "CM²", left - 6, top + 2, { size: 8, color: V.blueMid, align: "right", ls: 0.1 });
    const eng: Pt[] = engine.s.map((s, i) => [x(s), y(engine.area[i])]);
    beamPts(ctx, eng, V.blueMid, 1.2);
    const pts: Pt[] = a.s.map((s, i) => [x(s), y(a.area[i])]);
    beamPts(ctx, pts, V.amber, 1.6, { hot: true });
    a.channels.forEach((c, i) => {
      if (c > 1) dot(ctx, x(a.s[i]), y(a.area[i]), 2.4, V.blue);
    });
    const sx = x(cut * a.length);
    seg(ctx, sx, top, sx, bot, V.pink, 1.2);
  });
  const drag = useRef(false);
  const pick = (e: React.PointerEvent<HTMLCanvasElement>) => {
    const r = e.currentTarget.getBoundingClientRect();
    const { frame, engine } = p.current;
    const L = Math.max(frame.areaFn.length, engine.s[engine.s.length - 1] ?? 0);
    const s = ((e.clientX - r.left - PAD.left) / (r.width - PAD.left - PAD.right)) * L;
    p.current.onCut(Math.min(1, Math.max(0, s / frame.areaFn.length)));
  };
  return (
    <canvas
      ref={ref}
      className="lab-area"
      role="img"
      aria-label="Cross-sectional area along the tract: the 3D model in amber, the engine's current area function in blue. Click to move the cut."
      onPointerDown={(e) => {
        drag.current = true;
        e.currentTarget.setPointerCapture(e.pointerId);
        pick(e);
      }}
      onPointerMove={(e) => drag.current && pick(e)}
      onPointerUp={() => (drag.current = false)}
      onPointerCancel={() => (drag.current = false)}
    />
  );
}
