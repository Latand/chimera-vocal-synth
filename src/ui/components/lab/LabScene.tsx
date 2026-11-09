/**
 * The lab's 3D view of the engine's element meshes: each element drawn as a grid of lines in its
 * own colour over a black fill that hides what is behind it (a vector display with hidden lines),
 * the centreline, and the current cut (its air in amber, its plane as a dim frame).
 * Drag to turn, wheel or pinch to zoom, right-drag to pan.
 */
import { useEffect, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import type { Tract3dCut, Tract3dFrame } from "../../../wasm/engine.ts";
import { ELEMENTS } from "../../lab.ts";
import { V } from "../../colors.ts";
import type { LabState } from "../../store.ts";

export type View = "side" | "front" | "top" | "three";
const VIEWS: Record<View, [number, number, number]> = {
  side: [0, 0, 1],
  front: [1, 0.05, 0],
  top: [0.05, 1, 0.02],
  three: [0.75, 0.45, 0.85],
};

interface Props {
  frame: Tract3dFrame;
  cut: Tract3dCut | null;
  lab: LabState;
  view: { name: View; serial: number };
  scale: number;
}

export function LabScene({ frame, cut, lab, view, scale }: Props) {
  const wrap = useRef<HTMLDivElement>(null);
  const state = useRef({ frame, cut, lab, scale });
  state.current = { frame, cut, lab, scale };
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
    renderer.outputColorSpace = THREE.LinearSRGBColorSpace;
    renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
    renderer.setClearColor(0x000000, 1);
    renderer.localClippingEnabled = true;
    const canvas = renderer.domElement;
    canvas.className = "lab-canvas";
    canvas.setAttribute("role", "img");
    canvas.setAttribute("aria-label", "The 3D vocal tract by element. Drag to turn, wheel or pinch to zoom.");
    host.appendChild(canvas);
    const scene = new THREE.Scene();
    scene.fog = new THREE.Fog(0x000000, 30, 70);
    const camera = new THREE.PerspectiveCamera(32, 1, 0.5, 400);
    const controls = new OrbitControls(camera, canvas);
    controls.enableDamping = true;
    controls.dampingFactor = 0.1;
    controls.rotateSpeed = 0.7;
    controls.screenSpacePanning = true;
    controls.minDistance = 8;
    controls.maxDistance = 90;
    const clip = new THREE.Plane(new THREE.Vector3(0, 0, -1), 0);

    // one fill mesh and one line set per element, sharing positions
    const parts = ELEMENTS.map((e) => {
      const position = new THREE.BufferAttribute(new Float32Array(0), 3);
      const fillGeo = new THREE.BufferGeometry();
      const lineGeo = new THREE.BufferGeometry();
      const fill = new THREE.Mesh(
        fillGeo,
        new THREE.MeshBasicMaterial({ color: 0x000000, side: THREE.DoubleSide, polygonOffset: true, polygonOffsetFactor: 1, polygonOffsetUnits: 1 }),
      );
      const lines = new THREE.LineSegments(lineGeo, new THREE.LineBasicMaterial({ color: new THREE.Color(e.color), transparent: true, fog: true }));
      fill.renderOrder = 0;
      lines.renderOrder = 1;
      const group = new THREE.Group();
      group.add(fill, lines);
      scene.add(group);
      return { e, position, fillGeo, lineGeo, fill, lines, group, n: -1 };
    });
    const centre = new THREE.Line(new THREE.BufferGeometry(), new THREE.LineDashedMaterial({ color: new THREE.Color(V.pink), dashSize: 0.25, gapSize: 0.18, transparent: true, opacity: 0.8, depthTest: false }));
    centre.renderOrder = 3;
    const cutAir = new THREE.LineSegments(new THREE.BufferGeometry(), new THREE.LineBasicMaterial({ color: new THREE.Color(V.amber), depthTest: false }));
    cutAir.renderOrder = 4;
    const cutFrame = new THREE.LineLoop(new THREE.BufferGeometry(), new THREE.LineBasicMaterial({ color: new THREE.Color(V.amber), transparent: true, opacity: 0.45, depthTest: false }));
    cutFrame.renderOrder = 4;
    scene.add(centre, cutAir, cutFrame);

    const target = new THREE.Vector3();
    const look = (v: View) => {
      const k = state.current.scale;
      target.set(2.6 * k, 6.2 * k, 0);
      const d = VIEWS[v];
      const dir = new THREE.Vector3(d[0], d[1], d[2]).normalize();
      camera.position.copy(target).addScaledVector(dir, 26 * k);
      camera.up.set(0, 1, 0);
      controls.target.copy(target);
      controls.update();
    };

    let dirty = true;
    const update = () => {
      const { frame, cut, lab } = state.current;
      const meshes = frame.meshes ?? [];
      parts.forEach((p, i) => {
        const m = meshes[i];
        if (!m) return;
        const n = m.positions.length;
        if (n !== p.n) {
          // first frame or a new body: build the geometry; afterwards positions update in place
          p.position = new THREE.BufferAttribute(new Float32Array(m.positions), 3);
          p.position.setUsage(THREE.DynamicDrawUsage);
          p.fillGeo.setAttribute("position", p.position);
          p.fillGeo.setIndex(m.indices);
          p.lineGeo.setAttribute("position", p.position);
          p.lineGeo.setIndex(m.lines);
          p.n = n;
        } else {
          (p.position.array as Float32Array).set(m.positions);
          p.position.needsUpdate = true;
        }
        p.fillGeo.computeBoundingSphere();
        p.lineGeo.computeBoundingSphere();
        const shown = !lab.hidden.includes(p.e.key);
        p.group.visible = shown;
        p.fill.visible = !lab.xray;
        const lit = lab.element === p.e.key;
        const mat = p.lines.material as THREE.LineBasicMaterial;
        mat.opacity = lit ? 1 : lab.element === "midline" ? 0.85 : 0.55;
        const planes = lab.cutaway ? [clip] : [];
        mat.clippingPlanes = planes;
        (p.fill.material as THREE.MeshBasicMaterial).clippingPlanes = planes;
        const k = state.current.scale;
        p.group.position.set(0, lab.explode ? (i - 3.5) * 0.9 * k : 0, lab.explode ? (i % 2 ? 1 : -1) * 1.2 * k : 0);
      });
      const c = frame.centreline;
      centre.geometry.setAttribute("position", new THREE.Float32BufferAttribute(c.flatMap((p) => [p.x, p.y, 0]), 3));
      centre.computeLineDistances();
      const seg: number[] = [];
      const box: number[] = [];
      if (cut) {
        const at = (a: number, z: number) => [cut.origin.x + cut.u.x * a, cut.origin.y + cut.u.y * a, z];
        for (const row of cut.rows) for (const q of row.intervals) seg.push(...at(q.lo, row.z), ...at(q.hi, row.z));
        const zs = cut.rows.map((r) => r.z);
        const z0 = Math.min(...zs), z1 = Math.max(...zs);
        const reach = cut.regions.length ? Math.max(1.2, ...cut.regions.map((r) => Math.max(Math.abs(r.minU), Math.abs(r.maxU)))) : 1.2;
        box.push(...at(-reach, z0), ...at(reach, z0), ...at(reach, z1), ...at(-reach, z1));
      }
      cutAir.geometry.setAttribute("position", new THREE.Float32BufferAttribute(seg, 3));
      cutFrame.geometry.setAttribute("position", new THREE.Float32BufferAttribute(box, 3));
      dirty = true;
    };
    api.current = { update, look };

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
        renderer.render(scene, camera);
        dirty = false;
      }
      raf = requestAnimationFrame(loop);
    };
    look("three");
    update();
    resize();
    raf = requestAnimationFrame(loop);
    return () => {
      cancelAnimationFrame(raf);
      ro.disconnect();
      controls.dispose();
      for (const p of parts) {
        p.fillGeo.dispose();
        p.lineGeo.dispose();
        (p.fill.material as THREE.Material).dispose();
        (p.lines.material as THREE.Material).dispose();
      }
      for (const o of [centre, cutAir, cutFrame]) {
        o.geometry.dispose();
        (o.material as THREE.Material).dispose();
      }
      renderer.dispose();
      renderer.forceContextLoss();
      canvas.remove();
      api.current = null;
    };
  }, []);
  useEffect(() => api.current?.update(), [frame, cut, lab]);
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
