/**
 * Lab mode: the engine's parametric 3D vocal tract inside the head. TRACT shows it in 3D (the
 * head whole, opened along the midline, or the tract alone) and holds the listening controls;
 * CUT one cross-section to scale; AREA the area function against the one the voice uses now;
 * SHAPE the controls (postures, every element's 3D parameters, the 17 midline articulators);
 * SOUND the spectrum. The voice hears the 17 articulators, as in Play; the 3D-only parameters
 * (margins, grooves, widths) do not reach it yet.
 */
import { useEffect, useMemo, useRef, useState } from "react";
import { tract3dSpecs, tract3dPresets, ARTIC_SPECS, type HumanArtic, type Tract3dWant } from "../../wasm/engine.ts";
import { useStore, type LabState } from "../store.ts";
import { useModel } from "../useModel.ts";
import { sameBody } from "../body.ts";
import { VoiceModel } from "../analysis.ts";
import { engine } from "../../audio/AudioEngine.ts";
import { speakText, stopSpeaking } from "../speech.ts";
import { Screen, Chip } from "./Screen.tsx";
import { StartPrompt } from "./StartPrompt.tsx";
import { EncoderPages, type EncoderPage } from "./Encoder.tsx";
import { SpectrumView } from "./SpectrumView.tsx";
import { LabScene, type View } from "./lab/LabScene.tsx";
import { LabCutView, type CutDrawing } from "./lab/LabCutView.tsx";
import { LabAreaView } from "./lab/LabAreaView.tsx";
import { ELEMENTS, MIDLINE, MIDLINE_PAGES, offsetFor, paramLabel, shapePages, spokenForm } from "../lab.ts";

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
  // the posture's cuts and area function (the 3D view asks the engine for its own meshes)
  const result = useMemo(() => {
    const want: Tract3dWant = { meshes: false, cut: lab.cut };
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
    // a consonant is heard in a word: with the audio running, say it between two vowels
    const spoken = spokenForm(name);
    if (spoken && useStore.getState().started) void sayInLab(spoken, true);
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
            {(["side", "front", "three", "mouth"] as View[]).map((v) => (
              <Chip key={v} toggle={false} onClick={() => setView((x) => ({ name: v, serial: x.serial + 1 }))} aria-label={v === "three" ? "View from the front and side" : v === "mouth" ? "View of the mouth" : `View from the ${v}`}>
                {v === "three" ? "3/4" : v}
              </Chip>
            ))}
          </>
        }
      >
        <div className="lab-stage-box">
          <LabScene model={model} artic={a} frame={frame} cut={lab.plane === "perpendicular" ? cut : shown} plane={lab.plane === "sagittal" ? frame.plane : null} lab={lab} view={view} />
          <StartPrompt />
        </div>
        <LabListen />
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
          <Chip on={lab.head} onClick={() => setLab({ head: !lab.head })} title="The head round the tract">
            Head
          </Chip>
          <Chip on={lab.cutaway} onClick={() => setLab({ cutaway: !lab.cutaway })} title="Hide the near half to look inside (the head's layers show on the cut)">
            Cut away
          </Chip>
          <Chip on={lab.seeThrough} onClick={() => setLab({ seeThrough: !lab.seeThrough })} disabled={!lab.head || lab.cutaway} title="Make the skin translucent (with the head whole)">
            See-through
          </Chip>
          <Chip on={lab.lines} onClick={() => setLab({ lines: !lab.lines })} disabled={lab.element === MIDLINE} title="Draw the selected element's grid over its surface">
            Outline
          </Chip>
          <Chip on={lab.explode} onClick={() => setLab({ explode: !lab.explode })} disabled={lab.head} title="Pull the elements apart (with the head off)">
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
          {lab.head && lab.cutaway ? " On the cut: skin, fat (yellow), muscle (red), bone (ivory), cartilage (pale blue), brain (grey-pink); the hole is the tract's air." : ""}
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
            {shown.fold ? " · on the bend (this cut overlaps the next)" : ""}
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

/** Say `text` in the lab: the 3D model follows the voice while it speaks, then shows the posture again. */
let saying = 0;
async function sayInLab(text: string, ipa: boolean): Promise<boolean> {
  const token = ++saying;
  const s = useStore.getState();
  const held = s.hold;
  // speech breathes by itself: a held breath would drone under it
  s.setHold(false);
  await s.start();
  const now = useStore.getState();
  if (token !== saying || now.mode !== "lab" || !now.started) return true;
  now.setLab({ follow: true });
  let plan: Awaited<ReturnType<typeof speakText>> = null;
  try {
    plan = await speakText(text, { ipa, body: now.body, rate: 1, f0: now.source.f0 });
  } catch {
    now.setLab({ follow: false });
    return false;
  }
  if (!plan) {
    now.setLab({ follow: false });
    return false;
  }
  // when the phrase is over, back to the posture (and to the held voice, if it was held)
  const end = plan.plan.end + 0.25;
  setTimeout(() => {
    if (token !== saying) return;
    const t = useStore.getState();
    if (t.mode !== "lab") return;
    t.setLab({ follow: false });
    if (held) t.setHold(true);
  }, end * 1000);
  return true;
}

/** Listening, right under the 3D view: hold the voice on the posture, or say a word or IPA. */
function LabListen() {
  const hold = useStore((s) => s.hold);
  const setHold = useStore((s) => s.setHold);
  const start = useStore((s) => s.start);
  const starting = useStore((s) => s.starting);
  const startError = useStore((s) => s.startError);
  const setLab = useStore((s) => s.setLab);
  const [text, setText] = useState("la la la");
  const [ipa, setIpa] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const serial = useRef(0);
  useEffect(
    () => () => {
      saying++;
      stopSpeaking();
      useStore.getState().setHold(false);
    },
    [],
  );
  const stop = () => {
    serial.current++;
    saying++;
    stopSpeaking();
    setHold(false);
    setLab({ follow: false });
  };
  const listen = async () => {
    const token = ++serial.current;
    setError(null);
    if (hold) return stop();
    saying++;
    stopSpeaking();
    setLab({ follow: false });
    await start();
    const s = useStore.getState();
    if (token === serial.current && s.mode === "lab" && s.started) setHold(true);
  };
  const say = async () => {
    serial.current++;
    setError(null);
    if (!(await sayInLab(text, ipa))) setError(ipa ? "That IPA could not be spoken. Try asa or iʃi." : "Nothing to say. Type a word.");
  };
  return (
    <>
      <div className="lab-row lab-listen" role="group" aria-label="Listen">
        <Chip on={hold} tone="amber" onClick={() => void listen()} disabled={starting} title="Hold the voice on the posture shown">
          Listen
        </Chip>
        <Chip toggle={false} onClick={stop}>
          Stop
        </Chip>
        <form
          className="lab-say"
          onSubmit={(e) => {
            e.preventDefault();
            void say();
          }}
        >
          <input aria-label={ipa ? "IPA to say" : "Words to say"} value={text} onChange={(e) => setText(e.target.value)} maxLength={120} spellCheck={false} />
          <Chip
            on={ipa}
            onClick={() => {
              setIpa(!ipa);
              setText(ipa ? "la la la" : "asa");
            }}
            title="Type IPA instead of words"
          >
            IPA
          </Chip>
          <button type="submit" className="chip amber" disabled={starting}>
            Say
          </button>
        </form>
      </div>
      {(startError || error) && (
        <p className="error lab-note" role="alert">
          {error ?? "Audio could not start. Press Listen again."}
        </p>
      )}
    </>
  );
}

/** The voice's spectrum for the posture, and whether the 3D model follows the live voice. */
function LabSound() {
  const lab = useStore((s) => s.lab);
  const setLab = useStore((s) => s.setLab);
  return (
    <Screen title="Sound" sub="Resonances" className="lab-sound" label="Sound">
      <div className="lab-canvas-box lab-spec-box">
        <SpectrumView />
      </div>
      <div className="lab-row" role="group" aria-label="Voice">
        <Chip on={lab.follow} onClick={() => setLab({ follow: !lab.follow })} title="The 3D model follows the voice's articulators">
          Follow voice
        </Chip>
      </div>
      <p className="lab-note">Listen and Say are under the 3D view. The voice hears the 17 articulators; the 3D-only parameters (margins, grooves, widths) do not reach the sound yet.</p>
    </Screen>
  );
}
