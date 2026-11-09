//! The main thread's JSON calls: `chimera_call` takes `{"fn": name, ...args}` as UTF-8 JSON in a
//! buffer from `chimera_alloc`, and leaves the JSON result (or `{"error": message}`) in a buffer
//! read with `chimera_result_ptr`/`chimera_result_len`, valid until the next call. Nothing here
//! runs on the audio thread: these calls allocate.

use chimera::airway::frequency::{find_peaks, transfer_function};
use chimera::anatomy::human::{BodyMix, HumanArtic, HumanFrame};
use chimera::anatomy::geom::v;
use chimera::anatomy::tract3d::{Cut, Row, ShapeParams, Tract3d, Tract3dFrame};
use chimera::control::beat_player::BeatPlayer;
use chimera::creatures::voice::CreatureVoice;
use chimera::math;
use chimera::phonetics::beatbox::{self, BeatStep};
use chimera::phonetics::planner::{self, PhraseInput, PlanOptions};
use chimera::phonetics::sing::{self, MelismaOptions, NoteOnOptions, SongOptions, SungNote};
use chimera::phonetics::english::EnglishG2P;
use chimera::phonetics::{Segment, Syllable, ipa, songs, syllables, vowel_data};
use chimera::voice::human_airway::HumanAirway;
use serde_json::{Value, json};
use std::cell::RefCell;

struct Model {
    id: usize,
    body: BodyMix,
    fs: f64,
    airway: HumanAirway,
    /// the lab's 3D geometry, made on its first use (only the lab asks for it)
    tract3d: Option<Box<(Tract3d, Tract3dFrame)>>,
}

#[derive(Default)]
struct State {
    result: Vec<u8>,
    /// the page's models; a body is continuous, so the page drops the ones it no longer keeps
    /// (`dropModel`, from `useModel.ts`)
    models: Vec<Model>,
    next_model: usize,
    /// the page's English G2P (one dictionary, loaded once)
    g2p: EnglishG2P,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

type Res = Result<Value, String>;

fn arg<T: serde::de::DeserializeOwned>(req: &Value, key: &str) -> Result<T, String> {
    serde_json::from_value(req.get(key).cloned().unwrap_or(Value::Null)).map_err(|e| format!("{key}: {e}"))
}

fn to_json<T: serde::Serialize>(v: &T) -> Res {
    serde_json::to_value(v).map_err(|e| e.to_string())
}

fn model<'a>(st: &'a mut State, req: &Value) -> Result<&'a mut Model, String> {
    let id: usize = arg(req, "model")?;
    st.models.iter_mut().find(|m| m.id == id).ok_or_else(|| format!("no model {id}"))
}

