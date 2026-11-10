/**
 * The lab's 3D view: the engine's 3D tract inside the head (render/head/scene.ts), whole, opened
 * along the midline with its layers on the cut face, or the tract alone. Over it, the lab's own
 * drawings: the centreline, and the selected cut or plane where it lies (its air in amber, its
 * frame dimmer). Drag to turn, wheel or pinch to zoom, right-drag to pan.
 */
import { useEffect, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import type { HumanArtic, Tract3dCut, Tract3dFrame, Tract3dPlane } from "../../../wasm/engine.ts";
import type { LabState } from "../../store.ts";
import type { VoiceModel } from "../../analysis.ts";
import { HeadScene, type ViewName } from "../../render/head/scene.ts";
import { loadHead } from "../../render/head/asset.ts";

export type View = ViewName;

interface Props {
  model: VoiceModel;
  artic: HumanArtic;
  frame: Tract3dFrame;
  /** the perpendicular cut or the coronal/axial plane shown, or the sagittal plane's rows */
  cut: Tract3dCut | null;
  plane: Tract3dPlane;
  lab: LabState;
  view: { name: View; serial: number };
}

/** the cut's air as line segments, and its frame, in body cm */
function cutDrawing(cut: Tract3dCut | null, plane: Tract3dPlane, k: number): { segments: number[]; frame: number[] } {
  const segments: number[] = [], frame: number[] = [];
  if (plane && plane.kind === "sagittal") {
    // a sagittal plane lies at its own lateral offset: rows of air along x at each height
    const z = (plane.at ?? 0) * k;
    let x0 = Infinity, x1 = -Infinity, y0 = Infinity, y1 = -Infinity;
    for (const row of plane.rows) {
      for (let i = 0; i + 1 < row.x.length; i += 2) {
        segments.push(row.x[i], row.y, z, row.x[i + 1], row.y, z);
        x0 = Math.min(x0, row.x[i]);
        x1 = Math.max(x1, row.x[i + 1]);
      }
      y0 = Math.min(y0, row.y);
      y1 = Math.max(y1, row.y);
    }
    if (!Number.isFinite(x0)) [x0, x1, y0, y1] = [-2 * k, 8 * k, 0, 11 * k];
    frame.push(x0 - 0.4, y0 - 0.4, z, x1 + 0.4, y0 - 0.4, z, x1 + 0.4, y1 + 0.4, z, x0 - 0.4, y1 + 0.4, z);
    return { segments, frame };
  }
  if (!cut) return { segments, frame };
  const at = (a: number, z: number) => [cut.origin.x + cut.u.x * a, cut.origin.y + cut.u.y * a, z];
  let lo = Infinity, hi = -Infinity, z0 = Infinity, z1 = -Infinity;
  for (const row of cut.rows) {
    for (const q of row.intervals) {
      segments.push(...at(q.lo, row.z), ...at(q.hi, row.z));
      lo = Math.min(lo, q.lo);
      hi = Math.max(hi, q.hi);
      z0 = Math.min(z0, row.z);
      z1 = Math.max(z1, row.z);
    }
  }
  if (!Number.isFinite(lo)) [lo, hi, z0, z1] = [-1.2, 1.2, -1.5 * k, 1.5 * k];
  frame.push(...at(lo - 0.3, z0 - 0.3), ...at(hi + 0.3, z0 - 0.3), ...at(hi + 0.3, z1 + 0.3), ...at(lo - 0.3, z1 + 0.3));
  return { segments, frame };
}

export function LabScene({ model, artic, frame, cut, plane, lab, view }: Props) {
  const wrap = useRef<HTMLDivElement>(null);
  const state = useRef({ model, artic, frame, cut, plane, lab });
  state.current = { model, artic, frame, cut, plane, lab };
  const api = useRef<{ update: () => void; look: (v: View) => void } | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    const host = wrap.current!;
    let renderer: THREE.WebGLRenderer;
    try {
      renderer = new THREE.WebGLRenderer({ antialias: true });
    } catch {
      setFailed(true);
      return;
    }
    renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
    renderer.setClearColor(0x000000, 1);
    const canvas = renderer.domElement;
    canvas.className = "lab-canvas";
    canvas.setAttribute("role", "img");
    canvas.setAttribute("aria-label", "The head and its 3D vocal tract. Drag to turn, wheel or pinch to zoom.");
    host.appendChild(canvas);
    const scene = new HeadScene(renderer);
    const camera = scene.camera;
    const controls = new OrbitControls(camera, canvas);
    controls.enableDamping = true;
    controls.dampingFactor = 0.1;
    controls.rotateSpeed = 0.7;
    controls.screenSpacePanning = true;
    controls.minDistance = 6;
    controls.maxDistance = 220;

    let dirty = true;
    let live = true;
    let shownHead: boolean | null = null;
    const look = (v: View) => {
      const p = scene.pose(v, state.current.lab.head, 1, 1.02);
      camera.position.copy(p.position);
      camera.up.set(0, 1, 0);
      controls.target.copy(p.target);
      controls.update();
      dirty = true;
    };
    const update = () => {
      const { model, artic, frame, cut, plane, lab } = state.current;
      scene.update(model, artic, lab.shape, { head: lab.head, cutaway: lab.cutaway, seeThrough: lab.seeThrough, hidden: lab.hidden, lit: lab.lines ? lab.element : null, explode: lab.explode });
      const inside = !lab.head || lab.cutaway || lab.seeThrough;
      scene.overlay(inside ? { centreline: frame.centreline, ...cutDrawing(cut, plane, model.bodyParams.scale) } : null);
      if (shownHead !== lab.head) {
        shownHead = lab.head;
        look("three");
      }
      dirty = true;
    };
    api.current = { update, look };
    void loadHead().then(
      (mesh) => {
        if (!live) return;
        scene.setHead(mesh);
        update();
      },
      () => {},
    );

    const resize = () => {
      const w = host.clientWidth, h = host.clientHeight;
      renderer.setSize(w, h, false);
      camera.aspect = w / Math.max(1, h);
      camera.updateProjectionMatrix();
      dirty = true;
    };
    const ro = new ResizeObserver(resize);
    ro.observe(host);
    controls.addEventListener("change", () => (dirty = true));
    let raf = 0;
    const loop = () => {
      // damping keeps moving the camera for a moment after a drag
      if (controls.update() || dirty) {
        renderer.render(scene.scene, camera);
        dirty = false;
      }
      raf = requestAnimationFrame(loop);
    };
    resize();
    update();
    look("three");
    raf = requestAnimationFrame(loop);
    return () => {
      live = false;
      cancelAnimationFrame(raf);
      ro.disconnect();
      controls.dispose();
      scene.dispose();
      renderer.dispose();
      renderer.forceContextLoss();
      canvas.remove();
      api.current = null;
    };
  }, []);
  useEffect(() => api.current?.update(), [model, artic, frame, cut, plane, lab]);
  useEffect(() => {
    if (view.serial > 0) api.current?.look(view.name);
  }, [view]);
  return (
    <div className="lab-stage" ref={wrap}>
      {failed && (
        <p className="lab-note" role="alert">
          WebGL is unavailable here. The cuts, the area function and the controls still work.
        </p>
      )}
    </div>
  );
}
