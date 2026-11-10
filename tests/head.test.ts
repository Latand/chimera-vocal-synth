/**
 * The head round the 3D tract: the skin mesh (tools/gen-head.ts → head.generated.bin), its
 * place round the engine's tract, and the rig that moves it with the jaw and the lips. The
 * engine's side of it is the `tract3dView` call.
 */
import { test, expect } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { initEngine } from "./harness.ts";
await initEngine();
const api = await import("../src/wasm/engine.ts");
const { buildHead } = await import("../src/ui/render/head/asset.ts");
const { HeadRig, bared } = await import("../src/ui/render/head/rig.ts");

const MALE = { female: 0, child: 0 };
const bytes = readFileSync(join(import.meta.dir, "..", "src", "ui", "render", "head", "head.generated.bin"));
const mesh = buildHead(bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer);
const body = (name: "male" | "female" | "child") => {
  const b = name === "male" ? api.BODY_MALE : name === "female" ? api.BODY_FEMALE : api.BODY_CHILD;
  return { weights: [name === "male" ? 1 : 0, name === "female" ? 1 : 0, name === "child" ? 1 : 0] as [number, number, number], k: b.scale, ph: b.pharynx };
};

test("the head mesh is whole: three bodies, valid triangles, weights in range, an open mouth", () => {
  expect(mesh.n).toBeGreaterThan(15000);
  expect(mesh.bodies.length).toBe(3);
  for (const b of mesh.bodies) {
    expect(b.length).toBe(3 * mesh.n);
    expect(b.every(Number.isFinite)).toBe(true);
  }
  expect(mesh.indices.length % 3).toBe(0);
  let max = 0;
  for (const i of mesh.indices) max = Math.max(max, i);
  expect(max).toBe(mesh.n - 1);
  for (const w of [mesh.jaw, mesh.upper, mesh.lower, mesh.tint, mesh.inside]) {
    expect(w.length).toBe(mesh.n);
    expect(w.every((x) => x >= 0 && x <= 1.0001)).toBe(true);
  }
  // the lips' line has both lips, the rim is a loop inside them, the midline runs both ways
  expect(mesh.lipLine.upper.length).toBeGreaterThan(20);
  expect(mesh.lipLine.lower.length).toBeGreaterThan(20);
  expect(mesh.rim.length).toBe(76);
  expect(mesh.midFront.length).toBeGreaterThan(30);
  expect(mesh.midBack.length).toBeGreaterThan(80);
  // the mouth is open: the rim's edges each belong to one triangle only (a boundary)
  const edges = new Map<string, number>();
  for (let t = 0; t < mesh.indices.length; t += 3) {
    for (let e = 0; e < 3; e++) {
      const a = mesh.indices[t + e], b = mesh.indices[t + ((e + 1) % 3)];
      const key = a < b ? `${a},${b}` : `${b},${a}`;
      edges.set(key, (edges.get(key) ?? 0) + 1);
    }
  }
  const boundary = [...edges.values()].filter((c) => c === 1).length;
  expect(boundary).toBe(mesh.rim.length);
  expect([...edges.values()].every((c) => c <= 2)).toBe(true);
});

test("the head is the same on both sides: ears, eyelids and all (review: left parts were mirrored wrongly)", () => {
  for (const [bi, b] of mesh.bodies.entries()) {
    // a vertex (every 23rd is tried) has its mirror image across the midsagittal plane
    let worst = 0;
    for (let i = 0; i < mesh.n; i += 23) {
      const x = b[3 * i], y = b[3 * i + 1], z = -b[3 * i + 2];
      let best = Infinity;
      for (let j = 0; j < mesh.n; j++) best = Math.min(best, Math.abs(b[3 * j] - x) + Math.abs(b[3 * j + 1] - y) + Math.abs(b[3 * j + 2] - z));
      worst = Math.max(worst, best);
    }
    expect(worst).toBeLessThan(0.03);
    // the ears stand out on both sides, the same way
    let left = 0, right = 0, xl = 0, xr = 0, nl = 0, nr = 0;
    for (let i = 0; i < mesh.n; i++) {
      const z = b[3 * i + 2];
      left = Math.min(left, z);
      right = Math.max(right, z);
      // (the line is off the grid the stored coordinates lie on: a vertex at exactly 7.6 cm was
      // counted on one side and not on the other by the last bit of the subdivision's sums)
      if (z < -7.6037) (xl += b[3 * i]), nl++;
      if (z > 7.6037) (xr += b[3 * i]), nr++;
    }
    expect(right).toBeGreaterThan(7.8);
    expect(left + right).toBeCloseTo(0, 1);
    expect(nl).toBe(nr);
    expect(xl / nl).toBeCloseTo(xr / nr, 1);
    expect(bi).toBeLessThan(3);
  }
});

