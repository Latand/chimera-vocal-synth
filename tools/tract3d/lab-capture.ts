/**
 * Numbered frames of the lab's 3D view for before/after evidence: the same postures, cameras and
 * view options each run; options a build lacks are skipped (and said so in the log).
 * Usage: bun tools/tract3d/lab-capture.ts URL OUT_DIR [width height]
 */
import { chromium, type Page } from "playwright-core";
import { mkdirSync, writeFileSync } from "node:fs";

const [url, out, w = "1440", h = "1000"] = process.argv.slice(2);
if (!url || !out) throw new Error("usage: bun tools/tract3d/lab-capture.ts URL OUT_DIR [width height]");
mkdirSync(out, { recursive: true });
const log: string[] = [];
const browser = await chromium.launch({ headless: true });
let n = 0;
try {
  const page = await browser.newPage({ viewport: { width: Number(w), height: Number(h) }, deviceScaleFactor: 1 });
  page.on("pageerror", (e) => log.push(`pageerror: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && log.push(`console: ${m.text()}`));
  await page.goto(new URL("?mode=lab", url).toString(), { waitUntil: "networkidle" });
  await page.getByRole("heading", { name: "Tract", exact: true }).waitFor();
  const click = async (name: string | RegExp, scope?: string): Promise<boolean> => {
    const root = scope ? page.getByRole("group", { name: scope }) : page;
    const b = root.getByRole("button", { name, exact: typeof name === "string" }).first();
    if ((await b.count()) === 0) {
      log.push(`skipped: no button ${name}`);
      return false;
    }
    await b.click();
    return true;
  };
  const toggle = async (name: string, on: boolean) => {
    const b = page.getByRole("button", { name, exact: true }).first();
    if ((await b.count()) === 0) return log.push(`skipped: no toggle ${name}`), false;
    if ((await b.getAttribute("aria-pressed")) !== String(on)) await b.click();
    return true;
  };
  const shot = async (label: string, zoom = 0) => {
    const stage = page.locator(".lab-stage");
    if (zoom) {
      const box = (await stage.boundingBox())!;
      await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
      for (let i = 0; i < Math.abs(zoom); i++) await page.mouse.wheel(0, zoom > 0 ? -120 : 120);
    }
    await page.waitForTimeout(900);
    const file = `${String(++n).padStart(2, "0")}-${label}.png`;
    // a clip of the page (an element shot waits for a layout that the live canvas never settles)
    const box = (await stage.boundingBox())!;
    await page.screenshot({ path: `${out}/${file}`, clip: box });
    log.push(`${file}`);
  };
  const view = (v: string) => click(`View from the ${v}`);
  for (const [posture, tag] of [["open vowel /a/", "a"], [/\/i\//, "i"], [/\/u\//, "u"]] as const) {
    await click(posture, "Postures");
    await view("front and side");
    await shot(`${tag}-three`);
  }
  await click("open vowel /a/", "Postures");
  await view("side");
  await shot("a-side");
  await view("front");
  await shot("a-front");
  // the head: whole, cut open (its layers' cut faces), and see-through (absent before the head existed)
  if (await toggle("Head", true)) {
    await toggle("Cut away", false);
    await view("front and side");
    await shot("head-three");
    await view("side");
    await shot("head-side");
    await view("front");
    await shot("head-front");
    await toggle("Cut away", true);
    await view("side");
    await shot("head-layers-side");
    await view("front and side");
    await shot("head-layers-three");
    await shot("head-layers-close", 5);
    await toggle("Cut away", false);
    if (await toggle("See-through", true)) {
      await view("front and side");
      await shot("head-see-through");
      await toggle("See-through", false);
    }
    await toggle("Head", false);
  }
  // the inside: cut away, then the soft palate with the uvula and the larynx with the epiglottis
  await toggle("Cut away", true);
  await view("side");
  await shot("a-cutaway-side");
  await toggle("Cut away", false);
  for (const [el, tag] of [["Velum & uvula", "velum-uvula"], ["Larynx", "larynx-epiglottis"]] as const) {
    if (!(await click(el, "Controls for"))) continue;
    await click(/^Show only/);
    await view("front and side");
    await shot(`${tag}`, 4);
    await click("Show all");
  }
} finally {
  await browser.close();
  writeFileSync(`${out}/capture.log`, log.join("\n") + "\n");
  console.log(log.join("\n"));
}
