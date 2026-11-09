/**
 * Numbered frames of the 3D head and tract for before/after evidence: the lab's 3D view (the
 * head whole from three sides, the open mouth close up, the head cut open, see-through, the
 * tract alone, the planes), every human mode's face screen, and the lab's page at desktop and
 * phone width. The same postures, cameras and options each run; a control a build lacks is
 * skipped and said so in the log.
 * Usage: bun tools/tract3d/lab-capture.ts URL OUT_DIR
 */
import { chromium, type Page } from "playwright-core";
import { mkdirSync, writeFileSync } from "node:fs";

const [url, out] = process.argv.slice(2);
if (!url || !out) throw new Error("usage: bun tools/tract3d/lab-capture.ts URL OUT_DIR");
mkdirSync(out, { recursive: true });
const log: string[] = [];
// (no GL flags: forcing SwiftShader through ANGLE left the page without animation frames after a screenshot)
const browser = await chromium.launch({ headless: true });
let n = 0;
const open = async (path: string, width: number, height: number): Promise<Page> => {
  const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: 1 });
  page.on("pageerror", (e) => log.push(`pageerror: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && log.push(`console: ${m.text()}`));
  await page.goto(new URL(path, url).toString(), { waitUntil: "networkidle" });
  return page;
};
const name = (label: string) => `${String(++n).padStart(2, "0")}-${label}.png`;
try {
  // ---- the lab's 3D view
  const page = await open("?mode=lab", 1440, 1000);
  await page.getByRole("heading", { name: "Tract", exact: true }).waitFor();
  // the prompt would cover the head: start the audio (headless: no sound is heard) and stop the voice
  const start = page.getByRole("button", { name: /start and listen/i });
  if (await start.count()) {
    await start.click();
    await page.getByRole("group", { name: "Listen" }).getByRole("button", { name: "Stop", exact: true }).click();
  }
  const click = async (label: string | RegExp, scope?: string): Promise<boolean> => {
    const root = scope ? page.getByRole("group", { name: scope }) : page;
    const b = root.getByRole("button", { name: label, exact: typeof label === "string" }).first();
    if ((await b.count()) === 0) {
      log.push(`skipped: no button ${label}`);
      return false;
    }
    await b.click();
    return true;
  };
  const toggle = async (label: string, on: boolean) => {
    const b = page.getByRole("button", { name: label, exact: true }).first();
    if ((await b.count()) === 0) return log.push(`skipped: no toggle ${label}`), false;
    if (await b.isDisabled()) return false;
    if ((await b.getAttribute("aria-pressed")) !== String(on)) await b.click();
    return true;
  };
  const shot = async (label: string, zoom = 0) => {
    const stage = page.locator(".lab-stage");
    const box = (await stage.boundingBox())!;
    if (zoom) {
      await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
      for (let i = 0; i < Math.abs(zoom); i++) await page.mouse.wheel(0, zoom > 0 ? -120 : 120);
    }
    await page.waitForTimeout(1200);
    const file = name(label);
    await page.screenshot({ path: `${out}/${file}`, clip: box });
    log.push(file);
  };
  const view = (v: string) => click(v === "mouth" ? "View of the mouth" : `View from the ${v}`);
  const posture = (label: string | RegExp) => click(label, "Postures");
  await posture("open vowel /a/");
  // the head whole: three-quarter, front, side; the open mouth close up for three vowels
  await toggle("Cut away", false);
  for (const [v, tag] of [["front and side", "head-three"], ["front", "head-front"], ["side", "head-side"]] as const) {
    await view(v);
    await shot(tag);
  }
  for (const [p, tag] of [["open vowel /a/", "a"], [/close front \/i\//, "i"], [/close back rounded \/u\//, "u"]] as const) {
    await posture(p);
    await view("mouth");
    await shot(`mouth-${tag}`);
  }
  await posture("open vowel /a/");
  if (await toggle("See-through", true)) {
    await view("front and side");
    await shot("head-see-through");
    await toggle("See-through", false);
  }
  // the head cut open: its layers on the cut face, the tract's air the hole in it
  await toggle("Cut away", true);
  await view("side");
  await shot("cut-side");
  await shot("cut-side-close", 4);
  await view("front and side");
  await shot("cut-three");
  for (const [p, tag] of [[/close front \/i\//, "i"], [/grooved \/s\//, "s"], [/bunched/, "bunched-r"]] as const) {
    await posture(p);
    await view("side");
    await shot(`cut-side-${tag}`, 3);
  }
  // the planes where they lie: a sagittal plane off the midline, then a coronal one in the CUT screen
  await posture("open vowel /a/");
  // (a range input is set through its own setter, so the page's handler sees the change)
  const slide = (v: number) =>
    page.getByRole("slider", { name: "Cut position" }).evaluate((el, value) => {
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(el, String(value));
      el.dispatchEvent(new Event("input", { bubbles: true }));
    }, v);
  if (await click("Sagittal", "Plane")) {
    await slide(1.2);
    await view("front and side");
    await shot("plane-sagittal-right");
    await slide(-1.2);
    await shot("plane-sagittal-left");
  }
  if (await click("Coronal", "Plane")) {
    await slide(-1);
    await page.waitForTimeout(500);
    const file = name("cut-screen-coronal-back");
    await page.locator(".lab-cut-screen").screenshot({ path: `${out}/${file}` });
    log.push(file);
  }
  await click("Along tract", "Plane");
  // the tract alone
  await toggle("Head", false);
  await view("front and side");
  await shot("tract-alone");
  await toggle("Head", true);
  await view("front and side");
  await page.waitForTimeout(800);
  const labDesktop = name("lab-desktop");
  await page.screenshot({ path: `${out}/${labDesktop}`, fullPage: true });
  log.push(labDesktop);
  await page.close();

  // ---- every human mode's face screen, opened and whole
  const modes = await open("", 1440, 1000);
  for (const mode of ["Play", "Speak", "Sing", "Beatbox"]) {
    await modes.getByRole("radio", { name: mode }).click();
    await modes.waitForTimeout(3000);
    const file = name(`mode-${mode.toLowerCase()}`);
    await modes.screenshot({ path: `${out}/${file}` });
    log.push(file);
  }
  const cut = modes.getByRole("checkbox", { name: "Cut away" });
  if (await cut.count()) {
    await modes.getByRole("radio", { name: "Play" }).click();
    await cut.uncheck({ force: true });
    await modes.waitForTimeout(2500);
    const file = name("mode-play-whole");
    await modes.screenshot({ path: `${out}/${file}` });
    log.push(file);
  }
  await modes.close();

  // ---- a phone: the lab's whole page, and Play
  const phone = await open("?mode=lab", 390, 844);
  await phone.getByRole("heading", { name: "Tract", exact: true }).waitFor();
  await phone.waitForTimeout(2500);
  for (const [label, full] of [["lab-390", false], ["lab-390-page", true]] as const) {
    const file = name(label);
    await phone.screenshot({ path: `${out}/${file}`, fullPage: full });
    log.push(file);
  }
  await phone.getByRole("radio", { name: "Play" }).click();
  await phone.waitForTimeout(3000);
  const play = name("mode-play-390");
  await phone.screenshot({ path: `${out}/${play}` });
  log.push(play);
  await phone.close();
} finally {
  await browser.close();
  writeFileSync(`${out}/frames.txt`, log.join("\n") + "\n");
  console.log(log.join("\n"));
}