test("each body's head sits round its own tract: lips on the engine's lips, the larynx inside the neck", () => {
  for (const name of ["male", "female", "child"] as const) {
    const mix = { female: name === "female" ? 1 : 0, child: name === "child" ? 1 : 0 };
    const model = api.airwayModel(mix, 48000);
    try {
      const rig = new HeadRig(mesh, body(name));
      const k = rig.body.k;
      const rest = { ...api.NEUTRAL, lipAp: 0, lipProt: 0, jaw: 0 };
      const view = api.tract3dView(model, rest, {}, true);
      // the skin's lips at rest are where the engine's closed lips are (before any fitting)
      let sx = 0, sy = 0;
      const mid = [...mesh.lipLine.upper].filter((i) => Math.abs(rig.rest[3 * i + 2]) < 0.2 * k);
      for (const i of mid) (sx += rig.rest[3 * i] / mid.length), (sy += rig.rest[3 * i + 1] / mid.length);
      expect(Math.abs(sx - view.display.lips.upper[0].x)).toBeLessThan(0.5 * k);
      expect(Math.abs(sy - view.display.lips.upper[0].y)).toBeLessThan(0.3 * k);
      // the neck's skin on the midline, at the glottis' height: in front of the laryngeal tube with
      // room for cartilage and skin, and behind the spine
      const g = view.display.glottis;
      let front = -Infinity, back = Infinity;
      for (const i of [...mesh.midFront, ...mesh.midBack]) {
        if (mesh.stand[i] || Math.abs(rig.rest[3 * i + 1] - g.y) > 0.8 * k) continue;
        front = Math.max(front, rig.rest[3 * i]);
        back = Math.min(back, rig.rest[3 * i]);
      }
      expect(front - g.x).toBeGreaterThan(1.6 * k);
      expect(front - g.x).toBeLessThan(3.2 * k);
      expect(back - g.x).toBeLessThan(-6 * k);
      // nothing of the tract pokes out of the head's box, and the head stands on its stand below it
      let x0 = Infinity, x1 = -Infinity, y0 = Infinity, y1 = -Infinity, z1 = 0;
      for (let i = 0; i < mesh.n; i++) {
        x0 = Math.min(x0, rig.rest[3 * i]);
        x1 = Math.max(x1, rig.rest[3 * i]);
        y0 = Math.min(y0, rig.rest[3 * i + 1]);
        y1 = Math.max(y1, rig.rest[3 * i + 1]);
        z1 = Math.max(z1, Math.abs(rig.rest[3 * i + 2]));
      }
      for (const m of view.meshes) {
        if (m.element === "lips" || m.element === "cheeks") continue; // open sheets: they end outside the mouth
        for (let i = m.start; i < m.start + m.count; i++) {
          const [x, y, z] = [view.positions[3 * i], view.positions[3 * i + 1], view.positions[3 * i + 2]];
          expect(x > x0 && x < x1 && y > y0 && y < y1 && Math.abs(z) < z1).toBe(true);
        }
      }
    } finally {
      api.dropAirwayModel(model);
    }
  }
});

