import { test, expect } from "bun:test";
import { initEngine } from "./harness.ts";
await initEngine();
const api = await import("../src/wasm/engine.ts");
const { call } = await import("../src/wasm/chimera.ts");

const MALE = { female: 0, child: 0 };

test("lab geometry: specs, presets, cuts, planes and meshes over the JSON boundary", () => {
  const model = api.airwayModel(MALE, 48000);
  try {
    const before = api.airwayAreas(model, { ...api.NEUTRAL });
    const specs = api.tract3dSpecs();
    expect(specs.length).toBe(40);
    expect(new Set(specs.map((s) => s.key)).size).toBe(specs.length);
    for (const s of specs) expect(s.min <= s.default && s.default <= s.max).toBe(true);
    const presets = api.tract3dPresets(MALE);
    expect(presets.map((p) => p.name)).toContain("retroflex");
    for (const preset of presets) {
      expect(preset.label.length).toBeGreaterThan(0);
      const f = api.tract3d(model, preset.artic, preset.shape, { meshes: false, cut: 0.5 });
      expect(f.slices.length).toBe(f.areaFn.s.length);
      expect(f.areaFn.volume).toBeGreaterThan(0);
      expect(f.slices.every((s) => Number.isFinite(s.area) && s.area >= 0)).toBe(true);
      expect(f.meshes).toBeNull();
      // the selected cut carries its rows; it is the middle one
      expect(f.cut.rows.length).toBe(65);
      expect(f.cut.s).toBeCloseTo(f.slices[Math.round((f.slices.length - 1) * 0.5)].s, 9);
    }
    // the 3D model never touches the voice's own areas
    expect(api.airwayAreas(model, { ...api.NEUTRAL })).toEqual(before);

    const m = api.tract3d(model, { ...api.NEUTRAL }, {}, { meshes: true });
    expect(m.meshes!.map((x) => x.element)).toEqual(["tongue", "palate", "velum", "pharynx", "larynx", "lips", "teethJaw", "cheeks"]);
    for (const mesh of m.meshes!) {
      expect(mesh.positions.length % 3).toBe(0);
      expect(mesh.lines.length % 2).toBe(0);
      expect(mesh.indices.length % 3).toBe(0);
      expect(Math.max(...mesh.lines, ...mesh.indices) * 3).toBeLessThan(mesh.positions.length);
    }
    // the shape in use is derive + offset: a margin offset moves the value in use by as much
    const lowered = api.tract3d(model, { ...api.NEUTRAL }, { sideBlade: -0.5 }, { meshes: false });
    expect(lowered.shape.sideBlade - lowered.derived.sideBlade).toBeCloseTo(-0.5, 9);
    // planes
    const coronal = api.tract3d(model, { ...api.NEUTRAL }, {}, { meshes: false, plane: "coronal", at: 4.5 });
    const axial = api.tract3d(model, { ...api.NEUTRAL }, {}, { meshes: false, plane: "axial", at: 8.3 });
    const sagittal = api.tract3d(model, { ...api.NEUTRAL }, {}, { meshes: false, plane: "sagittal", at: 0 });
    expect(coronal.plane!.kind).toBe("coronal");
    expect(coronal.plane!.kind === "coronal" && coronal.plane!.cut.area).toBeGreaterThan(0);
    expect(axial.plane!.kind === "axial" && axial.plane!.cut.area).toBeGreaterThan(0);
    expect(sagittal.plane!.kind === "sagittal" && sagittal.plane!.rows.length).toBeGreaterThan(100);
    // bad requests are refused with a reason
    expect(() => call("tract3d", { model: model.model, artic: api.NEUTRAL, shape: { unknown: 1 } })).toThrow(/unknown/);
    expect(() => call("tract3d", { model: model.model, artic: {}, shape: {} })).toThrow();
  } finally {
    api.dropAirwayModel(model);
  }
  // a dropped model has no geometry either
  expect(() => api.tract3d(model, { ...api.NEUTRAL }, {}, { meshes: false })).toThrow();
});