fn dispatch(st: &mut State, req: &Value) -> Res {
    let name = req.get("fn").and_then(Value::as_str).ok_or("missing fn")?;
    match name {
        "version" => Ok(json!({ "layout": crate::LAYOUT_VERSION, "crate": env!("CARGO_PKG_VERSION") })),
        "catalog" => catalog(),
        // ---------------------------------------------------------------- the anatomy model
        "model" => {
            // the main-thread mirror of the worklet's airway: same topology and node counts
            let body: BodyMix = arg(req, "body")?;
            let fs: f64 = arg(req, "sampleRate")?;
            let m = if fs > 60000.0 { 1.0 } else { 2.0 };
            let i = match st.models.iter().position(|x| x.body == body && x.fs == fs * m) {
                Some(i) => i,
                None => {
                    st.models.push(Model { id: st.next_model, body, fs: fs * m, airway: HumanAirway::new(fs * m, body.body()), tract3d: None });
                    st.next_model += 1;
                    st.models.len() - 1
                }
            };
            let (id, a) = (st.models[i].id, &st.models[i].airway);
            Ok(json!({
                "model": id,
                "body": a.body,
                "nPh": a.n_ph,
                "nOr": a.n_or,
                "pharynxBranches": { "base": a.pharynx.branch_base, "n": a.pharynx.n },
                "oralBranches": { "base": a.oral.branch_base, "n": a.oral.n },
                "glottisPort": a.glottis_port,
            }))
        }
        "dropModel" => {
            let id: usize = arg(req, "model")?;
            st.models.retain(|m| m.id != id);
            Ok(Value::Null)
        }
        "tract3dSpecs" => to_json(&Tract3d::specs()),
        "tract3dPresets" => to_json(&chimera::anatomy::tract3d::presets(arg(req, "body")?)),
        "tract3d" => tract3d(st, req),
        "restTip" => to_json(&chimera::anatomy::human::rest_tip(arg(req, "jaw")?, arg(req, "tbX")?, arg(req, "tbY")?)),
        "frame" => {
            let a: HumanArtic = arg(req, "artic")?;
            let m = model(st, req)?;
            let f: HumanFrame = m.airway.tract.compute_new(&a);
            serde_json::to_value(&f).map_err(|e| e.to_string())
        }
        "apply" => {
            let a: HumanArtic = arg(req, "artic")?;
            let m = model(st, req)?;
            m.airway.apply(&a, 0.0);
            let air = &m.airway;
            Ok(json!({ "phAreas": air.ph_areas, "orAreas": air.or_areas, "phLen": air.ph_len, "orLen": air.or_len }))
        }
        "transfer" => {
            // the vocal-tract transfer function the UI draws, and its formants: 360 log-spaced
            // frequencies 60 Hz–8 kHz, injected at the first pharynx node, the glottis port cut
            let a: HumanArtic = arg(req, "artic")?;
            let m = model(st, req)?;
            m.airway.apply(&a, 0.0);
            let el = m.airway.net.linear_elements();
            let freqs: Vec<f64> = (0..360).map(|i| 60.0 * math::pow(8000.0 / 60.0, i as f64 / 359.0)).collect();
            let h = transfer_function(&el, m.airway.pharynx.first_node(), &freqs, &[m.airway.glottis_port]);
            let db: Vec<f64> = h.mag.iter().map(|x| 20.0 * math::log10(x + 1e-12)).collect();
            let formants: Vec<f64> = find_peaks(&freqs, &h.mag, 150.0).iter().map(|p| p.freq).take(5).collect();
            Ok(json!({ "freqs": freqs, "db": db, "formants": formants }))
        }
        // ---------------------------------------------------------------- text → phonemes
        "tokenizeIpa" => to_json(&ipa::tokenize_ipa(&arg::<String>(req, "text")?)),
        "g2pLoad" => {
            st.g2p.load_dict(&arg::<String>(req, "text")?);
            Ok(json!(true))
        }
        "g2pPhrases" => to_json(&st.g2p.to_phrases(&arg::<String>(req, "text")?)),
        "g2pWord" => to_json(&st.g2p.word(&arg::<String>(req, "word")?, arg::<Option<bool>>(req, "reduce")?.unwrap_or(true))),
        "lyricsToSyllables" => to_json(&syllables::lyrics_to_syllables(&arg::<String>(req, "text")?, &st.g2p)),
        // ---------------------------------------------------------------- creatures
        "creaturePreview" => {
            let id: String = arg(req, "id")?;
            let def = chimera::creatures::creature_by_id(&id).ok_or_else(|| format!("no creature {id}"))?;
            let params: std::collections::BTreeMap<String, f64> = arg::<Option<_>>(req, "params")?.unwrap_or_default();
            let initial: Vec<(usize, f64)> = params.iter().filter_map(|(k, v)| def.param_index(k).map(|i| (i, *v))).collect();
            let mut c = CreatureVoice::new(def, 48000.0, &initial);
            let mut t = c.new_telemetry();
            c.telemetry(&mut t);
            to_json(&t)
        }
        // ---------------------------------------------------------------- planning (speech, song)
        "planUtterance" => to_json(&planner::plan_utterance(&arg::<Vec<Segment>>(req, "segments")?, &arg::<PlanOptions>(req, "options")?)),
        // how many planned segments each segment becomes (a diphthong plans as its two vowels),
        // so the UI can find a word's span in a plan's timeline
        "plannedCounts" => to_json(&arg::<Vec<String>>(req, "ipa")?.iter().map(|s| planner::expand(s).map_or(1, |p| p.len())).collect::<Vec<_>>()),
        "planPhrases" => to_json(&planner::plan_phrases(&arg::<Vec<PhraseInput>>(req, "phrases")?, &arg::<PlanOptions>(req, "options")?)),
        "planSong" => to_json(&sing::plan_song(&arg::<Vec<SungNote>>(req, "notes")?, &arg::<SongOptions>(req, "options")?)),
        "planNoteOn" => to_json(&sing::plan_note_on(&arg::<Syllable>(req, "syl")?, arg(req, "midi")?, &arg::<NoteOnOptions>(req, "options")?)),
        "planNoteOff" => to_json(&sing::plan_note_off(&arg::<Syllable>(req, "syl")?, arg(req, "midi")?, arg::<BodyMix>(req, "body")?)),
        "planMelisma" => to_json(&sing::plan_melisma(arg(req, "midi")?, &arg::<MelismaOptions>(req, "options")?)),
        // ---------------------------------------------------------------- beatbox
        "parseBeat" => to_json(&beatbox::parse_beat(&arg::<String>(req, "text")?)),
        "formatBeat" => Ok(Value::String(beatbox::format_beat(&arg::<Vec<Option<BeatStep>>>(req, "steps")?))),
        "beatReady" => to_json(&beatbox::beat_ready(arg::<BodyMix>(req, "body")?)),
        "leadIn" => Ok(json!(BeatPlayer::lead_in(&arg::<String>(req, "id")?, arg::<BodyMix>(req, "body")?))),
        _ => Err(format!("unknown fn {name}")),
    }
}