test("the skin follows the engine's jaw and lips", () => {
  const model = api.airwayModel(MALE, 48000);
  try {
    const rig = new HeadRig(mesh, body("male"));
    const gapAt = (a: typeof api.NEUTRAL) => {
      const view = api.tract3dView(model, a);
      const out = rig.pose(view.display, a.lipWidth);
      const near = (ids: Int32Array) => {
        let y = 0, n = 0;
        for (const i of ids) if (Math.abs(out[3 * i + 2]) < 0.12) (y += out[3 * i + 1]), n++;
        return y / n;
      };
      return { skin: near(mesh.lipLine.upper) - near(mesh.lipLine.lower), engine: view.display.lips.upper[0].y - view.display.lips.lower[0].y, out: Float32Array.from(out), view };
    };
    // shut lips meet; lips open as in speech stand as far apart as the engine's
    const shut = gapAt({ ...api.NEUTRAL, lipAp: -0.2 });
    expect(shut.engine).toBe(0);
    expect(Math.abs(shut.skin)).toBeLessThan(0.05);
    const speech = gapAt({ ...api.NEUTRAL, jaw: 0.8, lipAp: 1.0, lipWidth: 0.8 });
    expect(speech.engine).toBeGreaterThan(0.9);
    expect(speech.engine).toBeLessThan(1.1);
    expect(Math.abs(speech.skin - speech.engine)).toBeLessThan(0.08);
    expect(bared(speech.view.display, 1).upper).toEqual(speech.view.display.lips.upper);
    // opened wider, the lips draw off the teeth: the skin's opening is the engine's and more,
    // by what `bared` says (slice-critic: an /a/ showed no upper teeth and a slit of a mouth)
    const open = gapAt({ ...api.NEUTRAL, jaw: 1.2, lipAp: 1.4, lipWidth: 0.8 });
    expect(open.engine).toBeGreaterThan(1.2);
    const shown = bared(open.view.display, 1);
    expect(shown.upper[0].y).toBeGreaterThan(open.view.display.lips.upper[0].y);
    expect(shown.lower[0].y).toBeLessThan(open.view.display.lips.lower[0].y);
    expect(Math.abs(open.skin - (shown.upper[0].y - shown.lower[0].y))).toBeLessThan(0.08);
    const corner = shown.upper.length - 1;
    expect(shown.upper[corner]).toEqual(open.view.display.lips.upper[corner]);
    // at its widest the upper lip stands above the upper incisors' gum line (their crowns are
    // 1 cm: a strip of gum shows), the lower lip under the lower incisors' crowns
    const wideOpen = api.tract3dView(model, { ...api.NEUTRAL, jaw: 2.6, lipAp: 2.4, lipWidth: 1 }).display;
    const teeth = (name: string) => wideOpen.outlines.find((o) => o.name === name)!.points[2];
    const wide = bared(wideOpen, 1);
    expect(wide.upper[0].y - teeth("upperTeeth").y).toBeGreaterThan(1.05);
    expect(teeth("lowerTeeth").y - wide.lower[0].y).toBeGreaterThan(0.4);
    // inside the lips' line nothing hangs in the opening: every such vertex lies behind its
    // lip's edge and outside the opening's height there
    for (let i = 0; i < mesh.n; i++) {
      if (mesh.inside[i] < 0.2 || Math.abs(open.out[3 * i + 2]) > 0.8) continue;
      const y = open.out[3 * i + 1];
      expect(y >= shown.upper[0].y - 0.12 || y <= shown.lower[0].y + 0.12).toBe(true);
      expect(open.out[3 * i]).toBeLessThan(Math.max(shown.upper[0].x, shown.lower[0].x));
    }
    // every vertex of the lips' line lies on that lip edge for its lateral offset
    const lips = { ...open.view.display.lips, ...shown };
    const edge = (pts: { x: number; y: number }[], az: number) => {
      const step = lips.z[1] - lips.z[0];
      const i = Math.min(lips.z.length - 2, Math.floor(az / step)), t = Math.min(1, az / step - i);
      return { x: pts[i].x + (pts[i + 1].x - pts[i].x) * t, y: pts[i].y + (pts[i + 1].y - pts[i].y) * t };
    };
    // (within a millimetre over the opening; up to 4 mm where the line turns at the corners)
    for (const [ids, pts] of [[mesh.lipLine.upper, lips.upper], [mesh.lipLine.lower, lips.lower]] as const) {
      const off: number[] = [];
      for (const i of ids) {
        const t = edge(pts, Math.abs(open.out[3 * i + 2]));
        off.push(Math.hypot(open.out[3 * i] - t.x, open.out[3 * i + 1] - t.y));
      }
      off.sort((a, b) => a - b);
      expect(off[Math.floor(off.length / 2)]).toBeLessThan(0.1);
      expect(off[off.length - 1]).toBeLessThan(0.4);
    }
    // the chin goes down with the jaw; the crown does not move
    let chin = -1;
    for (const i of mesh.midFront) if (!mesh.stand[i] && rig.rest[3 * i + 1] < 6 && (chin < 0 || rig.rest[3 * i] > rig.rest[3 * chin])) chin = i;
    expect(mesh.jaw[chin]).toBeGreaterThan(0.75);
    const closed = gapAt({ ...api.NEUTRAL, jaw: 0, lipAp: 0 });
    expect(open.out[3 * chin + 1]).toBeLessThan(closed.out[3 * chin + 1] - 0.9);
    const crown = mesh.midBack[Math.floor(mesh.midBack.length / 2)];
    expect(open.out[3 * crown + 1]).toBe(closed.out[3 * crown + 1]);
    // rounding draws the mouth's corners in
    const width = (a: typeof api.NEUTRAL) => {
      const o = gapAt(a).out;
      let w = 0;
      for (const i of mesh.lipLine.upper) w = Math.max(w, Math.abs(o[3 * i + 2]));
      return w;
    };
    expect(width({ ...api.NEUTRAL, lipWidth: 0, lipProt: 1.2, lipAp: 0.4 })).toBeLessThan(0.82 * width({ ...api.NEUTRAL, lipWidth: 1, lipAp: 0.4 }));
    // the same posture twice gives the same skin
    const again = gapAt({ ...api.NEUTRAL, jaw: 1.2, lipAp: 1.4, lipWidth: 0.8 });
    expect(again.out).toEqual(open.out);
  } finally {
    api.dropAirwayModel(model);
  }
});

