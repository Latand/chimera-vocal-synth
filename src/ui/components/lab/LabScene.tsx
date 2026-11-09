/**
 * The lab's 3D view of the engine's element meshes, in two styles:
 *  - the tract alone, each element drawn as a grid of lines in its own colour over a black fill
 *    that hides what is behind it (a vector display with hidden lines);
 *  - inside the head (`lab.head`): the elements shaded in tissue colours inside a head with skin,
 *    fat, muscle, the skull and brain, the mandible, the cervical spine, the hyoid and the
 *    laryngeal cartilages (render/head3d.ts). Cut away, every closed layer shows its cut face in
 *    its tissue's colour (stencil caps: a point of the plane is inside a closed surface when a
 *    ray through it crosses that surface an odd number of times).
 * Plus the centreline and the current cut (its air in amber, its plane as a dim frame).
 * Drag to turn, wheel or pinch to zoom, right-drag to pan.
 */
import { useEffect, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import type { Tract3dCut, Tract3dFrame } from "../../../wasm/engine.ts";
import { ELEMENTS } from "../../lab.ts";
import { V } from "../../colors.ts";
import type { LabState } from "../../store.ts";
import type { HeadPart, SectionRegion } from "../../render/head3d.ts";
import type { Vec } from "../../render/geom.ts";

export type View = "side" | "front" | "top" | "three";
const VIEWS: Record<View, [number, number, number]> = {
  side: [0, 0, 1],
  front: [1, 0.05, 0],
  top: [0.05, 1, 0.02],
  three: [0.75, 0.45, 0.85],
};

/** the elements' tissue colours inside the head (same order as ELEMENTS) */
const TISSUE_OF: Record<string, number> = {
  tongue: 0xc4585a,
  palate: 0xe0a092,
  velum: 0xdc8a8c,
  pharynx: 0xb45856,
  larynx: 0xd2a48e,
  lips: 0xc85e5e,
  teethJaw: 0xf2ede2,
  cheeks: 0xb8625e,
};

interface Props {
  frame: Tract3dFrame;
  cut: Tract3dCut | null;
  lab: LabState;
  view: { name: View; serial: number };
  scale: number;
  head: HeadPart[] | null;
  /** the midsagittal section on the cut face of the opened head */
  section: { air: Vec[][]; tissue: SectionRegion[] } | null;
}

interface HeadSlot {
  key: string;
  geo: THREE.BufferGeometry;
  visible: THREE.Mesh;
  back: THREE.Mesh | null;
  front: THREE.Mesh | null;
  cap: THREE.Mesh | null;
}

export function LabScene({ frame, cut, lab, view, scale, head, section }: Props) {
  const wrap = useRef<HTMLDivElement>(null);
  const state = useRef({ frame, cut, lab, scale, head, section });
  state.current = { frame, cut, lab, scale, head, section };
  const api = useRef<{ update: () => void; look: (v: View) => void } | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    const host = wrap.current!;
    let renderer: THREE.WebGLRenderer;
    try {
      renderer = new THREE.WebGLRenderer({ antialias: true, stencil: true });
    } catch {
      setFailed(true);
      return;
    }
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.toneMappingExposure = 1.05;
    renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
    renderer.setClearColor(0x000000, 1);
    renderer.localClippingEnabled = true;
    const canvas = renderer.domElement;
    canvas.className = "lab-canvas";
    canvas.setAttribute("role", "img");
    canvas.setAttribute("aria-label", "The 3D vocal tract by element, optionally inside the head. Drag to turn, wheel or pinch to zoom.");
    host.appendChild(canvas);
    const scene = new THREE.Scene();
    scene.fog = new THREE.Fog(0x000000, 30, 70);
    const camera = new THREE.PerspectiveCamera(32, 1, 0.5, 400);
    // light: a soft sky/ground fill, a key light that moves with the camera, a cool rim behind
    scene.add(new THREE.HemisphereLight(0xfff3ea, 0x2c1e1c, 1.25));
    const key = new THREE.DirectionalLight(0xffffff, 2.0);
    key.position.set(8, 14, 10);
    camera.add(key);
    scene.add(camera);
    const rim = new THREE.DirectionalLight(0xa9c8ff, 1.1);
    rim.position.set(-30, 25, -40);
    scene.add(rim);
    const controls = new OrbitControls(camera, canvas);
    controls.enableDamping = true;
    controls.dampingFactor = 0.1;
    controls.rotateSpeed = 0.7;
    controls.screenSpacePanning = true;
    controls.minDistance = 8;
    controls.maxDistance = 140;
    const clip = new THREE.Plane(new THREE.Vector3(0, 0, -1), 0);

    // one fill mesh, one shaded solid and one line set per element, sharing positions
    const parts = ELEMENTS.map((e) => {
      const position = new THREE.BufferAttribute(new Float32Array(0), 3);
      const fillGeo = new THREE.BufferGeometry();
      const lineGeo = new THREE.BufferGeometry();
      const fill = new THREE.Mesh(
        fillGeo,
        new THREE.MeshBasicMaterial({ color: 0x000000, side: THREE.DoubleSide, polygonOffset: true, polygonOffsetFactor: 1, polygonOffsetUnits: 1 }),
      );
      const solid = new THREE.Mesh(fillGeo, new THREE.MeshStandardMaterial({ color: TISSUE_OF[e.key] ?? 0xc08080, roughness: 0.62, metalness: 0, side: THREE.DoubleSide, fog: false }));
      const lines = new THREE.LineSegments(lineGeo, new THREE.LineBasicMaterial({ color: new THREE.Color(e.color), transparent: true, fog: true, toneMapped: false }));
      fill.renderOrder = 0;
      solid.renderOrder = 10;
      lines.renderOrder = 1001;
      const group = new THREE.Group();
      group.add(fill, solid, lines);
      scene.add(group);
      return { e, position, fillGeo, lineGeo, fill, solid, lines, group, n: -1 };
    });
    const centre = new THREE.Line(new THREE.BufferGeometry(), new THREE.LineDashedMaterial({ color: new THREE.Color(V.pink), dashSize: 0.25, gapSize: 0.18, transparent: true, opacity: 0.8, depthTest: false, toneMapped: false }));
    centre.renderOrder = 3003;
    const cutAir = new THREE.LineSegments(new THREE.BufferGeometry(), new THREE.LineBasicMaterial({ color: new THREE.Color(V.amber), depthTest: false, toneMapped: false }));
    cutAir.renderOrder = 3004;
    const cutFrame = new THREE.LineLoop(new THREE.BufferGeometry(), new THREE.LineBasicMaterial({ color: new THREE.Color(V.amber), transparent: true, opacity: 0.45, depthTest: false, toneMapped: false }));
    cutFrame.renderOrder = 3004;
    scene.add(centre, cutAir, cutFrame);

    // ---- the head: its parts are rebuilt as the posture changes
    const headGroup = new THREE.Group();
    scene.add(headGroup);
    let slots: HeadSlot[] = [];
    let headParts: HeadPart[] | null = null;
    const capGeo = new THREE.PlaneGeometry(90, 90);
    const stencilMat = (side: THREE.Side, op: THREE.StencilOp) =>
      new THREE.MeshBasicMaterial({
        side,
        colorWrite: false,
        depthWrite: false,
        depthTest: false,
        stencilWrite: true,
        stencilFunc: THREE.AlwaysStencilFunc,
        stencilFail: op,
        stencilZFail: op,
        stencilZPass: op,
        clippingPlanes: [clip],
      });
    const skinMat = new THREE.MeshPhysicalMaterial({ color: 0xd8a48a, roughness: 0.58, metalness: 0, sheen: 0.5, sheenRoughness: 0.6, sheenColor: new THREE.Color(0xffc9b3), clearcoat: 0.06, clearcoatRoughness: 0.7, fog: false, side: THREE.DoubleSide });
    const disposeSlots = () => {
      for (const s of slots) {
        s.geo.dispose();
        for (const m of [s.visible, s.back, s.front, s.cap]) if (m) (m.material as THREE.Material).dispose();
      }
      headGroup.clear();
      slots = [];
    };
    const buildHead = (hp: HeadPart[]) => {
      // same parts as before: replace the geometry only
      if (slots.length === hp.length && slots.every((s, i) => s.key === hp[i].key)) {
        hp.forEach((p, i) => {
          const g = new THREE.BufferGeometry();
          g.setAttribute("position", new THREE.BufferAttribute(p.positions, 3));
          g.setIndex(new THREE.BufferAttribute(p.indices, 1));
          if (p.colors) g.setAttribute("color", new THREE.BufferAttribute(p.colors, 3));
          g.computeVertexNormals();
          const s = slots[i];
          for (const m of [s.visible, s.back, s.front]) if (m) m.geometry = g;
          s.geo.dispose();
          s.geo = g;
        });
        return;
      }
      disposeSlots();
      let order = 100;
      for (const p of hp) {
        const g = new THREE.BufferGeometry();
        g.setAttribute("position", new THREE.BufferAttribute(p.positions, 3));
        g.setIndex(new THREE.BufferAttribute(p.indices, 1));
        g.computeVertexNormals();
        if (p.colors) g.setAttribute("color", new THREE.BufferAttribute(p.colors, 3));
        const visMat = p.skin ? skinMat.clone() : new THREE.MeshStandardMaterial({ color: p.color, roughness: p.key === "deep" ? 0.9 : 0.5, metalness: 0, side: p.key === "deep" ? THREE.BackSide : THREE.FrontSide, fog: false });
        if (p.colors) {
          // the colours carry the skin's tone: the material only lights them
          (visMat as THREE.MeshPhysicalMaterial).vertexColors = true;
          (visMat as THREE.MeshPhysicalMaterial).color.set(0xffffff);
        }
        const visible = new THREE.Mesh(g, visMat);
        visible.renderOrder = 10;
        headGroup.add(visible);
        let back: THREE.Mesh | null = null, front: THREE.Mesh | null = null, cap: THREE.Mesh | null = null;
        if (p.closed) {
          back = new THREE.Mesh(g, stencilMat(THREE.BackSide, THREE.IncrementWrapStencilOp));
          front = new THREE.Mesh(g, stencilMat(THREE.FrontSide, THREE.DecrementWrapStencilOp));
          back.renderOrder = order;
          front.renderOrder = order + 0.1;
          cap = new THREE.Mesh(
            capGeo,
            new THREE.MeshStandardMaterial({
              color: p.cap,
              roughness: 0.8,
              metalness: 0,
              fog: false,
              stencilWrite: true,
              stencilRef: 0,
              stencilFunc: THREE.NotEqualStencilFunc,
              stencilFail: THREE.ReplaceStencilOp,
              stencilZFail: THREE.ReplaceStencilOp,
              stencilZPass: THREE.ReplaceStencilOp,
            }),
          );
          cap.renderOrder = order + 0.2;
          cap.onAfterRender = (r) => r.clearStencil();
          headGroup.add(back, front, cap);
          order += 1;
        }
        slots.push({ key: p.key, geo: g, visible, back, front, cap });
      }
    };

    // the cut face's midsagittal section: air holes (depth only, so the layers' caps behind them
    // fail and the tract shows through) and the tract's tissues drawn over the caps
    const sectionGroup = new THREE.Group();
    scene.add(sectionGroup);
    // (depth writes need the depth test on: it always passes)
    const airMat = new THREE.MeshBasicMaterial({ colorWrite: false, depthWrite: true, depthTest: true, depthFunc: THREE.AlwaysDepth, side: THREE.DoubleSide });
    const regionGeo = (poly: Vec[]) => {
      const pts = poly.map((p) => new THREE.Vector2(p.x, p.y));
      return new THREE.ShapeGeometry(new THREE.Shape(pts));
    };
    const buildSection = (sec: { air: Vec[][]; tissue: SectionRegion[] }) => {
      for (const c of sectionGroup.children) {
        const m = c as THREE.Mesh;
        m.geometry.dispose();
        if (m.material !== airMat) (m.material as THREE.Material).dispose();
      }
      sectionGroup.clear();
      for (const poly of sec.air) {
        if (poly.length < 3) continue;
        const m = new THREE.Mesh(regionGeo(poly), airMat);
        m.position.z = 0.01;
        m.renderOrder = 50;
        sectionGroup.add(m);
      }
      for (const r of sec.tissue) {
        if (r.poly.length < 3) continue;
        const m = new THREE.Mesh(regionGeo(r.poly), new THREE.MeshStandardMaterial({ color: r.color, roughness: 0.75, metalness: 0, side: THREE.DoubleSide, fog: false }));
        m.position.z = 0.02;
        m.renderOrder = 900;
        sectionGroup.add(m);
      }
    };
    let shownSection: unknown = null;

    const target = new THREE.Vector3();
    const look = (v: View) => {
      const { scale: k, lab, head } = state.current;
      const withHead = lab.head && head && head.length > 0;
      if (withHead) target.set(1.2 * k, 8.6 * k, 0);
      else target.set(2.6 * k, 6.2 * k, 0);
      const d = VIEWS[v];
      const dir = new THREE.Vector3(d[0], d[1], d[2]).normalize();
      camera.position.copy(target).addScaledVector(dir, (withHead ? 60 : 26) * k);
      camera.up.set(0, 1, 0);
      controls.target.copy(target);
      controls.update();
      scene.fog = withHead ? null : new THREE.Fog(0x000000, 30, 70);
    };

    let dirty = true;
    let shownHead = false;
    const update = () => {
      const { frame, cut, lab, head } = state.current;
      const withHead = lab.head && !!head && head.length > 0;
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
        if (withHead) p.fillGeo.computeVertexNormals();
        const shown = !lab.hidden.includes(p.e.key);
        p.group.visible = shown;
        p.fill.visible = !lab.xray && !withHead;
        p.solid.visible = withHead && !lab.xray;
        const lit = lab.element === p.e.key;
        const mat = p.lines.material as THREE.LineBasicMaterial;
        mat.opacity = withHead ? (lit ? 0.5 : 0.12) : lit ? 1 : lab.element === "midline" ? 0.85 : 0.55;
        const planes = lab.cutaway ? [clip] : [];
        mat.clippingPlanes = planes;
        (p.fill.material as THREE.MeshBasicMaterial).clippingPlanes = planes;
        (p.solid.material as THREE.MeshStandardMaterial).clippingPlanes = planes;
        const k = state.current.scale;
        p.group.position.set(0, lab.explode ? (i - 3.5) * 0.9 * k : 0, lab.explode ? (i % 2 ? 1 : -1) * 1.2 * k : 0);
      });
      // the head
      if (withHead && head !== headParts) {
        buildHead(head!);
        headParts = head;
      }
      headGroup.visible = withHead;
      if (withHead) {
        const cutOpen = lab.cutaway;
        const see = lab.seeThrough && !cutOpen;
        for (const s of slots) {
          const layer = s.key === "skin" || s.key === "fat" || s.key === "muscle";
          const mat = s.visible.material as THREE.MeshStandardMaterial;
          // the layers' own surfaces only make cut faces; the skin seen is `skin-visible`
          s.visible.visible = s.key === "skin-visible" || (!layer && (s.key !== "deep" || cutOpen));
          mat.clippingPlanes = cutOpen ? [clip] : [];
          if (s.key === "skin-visible") {
            mat.transparent = see;
            mat.opacity = see ? 0.24 : 1;
            mat.depthWrite = !see;
            s.visible.renderOrder = see ? 2000 : 10;
          }
          for (const m of [s.back, s.front, s.cap]) if (m) m.visible = cutOpen;
        }
      }
      const sec = state.current.section;
      const showSection = withHead && lab.cutaway && !!sec;
      if (showSection && sec !== shownSection) {
        buildSection(sec!);
        shownSection = sec;
      }
      sectionGroup.visible = showSection;
      if (withHead !== shownHead) {
        shownHead = withHead;
        look("three");
      }
      const c = frame.centreline;
      centre.geometry.setAttribute("position", new THREE.Float32BufferAttribute(c.flatMap((p) => [p.x, p.y, 0]), 3));
      centre.computeLineDistances();
      centre.visible = !withHead || lab.cutaway || lab.seeThrough;
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
      cutAir.visible = cutFrame.visible = centre.visible;
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
    update();
    look("three");
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
        (p.solid.material as THREE.Material).dispose();
        (p.lines.material as THREE.Material).dispose();
      }
      for (const o of [centre, cutAir, cutFrame]) {
        o.geometry.dispose();
        (o.material as THREE.Material).dispose();
      }
      disposeSlots();
      for (const c of sectionGroup.children) (c as THREE.Mesh).geometry.dispose();
      airMat.dispose();
      capGeo.dispose();
      skinMat.dispose();
      renderer.dispose();
      renderer.forceContextLoss();
      canvas.remove();
      api.current = null;
    };
  }, []);
  useEffect(() => api.current?.update(), [frame, cut, lab, head, section]);
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