/// Everything constant the UI needs: articulators, bodies, defaults, sound catalogues.
fn catalog() -> Res {
    use chimera::anatomy::human::{ArticKey, BODY_CHILD, BODY_FEMALE, BODY_MALE, NEUTRAL};
    use chimera::control::articulators::ARTIC_SPECS;
    use chimera::voice::human_voice::DEFAULT_SOURCE;
    let specs: serde_json::Map<String, Value> = ArticKey::ALL.iter().map(|k| (k.name().to_string(), serde_json::to_value(ARTIC_SPECS[k.index()]).unwrap())).collect();
    let vowels: serde_json::Map<String, Value> = planner::vowel_table()
        .iter()
        .map(|(body, t)| (body.clone(), Value::Object(t.iter().map(|(ipa, v)| (ipa.clone(), json!({ "artic": v.artic, "formants": v.formants }))).collect())))
        .collect();
    Ok(json!({
        "neutral": NEUTRAL,
        "bodies": { "male": BODY_MALE, "female": BODY_FEMALE, "child": BODY_CHILD },
        "articKeys": ArticKey::NAMES,
        "articSpecs": specs,
        "defaultSource": DEFAULT_SOURCE,
        "exhaleLabel": planner::EXHALE_LABEL,
        "bassNote": beatbox::BASS_NOTE,
        "beatSounds": &beatbox::BEAT_SOUNDS[..],
        "beatPresets": &beatbox::BEAT_PRESETS[..],
        "vowelTargets": vowels,
        "consonantSymbols": chimera::phonetics::inventory::consonant_symbols(),
        "demoSongs": songs::DEMO_SONGS,
        "vowels": vowel_data::VOWELS,
        "creatures": &chimera::creatures::CREATURES[..],
    }))
}

/// Put a call's result (or error) in the result buffer; 0 on success, 1 on error.
pub(crate) fn set_result(out: Res) -> u32 {
    let (code, v) = match out {
        Ok(v) => (0, v),
        Err(e) => (1, json!({ "error": e })),
    };
    STATE.with(|s| s.borrow_mut().result = serde_json::to_vec(&v).unwrap_or_else(|_| b"{\"error\":\"unserialisable result\"}".to_vec()));
    code
}

