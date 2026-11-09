/**
 * Lab mode: the engine's parametric 3D vocal tract. TRACT shows the elements in 3D, CUT one
 * cross-section to scale, AREA the area function against the one the voice uses now, SHAPE the
 * controls (presets, every element's 3D parameters, the 17 midline articulators) and SOUND the
 * current engine for the same articulation. The 3D-only parameters do not reach the voice in this
 * phase: the voice hears the articulators, as in Play.
 */
import { useEffect, useMemo, useRef, useState } from "react";
import { tract3dSpecs, tract3dPresets, ARTIC_SPECS, type HumanArtic, type Tract3dFrame, type Tract3dWant } from "../../wasm/engine.ts";
import { useStore, type LabState } from "../store.ts";
import { useModel } from "../useModel.ts";
import { buildHead, buildSection } from "../render/head3d.ts";
import { sameBody } from "../body.ts";
import { VoiceModel } from "../analysis.ts";
import { engine } from "../../audio/AudioEngine.ts";
import { speakText, stopSpeaking } from "../speech.ts";
import { Screen, Chip } from "./Screen.tsx";
import { EncoderPages, type EncoderPage } from "./Encoder.tsx";
import { SpectrumView } from "./SpectrumView.tsx";
import { LabScene, type View } from "./lab/LabScene.tsx";
import { LabCutView, type CutDrawing } from "./lab/LabCutView.tsx";
import { LabAreaView } from "./lab/LabAreaView.tsx";
import { ELEMENTS, MIDLINE, MIDLINE_PAGES, offsetFor, paramLabel, shapePages } from "../lab.ts";

const PLANES: { key: LabState["plane"]; label: string }[] = [
  { key: "perpendicular", label: "Along tract" },
  { key: "sagittal", label: "Sagittal" },
  { key: "coronal", label: "Coronal" },
  { key: "axial", label: "Axial" },
];
/** each plane's slider: template cm */
const RANGES = { sagittal: [-2.8, 2.8, "Lateral"], coronal: [-1, 8.5, "Front–back"], axial: [1, 10.5, "Height"] } as const;

function fmt(v: number, d = 2) {
  return v.toFixed(d);
}

