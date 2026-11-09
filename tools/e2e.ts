/**
 * Headless-Chromium smoke test: loads the app, starts audio, interacts, collects console errors,
 * measures worklet load, and saves screenshots.
 * Usage: bun tools/e2e.ts [url] [outdir]
 */
import { chromium } from "playwright-core";
import { mkdirSync } from "node:fs";

const url = process.argv[2] ?? "http://127.0.0.1:5318/";
const outDir = process.argv[3] ?? "out/e2e";
mkdirSync(outDir, { recursive: true });

const browser = await chromium.launch({
  headless: true,
  args: ["--autoplay-policy=no-user-gesture-required", "--use-fake-ui-for-media-stream"],
});
const errors: string[] = [];
const logs: string[] = [];
async function run(viewport: { width: number; height: number }, tag: string) {
  const page = await browser.newPage({ viewport, deviceScaleFactor: 1 });
  page.on("console", (m) => {
    const t = `${m.type()}: ${m.text()}`;
    if (m.type() === "error") errors.push(`[${tag}] ${t}`);
    else logs.push(`[${tag}] ${t}`);
  });
  page.on("pageerror", (e) => errors.push(`[${tag}] pageerror: ${e.message}`));
  await page.goto(url, { waitUntil: "networkidle" });
  await page.waitForTimeout(400);
  await page.screenshot({ path: `${outDir}/${tag}-0-idle.png` });
  const startBtn = page.getByRole("button", { name: /breathe in and start/i });
  await startBtn.click();
  await page.waitForTimeout(1800);
  await page.screenshot({ path: `${outDir}/${tag}-1-running.png` });
  // the face screen opens on the 3D head: it must be drawn (not a black pane), and a drag turns it
  const headDrawn = async (where: string) => {
    let lit = 0;
    for (let i = 0; i < 40 && lit < 0.04; i++) {
      await page.waitForTimeout(150);
      lit = await page.evaluate(() => {
        const c = document.querySelector<HTMLCanvasElement>(".anatomy.three canvas");
        if (!c || !c.width) return 0;
        const d = c.getContext("2d")!.getImageData(0, 0, c.width, c.height).data;
        let n = 0, total = 0;
        for (let p = 0; p < d.length; p += 4 * 97) {
          total++;
          if (d[p] + d[p + 1] + d[p + 2] > 90) n++;
        }
        return n / total;
      });
    }
    logs.push(`[${tag}] 3d head in ${where}: ${(lit * 100).toFixed(0)} % of the pane lit`);
    if (!(lit >= 0.04)) errors.push(`[${tag}] the 3D head is not drawn in ${where} (${(lit * 100).toFixed(1)} % lit)`);
  };
  await headDrawn("play");
  const box = await page.locator(".anatomy canvas").boundingBox();
  if (box) {
    const sx = box.x + box.width * 0.47, sy = box.y + box.height * 0.42;
    await page.mouse.move(sx, sy);
    await page.mouse.down();
    await page.mouse.move(sx + 40, sy - 30, { steps: 10 });
    await page.mouse.up();
  }
  // the head whole, then opened again
  await page.getByRole("checkbox", { name: "Cut away" }).uncheck({ force: true });
  await headDrawn("play, whole head");
  await page.screenshot({ path: `${outDir}/${tag}-1b-whole.png` });
  await page.getByRole("checkbox", { name: "Cut away" }).check({ force: true });
  // the flat view keeps the handles: drag the tongue body there, then back to 3D
  await page.getByRole("checkbox", { name: "3D" }).uncheck({ force: true });
  await page.waitForTimeout(300);
  const flat = await page.locator(".anatomy canvas").boundingBox();
  if (flat) {
    // move over likely tongue area and drag up/forward
    const sx = flat.x + flat.width * 0.47, sy = flat.y + flat.height * 0.42;
    await page.mouse.move(sx, sy);
    await page.mouse.down();
    await page.mouse.move(sx + 40, sy - 30, { steps: 10 });
    await page.mouse.up();
  }
  await page.getByRole("checkbox", { name: "3D" }).check({ force: true });
  // vowel map drag
  const vm = await page.locator(".vowel-map").boundingBox();
  if (vm) {
    await page.mouse.move(vm.x + vm.width * 0.2, vm.y + vm.height * 0.2);
    await page.mouse.down();
    await page.mouse.move(vm.x + vm.width * 0.8, vm.y + vm.height * 0.8, { steps: 20 });
    await page.mouse.up();
  }
  await page.keyboard.down("d");
  await page.waitForTimeout(900);
  await page.screenshot({ path: `${outDir}/${tag}-2-played.png` });
  await page.keyboard.up("d");
  const loadText = await page.locator(".load").textContent().catch(() => null);
  logs.push(`[${tag}] load: ${loadText}`);
  // speak mode
  await page.getByRole("radio", { name: "Speak" }).click();
  await page.waitForTimeout(200);
  await headDrawn("speak");
  await page.getByRole("button", { name: /^Speak$/ }).click();
  await page.waitForTimeout(1500);
  await page.screenshot({ path: `${outDir}/${tag}-3-speak.png`, fullPage: true });
  const trans = await page.locator(".transcription").textContent();
  logs.push(`[${tag}] transcription: ${trans}`);
  // speech breathes by itself: Play's held breath must not drone on once the phrase is over
  const speechRms: number[] = [];
  for (let i = 0; i < 12; i++) {
    await page.waitForTimeout(250);
    speechRms.push(
      await page.evaluate(() => {
        // @ts-expect-error debug handle
        return window.__chimeraEngine?.latest?.t?.outRms ?? -1;
      }),
    );
  }
  logs.push(`[${tag}] speech rms then silence ${speechRms.map((r) => r.toFixed(4)).join(",")}`);
  if (!(Math.max(...speechRms.slice(-3)) < 1e-3)) errors.push(`[${tag}] voice still sounding after the phrase: ${speechRms.slice(-3).join(",")}`);
  await page.locator("button.ipa", { hasText: "ʃ" }).first().click();
  await page.waitForTimeout(700);
  await page.screenshot({ path: `${outDir}/${tag}-4-pad.png` });
  // sing mode: a key sings the marked syllable and moves the cursor; legato; a demo song
  await page.getByRole("radio", { name: "Sing" }).click();
  await page.waitForTimeout(600);
  await headDrawn("sing");
  const rms = () =>
    page.evaluate(() => {
      // @ts-expect-error debug handle
      return window.__chimeraEngine?.latest?.t?.outRms ?? -1;
    });
  const nextSyl = () => page.locator(".syl.next").textContent().catch(() => null);
  const sylBefore = await nextSyl();
  await page.keyboard.down("d");
  await page.waitForTimeout(450);
  const singing = await rms();
  await page.keyboard.down("g"); // legato into the next syllable
  await page.waitForTimeout(350);
  await page.keyboard.up("d");
  await page.waitForTimeout(250);
  await page.keyboard.up("g");
  await page.waitForTimeout(900);
  const after = await rms();
  const sylAfter = await nextSyl();
  logs.push(`[${tag}] sing: next syllable ${sylBefore} -> ${sylAfter}, rms while held ${singing.toFixed(4)}, then silence ${after.toFixed(4)}`);
  if (!(singing > 0.01)) errors.push(`[${tag}] a sung key made no sound (${singing})`);
  if (!(after < 1e-3)) errors.push(`[${tag}] voice still sounding after the sung notes: ${after}`);
  if (sylBefore === sylAfter) errors.push(`[${tag}] the syllable cursor did not move (${sylBefore})`);
  await page.getByRole("button", { name: "Made of air" }).click();
  await page.waitForTimeout(2200);
  const demoRms = await rms();
  const demoNow = await page.locator(".syl.now").textContent().catch(() => null);
  await page.screenshot({ path: `${outDir}/${tag}-5-sing.png`, fullPage: tag !== "desktop" });
  await page.getByRole("button", { name: /^Stop “Made of air”$/ }).click();
  await page.waitForTimeout(700);
  const demoStopped = await rms();
  logs.push(`[${tag}] sing demo: rms ${demoRms.toFixed(4)} on "${demoNow}", after stop ${demoStopped.toFixed(4)}`);
  if (!(demoRms > 0.01) || !demoNow) errors.push(`[${tag}] demo song not singing (rms ${demoRms}, syllable ${demoNow})`);
  if (!(demoStopped < 1e-3)) errors.push(`[${tag}] demo still sounding after stop: ${demoStopped}`);
  // bestiary: every creature, its first call, a key in held-voice mode, and the sustain toggle
  await page.getByRole("radio", { name: "Bestiary" }).click();
  await page.waitForTimeout(300);
  const names = await page.locator(".specimen-name").allTextContents();
  for (const [i, name] of names.entries()) {
    await page.locator(".specimen").nth(i).click();
    await page.waitForTimeout(250);
    await page.locator(".call").first().click();
    await page.waitForTimeout(1300);
    const slug = name.toLowerCase().replace(/[^a-z]+/g, "-");
    if (tag === "desktop" || i === 0) await page.screenshot({ path: `${outDir}/${tag}-5-${slug}.png`, fullPage: tag !== "desktop" });
    const state = await page.evaluate(() => {
      // @ts-expect-error debug handle
      const e = window.__chimeraEngine;
      const t = e?.latestCreature?.t;
      return t ? { id: t.id, rms: t.outRms, call: t.call, load: e.load } : null;
    });
    logs.push(`[${tag}] creature ${name}: ${JSON.stringify(state)}`);
  }
  await page.keyboard.down("g");
  await page.waitForTimeout(600);
  const held = await page.evaluate(() => {
    // @ts-expect-error debug handle
    const t = window.__chimeraEngine?.latestCreature?.t;
    return t ? { gate: t.gate, rms: t.outRms } : null;
  });
  await page.keyboard.up("g");
  logs.push(`[${tag}] held key: ${JSON.stringify(held)}`);
  // beatbox: preset, play, playhead + level from telemetry, a pad, stop
  await page.getByRole("radio", { name: "Beatbox" }).click();
  await page.waitForTimeout(300);
  await headDrawn("beatbox");
  await page.getByRole("button", { name: "Breakbeat" }).click();
  await page.getByRole("button", { name: "Play beat" }).click();
  const beatSamples: { step: number; rms: number }[] = [];
  for (let i = 0; i < 8; i++) {
    await page.waitForTimeout(250);
    beatSamples.push(
      await page.evaluate(() => {
        // @ts-expect-error debug handle
        const t = window.__chimeraEngine?.latest;
        return { step: t?.beatStep ?? -2, rms: t?.t?.outRms ?? -1 };
      }),
    );
  }
  await page.screenshot({ path: `${outDir}/${tag}-6-beatbox.png`, fullPage: tag !== "desktop" });
  logs.push(`[${tag}] beatbox steps ${beatSamples.map((b) => b.step).join(",")} rms ${beatSamples.map((b) => b.rms.toFixed(3)).join(",")}`);
  await page.locator(".pad").first().dispatchEvent("pointerdown");
  await page.waitForTimeout(300);
  await page.getByRole("button", { name: "Stop" }).click();
  await page.waitForTimeout(300);
  // Lab: the 3D head and tract, the cuts in every plane, the controls, and the voice heard there.
  // The sound is judged from telemetry as it arrives (each wait is bounded), never from a value
  // read after a fixed sleep: a late telemetry message would still hold the state before the click.
  const voice = () =>
    page.evaluate(() => {
      // @ts-expect-error debug handle
      const t = window.__chimeraEngine?.latest?.t;
      return t ? { rms: t.outRms as number, gate: t.gate as number, clock: t.clock as number } : null;
    });
  const until = async (what: string, ok: (v: { rms: number; gate: number; clock: number }) => boolean, ms = 6000): Promise<boolean> => {
    const end = Date.now() + ms;
    let last: Awaited<ReturnType<typeof voice>> = null;
    while (Date.now() < end) {
      last = await voice();
      if (last && ok(last)) return true;
      await page.waitForTimeout(60);
    }
    errors.push(`[${tag}] lab: ${what} (last telemetry ${JSON.stringify(last)})`);
    return false;
  };
  await page.getByRole("radio", { name: "Lab", exact: true }).click();
  await page.getByRole("heading", { name: "Tract", exact: true }).waitFor();
  await page.getByRole("button", { name: "open vowel /a/" }).click();
  for (const plane of ["Sagittal", "Coronal", "Axial", "Along tract"]) {
    await page.getByRole("group", { name: "Plane" }).getByRole("button", { name: plane }).click();
    if (!(await page.locator(".lab-cut").isVisible())) errors.push(`[${tag}] lab cut missing for ${plane}`);
  }
  const area0 = await page.locator(".lab-cut-screen .lab-readout").textContent();
  // a vowel posture is held: Listen opens the breath on it, Stop closes it
  await until("the voice was not silent on entering the lab", (v) => v.rms < 1e-3);
  await page.getByRole("group", { name: "Listen" }).getByRole("button", { name: "Listen", exact: true }).click();
  await until("Listen did not open the breath", (v) => v.gate > 0.5);
  await until("Listen on /a/ is silent", (v) => v.rms > 0.01);
  await page.getByRole("button", { name: "close back rounded /u/" }).click();
  await until("the held voice stopped when the posture changed", (v) => v.gate > 0.5 && v.rms > 0.005);
  await page.screenshot({ path: `${outDir}/${tag}-7-lab.png`, fullPage: true });
  await page.getByRole("group", { name: "Listen" }).getByRole("button", { name: "Stop", exact: true }).click();
  await until("Stop did not close the breath", (v) => v.gate < 0.5);
  await until("the voice still sounds after Stop", (v) => v.rms < 1e-3);
  // a consonant posture is said between two vowels (held alone it is nearly silent), then it is quiet again
  await page.getByRole("button", { name: "grooved /s/" }).click();
  const area1 = await page.locator(".lab-cut-screen .lab-readout").textContent();
  if (area0 === area1) errors.push(`[${tag}] lab cut did not change with the posture`);
  await until("the /s/ posture was not spoken", (v) => v.rms > 0.01);
  await until("the voice still sounds after /asa/", (v) => v.rms < 1e-3, 8000);
  // words, typed
  await page.getByRole("textbox", { name: "Words to say" }).fill("hello");
  await page.getByRole("group", { name: "Listen" }).getByRole("button", { name: "Say", exact: true }).click();
  await until("Say (words) is silent", (v) => v.rms > 0.01);
  await until("the voice still sounds after the word", (v) => v.rms < 1e-3, 8000);
  await page.getByRole("button", { name: "Show only Tongue" }).click();
  await page.getByRole("button", { name: "Show all", exact: true }).click();
  await page.getByRole("group", { name: "Controls for" }).getByRole("button", { name: "Midline" }).click();
  for (const v of ["View from the side", "View of the mouth", "View from the front and side"]) await page.getByRole("button", { name: v }).click();
  if (await page.evaluate(() => document.documentElement.scrollWidth > innerWidth)) errors.push(`[${tag}] lab scrolls horizontally`);
  // back to the human voice
  await page.getByRole("radio", { name: "Play" }).click();
  await page.waitForTimeout(600);
  const human = await page.evaluate(() => {
    // @ts-expect-error debug handle
    const e = window.__chimeraEngine;
    return { human: !!e?.latest, creature: !!e?.latestCreature };
  });
  logs.push(`[${tag}] back to play: ${JSON.stringify(human)}`);
  await page.close();
}