/// Run one JSON call (see the module docs). Returns 0 on success, 1 if the result is an error.
///
/// # Safety
/// `ptr..ptr+len` must be readable (a buffer from `chimera_alloc`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chimera_call(ptr: *const u8, len: usize) -> u32 {
    let bytes = unsafe { core::slice::from_raw_parts(ptr, len) };
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        let out = match serde_json::from_slice::<Value>(bytes) {
            Err(e) => Err(format!("bad request: {e}")),
            Ok(req) => dispatch(&mut st, &req),
        };
        let (code, v) = match out {
            Ok(v) => (0, v),
            Err(e) => (1, json!({ "error": e })),
        };
        st.result = serde_json::to_vec(&v).unwrap_or_else(|_| b"{\"error\":\"unserialisable result\"}".to_vec());
        code
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn chimera_result_ptr() -> *const u8 {
    STATE.with(|s| s.borrow().result.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn chimera_result_len() -> usize {
    STATE.with(|s| s.borrow().result.len())
}

/// The lab's geometry for a posture: the area function and cut summaries every call, the
/// selected cut's rows, one anatomical plane, and (when asked) the element meshes.
fn tract3d(st: &mut State, req: &Value) -> Res {
    let a: HumanArtic = arg(req, "artic")?;
    let shape = arg::<Option<ShapeParams>>(req, "shape")?.unwrap_or_default();
    let want = req.get("want").cloned().unwrap_or(Value::Null);
    let m = model(st, req)?;
    let body = m.body.body();
    let geometry = m.tract3d.get_or_insert_with(|| Box::new((Tract3d::new(body), Tract3dFrame::default())));
    let (t, f) = (&mut geometry.0, &mut geometry.1);
    t.compute(&a, &shape, f)?;
    let n = f.area_fn.s.len();
    let pick = want.get("cut").and_then(Value::as_f64).unwrap_or(0.8).clamp(0., 1.);
    let cut = &f.cuts()[((n - 1) as f64 * pick).round() as usize];
    let slices: Vec<Value> = f.cuts().iter().map(|c| json!({"s": c.s, "area": c.area, "channels": c.channels, "width": c.width, "height": c.height, "fold": c.fold, "origin": c.origin, "u": c.u})).collect();
    let k = body.scale;
    let at = want.get("at").and_then(Value::as_f64);
    let plane = match want.get("plane").and_then(Value::as_str).unwrap_or("none") {
        kind @ ("coronal" | "axial") => {
            let mut c = Cut::default();
            let (o, u) = if kind == "coronal" { (v(at.unwrap_or(4.5) * k, 8. * k), v(0., 1.)) } else { (v(4. * k, at.unwrap_or(8.3) * k), v(1., 0.)) };
            t.cut_plane(o, u, &mut c);
            json!({"kind": kind, "at": at, "cut": c})
        }
        "sagittal" => {
            let z = at.unwrap_or(0.) * k;
            let mut rows = Vec::new();
            let mut row = Row::default();
            let (y0, y1) = (f.centreline[0].y - 0.5 * k, 12. * k);
            let mut y = y0;
            while y <= y1 {
                t.air_intervals(y, z, &mut row);
                if row.count > 0 {
                    rows.push(json!({"y": y, "x": row.intervals[..row.count].iter().flat_map(|q| [q.lo, q.hi]).collect::<Vec<f64>>()}));
                }
                y += 0.05 * k;
            }
            json!({"kind": "sagittal", "at": at, "step": 0.05 * k, "rows": rows})
        }
        _ => Value::Null,
    };
    let meshes = if want.get("meshes").and_then(Value::as_bool).unwrap_or(true) { to_json(&t.meshes())? } else { Value::Null };
    Ok(json!({
        "shape": f.shape,
        "derived": t.derive(&a),
        "centreline": f.centreline,
        "areaFn": f.area_fn,
        "slices": slices,
        "cut": cut,
        "plane": plane,
        "cavities": f.cavities,
        "meshes": meshes,
    }))
}
