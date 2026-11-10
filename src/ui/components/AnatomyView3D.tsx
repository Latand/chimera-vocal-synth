/**
 * The face screen in 3D: the head with the engine's 3D vocal tract inside it (render/head/),
 * moved by the live voice's articulators. Whole, or opened along the midline so the tract shows
 * ("Cut away"). Drag to turn, wheel or pinch to zoom, right-drag to pan, double-click to go
 * back. The articulators can't be dragged here; the flat view keeps the handles.
 *
 * three.js draws into its own canvas, which goes under the screen's 2D canvas each frame: the
 * readouts, the words at the mouth and their glow are the flat view's own.
 */
import { useEffect, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { HeadScene } from "../render/head/scene.ts";
import { loadHead } from "../render/head/asset.ts";
import { faceHandles, type FaceView } from "../render/anatomy.ts";
import { Petting } from "../render/petting.ts";
import { drawReadouts } from "../render/readouts.ts";
import { Glow, label } from "../render/vector.ts";
import { engine } from "../../audio/AudioEngine.ts";
import { sameBody } from "../body.ts";
import { useStore } from "../store.ts";
import { VoiceModel } from "../analysis.ts";
import { useModel } from "../useModel.ts";
import { V } from "../colors.ts";
import { articulatorSlots, tractInfo, type InfoFn, type OverlayFn, type SlotsFn } from "./AnatomyView.tsx";
import { ARTIC_KEYS, type HumanArtic } from "../../wasm/engine.ts";
import type { TelemetryMessage } from "../../worklet/protocol.ts";

/** how a fed head is shown */
export interface HeadView {
  /** the narrowest place's area, written beside it */
  calipers: boolean;
  /** opened along the midline (default), or the head whole */
  cutaway?: boolean;
}

/**
 * A head driven by something other than the live engine (the duet page's recordings): its own
 * body's model and view, and each frame's telemetry. `tele` null holds the head in `artic`
 * with the voice at rest; a new `epoch` (a seek) restarts what follows the voice.
 */
export interface HeadFeed {
  model: VoiceModel;
  view: HeadView;
  frame(): { tele: TelemetryMessage | null; artic: HumanArtic; epoch: number };
  /** cap on the device pixel ratio (two heads on a phone draw half the pixels at 1.5) */
  maxDpr?: number;
  /** which way the head turns to show its face: 1 to the viewer's right, −1 to their left */
  facing?: 1 | -1;
}

const ease = (t: number) => (t < 0.5 ? 4 * t * t * t : 1 - (-2 * t + 2) ** 3 / 2);

export function AnatomyView3D(props: { slots?: SlotsFn; info?: InfoFn; showFormants?: boolean; overlay?: OverlayFn; feed?: HeadFeed; readouts?: boolean }) {
  const feedRef = useRef<HeadFeed | undefined>(props.feed);
  feedRef.current = props.feed;
  const readoutsRef = useRef(props.readouts ?? true);
  readoutsRef.current = props.readouts ?? true;
  const overlayRef = useRef<OverlayFn | undefined>(props.overlay);
  overlayRef.current = props.overlay;
  const formantsRef = useRef(props.showFormants ?? true);
  formantsRef.current = props.showFormants ?? true;
  const slotsRef = useRef<SlotsFn>(props.slots ?? articulatorSlots);
  slotsRef.current = props.slots ?? articulatorSlots;
  const infoRef = useRef<InfoFn>(props.info ?? tractInfo);
  infoRef.current = props.info ?? tractInfo;
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const wrapRef = useRef<HTMLDivElement>(null);
  const model = useModel();
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    const canvas = canvasRef.current!;
    const ctx = canvas.getContext("2d")!;
    const wrap = wrapRef.current!;
    const glCanvas = document.createElement("canvas");
    let gl: THREE.WebGLRenderer;
    try {
      gl = new THREE.WebGLRenderer({ canvas: glCanvas, antialias: true, alpha: false, powerPreference: "high-performance" });
    } catch {
      setFailed(true);
      return;
    }
    gl.setClearColor(0x000000, 1);
    const scene = new HeadScene(gl);
    const facing = feedRef.current?.facing ?? 1;
    const glow = new Glow();
    const camera = scene.camera;
    const controls = new OrbitControls(camera, canvas);
    controls.enableDamping = true;
    controls.dampingFactor = 0.08;
    controls.rotateSpeed = 0.7;
    controls.zoomSpeed = 0.8;
    controls.screenSpacePanning = true;
    controls.minDistance = 12;
    controls.maxDistance = 260;
    let live = true;
    let headReady = false;
    void loadHead().then(
      (mesh) => {
        if (!live) return;
        scene.setHead(mesh);
        headReady = true;
      },
      () => {},
    );

    // camera moves (the opening turn from the side, double-click home): orbit round the target
    const tween = { on: false, t: 0, dur: 1, p0: new THREE.Vector3(), q0: new THREE.Vector3(), p1: new THREE.Vector3(), q1: new THREE.Vector3() };
    const s0 = new THREE.Spherical(), s1 = new THREE.Spherical(), off = new THREE.Vector3();
    const moveTo = (pos: THREE.Vector3, target: THREE.Vector3, dur: number) => {
      tween.p0.copy(camera.position);
      tween.q0.copy(controls.target);
      tween.p1.copy(pos);
      tween.q1.copy(target);
      tween.t = 0;
      tween.dur = dur;
      tween.on = true;
    };
    const stepTween = (dt: number) => {
      if (!tween.on) return;
      tween.t = Math.min(1, tween.t + dt / tween.dur);
      const e = ease(tween.t);
      s0.setFromVector3(off.copy(tween.p0).sub(tween.q0));
      s1.setFromVector3(off.copy(tween.p1).sub(tween.q1));
      let dth = s1.theta - s0.theta;
      dth -= Math.round(dth / (2 * Math.PI)) * 2 * Math.PI;
      s0.set(s0.radius + (s1.radius - s0.radius) * e, s0.phi + (s1.phi - s0.phi) * e, s0.theta + dth * e);
      controls.target.lerpVectors(tween.q0, tween.q1, e);
      camera.position.copy(controls.target).add(off.setFromSpherical(s0));
      if (tween.t >= 1) tween.on = false;
    };
    controls.addEventListener("start", () => (tween.on = false));
    let cutaway = true;
    /** the view the head is shown from: opened, mostly from the cut's side; whole, from the front and side */
    const homePose = () => {
      // with the readouts the head is drawn smaller, to leave their corner free (its stand ran
      // under the first readout's digits)
      const p = scene.pose(cutaway ? "side" : "three", true, facing, readoutsRef.current && w >= 560 ? 1.4 : 1.08);
      if (cutaway) {
        // a little towards the face, so the cut face keeps its depth
        const d = p.position.clone().sub(p.target);
        d.applyAxisAngle(new THREE.Vector3(0, 1, 0), 0.38 * facing);
        d.y += d.length() * 0.08;
        p.position.copy(p.target).add(d);
      }
      return p;
    };
    const home = () => {
      const p = homePose();
      moveTo(p.position, p.target, 0.9);
    };
    canvas.addEventListener("dblclick", home);

    // petting: a mouse wiggled over the top of the head (no button down: that turns the head)
    const pet = new Petting();
    let crown: { x: number; y: number; cmPx: number } | null = null;
    let lastPt: [number, number] | null = null;
    const onHover = (e: PointerEvent) => {
      if (e.buttons !== 0 || e.pointerType === "touch" || !crown) {
        lastPt = null;
        return;
      }
      const p: [number, number] = [e.offsetX, e.offsetY];
      // over the crown: within 6 cm of its top, and not below the brow
      const near = Math.hypot(p[0] - crown.x, p[1] - crown.y) < 6 * crown.cmPx && p[1] < crown.y + 4.5 * crown.cmPx;
      if (near && lastPt) pet.stroke(Math.hypot(p[0] - lastPt[0], p[1] - lastPt[1]) / crown.cmPx);
      lastPt = near ? p : null;
    };
    const offHead = () => (lastPt = null);
    canvas.addEventListener("pointermove", onHover);
    canvas.addEventListener("pointerleave", offHead);

    let w = 0, h = 0, dpr = 1;
    const maxDpr = () => Math.min(feedRef.current?.maxDpr ?? 2, window.devicePixelRatio || 1);
    /** a fed head (no settings of its own to keep) goes back to its framing when its pane changes shape */
    let refit = false;
    const fit = () => {
      const b = wrap.getBoundingClientRect();
      dpr = maxDpr();
      refit = !!feedRef.current && (b.width !== w || b.height !== h);
      w = b.width;
      h = b.height;
      canvas.width = Math.max(1, Math.round(w * dpr));
      canvas.height = Math.max(1, Math.round(h * dpr));
      gl.setPixelRatio(dpr);
      gl.setSize(Math.max(1, w), Math.max(1, h), false);
      camera.aspect = Math.max(1, w) / Math.max(1, h);
      // on a wide screen the head sits left of centre, leaving the right for the readouts
      if (w < 560 || !readoutsRef.current) camera.clearViewOffset();
      else camera.setViewOffset(w, h, 0.17 * w, 0.03 * h, w, h);
      camera.updateProjectionMatrix();
    };
    const ro = new ResizeObserver(fit);
    ro.observe(wrap);
    const pxPerCm = (x: number, y: number) => {
      const e = camera.matrixWorld.elements;
      const [x0, y0] = scene.project(x, y, 0, w, h);
      const [x1, y1] = scene.project(x + e[0], y + e[1], e[2], w, h);
      return Math.hypot(x1 - x0, y1 - y0);
    };
    let raf = 0;
    let last = performance.now();
    let fTick = 0;
    let formants: number[] = [];
    let posed: VoiceModel | null = null;
    const posedTarget = new THREE.Vector3();
    /** what the scene last drew: the posture and how it was shown */
    let drawn = "";
    const loop = (now: number) => {
      const dt = Math.min(0.05, (now - last) / 1000);
      const elapsed = Math.min(0.5, (now - last) / 1000);
      last = now;
      if (maxDpr() !== dpr) fit();
      const st = useStore.getState();
      const feed = feedRef.current;
      let tele: TelemetryMessage | null, a: HumanArtic;
      if (feed) {
        const fr = feed.frame();
        tele = fr.tele;
        a = tele ? VoiceModel.articFromArray(tele.t.artic) : fr.artic;
      } else {
        tele = engine.latest && sameBody(engine.latest.body, st.body) ? engine.latest : null;
        a = tele ? VoiceModel.articFromArray(tele.t.artic) : st.artic;
      }
      const view: HeadView = feed ? feed.view : st.view;
      const m = feed ? feed.model : model.current;
      if (m && w > 0 && h > 0) {
        const cut = view.cutaway ?? true;
        if (cut !== cutaway) {
          cutaway = cut;
          if (posed) home();
        }
        if (posed !== m) {
          // the camera's poses depend on the body the scene holds: draw it first
          scene.update(m, a, {}, { head: true, cutaway, seeThrough: false, hidden: [], lit: null, explode: false, side: facing });
          drawn = "";
          const p = homePose();
          if (!posed) {
            // open on the flat view's side view, then turn to show the head
            const side = scene.pose("side", true, facing);
            camera.position.copy(side.position);
            controls.target.copy(side.target);
            moveTo(p.position, p.target, 1.6);
          } else {
            // another body: keep the view, centred on the new head
            off.copy(p.target).sub(posedTarget);
            camera.position.add(off);
            controls.target.add(off);
          }
          posedTarget.copy(p.target);
          posed = m;
        } else if (refit) {
          const p = homePose();
          moveTo(p.position, p.target, 0.4);
        }
        refit = false;
        stepTween(dt);
        controls.update(dt);
        // the engine's shapes are recomputed only when the posture or the way it is shown changes
        const key = `${headReady}|${cutaway}|${ARTIC_KEYS.map((k) => a[k].toFixed(4)).join(",")}`;
        if (key !== drawn) {
          scene.update(m, a, {}, { head: true, cutaway, seeThrough: false, hidden: [], lit: null, explode: false, side: facing });
          drawn = key;
        }
        gl.render(scene.scene, camera);
        const f = m.frame(a);
        const readouts = readoutsRef.current;
        if (readouts && (++fTick % 4 === 0 || formants.length === 0)) formants = m.transfer(a).formants;
        // the overlays first, glowing; the head goes under them
        ctx.setTransform(1, 0, 0, 1, 0, 0);
        ctx.globalCompositeOperation = "source-over";
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
        if (view.calipers && cutaway) {
          // the narrowest place of the tract (past the larynx), with its area
          let best = -1;
          for (let i = 0; i < f.grid.length; i++) if (f.grid[i].s > 2.5 && (best < 0 || f.grid[i].area < f.grid[best].area)) best = i;
          if (best >= 0) {
            const g = f.grid[best];
            const [cx, cy] = scene.project(g.mid.x, g.mid.y, 0, w, h);
            label(ctx, g.area <= 0.0005 ? "CLOSED" : `${g.area.toFixed(2)} CM²`, cx + 10, cy - 6, { size: 11, color: V.white, ls: 0.12 });
          }
        }
        if (readouts) {
          const info = infoRef.current(m, f, tele).map((s) => s.toUpperCase());
          const slots = slotsRef.current(m, a, f, formants, tele);
          drawReadouts(ctx, { w, h, compact: w < 560, slotX: w - Math.min(340, w * 0.42), slotY: h - Math.min(236, h * 0.3) }, formantsRef.current ? formants : [], info, slots);
        }
        const lips = faceHandles(m, f, a).find((x) => x.id === "lips")!.pos;
        const faceView: FaceView = {
          handles: faceHandles,
          X: (p) => scene.project(p.x, p.y, 0, w, h)[0],
          Y: (p) => scene.project(p.x, p.y, 0, w, h)[1],
          pxPerCm: pxPerCm(lips.x, lips.y),
        };
        overlayRef.current?.(ctx, faceView, m, f, a, { w, h, dt });
        // the hearts start beside the crown on the side the face looks to, and drift that way
        const k = m.tract.scale;
        const [crx, cry] = scene.project(-3 * k, 24.4 * k, 0, w, h);
        crown = { x: crx, y: cry, cmPx: pxPerCm(-3 * k, 24.4 * k) };
        pet.update(elapsed, [crown.x + facing * 7 * crown.cmPx, crown.y + 0.5 * crown.cmPx], crown.cmPx);
        pet.draw(ctx, facing, crown.cmPx);
        glow.apply(canvas, ctx, dpr);
        ctx.setTransform(1, 0, 0, 1, 0, 0);
        ctx.globalCompositeOperation = "destination-over";
        ctx.drawImage(glCanvas, 0, 0, canvas.width, canvas.height);
        ctx.globalCompositeOperation = "source-over";
      }
      raf = requestAnimationFrame(loop);
    };
    raf = requestAnimationFrame(loop);
    return () => {
      live = false;
      cancelAnimationFrame(raf);
      ro.disconnect();
      canvas.removeEventListener("dblclick", home);
      canvas.removeEventListener("pointermove", onHover);
      canvas.removeEventListener("pointerleave", offHead);
      controls.dispose();
      scene.dispose();
      gl.dispose();
      // browsers allow few live WebGL contexts: give this one back now, not at garbage collection
      gl.forceContextLoss();
    };
  }, [model]);

  if (failed)
    return (
      <div className="anatomy three">
        <p className="no-webgl">3D needs WebGL, which this browser has turned off. Untick 3D for the flat view.</p>
      </div>
    );
  return (
    <div ref={wrapRef} className="anatomy three">
      <canvas ref={canvasRef} aria-label="The head and its vocal tract in 3D. Drag to turn the head, scroll or pinch to zoom, double-click to go back to the starting view." role="img" />
    </div>
  );
}
