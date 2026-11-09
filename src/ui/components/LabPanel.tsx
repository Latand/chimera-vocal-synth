import { useEffect, useMemo, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import {
  tract3dSpecs,
  tract3dPresets,
  ARTIC_KEYS,
  ARTIC_SPECS,
  type HumanArtic,
  type Tract3dFrame,
  type Tract3dCut,
} from "../../wasm/engine.ts";
import { useStore } from "../store.ts";
import { useModel } from "../useModel.ts";
import { sameBody } from "../body.ts";
import { VoiceModel } from "../analysis.ts";
import { engine } from "../../audio/AudioEngine.ts";
import { speakText, stopSpeaking } from "../speech.ts";
import { Screen, Chip } from "./Screen.tsx";
import { EncoderPages, type EncoderPage } from "./Encoder.tsx";
import { SpectrumView } from "./SpectrumView.tsx";

const COLORS = [
  0xf077a9, 0x68b6e8, 0x9dbbf6, 0x65a6ce, 0x8bdfbc, 0xefb778, 0xe0d2a9,
  0x7195bd,
];
const label = (s: string) =>
  s.replace(/([A-Z])/g, " $1").replace(/^./, (x) => x.toUpperCase());
function Scene({ frame, cut }: { frame: Tract3dFrame; cut: Tract3dCut }) {
  const ref = useRef<HTMLDivElement>(null);
  const state = useRef({ frame, cut, lab: useStore.getState().lab });
  const lab = useStore((s) => s.lab);
  state.current = { frame, cut, lab };
  const update = useRef<() => void>(() => {});
  const [error, setError] = useState(false);
  useEffect(() => {
    const wrap = ref.current!;
    let renderer: THREE.WebGLRenderer;
    try {
      renderer = new THREE.WebGLRenderer({ antialias: true });
    } catch {
      setError(true);
      return;
    }
    renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
    renderer.setClearColor(0x000000);
    renderer.localClippingEnabled = true;
    renderer.domElement.setAttribute(
      "aria-label",
      "Parametric tract. Drag to rotate; wheel or pinch to zoom.",
    );
    renderer.domElement.setAttribute("role", "img");
    wrap.appendChild(renderer.domElement);
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(35, 1, 0.1, 1000);
    camera.position.set(19, 12, 22);
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.target.set(2, 5, 0);
    controls.minDistance = 8;
    controls.maxDistance = 70;
    controls.update();
    const meshes: THREE.Mesh[] = [];
    for (let i = 0; i < 8; i++) {
      const g = new THREE.BufferGeometry();
      const m = new THREE.MeshBasicMaterial({
        color: COLORS[i],
        wireframe: true,
        transparent: true,
        opacity: 0.48,
        side: THREE.DoubleSide,
      });
      const mesh = new THREE.Mesh(g, m);
      meshes.push(mesh);
      scene.add(mesh);
    }
    const line = new THREE.LineSegments(
      new THREE.BufferGeometry(),
      new THREE.LineBasicMaterial({ color: 0xffc573 }),
    );
    scene.add(line);
    const clip = new THREE.Plane(new THREE.Vector3(0, 0, -1), 0);
    const draw = () => renderer.render(scene, camera);
    update.current = () => {
      const { frame, cut, lab } = state.current;
      frame.meshes.forEach((data, i) => {
        const m = meshes[i];
        const g = m.geometry;
        const old = g.getAttribute("position") as
          | THREE.BufferAttribute
          | undefined;
        if (old?.array.length === data.positions.length) {
          old.array.set(data.positions);
          old.needsUpdate = true;
        } else {
          g.setAttribute(
            "position",
            new THREE.Float32BufferAttribute(data.positions, 3),
          );
        }
        g.setIndex(data.indices);
        g.computeBoundingSphere();
        m.visible = !lab.hidden.includes(data.element);
        (m.material as THREE.MeshBasicMaterial).clippingPlanes = lab.cutaway
          ? [clip]
          : [];
        m.position.set(
          0,
          lab.explode ? (i - 3.5) * 0.65 : 0,
          lab.explode ? (i % 2 ? 1 : -1) * 0.7 : 0,
        );
      });
      const vertices: number[] = [];
      for (const row of cut.rows)
        for (const q of row.intervals) {
          vertices.push(
            cut.origin.x + cut.u.x * q.lo,
            cut.origin.y + cut.u.y * q.lo,
            row.z,
            cut.origin.x + cut.u.x * q.hi,
            cut.origin.y + cut.u.y * q.hi,
            row.z,
          );
        }
      line.geometry.setAttribute(
        "position",
        new THREE.Float32BufferAttribute(vertices, 3),
      );
      line.geometry.computeBoundingSphere();
      draw();
    };
    const resize = () => {
      renderer.setSize(wrap.clientWidth, wrap.clientHeight);
      camera.aspect = wrap.clientWidth / Math.max(1, wrap.clientHeight);
      camera.updateProjectionMatrix();
      draw();
    };
    const ro = new ResizeObserver(resize);
    ro.observe(wrap);
    controls.addEventListener("change", draw);
    update.current();
    resize();
    return () => {
      ro.disconnect();
      controls.dispose();
      for (const m of meshes) {
        m.geometry.dispose();
        (m.material as THREE.Material).dispose();
      }
      line.geometry.dispose();
      (line.material as THREE.Material).dispose();
      renderer.dispose();
      renderer.forceContextLoss();
      renderer.domElement.remove();
      update.current = () => {};
    };
  }, []);
  useEffect(() => update.current(), [frame, cut, lab]);
  return (
    <div className="lab-scene" ref={ref}>
      {error && (
        <p role="alert">
          WebGL is unavailable. The cuts and area measurements below remain
          available.
        </p>
      )}
    </div>
  );
}
function CutView({ frame, cut }: { frame: Tract3dFrame; cut: Tract3dCut }) {
  const plane = useStore((s) => s.lab.plane);
  if (plane === "midsagittal")
    return (
      <svg
        className="lab-cut"
        viewBox="-4 -12 16 16"
        role="img"
        aria-label="Midsagittal tissue outlines in centimetres"
      >
        {frame.midsagittal.map(([name, points], i) => (
          <polyline
            key={`${name}-${i}`}
            points={points.map((p) => `${p.x},${-p.y}`).join(" ")}
            fill="none"
            stroke={`#${COLORS[i % COLORS.length].toString(16)}`}
            strokeWidth={0.035}
          />
        ))}
      </svg>
    );
  return (
    <svg
      className="lab-cut"
      viewBox={plane === "perpendicular" ? "-4 -4 8 8" : `-4 ${Math.min(-4, ...cut.regions.map(r => -r.maxU))} 8 ${Math.max(4, ...cut.regions.map(r => -r.minU)) - Math.min(-4, ...cut.regions.map(r => -r.maxU))}`}
      role="img"
      aria-label={`${plane} cut, ${cut.area.toFixed(3)} square centimetres, ${cut.channels} channels`}
    >
      <path d="M-4 0H4M0-4V4" stroke="#26475c" strokeWidth={0.015} />
      {[-3, -2, -1, 1, 2, 3].map((i) => (
        <path
          key={i}
          d={`M${i} -4V4M-4 ${i}H4`}
          stroke="#132632"
          strokeWidth={0.01}
        />
      ))}
      {cut.rows.flatMap((r, i) =>
        r.intervals.map((q, j) => {
          const dz = i > 0 ? r.z - cut.rows[i - 1].z : 0.1;
          return (
            <rect
              key={`${i}-${j}`}
              x={r.z - dz / 2}
              y={-q.hi}
              width={dz + 0.002}
              height={q.hi - q.lo}
              fill={q.component % 2 ? "#68b6e8" : "#ffc573"}
            />
          );
        }),
      )}
    </svg>
  );
}
function AreaView({
  frame,
  engineAreas,
  s,
}: {
  frame: Tract3dFrame;
  engineAreas: { s: number[]; area: number[] };
  s: number;
}) {
  const a = frame.areaFn;
  const max = Math.max(1, ...a.area, ...engineAreas.area);
  const x = (v: number) => 38 + (v / a.length) * 490;
  const y = (v: number) => 148 - (v / max) * 125;
  return (
    <svg
      className="lab-area"
      viewBox="0 0 550 180"
      role="img"
      aria-label="3D area in amber, current engine area in blue. Distance in centimetres and area in square centimetres."
    >
      <path d="M38 12V148H530" fill="none" stroke="#33566e" />
      <polyline
        points={engineAreas.s
          .map((s, i) => `${x(s)},${y(engineAreas.area[i])}`)
          .join(" ")}
        fill="none"
        stroke="#547589"
        strokeWidth={1.3}
      />
      <polyline
        points={a.s.map((s, i) => `${x(s)},${y(a.area[i])}`).join(" ")}
        fill="none"
        stroke="#ffc573"
        strokeWidth={2}
      />
      <path d={`M${x(s)} 12V148`} stroke="#f077a9" />
      {a.s.map((s, i) =>
        a.channels[i] > 1 ? (
          <circle key={i} cx={x(s)} cy={y(a.area[i])} r={2.5} fill="#68b6e8" />
        ) : null,
      )}
      <g fill="#8bacc0" fontSize={12}>
        <text x={4} y={18}>
          {max.toFixed(1)}
        </text>
        <text x={6} y={147}>
          0
        </text>
        <text x={38} y={170}>
          0 cm
        </text>
        <text x={460} y={170}>
          {a.length.toFixed(2)} cm
        </text>
        <text x={45} y={18}>
          cm²
        </text>
      </g>
    </svg>
  );
}
export function LabPanel() {
  const body = useStore((s) => s.body);
  const artic = useStore((s) => s.artic);
  const lab = useStore((s) => s.lab);
  const setLab = useStore((s) => s.setLab);
  const setArtic = useStore((s) => s.setArtic);
  const start = useStore((s) => s.start);
  const hold = useStore((s) => s.hold);
  const setHold = useStore((s) => s.setHold);
  const starting = useStore((s) => s.starting);
  const startError = useStore((s) => s.startError);
  const [live, setLive] = useState<HumanArtic | null>(null);
  const [soundError, setSoundError] = useState<string | null>(null);
  const [ipa, setIpa] = useState("asa");
  const serial = useRef(0);
  const model = useModel().current!;
  const specs = useMemo(() => tract3dSpecs(), []);
  const presets = useMemo(() => tract3dPresets(body), [body]);
  useEffect(() => {
    if (!lab.follow) {
      setLive(null);
      return;
    }
    let key = "";
    const id = setInterval(() => {
      const t = engine.latest;
      if (!t || !sameBody(t.body, body)) return;
      const next = VoiceModel.articFromArray(t.t.artic);
      const k = JSON.stringify(next);
      if (k !== key) {
        key = k;
        setLive(next);
      }
    }, 100);
    return () => clearInterval(id);
  }, [lab.follow, body]);
  useEffect(
    () => () => {
      serial.current++;
      stopSpeaking();
      useStore.getState().setHold(false);
    },
    [],
  );
  const a = lab.follow && live ? live : artic;
  const result = useMemo(() => {
    try {
      return {
        frame: model.geometry(a, lab.shape, {
          x: lab.position,
          y: lab.position,
        }),
        error: null,
      };
    } catch (e) {
      return {
        frame: null,
        error:
          e instanceof Error ? e.message : "Geometry could not be computed",
      };
    }
  }, [model, a, lab.shape, lab.position]);
  const engineAreas = useMemo(() => {
    const a0 = model.areas(a);
    const s: number[] = [];
    const area: number[] = [];
    for (let i = 0; i < a0.phAreas.length; i++) {
      s.push(((i + 0.5) * a0.phLen) / a0.phAreas.length);
      area.push(a0.phAreas[i]);
    }
    for (let i = 0; i < a0.orAreas.length; i++) {
      s.push(a0.phLen + ((i + 0.5) * a0.orLen) / a0.orAreas.length);
      area.push(a0.orAreas[i]);
    }
    return { s, area };
  }, [model, a]);
  const pages: EncoderPage[] = [];
  for (const element of [...new Set(specs.map((s) => s.element))]) {
    const ss = specs.filter((s) => s.element === element);
    for (let i = 0; i < ss.length; i += 4)
      pages.push({
        name: `${label(element)}${ss.length > 4 ? ` ${i / 4 + 1}` : ""}`,
        encoders: ss
          .slice(i, i + 4)
          .map((s) => ({
            label: label(s.key),
            value: lab.shape[s.key] ?? s.default,
            min: s.min,
            max: s.max,
            onChange: (v) =>
              setLab({ shape: { ...lab.shape, [s.key]: v }, follow: false }),
            format: (v) => ({ value: v.toFixed(2), unit: s.unit }),
          })),
      });
  }
  for (let i = 0; i < ARTIC_KEYS.length; i += 4)
    pages.push({
      name: `Midline ${i / 4 + 1}`,
      encoders: ARTIC_KEYS.slice(i, i + 4).map((k) => ({
        label: label(k),
        value: artic[k],
        min: ARTIC_SPECS[k].min,
        max: ARTIC_SPECS[k].max,
        onChange: (v) => {
          setLab({ follow: false });
          setArtic({ [k]: v }, false);
        },
        format: (v) => ({ value: v.toFixed(2), unit: "" }),
      })),
    });
  const frame = result.frame;
  if (!frame)
    return (
      <Screen title="Lab">
        <p role="alert">{result.error}</p>
        <button onClick={() => setLab({ shape: {} })}>Reset shape</button>
      </Screen>
    );
  const index = Math.min(
    frame.slices.length - 1,
    Math.round(lab.slice * (frame.slices.length - 1)),
  );
  const cut =
    lab.plane === "coronal"
      ? frame.coronal
      : lab.plane === "axial"
        ? frame.axial
        : frame.slices[index];
  const select = (name: string) => {
    const p = presets.find((p) => p.name === name)!;
    setLab({ shape: p.shape, follow: false });
    setArtic(p.artic, false);
  };
  const sustain = async () => {
    const token = ++serial.current;
    if (hold) {
      setHold(false);
      return;
    }
    await start();
    if (
      token === serial.current &&
      useStore.getState().mode === "lab" &&
      useStore.getState().started
    )
      setHold(true);
  };
  const say = async () => {
    const token = ++serial.current;
    setSoundError(null);
    setHold(false);
    await start();
    if (
      token !== serial.current ||
      useStore.getState().mode !== "lab" ||
      !sameBody(useStore.getState().body, body) ||
      !useStore.getState().started
    )
      return;
    setLab({ follow: true });
    try {
      await speakText(ipa, {
        ipa: true,
        body,
        rate: 1,
        f0: useStore.getState().source.f0,
      });
    } catch {
      setSoundError("Speech could not start. Try Say again.");
    }
  };
  return (
    <div className="lab-layout">
      <Screen
        title="Tract"
        sub="Parametric tissues / cm"
        className="lab-tract"
        focus
      >
        <div className="lab-toolbar">
          <Chip
            on={lab.cutaway}
            onClick={() => setLab({ cutaway: !lab.cutaway })}
          >
            Cut away
          </Chip>
          <Chip
            on={lab.explode}
            onClick={() => setLab({ explode: !lab.explode })}
          >
            Explode
          </Chip>
          <Chip toggle={false} onClick={() => setLab({ hidden: [] })}>
            Show all
          </Chip>
        </div>
        <Scene frame={frame} cut={cut} />
        <p className="lab-hint">
          Drag to rotate · scroll or pinch to zoom · amber marks the cut
        </p>
        <div className="lab-elements">
          {frame.meshes.map((m, i) => (
            <div key={m.element}>
              <Chip
                on={!lab.hidden.includes(m.element)}
                onClick={() =>
                  setLab({
                    hidden: lab.hidden.includes(m.element)
                      ? lab.hidden.filter((e) => e !== m.element)
                      : [...lab.hidden, m.element],
                  })
                }
                style={{ color: `#${COLORS[i % COLORS.length].toString(16)}` }}
              >
                {label(m.element)}
              </Chip>
              <button
                className="chip lab-solo"
                aria-label={`Solo ${label(m.element)}`}
                onClick={() =>
                  setLab({
                    hidden: frame.meshes
                      .filter((x) => x.element !== m.element)
                      .map((x) => x.element),
                  })
                }
              >
                solo
              </button>
            </div>
          ))}
        </div>
      </Screen>
      <Screen title="Cut" sub={lab.plane} className="lab-slice">
        <div className="lab-toolbar">
          {(["perpendicular", "midsagittal", "coronal", "axial"] as const).map(
            (plane) => (
              <Chip
                key={plane}
                on={lab.plane === plane}
                onClick={() => setLab({ plane })}
              >
                {plane}
              </Chip>
            ),
          )}
        </div>
        <CutView frame={frame} cut={cut} />
        {lab.plane !== "midsagittal" && (
          <p className="lab-readout">
            {cut.area.toFixed(3)} cm² · {cut.channels} channels ·{" "}
            {cut.width.toFixed(2)} × {cut.height.toFixed(2)} cm
            {cut.fold ? " · crossing cuts" : ""}
          </p>
        )}
        <label className="lab-position">
          {lab.plane === "perpendicular"
            ? `Distance ${cut.s.toFixed(2)} cm`
            : `Plane ${lab.position.toFixed(2)} cm`}
          <input
            aria-label="Cut position"
            type="range"
            min={0}
            max={lab.plane === "perpendicular" ? 1 : 12}
            step={0.01}
            value={lab.plane === "perpendicular" ? lab.slice : lab.position}
            onChange={(e) =>
              setLab(
                lab.plane === "perpendicular"
                  ? { slice: Number(e.target.value) }
                  : { position: Number(e.target.value) },
              )
            }
          />
        </label>
      </Screen>
      <Screen title="Area" sub="3D / current engine" className="lab-af">
        <AreaView frame={frame} engineAreas={engineAreas} s={cut.s} />
        <p className="lab-readout">
          {frame.areaFn.length.toFixed(2)} cm · {frame.areaFn.volume.toFixed(1)}{" "}
          cm³ · min {frame.areaFn.minArea.toFixed(3)} cm² at{" "}
          {frame.areaFn.minS.toFixed(2)} cm
        </p>
        <p className="lab-hint">
          Amber: 3D · blue: engine · dots: multiple channels
        </p>
      </Screen>
      <Screen
        title="Shape"
        sub="Lateral offsets + midline"
        className="lab-shape"
      >
        <div className="lab-presets">
          {presets.map((p) => (
            <Chip key={p.name} toggle={false} onClick={() => select(p.name)}>
              {p.name}
            </Chip>
          ))}
          <Chip toggle={false} onClick={() => setLab({ shape: {} })}>
            Reset offsets
          </Chip>
        </div>
        <EncoderPages label="Shape" pages={pages} />
      </Screen>
      <Screen title="Sound" sub="Current engine" className="lab-sound">
        <div className="lab-spectrum">
          <SpectrumView />
        </div>
        <p className="lab-hint">
          Sound: current engine. Shape-only parameters are heard from phase 2.
        </p>
        <div className="lab-toolbar">
          <Chip on={hold} onClick={() => void sustain()} disabled={starting}>
            Hold
          </Chip>
          <Chip on={lab.follow} onClick={() => setLab({ follow: !lab.follow })}>
            Follow live
          </Chip>
          <Chip
            toggle={false}
            onClick={() => {
              serial.current++;
              stopSpeaking();
              setHold(false);
            }}
          >
            Stop
          </Chip>
        </div>
        <label className="lab-say">
          IPA
          <input
            value={ipa}
            onChange={(e) => setIpa(e.target.value)}
            aria-label="Lab IPA"
            maxLength={120}
          />
          <button className="chip" disabled={starting} onClick={() => void say()}>
            Say
          </button>
        </label>
        {(startError || soundError) && (
          <p className="error" role="alert">
            {soundError ?? "Audio could not start. Try Hold or Say again."}
          </p>
        )}
      </Screen>
    </div>
  );
}