export function LabPanel() {
  const body = useStore((s) => s.body);
  const artic = useStore((s) => s.artic);
  const lab = useStore((s) => s.lab);
  const setLab = useStore((s) => s.setLab);
  const setArtic = useStore((s) => s.setArtic);
  const model = useModel().current!;
  const specs = useMemo(() => tract3dSpecs(), []);
  const presets = useMemo(() => tract3dPresets(body), [body]);
  const [live, setLive] = useState<HumanArtic | null>(null);
  const [view, setView] = useState<{ name: View; serial: number }>({ name: "three", serial: 0 });
  const [preset, setPreset] = useState<string | null>(null);

  // follow the live voice: its articulators from telemetry, ten times a second
  useEffect(() => {
    if (!lab.follow) {
      setLive(null);
      return;
    }
    let key = "";
    const id = setInterval(() => {
      const t = engine.latest;
      if (!t || !sameBody(t.body, useStore.getState().body)) return;
      const next = VoiceModel.articFromArray(t.t.artic);
      const k = JSON.stringify(next);
      if (k !== key) {
        key = k;
        setLive(next);
      }
    }, 100);
    return () => clearInterval(id);
  }, [lab.follow]);

  const a = lab.follow && live ? live : artic;
  // the posture's geometry and meshes; the selected cut and plane come with it
  const result = useMemo(() => {
    const want: Tract3dWant = { meshes: true, cut: lab.cut };
    if (lab.plane !== "perpendicular") {
      want.plane = lab.plane;
      want.at = lab.at[lab.plane];
    }
    try {
      return { frame: model.geometry(a, lab.shape, want), error: null };
    } catch (e) {
      return { frame: null, error: e instanceof Error ? e.message : String(e) };
    }
  }, [model, a, lab.shape, lab.cut, lab.plane, lab.at]);
  // the head around the tract (display only), rebuilt with the posture
  const head = useMemo(() => (lab.head ? buildHead(model, a) : null), [model, a, lab.head]);
  const section = useMemo(() => (lab.head && lab.cutaway ? buildSection(model, a) : null), [model, a, lab.head, lab.cutaway]);
  const engineAreas = useMemo(() => {
    const e = model.areas(a);
    const s: number[] = [];
    const area: number[] = [];
    e.phAreas.forEach((v, i) => {
      s.push(((i + 0.5) * e.phLen) / e.phAreas.length);
      area.push(v);
    });
    e.orAreas.forEach((v, i) => {
      s.push(e.phLen + ((i + 0.5) * e.orLen) / e.orAreas.length);
      area.push(v);
    });
    return { s, area };
  }, [model, a]);

  const frame = result.frame;
  if (!frame)
    return (
      <Screen title="Lab" sub="3D tract">
        <p className="lab-note" role="alert">
          The geometry could not be computed: {result.error}
        </p>
        <div className="lab-row">
          <Chip toggle={false} onClick={() => setLab({ shape: {} })}>
            Reset 3D offsets
          </Chip>
        </div>
      </Screen>
    );

  const k = model.bodyParams.scale;
  const cut = frame.cut;
  const drawing: CutDrawing | null =
    lab.plane === "perpendicular"
      ? { kind: "cut", cut, axes: ["lateral →", "↑ outer wall"] }
      : frame.plane && frame.plane.kind === "sagittal"
        ? { kind: "sagittal", plane: frame.plane }
        : frame.plane
          ? { kind: "cut", cut: frame.plane.cut, axes: ["lateral →", lab.plane === "coronal" ? "↑ up" : "↑ front"] }
          : null;
  const shown = lab.plane === "perpendicular" || !frame.plane || frame.plane.kind === "sagittal" ? cut : frame.plane.cut;

  // ---- controls: the focused element's 3D parameters, or the midline articulators
  const encoderPages: EncoderPage[] =
    lab.element === MIDLINE
      ? MIDLINE_PAGES.map((p) => ({
          name: p.name,
          encoders: p.keys.map((key) => ({
            label: paramLabel(key),
            value: a[key],
            min: ARTIC_SPECS[key].min,
            max: ARTIC_SPECS[key].max,
            onChange: (v: number) => {
              setPreset(null);
              setLab({ follow: false });
              setArtic({ [key]: v }, false);
            },
            format: (v: number) => ({ value: fmt(v), unit: key === "lipWidth" || key === "lateral" || key === "ttCurl" || key === "dorsum" || key === "epilarynx" || key === "ventricular" || key === "labiodental" ? "" : "cm" }),
          })),
        }))
      : shapePages(specs, lab.element).map((p) => ({
          name: p.name,
          encoders: p.specs.map((sp) => ({
            label: paramLabel(sp.key),
            value: frame.shape[sp.key] ?? sp.default,
            min: sp.min,
            max: sp.max,
            onChange: (v: number) => {
              setPreset(null);
              setLab({ shape: { ...lab.shape, [sp.key]: offsetFor(v, frame.derived[sp.key] ?? sp.default, sp.default) } });
            },
            format: (v: number) => ({ value: fmt(v), unit: sp.unit }),
          })),
        }));
  const element = ELEMENTS.find((e) => e.key === lab.element);
  const changed = Object.keys(lab.shape).filter((key) => {
    const sp = specs.find((s) => s.key === key);
    return sp && Math.abs(lab.shape[key] - sp.default) > 1e-9;
  });

  const select = (name: string) => {
    const p = presets.find((x) => x.name === name);
    if (!p) return;
    setPreset(name);
    setLab({ shape: p.shape, follow: false });
    setArtic(p.artic, false);
  };
  const cavity = (kind: string) => frame.cavities.filter((c) => c.kind === kind);
  const pir = cavity("piriform");

  return (
    <div className="lab-layout">
      <Screen
        title="Tract"
        sub={`3D model · ${lab.follow ? "following the voice" : preset ? (presets.find((p) => p.name === preset)?.label ?? preset) : "your posture"}`}
        className="lab-tract"
        focus
        label="3D tract"
        tools={
          <>
            {(["side", "front", "top", "three"] as View[]).map((v) => (
              <Chip key={v} toggle={false} onClick={() => setView((x) => ({ name: v, serial: x.serial + 1 }))} aria-label={`View from the ${v === "three" ? "front and side" : v}`}>
                {v === "three" ? "3/4" : v}
              </Chip>
            ))}
          </>
        }
      >
        <LabScene frame={frame} cut={lab.plane === "perpendicular" ? cut : shown} lab={lab} view={view} scale={k} head={head} section={section} />
        <div className="lab-row lab-elements" role="group" aria-label="Elements shown">
          {ELEMENTS.map((e) => (
            <Chip
              key={e.key}
              on={!lab.hidden.includes(e.key)}
              style={{ "--el": e.color } as React.CSSProperties}
              className="lab-el"
              title={e.about}
              onClick={() => setLab({ hidden: lab.hidden.includes(e.key) ? lab.hidden.filter((x) => x !== e.key) : [...lab.hidden, e.key] })}
            >
              {e.label}
            </Chip>
          ))}
        </div>
        <div className="lab-row" role="group" aria-label="3D view options">
          <Chip on={lab.head} onClick={() => setLab({ head: !lab.head })} title="The head around the tract: skin, fat, muscle, skull and brain, jaw, spine, hyoid and larynx">
            Head
          </Chip>
          <Chip on={lab.cutaway} onClick={() => setLab({ cutaway: !lab.cutaway })} title="Hide the right half to look inside (the head's layers show where they are cut)">
            Cut away
          </Chip>
          <Chip on={lab.seeThrough} onClick={() => setLab({ seeThrough: !lab.seeThrough })} disabled={!lab.head || lab.cutaway} title="Make the skin translucent (with the head whole)">
            See-through
          </Chip>
          <Chip on={lab.xray} onClick={() => setLab({ xray: !lab.xray })} title="Show the lines behind surfaces too">
            X-ray
          </Chip>
          <Chip on={lab.explode} onClick={() => setLab({ explode: !lab.explode })} title="Pull the elements apart">
            Explode
          </Chip>
          <Chip
            toggle={false}
            onClick={() => setLab({ hidden: ELEMENTS.filter((e) => e.key !== lab.element && lab.element !== MIDLINE).map((e) => e.key) })}
            disabled={lab.element === MIDLINE}
            aria-label={element ? `Show only ${element.label}` : "Show only the selected element"}
          >
            Only {element?.label ?? "selected"}
          </Chip>
          <Chip toggle={false} onClick={() => setLab({ hidden: [] })}>
            Show all
          </Chip>
        </div>
        <p className="lab-note">
          Drag to turn · wheel or pinch to zoom · amber: the cut, pink: the centreline.
          {lab.head && lab.cutaway ? " Cut faces: skin, fat (yellow), muscle (red), bone (ivory), cartilage (pale blue), brain (grey-pink)." : ""}
        </p>
      </Screen>

      <Screen title="Cut" sub={PLANES.find((p) => p.key === lab.plane)!.label} className="lab-cut-screen" label="Cross-section">
        <div className="lab-row" role="group" aria-label="Plane">
          {PLANES.map((p) => (
            <Chip key={p.key} on={lab.plane === p.key} onClick={() => setLab({ plane: p.key })}>
              {p.label}
            </Chip>
          ))}
        </div>
        <div className="lab-canvas-box lab-cut-box">
          <LabCutView drawing={drawing} size={lab.plane === "perpendicular" ? 7 : 8} />
        </div>
        <label className="lab-slider">
          <span>{lab.plane === "perpendicular" ? `At ${fmt(cut.s, 1)} cm from the glottis` : `${RANGES[lab.plane][2]} ${fmt(lab.at[lab.plane], 1)} cm`}</span>
          <input
            type="range"
            aria-label="Cut position"
            min={lab.plane === "perpendicular" ? 0 : RANGES[lab.plane][0]}
            max={lab.plane === "perpendicular" ? 1 : RANGES[lab.plane][1]}
            step={lab.plane === "perpendicular" ? 0.005 : 0.05}
            value={lab.plane === "perpendicular" ? lab.cut : lab.at[lab.plane]}
            onChange={(e) => {
              const v = Number(e.target.value);
              if (lab.plane === "perpendicular") setLab({ cut: v });
              else setLab({ at: { ...lab.at, [lab.plane]: v } });
            }}
          />
        </label>
        {lab.plane === "sagittal" ? (
          <p className="lab-readout">Air in the plane {fmt(Math.abs(lab.at.sagittal), 1)} cm {lab.at.sagittal < 0 ? "left of" : lab.at.sagittal > 0 ? "right of" : "on"} the midline.</p>
        ) : (
          <p className="lab-readout">
            <b>{fmt(shown.area, 2)} cm²</b> · {shown.channels} {shown.channels === 1 ? "channel" : "channels"} · {fmt(shown.width, 1)} × {fmt(shown.height, 1)} cm
            {shown.fold ? " · neighbouring cuts cross at this bend" : ""}
            {shown.regions.length > 1 ? ` · regions ${shown.regions.map((r) => `${r.tag} ${fmt(r.area)}`).join(", ")} cm²` : ""}
          </p>
        )}
      </Screen>

      <Screen title="Area" sub="3D model / voice now" className="lab-area-screen" label="Area function">
        <div className="lab-canvas-box lab-area-box">
          <LabAreaView frame={frame} engine={engineAreas} cut={lab.cut} onCut={(c) => setLab({ cut: c, plane: "perpendicular" })} />
        </div>
        <p className="lab-readout">
          {fmt(frame.areaFn.length, 1)} cm long · {fmt(frame.areaFn.volume, 1)} cm³ · narrowest {fmt(frame.areaFn.minArea, 2)} cm² at {fmt(frame.areaFn.minS, 1)} cm
          {pir.length ? ` · piriform fossae ${pir.map((c) => fmt(c.volume, 1)).join(" + ")} cm³` : ""}
        </p>
        <p className="lab-note">Amber: the 3D model. Blue: the area function the voice uses now (its 1D area law). Dots: cuts with several channels.</p>
      </Screen>

      <Screen title="Shape" sub={lab.element === MIDLINE ? "Midline articulators" : `${element?.label ?? ""} · 3D parameters`} className="lab-shape" label="Shape controls">
        <div className="lab-row lab-presets" role="group" aria-label="Postures">
          {presets.map((p) => (
            <Chip key={p.name} on={preset === p.name} onClick={() => select(p.name)} aria-label={p.label} title={p.label}>
              {p.name}
            </Chip>
          ))}
        </div>
        <div className="lab-row" role="group" aria-label="Controls for">
          {ELEMENTS.map((e) => (
            <Chip key={e.key} on={lab.element === e.key} onClick={() => setLab({ element: e.key })} style={{ "--el": e.color } as React.CSSProperties} className="lab-el">
              {e.label}
            </Chip>
          ))}
          <Chip on={lab.element === MIDLINE} onClick={() => setLab({ element: MIDLINE })}>
            Midline
          </Chip>
        </div>
        <EncoderPages key={lab.element} label={lab.element === MIDLINE ? "Midline" : (element?.label ?? "Shape")} pages={encoderPages} />
        <p className="lab-note">
          {lab.element === MIDLINE
            ? "The engine's 17 articulators: the 3D shape follows them, and so does the voice."
            : `${element?.about ?? ""} Values shown are in use: what the articulators imply plus your offset.`}
        </p>
        <div className="lab-row">
          <Chip toggle={false} onClick={() => setLab({ shape: {} })} disabled={changed.length === 0}>
            Reset 3D offsets{changed.length ? ` (${changed.length})` : ""}
          </Chip>
        </div>
      </Screen>

      <LabSound />
    </div>
  );
}

