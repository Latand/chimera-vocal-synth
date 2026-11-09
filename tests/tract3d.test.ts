import { test, expect } from "bun:test";
import { initEngine } from "./harness.ts";
await initEngine();
const api = await import("../src/wasm/engine.ts");
const { call } = await import("../src/wasm/chimera.ts");
test("lab geometry shares body mapping and leaves existing model areas unchanged", () => {
  const model = api.airwayModel({ female: 0, child: 0 }, 48000);
  try {
    const before = api.airwayAreas(model, { ...api.NEUTRAL });
    const specs = api.tract3dSpecs();
    expect(specs.length).toBe(41);
    expect(new Set(specs.map((s) => s.key)).size).toBe(specs.length);
    const presets = api.tract3dPresets({ female: 0, child: 0 });
    for (const preset of presets) {
      const f = api.tract3d(model, preset.artic, preset.shape, {
        meshes: false,
      });
      expect(f.slices.length).toBe(f.areaFn.s.length);
      expect(f.areaFn.volume).toBeGreaterThan(0);
      expect(f.slices.every((s) => Number.isFinite(s.area))).toBe(true);
    }
    expect(api.airwayAreas(model, { ...api.NEUTRAL })).toEqual(before);
    const m = api.tract3d(model, { ...api.NEUTRAL }, {}, { meshes: true });
    const plane = api.tract3d(model, { ...api.NEUTRAL }, {}, { meshes: false, x: 4, y: 4 });
    expect(plane.coronal.area).toBeGreaterThan(0);
    expect(plane.axial.area).toBeGreaterThan(0);
    expect(m.meshes.map((m) => m.element)).toEqual([
      "tongue",
      "palate",
      "velum",
      "pharynx",
      "larynx",
      "lips",
      "teethJaw",
      "cheeks",
    ]);
    expect(() =>
      call("tract3d", {
        model: model.model,
        artic: api.NEUTRAL,
        shape: { unknown: 1 },
      }),
    ).toThrow();
    expect(() =>
      call("tract3d", { model: model.model, artic: {}, shape: {} }),
    ).toThrow();
  } finally {
    api.dropAirwayModel(model);
  }
  expect(() =>
    api.tract3d(model, { ...api.NEUTRAL }, {}, { meshes: false }),
  ).toThrow();
});