// the very first key in Sing mode, before audio has started, still sings once audio is up
{
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  page.on("pageerror", (e) => errors.push(`[first-key] pageerror: ${e.message}`));
  await page.goto(url, { waitUntil: "networkidle" });
  await page.getByRole("radio", { name: "Sing" }).click();
  await page.keyboard.down("d");
  await page.waitForTimeout(1200);
  const level = await page.evaluate(() => {
    // @ts-expect-error debug handle
    return window.__chimeraEngine?.latest?.t?.outRms ?? -1;
  });
  await page.keyboard.up("d");
  logs.push(`[first-key] sing before start: rms ${level.toFixed(4)}`);
  if (!(level > 0.01)) errors.push(`[first-key] the first sung key, pressed before audio started, made no sound (${level})`);
  await page.close();
}

// a visitor who opens the lab first: the prompt's one button starts the voice on the posture shown
{
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  page.on("pageerror", (e) => errors.push(`[lab-first] pageerror: ${e.message}`));
  await page.goto(new URL("?mode=lab", url).toString(), { waitUntil: "networkidle" });
  await page.getByRole("button", { name: /start and listen/i }).click();
  const end = Date.now() + 8000;
  let level = -1;
  while (Date.now() < end && !(level > 0.01)) {
    await page.waitForTimeout(80);
    level = await page.evaluate(() => {
      // @ts-expect-error debug handle
      return window.__chimeraEngine?.latest?.t?.outRms ?? -1;
    });
  }
  logs.push(`[lab-first] lab start prompt: rms ${level.toFixed(4)}`);
  if (!(level > 0.01)) errors.push(`[lab-first] the lab's start prompt made no sound (${level})`);
  await page.close();
}

await run({ width: 1440, height: 980 }, "desktop");
await run({ width: 390, height: 844 }, "mobile");
await browser.close();
console.log("console errors:", errors.length ? "\n" + errors.join("\n") : "none");
console.log(logs.filter((l) => /load|warn|transcription|creature|held|back to|beatbox|silence|sing|lab/i.test(l)).slice(0, 40).join("\n"));
// a run that collected errors fails, so CI and agents can't read it as a pass
if (errors.length) process.exit(1);
