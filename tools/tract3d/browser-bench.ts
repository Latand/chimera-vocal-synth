/**
 * Times the lab's geometry call in the browser, across the whole JSON boundary (encode, wasm,
 * decode, parse), on the served wasm asset. usage: bun tools/tract3d/browser-bench.ts URL OUT.json
 */
import { chromium } from "playwright-core";
import { writeFileSync } from "node:fs";

const [url, out] = process.argv.slice(2);
if (!url || !out) throw new Error("usage: bun tools/tract3d/browser-bench.ts URL OUT.json");
const browser = await chromium.launch({ headless: true });
try {
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  await page.goto(new URL("?mode=lab", url).href, { waitUntil: "networkidle" });
  const result = await page.evaluate(async () => {
    const asset = performance.getEntriesByType("resource").find((r) => r.name.endsWith(".wasm"))?.name;
    if (!asset) throw new Error("the page loaded no wasm asset");
    const ex = new WebAssembly.Instance(await WebAssembly.compile(await (await fetch(asset)).arrayBuffer()), {}).exports;
    const memory = ex.memory as WebAssembly.Memory;
    const fn = (name: string) => ex[name] as (...args: number[]) => number;
    const enc = new TextEncoder(), dec = new TextDecoder();
    let bytes = 0;
    const call = (name: string, args: Record<string, unknown> = {}) => {
      const data = enc.encode(JSON.stringify({ fn: name, ...args }));
      const ptr = fn("chimera_alloc")(data.length);
      new Uint8Array(memory.buffer, ptr, data.length).set(data);
      const status = fn("chimera_call")(ptr, data.length);
      fn("chimera_free")(ptr, data.length);
      bytes = fn("chimera_result_len")();
      const value = JSON.parse(dec.decode(new Uint8Array(memory.buffer, fn("chimera_result_ptr")(), bytes)));
      if (status) throw new Error(value.error);
      return value;
    };
    const catalog = call("catalog");
    const model = call("model", { body: { female: 0, child: 0 }, sampleRate: 48000 });
    const time = (want: Record<string, unknown>) => {
      const args = { model: model.model, artic: catalog.neutral, shape: {}, want };
      call("tract3d", args); // warm
      const samples: number[] = [];
      for (let i = 0; i < 20; i++) {
        const t = performance.now();
        call("tract3d", args);
        samples.push(performance.now() - t);
      }
      samples.sort((a, b) => a - b);
      return { medianMs: samples[10], p95Ms: samples[19], resultBytes: bytes };
    };
    const withMeshes = time({ meshes: true, cut: 0.82 });
    const cutOnly = time({ meshes: false, cut: 0.82 });
    call("dropModel", { model: model.model });
    return { body: "male", calls: 20, withMeshes, cutOnly, targetMs: 16, asset: asset.split("/").pop() };
  });
  writeFileSync(out, JSON.stringify(result, null, 2));
  console.log(JSON.stringify(result));
} finally {
  await browser.close();
}