test("tract3dView: the meshes' positions as floats, topology on request, the same geometry as the lab's call", () => {
  const model = api.airwayModel(MALE, 48000);
  try {
    const a = { ...api.NEUTRAL, jaw: 0.8 };
    const bare = api.tract3dView(model, a);
    const full = api.tract3dView(model, a, {}, true);
    expect(bare.meshes.map((m) => m.element)).toEqual(["tongue", "palate", "velum", "pharynx", "larynx", "lips", "teethJaw", "cheeks"]);
    expect(bare.meshes.every((m) => m.indices === undefined)).toBe(true);
    let total = 0;
    for (const m of full.meshes) {
      expect(m.start).toBe(total);
      total += m.count;
      const covered = m.groups!.reduce((s, g) => s + g.count, 0);
      expect(covered).toBe(m.indices!.length);
      expect(Math.max(...m.indices!, ...m.lines!)).toBeLessThan(m.count);
    }
    expect(full.positions.length).toBe(3 * total);
    expect(full.positions).toEqual(bare.positions);
    expect(full.meshes.find((m) => m.element === "teethJaw")!.groups!.map((g) => g.material)).toEqual(["enamel", "gum"]);
    // one geometry: the lab's JSON call draws the same vertices
    const lab = api.tract3d(model, a, {}, { meshes: true });
    const flat = lab.meshes!.flatMap((m) => m.positions);
    expect(flat.length).toBe(full.positions.length);
    let worst = 0;
    for (let i = 0; i < flat.length; i++) worst = Math.max(worst, Math.abs(flat[i] - full.positions[i]));
    expect(worst).toBeLessThan(1e-5);
    // the section is the midsagittal lamina: its first air polygon is the envelope
    expect(full.display.air[0].name).toBe("envelope");
    expect(full.display.tissue.map((t) => t.name)).toContain("tongue");
    expect(full.display.outlines.map((t) => t.name)).toEqual(["tongue", "velum", "upperTeeth", "lowerTeeth", "jawBone", "nasalCavity"]);
    // a 3D offset reaches the view: rounding narrows the lips' opening
    const round = api.tract3dView(model, a, { lipRound: 1 });
    expect(round.display.lips.halfWidth).toBeCloseTo(0.35 * full.display.lips.halfWidth, 9);
    // and the engine's own rounded lips round the 3D ones without any offset (slice-critic:
    // /u/ was drawn with spread lips)
    const u = api.tract3dView(model, { ...a, lipProt: 1.4, lipWidth: 0.2, lipAp: 0.3 });
    const spread = api.tract3dView(model, { ...a, lipProt: 0, lipWidth: 0.2, lipAp: 0.3 });
    expect(u.display.lips.halfWidth).toBeLessThan(0.6 * spread.display.lips.halfWidth);
    expect(() => api.tract3dView(model, a, { unknown: 1 })).toThrow(/unknown/);
  } finally {
    api.dropAirwayModel(model);
  }
});
