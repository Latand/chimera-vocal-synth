/** Rasterize local validation SVGs with Chromium. No network or dataset discovery. */
import { chromium } from "playwright-core";
import { readdirSync, readFileSync } from "node:fs";
import { resolve, join } from "node:path";
const root = resolve(process.argv[2] ?? "");
if (!process.argv[2])
  throw new Error("usage: bun tools/tract3d/capture.ts EVIDENCE_DIRECTORY");
const browser = await chromium.launch({ headless: true });
try {
  const page = await browser.newPage({
    viewport: { width: 1500, height: 900 },
    deviceScaleFactor: 1,
  });
  for (const file of readdirSync(root)
    .filter((f) => f.endsWith(".svg"))
    .sort()) {
    await page.setContent(readFileSync(join(root, file), "utf8"));
    const svg = page.locator("svg");
    await svg.screenshot({ path: join(root, file.replace(/\.svg$/, ".png")) });
  }
} finally {
  await browser.close();
}
