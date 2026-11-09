/**
 * Local-only panels: USC 75-speaker MRI planes (CC BY 4.0, doi:10.1038/s41597-021-00976-x) next
 * to the model's midsagittal air and a coronal cut through the oral cavity, at the same scale.
 * The scanner planes are not registered to the model, and the volumes are not segmented (teeth
 * read as air), so these panels support visual comparison only, never an area error.
 *
 * usage: bun tools/tract3d/usc-compare.ts <chimera binary> <recon dir> <out dir>
 */
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const [binary, input, output] = process.argv.slice(2);
if (!binary || !input || !output) throw new Error("usage: bun tools/tract3d/usc-compare.ts CLI RECON_DIRECTORY OUT_DIRECTORY");
const cli = resolve(binary);
const out = resolve(output);
mkdirSync(out, { recursive: true });
function run(args: string[]) {
  const p = Bun.spawnSync([cli, "tract3d", ...args], { stdout: "pipe", stderr: "pipe" });
  if (p.exitCode !== 0) throw new Error(new TextDecoder().decode(p.stderr));
}
type Frame = { sagittal: { y: number; x: number[] }[]; coronal: { rows: { z: number; intervals: { lo: number; hi: number }[] }[] }; incisor: { x: number; y: number } };
const models = new Map<string, Frame>();
for (const name of ["bunched", "retroflex", "i", "a", "u", "ə", "s", "ʃ", "l"]) {
  const path = join(out, `model-${name}.json`);
  run(["model", name, path]);
  models.set(name, JSON.parse(readFileSync(path, "utf8")));
}
// MRI pixels are 1.5625 mm; everything is drawn at 3 px per pixel (19.2 px/cm)
const PX = 3;
const CM = (10 / 1.5625) * PX;
function sagittal(f: Frame, x0: number, y0: number, w: number, h: number) {
  // the model's upper incisor edge goes where the panel's reference mark is (unregistered)
  const ox = x0 + w * 0.22 - f.incisor.x * CM, oy = y0 + h * 0.42 + f.incisor.y * CM;
  let s = `<rect x="${x0}" y="${y0}" width="${w}" height="${h}" fill="none" stroke="#1d3a4c"/>`;
  for (const r of f.sagittal) for (let j = 0; j + 1 < r.x.length; j += 2) {
    const x = ox + r.x[j] * CM, wd = (r.x[j + 1] - r.x[j]) * CM, y = oy - r.y * CM;
    if (x > x0 && x + wd < x0 + w && y > y0 && y < y0 + h) s += `<rect x="${x.toFixed(1)}" y="${y.toFixed(1)}" width="${wd.toFixed(1)}" height="${(0.05 * CM + 0.3).toFixed(1)}" fill="#ffc573"/>`;
  }
  return s;
}
function coronal(f: Frame, cx: number, cy: number) {
  let s = "";
  for (const r of f.coronal.rows) for (const q of r.intervals) s += `<rect x="${(cx + r.z * CM - 0.05 * CM).toFixed(1)}" y="${(cy - q.hi * CM).toFixed(1)}" width="${(0.1 * CM).toFixed(1)}" height="${((q.hi - q.lo) * CM).toFixed(1)}" fill="#8bdfbc"/>`;
  return s;
}
const symbolOf = (n: string) => (n.includes("_beet_") ? "i" : n.includes("_pot_") ? "a" : n.includes("_boot_") ? "u" : n.includes("_asa_") ? "s" : n.includes("_asha_") ? "ʃ" : n.includes("_ala_") ? "l" : n.includes("_ara_") || n.includes("_bird_") ? "ɹ" : "ə");
let n = 0;
const rows: object[] = [];
for (const file of readdirSync(input).filter((f) => f.endsWith(".mat")).sort()) {
  n++;
  const stem = `${String(n).padStart(2, "0")}-${file.replace(/_recon\.mat$/, "")}`;
  const dir = join(out, stem);
  mkdirSync(dir, { recursive: true });
  run(["usc", join(input, file), dir, "--ap", "48"]);
  const img = (f: string, x: number, y: number, w: number, h: number) => `<image x="${x}" y="${y}" width="${w}" height="${h}" style="image-rendering:pixelated" href="data:image/png;base64,${readFileSync(join(dir, f)).toString("base64")}"/>`;
  const symbol = symbolOf(file);
  const shown = symbol === "ɹ" ? ["bunched", "retroflex"] : [symbol];
  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${1000 + 500 * shown.length}" height="660"><rect width="100%" height="100%" fill="#05090d"/><g fill="#cfe3ef" font-family="sans-serif" font-size="14">`;
  svg += `<text x="16" y="24">${stem} · USC 75-speaker MRI, CC BY 4.0, doi:10.1038/s41597-021-00976-x · model /${symbol}/ (same scale, 1 cm = ${CM.toFixed(1)} px)</text>`;
  svg += `<text x="16" y="52">MRI midsagittal</text><text x="520" y="52">MRI coronal, two depths</text>`;
  svg += img("midsagittal.png", 16, 60, 160 * PX, 160 * PX);
  svg += img("coronal-0.png", 520, 60, 80 * PX, 160 * PX) + img("coronal-1.png", 770, 60, 80 * PX, 160 * PX);
  shown.forEach((name, i) => {
    const x = 1020 + i * 500;
    const f = models.get(name)!;
    svg += `<text x="${x}" y="52">model ${name === "bunched" || name === "retroflex" ? `${name} /ɹ/` : `/${name}/`}: midsagittal air (amber), coronal cut x = 4.5 cm (green)</text>`;
    svg += sagittal(f, x, 60, 480, 330);
    svg += coronal(f, x + 240, 520);
  });
  svg += `<text x="16" y="610">Planes are not registered to the model. Teeth read as air in these unsegmented volumes: no area is measured.</text></g></svg>`;
  writeFileSync(join(out, `${stem}.svg`), svg);
  rows.push({ case: stem, symbol, models: shown, coronalIndices: [48, 60], registered: false });
}
writeFileSync(join(out, "panels.json"), JSON.stringify({ source: "doi:10.1038/s41597-021-00976-x", licence: "CC BY 4.0", cases: rows }, null, 2));
console.log(`wrote ${n} panels`);