/** The current engine for the posture shown: hold it, say IPA, or let the 3D model follow. */
function LabSound() {
  const body = useStore((s) => s.body);
  const lab = useStore((s) => s.lab);
  const setLab = useStore((s) => s.setLab);
  const start = useStore((s) => s.start);
  const hold = useStore((s) => s.hold);
  const setHold = useStore((s) => s.setHold);
  const starting = useStore((s) => s.starting);
  const startError = useStore((s) => s.startError);
  const [ipa, setIpa] = useState("asa");
  const [error, setError] = useState<string | null>(null);
  const serial = useRef(0);
  useEffect(
    () => () => {
      serial.current++;
      stopSpeaking();
      useStore.getState().setHold(false);
    },
    [],
  );
  const sustain = async () => {
    const token = ++serial.current;
    setError(null);
    if (hold) {
      setHold(false);
      return;
    }
    await start();
    const s = useStore.getState();
    if (token === serial.current && s.mode === "lab" && s.started) setHold(true);
  };
  const say = async () => {
    const token = ++serial.current;
    setError(null);
    setHold(false);
    await start();
    const s = useStore.getState();
    if (token !== serial.current || s.mode !== "lab" || !sameBody(s.body, body) || !s.started) return;
    setLab({ follow: true });
    try {
      await speakText(ipa, { ipa: true, body, rate: 1, f0: s.source.f0 });
    } catch {
      setError("Speech could not start. Try Say again.");
    }
  };
  return (
    <Screen title="Sound" sub="Current engine" className="lab-sound" label="Sound">
      <div className="lab-canvas-box lab-spec-box">
        <SpectrumView />
      </div>
      <div className="lab-row" role="group" aria-label="Voice">
        <Chip on={hold} onClick={() => void sustain()} disabled={starting}>
          Hold
        </Chip>
        <Chip on={lab.follow} onClick={() => setLab({ follow: !lab.follow })} title="The 3D model follows the voice's articulators">
          Follow voice
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
      <div className="lab-say">
        <label htmlFor="lab-ipa">IPA</label>
        <input id="lab-ipa" value={ipa} onChange={(e) => setIpa(e.target.value)} maxLength={120} spellCheck={false} />
        <Chip toggle={false} disabled={starting} onClick={() => void say()}>
          Say
        </Chip>
      </div>
      {(startError || error) && (
        <p className="error" role="alert">
          {error ?? "Audio could not start. Try Hold or Say again."}
        </p>
      )}
      <p className="lab-note">The voice is the current engine: it hears the 17 articulators. The 3D-only parameters (margins, grooves, widths) reach the sound in phase 2.</p>
    </Screen>
  );
}
