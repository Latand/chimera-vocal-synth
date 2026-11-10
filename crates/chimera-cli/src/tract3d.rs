//! `chimera tract3d …`: exports of the parametric 3D tract and its comparison with MRI
//! references kept outside the repository. No dataset is embedded; see tools/tract3d/README.md.
mod acoustics;
mod report;
mod sheet;
mod usc;
mod volume;
use chimera::anatomy::human::{ArticKey, BodyMix, HumanArtic};
use chimera::anatomy::tract3d::{posture, presets, Cut, ShapeParams, Tract3d, Tract3dFrame, SPECS};
use chimera::util::optimize::{nelder_mead, NelderMeadOpts};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use volume::{Measurement, Volume};

pub fn write(path: &Path, v: &impl serde::Serialize) -> Result<(), String> {
    fs::write(path, serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?).map_err(|e| format!("{}: {e}", path.display()))
}
fn mkdir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| e.to_string())
}
fn option<'a>(args: &'a [String], key: &str, default: &'a str) -> &'a str {
    args.iter().position(|x| x == key).and_then(|i| args.get(i + 1)).map(String::as_str).unwrap_or(default)
}

/// DVTD sound number (1-based) → the engine's nearest symbol (tense and lax share one).
pub const IPA: [&str; 22] = ["a", "e", "i", "o", "u", "ɛ", "ø", "y", "l", "f", "s", "ʃ", "ç", "x", "a", "ɛ", "ɪ", "ɔ", "ʊ", "ʏ", "œ", "ə"];
/// The design's headline set: /aː iː uː eː oː ə/ and /s ʃ ç x f l/ (DVTD numbers).
pub const HEADLINE: [usize; 12] = [1, 3, 5, 2, 4, 22, 11, 12, 13, 14, 10, 9];
/// Sounds the speaker's anatomy is fitted on (once): /ə aː iː uː s l/.
const ANATOMY_SET: [usize; 6] = [22, 1, 3, 5, 11, 9];

/// Articulators the per-sound fit moves (the velum stays at the target: DVTD excludes the nose).
const ARTIC_KEYS: [ArticKey; 13] = [
    ArticKey::TbX,
    ArticKey::TbY,
    ArticKey::TtX,
    ArticKey::TtY,
    ArticKey::TtCurl,
    ArticKey::Jaw,
    ArticKey::Root,
    ArticKey::LipAp,
    ArticKey::LipProt,
    ArticKey::LipWidth,
    ArticKey::Larynx,
    ArticKey::Epilarynx,
    ArticKey::Lateral,
];
/// 3D parameters the per-sound fit moves: the tongue's, the lips' rounding, and how far the
/// pharynx's side walls are drawn in (they move with the sound; the ring's own size is anatomy).
const SHAPE_KEYS: [&str; 13] = ["sideRoot", "sideBack", "sideDorsum", "sideBlade", "sideTip", "grooveRoot", "grooveBack", "grooveDorsum", "grooveBlade", "grooveTip", "grooveWidth", "lipRound", "lateralSqueeze"];
/// Anatomy fitted once per speaker.
pub const ANATOMY_KEYS: [&str; 14] = ["pharynxWidthNaso", "pharynxWidthOro", "pharynxWidthHypo", "pharynxShape", "archWidth", "lowerArchWidth", "vaultDepth", "vaultShape", "epilarynxWidth", "epiglottisCurl", "aryFoldHeight", "piriformWidth", "piriformDepth", "tongueWidth"];

fn model_args(name: &str, body: BodyMix) -> Result<(HumanArtic, ShapeParams), String> {
    if let Some(p) = presets(body).into_iter().find(|p| p.name == name) {
        return Ok((p.artic, p.shape));
    }
    let bytes = fs::read(name).map_err(|e| format!("preset or articulation file {name:?}: {e}"))?;
    let v: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let a = serde_json::from_value(v.get("artic").unwrap_or(&v).clone()).map_err(|e| format!("artic: {e}"))?;
    let shape = serde_json::from_value(v.get("shape").cloned().unwrap_or(json!({}))).map_err(|e| format!("shape: {e}"))?;
    Ok((a, shape))
}
pub fn frame_json(m: &Tract3d, f: &Tract3dFrame, meshes: bool) -> Value {
    let k = m.tract.body.scale;
    let mut coronal = Cut::default();
    let mut axial = Cut::default();
    m.cut_plane(chimera::anatomy::geom::v(4.5 * k, 8. * k), chimera::anatomy::geom::v(0., 1.), &mut coronal);
    m.cut_plane(chimera::anatomy::geom::v(4. * k, 8.3 * k), chimera::anatomy::geom::v(1., 0.), &mut axial);
    // the air of the midsagittal plane, row by row (for drawing next to MRI)
    let mut sagittal = Vec::new();
    let mut row = chimera::anatomy::tract3d::Row::default();
    let mut y = f.centreline[0].y - 0.5 * k;
    while y <= 12. * k {
        m.air_intervals(y, 0., &mut row);
        if row.count > 0 {
            sagittal.push(json!({"y": y, "x": row.intervals[..row.count].iter().flat_map(|q| [q.lo, q.hi]).collect::<Vec<f64>>()}));
        }
        y += 0.05 * k;
    }
    json!({"centreline":f.centreline,"areaFn":f.area_fn,"slices":f.cuts(),"shape":f.shape,"cavities":f.cavities,"meshes":if meshes {json!(m.meshes())} else {Value::Null},"midsagittal":m.midsagittal(),"sagittal":sagittal,"coronal":coronal,"axial":axial,"incisor":m.parts.palate.last()})
}

/// The reference pipeline on one DVTD sound folder: voxels, centreline, cuts.
pub fn reference(dir: &Path) -> Result<(Volume, Measurement), String> {
    let name = dir.file_name().ok_or("missing sound folder")?.to_string_lossy();
    let open = volume::stl(&dir.join(format!("{name}-inner-surface.stl")))?;
    let closed = volume::stl(&dir.join(format!("{name}-inner-surface-closed.stl")))?;
    let ends = volume::openings(&open)?;
    let v = volume::voxelize(&closed, 0.05);
    let line = volume::centreline(&v, ends)?;
    let measurement = volume::measure(&v, &line);
    Ok((v, measurement))
}
/// The same pipeline on the model's voxels, between its own glottis and lip opening.
pub fn model_measure(m: &Tract3d, f: &Tract3dFrame) -> Result<(Volume, Measurement), String> {
    let vol = volume::model(m);
    let first = f.centreline[0];
    let last = *f.centreline.last().unwrap();
    let line = volume::centreline(&vol, [[first.x, first.y, 0.], [last.x, last.y, 0.]])?;
    let r = volume::measure(&vol, &line);
    Ok((vol, r))
}
fn interp(s: &[f64], a: &[f64], x: f64) -> f64 {
    let i = s.partition_point(|v| *v < x).clamp(1, s.len() - 1);
    let t = ((x - s[i - 1]) / (s[i] - s[i - 1]).max(1e-12)).clamp(0., 1.);
    a[i - 1] * (1. - t) + a[i] * t
}
pub fn af(m: &Measurement) -> (Vec<f64>, Vec<f64>) {
    (m.slices.iter().map(|x| x.s).collect(), m.slices.iter().map(|x| x.area).collect())
}
/// d: RMS over 101 points of normalised length of log2(A₁/A₂), areas floored at 0.05 cm².
pub fn distance(sa: &[f64], aa: &[f64], sb: &[f64], ab: &[f64]) -> f64 {
    let la = *sa.last().unwrap();
    let lb = *sb.last().unwrap();
    ((0..101)
        .map(|i| {
            let t = i as f64 / 100.;
            (interp(sa, aa, t * la).max(0.05) / interp(sb, ab, t * lb).max(0.05)).log2().powi(2)
        })
        .sum::<f64>()
        / 101.)
        .sqrt()
}
/// Where along the tract a model's area function departs from the reference's: the mean of
/// log2(A_model / A_reference) over four stretches of normalised length (the larynx to 0.2, the
/// pharynx to 0.55, the mouth to 0.9, the lips), areas floored at 0.05 cm² as in `distance`.
pub fn by_region(sa: &[f64], aa: &[f64], sb: &[f64], ab: &[f64]) -> [f64; 4] {
    let (la, lb) = (*sa.last().unwrap(), *sb.last().unwrap());
    let edges = [0., 0.2, 0.55, 0.9, 1.0001];
    let mut out = [0.; 4];
    for (j, o) in out.iter_mut().enumerate() {
        let (mut sum, mut n) = (0., 0);
        for i in 0..101 {
            let t = i as f64 / 100.;
            if t >= edges[j] && t < edges[j + 1] {
                sum += (interp(sa, aa, t * la).max(0.05) / interp(sb, ab, t * lb).max(0.05)).log2();
                n += 1;
            }
        }
        *o = sum / n.max(1) as f64;
    }
    out
}
/// F1–F3 of an area function through the engine's chain-matrix code (closed glottis end,
/// radiating lips): the same solver for model and reference.
pub fn resonances(s: &[f64], a: &[f64]) -> Vec<f64> {
    use chimera::airway::{
        frequency::{find_peaks, transfer_function},
        network::{AirwayNetwork, Area, TubeSpec},
    };
    let n = 64;
    let length = *s.last().unwrap();
    let areas = (0..n).map(|i| interp(s, a, length * (i as f64 + 0.5) / n as f64).max(0.02)).collect();
    let mut net = AirwayNetwork::new(96000.);
    let t = net.add_tube(TubeSpec::new("measurement", n, length, Area::Profile(areas)));
    net.add_radiation(t.last_node());
    net.finalize();
    let freqs: Vec<_> = (0..991).map(|i| 50. + i as f64 * 5.).collect();
    let tf = transfer_function(&net.linear_elements(), t.first_node(), &freqs, &[]);
    find_peaks(&freqs, &tf.mag, 150.).iter().take(3).map(|p| p.freq).collect()
}

/// A posture: the articulators and the 3D offsets (anatomy included).
#[derive(Clone)]
struct Posture {
    artic: HumanArtic,
    shape: ShapeParams,
}
fn spec(key: &str) -> &'static chimera::anatomy::tract3d::ParamSpec {
    SPECS.iter().find(|s| s.key == key).expect("known key")
}
/// Thinnest opening (cm) the comparison resolves: the reference pipeline traces its centreline
/// on a 1 mm grid (0.5 mm voxels, every other one; the MRI's own voxels are 1.2 mm), so a slit
/// thinner than this disconnects the model's voxels while the fast slicer still counts it.
const RESOLVED: f64 = 0.15;
/// Weight of the resonance term in the fit's objective: a 10 % error of one resonance costs as
/// much as 0.08 of area-function distance.
const RESONANCE_WEIGHT: f64 = 0.8;
/// Fast objective: area-function distance, tract length and cut proportions at six stations,
/// a penalty for openings the voxel pipeline cannot resolve (closed or thinner than 1.5 mm), and
/// the distance of F1–F3 from the reference's (both through the same 1D solver). The area
/// distance alone weighs every centimetre of the tract alike, while the resonances depend most
/// on a few places (the lips of a rounded vowel, the narrowest constriction): without the last
/// term a fit could match the area function well and still sit a third above the reference's F1.
fn objective(f: &Tract3dFrame, r: &Measurement, rs: &[f64], ra: &[f64], peaks: &[f64], resonance_weight: f64) -> f64 {
    let d = distance(rs, ra, &f.area_fn.s, &f.area_fn.area);
    // hydraulic thickness 2A/P: a slit l × t gives t whichever way it runs, a disc its radius
    let unresolved: f64 = f
        .cuts()
        .iter()
        .map(|c| if c.area <= 0. || c.perimeter <= 0. { 1. } else { (RESOLVED - 2. * c.area / c.perimeter).max(0.) / RESOLVED })
        .sum();
    let length = (f.area_fn.length / r.length).ln().abs();
    let mut shape = 0.;
    for fraction in [0.15, 0.3, 0.45, 0.6, 0.75, 0.9] {
        let q = &r.slices[((r.slices.len() - 1) as f64 * fraction).round() as usize];
        let c = &f.cuts()[((f.area_fn.s.len() - 1) as f64 * fraction).round() as usize];
        shape += ((c.width / c.height.max(0.05)).max(0.05) / (q.width / q.height.max(0.05)).max(0.05)).log2().abs();
    }
    let mut resonance = 0.;
    if resonance_weight > 0. {
        let own = resonances(&f.area_fn.s, &f.area_fn.area);
        // a resonance the model's area function lacks counts as one 35 % off
        resonance = peaks.iter().enumerate().map(|(i, p)| own.get(i).map_or(0.3, |q| (q / p).ln().abs())).sum();
    }
    d + 1.5 * length + 0.05 * shape / 6. + 0.5 * unresolved + resonance_weight * resonance
}
/// The posture a share `t` of the way from `a` to `b`, over the articulators and 3D parameters
/// the per-sound fit moves.
fn blend(a: &Posture, b: &Posture, t: f64) -> Posture {
    let mut p = a.clone();
    for k in ARTIC_KEYS {
        p.artic.set(k, a.artic.get(k) + (b.artic.get(k) - a.artic.get(k)) * t);
    }
    for k in SHAPE_KEYS {
        let (x, y) = (a.shape.get(k).unwrap(), b.shape.get(k).unwrap());
        p.shape.set(k, x + (y - x) * t);
    }
    p
}
/// Per-sound fit: articulators and tongue shape, anatomy fixed; two restarts from the best.
fn fit_posture(m: &mut Tract3d, f: &mut Tract3dFrame, start: &Posture, r: &Measurement, iters: usize) -> (Posture, f64) {
    let (rs, ra) = af(r);
    let peaks = resonances(&rs, &ra);
    let mut x: Vec<f64> = ARTIC_KEYS.iter().map(|&k| start.artic.get(k)).collect();
    x.extend(SHAPE_KEYS.iter().map(|k| start.shape.get(k).unwrap()));
    let mut lo: Vec<f64> = ARTIC_KEYS.iter().map(|&k| chimera::control::articulators::artic_spec(k).min).collect();
    let mut hi: Vec<f64> = ARTIC_KEYS.iter().map(|&k| chimera::control::articulators::artic_spec(k).max).collect();
    lo.extend(SHAPE_KEYS.iter().map(|k| spec(k).min));
    hi.extend(SHAPE_KEYS.iter().map(|k| spec(k).max));
    let decode = |x: &[f64]| {
        let mut p = start.clone();
        for (i, &k) in ARTIC_KEYS.iter().enumerate() {
            p.artic.set(k, x[i]);
        }
        for (i, k) in SHAPE_KEYS.iter().enumerate() {
            p.shape.set(k, x[ARTIC_KEYS.len() + i]);
        }
        p
    };
    // The resonance term is in from the first round. Ruled out: bringing it in round by round
    // (0, half, all), which kept more of the area-function distance (median 0.436 / 0.402
    // against 0.453 / 0.447) but left the male /u/ 25 % above the reference's F1, where the
    // whole term from the start reaches 2 %.
    let mut best = f64::INFINITY;
    for round in 0..3 {
        let step: Vec<f64> = (0..x.len()).map(|i| if i < ARTIC_KEYS.len() { 0.25 } else { 0.15 } / (1 + round) as f64).collect();
        let res = nelder_mead(
            |x| {
                let p = decode(x);
                if m.compute(&p.artic, &p.shape, f).is_err() {
                    return 1e6;
                }
                objective(f, r, &rs, &ra, &peaks, RESONANCE_WEIGHT)
            },
            &x,
            NelderMeadOpts { lo: Some(lo.clone()), hi: Some(hi.clone()), step: Some(step), max_iter: Some(iters / 3), ..Default::default() },
        );
        if res.fx < best {
            best = res.fx;
            x = res.x;
        }
    }
    (decode(&x), best)
}
/// Anatomy fit, once per speaker: shared parameters over several sounds' postures.
fn fit_anatomy(m: &mut Tract3d, f: &mut Tract3dFrame, postures: &[Posture], refs: &[&Measurement], anatomy: &ShapeParams, iters: usize) -> (ShapeParams, f64) {
    let afs: Vec<_> = refs.iter().map(|r| af(r)).collect();
    let peaks: Vec<_> = afs.iter().map(|(s, a)| resonances(s, a)).collect();
    let x0: Vec<f64> = ANATOMY_KEYS.iter().map(|k| anatomy.get(k).unwrap()).collect();
    let lo: Vec<f64> = ANATOMY_KEYS.iter().map(|k| spec(k).min).collect();
    let hi: Vec<f64> = ANATOMY_KEYS.iter().map(|k| spec(k).max).collect();
    let apply = |x: &[f64], s: &ShapeParams| {
        let mut s = *s;
        for (i, k) in ANATOMY_KEYS.iter().enumerate() {
            s.set(k, x[i]);
        }
        s
    };
    let res = nelder_mead(
        |x| {
            let mut sum = 0.;
            for (j, p) in postures.iter().enumerate() {
                if m.compute(&p.artic, &apply(x, &p.shape), f).is_err() {
                    return 1e6;
                }
                sum += objective(f, refs[j], &afs[j].0, &afs[j].1, &peaks[j], RESONANCE_WEIGHT);
            }
            sum / postures.len() as f64
        },
        &x0,
        NelderMeadOpts { lo: Some(lo), hi: Some(hi), step: Some(vec![0.2; x0.len()]), max_iter: Some(iters), ..Default::default() },
    );
    (apply(&res.x, anatomy), res.fx)
}
fn with_anatomy(shape: &ShapeParams, anatomy: &ShapeParams) -> ShapeParams {
    let mut s = *shape;
    for k in ANATOMY_KEYS {
        s.set(k, anatomy.get(k).unwrap());
    }
    s
}

pub fn run(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("model") => {
            if args.len() < 3 {
                return Err("tract3d model <preset|artic.json> <out.json> [--body male|female|child]".into());
            }
            let body = crate::body_mix(option(args, "--body", "male"))?;
            let (a, p) = model_args(&args[1], body)?;
            let mut m = Tract3d::new(body.body());
            let mut f = Tract3dFrame::default();
            m.compute(&a, &p, &mut f)?;
            write(Path::new(&args[2]), &frame_json(&m, &f, true))
        }
        Some("sheet") => {
            if args.len() < 3 {
                return Err("tract3d sheet <preset|artic.json> <out.svg> [--body male|female|child]".into());
            }
            let body = crate::body_mix(option(args, "--body", "male"))?;
            let (a, p) = model_args(&args[1], body)?;
            let mut m = Tract3d::new(body.body());
            let mut f = Tract3dFrame::default();
            m.compute(&a, &p, &mut f)?;
            fs::write(&args[2], sheet::sheet(&m, &f, &format!("model: {}", args[1]))).map_err(|e| e.to_string())
        }
        Some("cut") => {
            if args.len() < 4 {
                return Err("tract3d cut <preset|artic.json> <s cm> <out.svg> [--body male|female|child]".into());
            }
            let body = crate::body_mix(option(args, "--body", "male"))?;
            let (a, p) = model_args(&args[1], body)?;
            let mut m = Tract3d::new(body.body());
            let mut f = Tract3dFrame::default();
            m.compute(&a, &p, &mut f)?;
            let s: f64 = args[2].parse().map_err(|_| "invalid s")?;
            let i = f.area_fn.s.iter().position(|x| *x >= s).unwrap_or(f.area_fn.s.len() - 1);
            fs::write(&args[3], sheet::cut_detail(&m, &f.cuts()[i], &format!("{} at s = {s}", args[1]))).map_err(|e| e.to_string())
        }
        Some("presets") => {
            if args.len() < 2 {
                return Err("tract3d presets <out-dir> [--body male|female|child]".into());
            }
            let name = option(args, "--body", "male");
            let body = crate::body_mix(name)?;
            let out = Path::new(&args[1]);
            mkdir(out)?;
            let mut m = Tract3d::new(body.body());
            let mut f = Tract3dFrame::default();
            for (i, p) in presets(body).iter().enumerate() {
                m.compute(&p.artic, &p.shape, &mut f)?;
                fs::write(out.join(format!("{:02}-{}-{name}.svg", i + 1, p.name)), sheet::sheet(&m, &f, &format!("preset {} ({}, {name})", p.name, p.label))).map_err(|e| e.to_string())?;
            }
            let checks = chimera::anatomy::tract3d::check_presets(body);
            write(&out.join(format!("features-{name}.json")), &checks)?;
            let mut md = String::from("| Preset | Feature | Measured | Line | Result |\n|---|---|---|---|---|\n");
            for c in &checks {
                md.push_str(&format!("| {} | {} | {} | {} | {} |\n", c.preset, c.feature, c.measured, c.line, if c.pass { "pass" } else { "FAIL" }));
            }
            println!("{md}");
            fs::write(out.join(format!("features-{name}.md")), md).map_err(|e| e.to_string())
        }
        Some("reference") => {
            if args.len() < 3 {
                return Err("tract3d reference <sound-dir> <out-dir>".into());
            }
            let out = Path::new(&args[2]);
            mkdir(out)?;
            let (_v, r) = reference(Path::new(&args[1]))?;
            write(&out.join("reference.json"), &r)?;
            report::reference(&r, &out.join("reference.svg"))
        }
        Some("validate") => {
            if args.len() < 3 {
                return Err("tract3d validate <dvtd-root> <out-dir> [--iterations N] [--anatomy-iterations N] [--jobs N] [--only s1-01,…]".into());
            }
            validate(
                Path::new(&args[1]),
                Path::new(&args[2]),
                option(args, "--iterations", "1500").parse().map_err(|_| "invalid iteration count")?,
                option(args, "--anatomy-iterations", "400").parse().map_err(|_| "invalid iteration count")?,
                option(args, "--jobs", "3").parse().map_err(|_| "invalid job count")?,
                option(args, "--only", ""),
            )
        }
        Some("table") => {
            // the docs' table again from a finished run's table.json
            if args.len() < 2 {
                return Err("tract3d table <validate-out-dir>".into());
            }
            let out = Path::new(&args[1]);
            let table: Value = serde_json::from_slice(&fs::read(out.join("table.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            report::table(&table, &out.join("table.md"))
        }
        Some("fit-derive") => {
            if args.len() < 2 {
                return Err("tract3d fit-derive <validate-out-dir> [--iterations N]".into());
            }
            fit_derive(Path::new(&args[1]), option(args, "--iterations", "600").parse().map_err(|_| "invalid iteration count")?)
        }
        Some("tune") => {
            if args.len() < 2 {
                return Err("tract3d tune <preset> [--body male|female|child] [--iterations N]".into());
            }
            tune(&args[1], crate::body_mix(option(args, "--body", "male"))?, option(args, "--iterations", "400").parse().map_err(|_| "invalid iteration count")?)
        }
        Some("usc") => usc::run(&args[1..]),
        Some("acoustics") => acoustics::run(&args[1..]),
        Some("walls") => acoustics::walls(&args[1..]),
        Some("benchmark") => {
            let mut m = Tract3d::new(crate::body("male")?);
            let mut f = Tract3dFrame::default();
            let mut times = Vec::new();
            for p in presets(BodyMix::MALE) {
                m.compute(&p.artic, &p.shape, &mut f)?;
                let t = std::time::Instant::now();
                for _ in 0..50 {
                    m.compute(&p.artic, &p.shape, &mut f)?;
                }
                times.push(t.elapsed().as_secs_f64() * 1000. / 50.);
            }
            times.sort_by(f64::total_cmp);
            let t = std::time::Instant::now();
            let meshes = m.meshes();
            let mesh_ms = t.elapsed().as_secs_f64() * 1000.;
            println!(
                "compute: {} laminae, {} cuts; median {:.2} ms, max {:.2} ms over {} presets x 50 calls; meshes {:.2} ms ({} vertices)",
                m.lamina_count(),
                f.area_fn.s.len(),
                times[times.len() / 2],
                times[times.len() - 1],
                times.len(),
                mesh_ms,
                meshes.iter().map(|x| x.positions.len() / 3).sum::<usize>()
            );
            Ok(())
        }
        _ => Err("tract3d model|sheet|cut|presets|reference|validate|table|fit-derive|usc|acoustics|walls|benchmark".into()),
    }
}

/// Moves a preset's articulators and tongue shape until its feature checks pass (least slack,
/// staying near the starting posture). Prints the result for presets.rs; changes nothing.
fn tune(name: &str, body: BodyMix, iters: usize) -> Result<(), String> {
    use chimera::anatomy::tract3d::check_one;
    let all = presets(body);
    let p0 = all.iter().find(|p| p.name == name).ok_or("unknown preset")?;
    let mut m = Tract3d::new(body.body());
    let mut f = Tract3dFrame::default();
    let s_at = {
        let s = all.iter().find(|p| p.name == "s").unwrap();
        m.compute(&s.artic, &s.shape, &mut f)?;
        let l = f.area_fn.length;
        f.cuts().iter().filter(|c| c.s / l >= 0.8 && c.s / l <= 0.97).min_by(|a, b| a.area.total_cmp(&b.area)).map_or(0.9, |c| c.s / l)
    };
    let artic = [ArticKey::TbX, ArticKey::TbY, ArticKey::TtX, ArticKey::TtY, ArticKey::Jaw, ArticKey::TtCurl, ArticKey::Root];
    let shape = ["sideBack", "sideDorsum", "sideBlade", "sideTip", "grooveBlade", "grooveTip", "grooveWidth"];
    let mut x0: Vec<f64> = artic.iter().map(|&k| p0.artic.get(k)).collect();
    x0.extend(shape.iter().map(|k| p0.shape.get(k).unwrap()));
    let mut lo: Vec<f64> = artic.iter().map(|&k| chimera::control::articulators::artic_spec(k).min).collect();
    let mut hi: Vec<f64> = artic.iter().map(|&k| chimera::control::articulators::artic_spec(k).max).collect();
    lo.extend(shape.iter().map(|k| spec(k).min));
    hi.extend(shape.iter().map(|k| spec(k).max));
    let decode = |x: &[f64]| {
        let (mut a, mut s) = (p0.artic, p0.shape);
        for (i, &k) in artic.iter().enumerate() {
            a.set(k, x[i]);
        }
        for (i, k) in shape.iter().enumerate() {
            s.set(k, x[artic.len() + i]);
        }
        (a, s)
    };
    let mut checks = Vec::new();
    let mut score = |x: &[f64], checks: &mut Vec<chimera::anatomy::tract3d::Check>| -> f64 {
        let (a, s) = decode(x);
        if m.compute(&a, &s, &mut f).is_err() {
            return 1e6;
        }
        checks.clear();
        check_one(&m, &f, name, &s, s_at, body.body().scale, checks);
        let near: f64 = x.iter().zip(&x0).map(|(a, b)| (a - b) * (a - b)).sum();
        checks.iter().map(|c| c.slack).sum::<f64>() + 0.01 * near
    };
    let before = score(&x0, &mut checks);
    let res = nelder_mead(|x| score(x, &mut Vec::new()), &x0, NelderMeadOpts { lo: Some(lo), hi: Some(hi), step: Some(vec![0.15; x0.len()]), max_iter: Some(iters), ..Default::default() });
    let after = score(&res.x, &mut checks);
    let (a, s) = decode(&res.x);
    println!("{name}: score {before:.3} -> {after:.3}");
    for (i, k) in artic.iter().enumerate() {
        println!("  {} {:.2} -> {:.2}", k.name(), x0[i], a.get(*k));
    }
    for (i, k) in shape.iter().enumerate() {
        println!("  {k} {:.2} -> {:.2}", x0[artic.len() + i], s.get(k).unwrap());
    }
    for c in &checks {
        println!("  {}: {} [{}]", c.feature, c.measured, if c.pass { "pass" } else { "FAIL" });
    }
    Ok(())
}

/// Cut-shape metrics at the plan's stations (M3): IoU after equal-area scaling and centroid
/// alignment, the log2 aspect-ratio difference, channel counts.
pub fn shape_metrics(a: &Measurement, b: &Measurement) -> Value {
    let fractions = [0.15, 0.3, 0.45, 0.6, 0.75, 0.9];
    let n = volume::MASK;
    let rows: Vec<_> = fractions
        .iter()
        .map(|&fraction| {
            let x = &a.slices[((a.slices.len() - 1) as f64 * fraction).round() as usize];
            let y = &b.slices[((b.slices.len() - 1) as f64 * fraction).round() as usize];
            let centroid = |q: &volume::Slice| {
                let mut sum = [0.; 3];
                for (i, &v) in q.mask.iter().enumerate() {
                    if v != 0 {
                        sum[0] += (i / n) as f64;
                        sum[1] += (i % n) as f64;
                        sum[2] += 1.;
                    }
                }
                if sum[2] > 0. { [sum[0] / sum[2], sum[1] / sum[2], sum[2]] } else { [160., 160., 0.] }
            };
            let ca = centroid(x);
            let cb = centroid(y);
            let scale = if cb[2] > 0. { (ca[2] / cb[2]).sqrt() } else { 1. };
            let (mut inter, mut union) = (0, 0);
            for i in 0..n {
                for j in 0..n {
                    let u = ((i as f64 - ca[0]) / scale + cb[0]).round() as isize;
                    let w = ((j as f64 - ca[1]) / scale + cb[1]).round() as isize;
                    let aa = x.mask[i * n + j] != 0;
                    let bb = (0..n as isize).contains(&u) && (0..n as isize).contains(&w) && y.mask[u as usize * n + w as usize] != 0;
                    inter += usize::from(aa && bb);
                    union += usize::from(aa || bb);
                }
            }
            json!({"fraction":fraction,"iou":(union > 0).then(|| inter as f64 / union as f64),"aspectLog2Error":(x.height > 0. && y.height > 0. && x.width > 0. && y.width > 0.).then(|| ((x.width / x.height) / (y.width / y.height)).log2().abs()),"referenceChannels":x.channels,"modelChannels":y.channels})
        })
        .collect();
    json!(rows)
}

struct Case {
    index: usize,
    dir: PathBuf,
    name: String,
    sound: usize,
    speaker: usize,
}
fn cases(root: &Path, only: &str) -> Result<Vec<Case>, String> {
    let extracted = if root.join("extracted").exists() { root.join("extracted") } else { root.to_owned() };
    let mut out = Vec::new();
    for (speaker, subject) in ["subject-1", "subject-2"].iter().enumerate() {
        let mut dirs = Vec::new();
        for entry in fs::read_dir(extracted.join(subject)).map_err(|e| format!("{subject}: {e}"))? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.is_dir() && path.file_name().unwrap().to_string_lossy().starts_with('s') {
                dirs.push(path);
            }
        }
        dirs.sort();
        for dir in dirs {
            let name = dir.file_name().unwrap().to_string_lossy().to_string();
            let sound: usize = name.split('-').nth(1).and_then(|x| x.parse().ok()).ok_or("invalid sound folder name")?;
            if !(1..=22).contains(&sound) {
                return Err(format!("unexpected sound number in {name}"));
            }
            if !only.is_empty() && !only.split(',').any(|o| name.starts_with(o)) {
                continue;
            }
            out.push(Case { index: speaker * 22 + sound, dir, name, sound, speaker });
        }
    }
    Ok(out)
}

/// Runs `f` over items on `jobs` threads; results in item order.
fn parallel<T: Sync, R: Send>(items: &[T], jobs: usize, f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let next = std::sync::atomic::AtomicUsize::new(0);
    let results: Vec<std::sync::Mutex<Option<R>>> = items.iter().map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|s| {
        for _ in 0..jobs.max(1) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if i >= items.len() {
                    break;
                }
                let r = f(&items[i]);
                *results[i].lock().unwrap() = Some(r);
            });
        }
    });
    results.into_iter().map(|m| m.into_inner().unwrap().unwrap()).collect()
}

fn body_of(speaker: usize) -> (BodyMix, &'static str) {
    if speaker == 0 { (BodyMix::MALE, "male") } else { (BodyMix::FEMALE, "female") }
}
/// Median (mean of the middle two for an even count); non-finite values are left out.
pub fn median(mut x: Vec<f64>) -> Option<f64> {
    x.retain(|v| v.is_finite());
    x.sort_by(f64::total_cmp);
    if x.is_empty() {
        None
    } else if x.len() % 2 == 1 {
        Some(x[x.len() / 2])
    } else {
        Some(0.5 * (x[x.len() / 2 - 1] + x[x.len() / 2]))
    }
}

fn validate(root: &Path, out: &Path, iters: usize, anatomy_iters: usize, jobs: usize, only: &str) -> Result<(), String> {
    mkdir(out)?;
    let started = std::time::Instant::now();
    let all = cases(root, only)?;
    println!("{} cases; reference pipeline ...", all.len());
    let refs: Vec<Result<(Volume, Measurement), String>> = parallel(&all, jobs, |c| reference(&c.dir));
    for (c, r) in all.iter().zip(&refs) {
        match r {
            Err(e) => return Err(format!("{}: {e}", c.name)),
            Ok((_, m)) => write(&out.join(format!("{:02}-{}-reference.json", c.index, c.name)), m)?,
        }
    }
    let refs: Vec<(Volume, Measurement)> = refs.into_iter().map(Result::unwrap).collect();
    println!("references done in {:.0} s", started.elapsed().as_secs_f64());
    // ---- anatomy, once per speaker, on /ə aː iː uː s l/ (articulation refitted around it)
    let mut anatomy = [ShapeParams::default(); 2];
    for speaker in 0..2 {
        let set: Vec<usize> = (0..all.len()).filter(|&i| all[i].speaker == speaker && ANATOMY_SET.contains(&all[i].sound)).collect();
        if set.is_empty() {
            continue;
        }
        let (body, label) = body_of(speaker);
        let mut current = ShapeParams::default();
        let mut postures: Vec<Posture> = set.iter().map(|&i| Posture { artic: posture(IPA[all[i].sound - 1], body), shape: ShapeParams::default() }).collect();
        let mut last = 0.;
        for round in 0..2 {
            let starts: Vec<(usize, Posture)> = set.iter().zip(&postures).map(|(&i, p)| (i, Posture { artic: p.artic, shape: with_anatomy(&p.shape, &current) })).collect();
            postures = parallel(&starts, jobs, |(i, start)| {
                let mut m = Tract3d::new(body.body());
                let mut f = Tract3dFrame::default();
                fit_posture(&mut m, &mut f, start, &refs[*i].1, iters / 2).0
            });
            let rs: Vec<&Measurement> = set.iter().map(|&i| &refs[i].1).collect();
            let mut m = Tract3d::new(body.body());
            let mut f = Tract3dFrame::default();
            let (a, fx) = fit_anatomy(&mut m, &mut f, &postures, &rs, &current, anatomy_iters / (1 + round));
            current = a;
            last = fx;
            println!("{label} anatomy round {}: mean objective {fx:.3} ({:.0} s)", round + 1, started.elapsed().as_secs_f64());
        }
        anatomy[speaker] = current;
        let values: serde_json::Map<String, Value> = ANATOMY_KEYS.iter().map(|k| (k.to_string(), json!(current.get(k).unwrap()))).collect();
        write(&out.join(format!("anatomy-{label}.json")), &json!({"speaker":label,"fittedOn":set.iter().map(|&i| all[i].name.clone()).collect::<Vec<_>>(),"objective":last,"iterations":anatomy_iters,"anatomy":values}))?;
    }
    // ---- every sound: default posture (engine targets + derive) and fitted posture
    let order: Vec<usize> = (0..all.len()).collect();
    let rows: Vec<Result<Value, String>> = parallel(&order, jobs, |&i| -> Result<Value, String> {
        let c = &all[i];
        let (body, label) = body_of(c.speaker);
        let symbol = IPA[c.sound - 1];
        let (rvol, r) = &refs[i];
        let (rs, ra) = af(r);
        let mut m = Tract3d::new(body.body());
        let mut f = Tract3dFrame::default();
        let start = Posture { artic: posture(symbol, body), shape: with_anatomy(&ShapeParams::default(), &anatomy[c.speaker]) };
        m.compute(&start.artic, &start.shape, &mut f)?;
        let default_fast = distance(&rs, &ra, &f.area_fn.s, &f.area_fn.area);
        let default_voxel = model_measure(&m, &f).ok();
        let default_d = default_voxel.as_ref().map(|(_, q)| {
            let (s, a) = af(q);
            distance(&rs, &ra, &s, &a)
        });
        let default_regions = default_voxel.as_ref().map(|(_, q)| {
            let (s, a) = af(q);
            by_region(&s, &a, &rs, &ra)
        });
        let (best, objective) = fit_posture(&mut m, &mut f, &start, r, iters);
        // The fast objective sees the tract at its cuts, 2.5 mm apart; between two of them a
        // channel can still be too thin for the voxel pipeline to follow, and the fit then has
        // no measured area function. Keep a fit the voxel pipeline can measure: the best one,
        // or the nearest to it on the way back to the starting posture.
        let mut fit = best.clone();
        let mut backed_off = 0.;
        let mut measured = Err("not measured".to_string());
        for back in [0., 0.1, 0.2, 0.35, 0.5, 0.7, 0.85, 1.] {
            fit = blend(&best, &start, back);
            m.compute(&fit.artic, &fit.shape, &mut f)?;
            measured = model_measure(&m, &f);
            backed_off = back;
            if measured.is_ok() {
                break;
            }
        }
        let fast_d = distance(&rs, &ra, &f.area_fn.s, &f.area_fn.area);
        // T-consistency: the model's voxels cut along the fast slicer's own centreline
        let line: Vec<[f64; 3]> = f.centreline.iter().map(|p| [p.x, p.y, 0.]).collect();
        let same = match &measured {
            Ok((vol, _)) => Some(volume::measure(vol, &line)),
            Err(_) => None,
        };
        let (mut errs, mut equal) = (Vec::new(), 0);
        if let Some(same) = &same {
            for (i, c) in f.cuts().iter().enumerate() {
                let q = &same.slices[i.min(same.slices.len() - 1)];
                if q.area > 0.3 {
                    errs.push(((c.area - q.area) / q.area).abs());
                }
                equal += usize::from(c.channels == q.channels);
            }
        }
        let consistency = median(errs);
        let channel_agreement = same.as_ref().map(|_| equal as f64 / f.cuts().len() as f64);
        let fit_d = measured.as_ref().ok().map(|(_, q)| {
            let (s, a) = af(q);
            distance(&rs, &ra, &s, &a)
        });
        let fit_regions = measured.as_ref().ok().map(|(_, q)| {
            let (s, a) = af(q);
            by_region(&s, &a, &rs, &ra)
        });
        let reference_peaks = resonances(&rs, &ra);
        let measured_peaks = measured.as_ref().ok().map(|(_, q)| {
            let (s, a) = af(q);
            resonances(&s, &a)
        });
        let default_peaks = default_voxel.as_ref().map(|(_, q)| {
            let (s, a) = af(q);
            resonances(&s, &a)
        });
        let pct = |p: &Option<Vec<f64>>| p.as_ref().map(|p| reference_peaks.iter().zip(p).map(|(a, b)| (b - a) / a * 100.).collect::<Vec<_>>());
        // constriction (M2): position and area of the minimum, s ≥ 0.5 cm
        let constriction = |m: &Measurement| {
            let i = (0..m.slices.len()).filter(|&i| m.slices[i].s >= 0.5).min_by(|&a, &b| m.slices[a].area.total_cmp(&m.slices[b].area)).unwrap();
            json!({"fraction": m.slices[i].s / m.length, "area": m.slices[i].area, "channels": m.slices[i].channels})
        };
        // the research stage's probe output for the same mesh, if kept: the port's oracle
        let oracle = root.join("derived").join(format!("{}.json", c.name));
        let mut port = Value::Null;
        if oracle.exists() {
            let old: Value = serde_json::from_slice(&fs::read(oracle).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            if let Some(os) = old["slices"].as_array() {
                let (mut diffs, mut same, mut nc) = (Vec::new(), 0, 0);
                for q in &r.slices {
                    if let Some(o) = os.get((q.s / 0.25).round() as usize) {
                        if let Some(a) = o["area_cm2"].as_f64() {
                            if a > 0.3 {
                                diffs.push((q.area - a).abs() / a);
                            }
                            same += usize::from(o["channels"].as_u64() == Some(q.channels as u64));
                            nc += 1;
                        }
                    }
                }
                port = json!({"medianAreaRelativeError": median(diffs), "channelAgreement": same as f64 / nc.max(1) as f64, "lengthErrorCm": r.length - old["length_cm"].as_f64().unwrap_or(0.)});
            }
        }
        let stem = format!("{:02}-{}", c.index, c.name);
        write(&out.join(format!("{stem}-posture.json")), &json!({"artic": fit.artic, "shape": fit.shape}))?;
        write(&out.join(format!("{stem}-fit.json")), &frame_json(&m, &f, false))?;
        if let Ok((_, q)) = &measured {
            write(&out.join(format!("{stem}-fit-voxel.json")), q)?;
        }
        report::comparison(
            r,
            rvol,
            measured.as_ref().ok().map(|(v, q)| (q, v)),
            default_voxel.as_ref().map(|(_, q)| q),
            &format!("{} /{}/ ({label}) vs the parametric model", c.name, symbol),
            &json!({"defaultD": default_d, "fitD": fit_d, "referencePeaks": reference_peaks, "fitPeaks": measured_peaks}),
            &out.join(format!("{stem}.svg")),
        )?;
        let r3 = |x: Option<f64>| x.map(|x| (x * 1000.).round() / 1000.);
        println!("{} /{symbol}/: default d {:?}, fit d {:?} (fast {:.3}), slicer {:?} ({:.0} s)", c.name, r3(default_d), r3(fit_d), fast_d, r3(consistency), started.elapsed().as_secs_f64());
        Ok(json!({
            "case": c.name, "index": c.index, "sound": c.sound, "ipa": symbol, "body": label,
            "defaultD": default_d, "defaultFastD": default_fast, "fitD": fit_d, "fitFastD": fast_d, "objective": objective,
            "defaultRegionLog2": default_regions, "fitRegionLog2": fit_regions,
            "measurementError": measured.as_ref().err(), "backedOff": backed_off,
            "referenceLength": r.length, "fitLength": measured.as_ref().ok().map(|(_, q)| q.length),
            "referenceVolume": r.volume, "fitVolume": measured.as_ref().ok().map(|(_, q)| q.volume),
            "slicerMedianAreaError": consistency, "slicerChannelAgreement": channel_agreement,
            "referencePeaksHz": reference_peaks, "fitPeaksHz": measured_peaks, "defaultPeaksHz": default_peaks,
            "fitResonanceErrorPercent": pct(&measured_peaks), "defaultResonanceErrorPercent": pct(&default_peaks),
            "referenceConstriction": constriction(r), "fitConstriction": measured.as_ref().ok().map(|(_, q)| constriction(q)),
            "lowerPharynxChannels": {"reference": (2..=12).map(|i| r.slices.get(i).map_or(0, |q| q.channels)).collect::<Vec<_>>(), "fit": measured.as_ref().ok().map(|(_, q)| (2..=12).map(|i| q.slices.get(i).map_or(0, |q| q.channels)).collect::<Vec<_>>())},
            "shapeMetrics": measured.as_ref().ok().map(|(_, q)| shape_metrics(r, q)),
            "referencePort": port,
        }))
    });
    let mut table_rows: Vec<Value> = Vec::new();
    for (c, r) in all.iter().zip(rows) {
        table_rows.push(r.map_err(|e| format!("{}: {e}", c.name))?);
    }
    let mut rows = table_rows;
    // ---- identification, between-speaker spread, per-speaker summaries
    let fitted: Vec<Option<(Vec<f64>, Vec<f64>)>> = all
        .iter()
        .map(|c| {
            fs::read(out.join(format!("{:02}-{}-fit-voxel.json", c.index, c.name))).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok()).map(|v| {
                let sl = v["slices"].as_array().cloned().unwrap_or_default();
                (sl.iter().map(|q| q["s"].as_f64().unwrap_or(0.)).collect(), sl.iter().map(|q| q["area"].as_f64().unwrap_or(0.)).collect())
            })
        })
        .collect();
    let reference_afs: Vec<_> = refs.iter().map(|r| af(&r.1)).collect();
    for i in 0..rows.len() {
        let speaker = all[i].speaker;
        let nearest = fitted[i].as_ref().and_then(|(s, a)| {
            (0..all.len()).filter(|&j| all[j].speaker == speaker).min_by(|&x, &y| distance(s, a, &reference_afs[x].0, &reference_afs[x].1).total_cmp(&distance(s, a, &reference_afs[y].0, &reference_afs[y].1)))
        });
        rows[i]["identified"] = json!(nearest.map(|j| j == i));
        rows[i]["nearestReference"] = json!(nearest.map(|j| all[j].name.clone()));
        if let Some(peer) = (0..all.len()).find(|&j| all[j].speaker != speaker && all[j].sound == all[i].sound) {
            let (s, a) = &reference_afs[i];
            let (ps, pa) = &reference_afs[peer];
            let mean = |x: &[f64]| x.iter().sum::<f64>() / x.len() as f64;
            let ratio = mean(a) / mean(pa);
            let scaled: Vec<f64> = pa.iter().map(|x| x * ratio).collect();
            rows[i]["betweenSpeakerD"] = json!(distance(s, a, ps, pa));
            rows[i]["betweenSpeakerScaledD"] = json!(distance(s, a, ps, &scaled));
            rows[i]["betweenSpeakerShapeMetrics"] = shape_metrics(&refs[i].1, &refs[peer].1);
        }
    }
    let mut groups = Vec::new();
    for speaker in 0..2 {
        let (_, label) = body_of(speaker);
        let rr: Vec<&Value> = rows.iter().filter(|r| r["body"] == label).collect();
        if rr.is_empty() {
            continue;
        }
        let col = |key: &str| rr.iter().filter_map(|r| r[key].as_f64()).collect::<Vec<_>>();
        let res = |key: &str| rr.iter().filter_map(|r| r[key].as_array()).flatten().filter_map(Value::as_f64).map(f64::abs).collect::<Vec<_>>();
        let fit = col("fitD");
        let identified = rr.iter().filter(|r| r["identified"] == true).count();
        let fe = res("fitResonanceErrorPercent");
        let headline: Vec<f64> = rr.iter().filter(|r| HEADLINE.contains(&(r["sound"].as_u64().unwrap_or(0) as usize))).filter_map(|r| r["fitD"].as_f64()).collect();
        // cut shape against the speaker (and between the two speakers), the lower pharynx's count
        let shape = |key: &str, field: &str| rr.iter().filter_map(|r| r[key].as_array()).flatten().filter_map(|m| m[field].as_f64()).collect::<Vec<_>>();
        let (mut split, mut within) = (0, 0);
        for r in &rr {
            if let (Some(a), Some(b)) = (r["lowerPharynxChannels"]["reference"].as_array(), r["lowerPharynxChannels"]["fit"].as_array()) {
                for (x, y) in a.iter().zip(b) {
                    if let (Some(x), Some(y)) = (x.as_i64(), y.as_i64()) {
                        if x >= 2 {
                            split += 1;
                            within += i32::from((x - y).abs() <= 1);
                        }
                    }
                }
            }
        }
        groups.push(json!({
            "medianCutIou": median(shape("shapeMetrics", "iou")), "medianCutAspectError": median(shape("shapeMetrics", "aspectLog2Error")),
            "medianBetweenSpeakerCutIou": median(shape("betweenSpeakerShapeMetrics", "iou")), "medianBetweenSpeakerCutAspectError": median(shape("betweenSpeakerShapeMetrics", "aspectLog2Error")),
            "lowerPharynxSplitCuts": split, "lowerPharynxWithinOne": within,
            "defaultWithinLine": rr.iter().filter(|r| r["defaultD"].as_f64().is_some_and(|d| d <= r["betweenSpeakerD"].as_f64().unwrap_or(1.).max(1.))).count(),
            "fitWithin045": rr.iter().filter(|r| r["fitD"].as_f64().is_some_and(|d| d <= 0.45)).count(),
            "soundsWithResonanceOver10": rr.iter().filter(|r| r["fitResonanceErrorPercent"].as_array().is_some_and(|a| a.iter().filter_map(Value::as_f64).any(|x| x.abs() > 10.))).count(),
            "body": label, "cases": rr.len(), "measured": fit.len(),
            "medianFitD": median(fit.clone()), "maxFitD": fit.iter().cloned().fold(f64::NAN, f64::max), "medianHeadlineFitD": median(headline),
            "identified": identified,
            "medianDefaultD": median(col("defaultD")),
            "medianFitResonanceErrorPercent": median(fe.clone()), "maxFitResonanceErrorPercent": fe.iter().cloned().fold(f64::NAN, f64::max),
            "medianDefaultResonanceErrorPercent": median(res("defaultResonanceErrorPercent")),
            "medianSlicerAreaError": median(col("slicerMedianAreaError")), "worstSlicerAreaError": col("slicerMedianAreaError").iter().cloned().fold(f64::NAN, f64::max),
            "medianSlicerChannelAgreement": median(col("slicerChannelAgreement")),
            "medianBetweenSpeakerScaledD": median(col("betweenSpeakerScaledD")),
        }));
    }
    let table = json!({
        "schema": 2, "complete": true, "iterations": iters, "anatomyIterations": anatomy_iters,
        "measurement": "0.5 mm voxels; 2.5 mm centreline steps; 0.25 mm cut pixels; band ±6 mm × ±30 mm; d = RMS log2 area ratio over 101 points of normalised length, 0.05 cm² floor",
        "seconds": started.elapsed().as_secs_f64(),
        "groups": groups, "rows": rows,
    });
    write(&out.join("table.json"), &table)?;
    report::table(&table, &out.join("table.md"))?;
    println!("validate done in {:.0} s", started.elapsed().as_secs_f64());
    Ok(())
}

/// Fits the `derive` constants to the median default d over every reference in a
/// validate output (fast slicer, fitted anatomy). Prints the constants for params.rs.
fn fit_derive(dir: &Path, iters: usize) -> Result<(), String> {
    use chimera::anatomy::tract3d::{derive, derive_with, DERIVE};
    let mut anatomy = Vec::new();
    for label in ["male", "female"] {
        let v: Value = serde_json::from_slice(&fs::read(dir.join(format!("anatomy-{label}.json"))).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let mut s = ShapeParams::default();
        for k in ANATOMY_KEYS {
            s.set(k, v["anatomy"][k].as_f64().ok_or("missing anatomy value")?);
        }
        anatomy.push(s);
    }
    let mut refs = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if let Some(stem) = name.strip_suffix("-reference.json") {
            let v: Value = serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            let sl = v["slices"].as_array().cloned().unwrap_or_default();
            let speaker = usize::from(stem.contains("-s2-"));
            let sound: usize = stem.split('-').nth(2).and_then(|x| x.parse().ok()).ok_or("bad reference name")?;
            refs.push((speaker, sound, sl.iter().map(|q| q["s"].as_f64().unwrap()).collect::<Vec<_>>(), sl.iter().map(|q| q["area"].as_f64().unwrap()).collect::<Vec<_>>()));
        }
    }
    refs.sort_by_key(|r| (r.0, r.1));
    let mut models = [Tract3d::new(BodyMix::MALE.body()), Tract3d::new(BodyMix::FEMALE.body())];
    let mut f = Tract3dFrame::default();
    let mut eval = |c: &[f64; 10]| -> f64 {
        let mut ds = Vec::new();
        for (speaker, sound, s, a) in &refs {
            let (body, _) = body_of(*speaker);
            let artic = posture(IPA[sound - 1], body);
            // offsets that turn the code's derive into derive_with(c), plus the anatomy
            let mut shape = with_anatomy(&ShapeParams::default(), &anatomy[*speaker]);
            let base = derive(&artic);
            let want = derive_with(&artic, c);
            for sp in SPECS {
                if !ANATOMY_KEYS.contains(&sp.key) {
                    shape.set(sp.key, sp.default + want.get(sp.key).unwrap() - base.get(sp.key).unwrap());
                }
            }
            if models[*speaker].compute(&artic, &shape, &mut f).is_err() {
                return 1e6;
            }
            ds.push(distance(s, a, &f.area_fn.s, &f.area_fn.area));
        }
        median(ds).unwrap_or(1e6)
    };
    let before = eval(&DERIVE);
    let res = nelder_mead(
        |x| eval(&x.try_into().unwrap()),
        &DERIVE,
        NelderMeadOpts { lo: Some(vec![-0.5, -0.5, 0., 0., -0.5, 0., 0., -0.5, 0., 0.]), hi: Some(vec![1.4, 1.4, 0.8, 0.8, 1.4, 1.4, 1.4, 1.4, 1.5, 1.2]), step: Some(vec![0.15; 10]), max_iter: Some(iters), ..Default::default() },
    );
    println!("derive constants: before {DERIVE:?} median default d (fast) {before:.3}\nafter  {:?} median default d (fast) {:.3} in {} iterations", res.x.iter().map(|x| (x * 1000.).round() / 1000.).collect::<Vec<_>>(), res.fx, res.iters);
    write(&dir.join("derive-fit.json"), &json!({"before": DERIVE, "beforeMedianFastD": before, "after": res.x, "afterMedianFastD": res.fx, "iterations": res.iters, "cases": refs.len()}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chimera::anatomy::human::NEUTRAL;
    #[test]
    fn larynx_voxel_junction_survives_subvoxel_translation() {
        for body in [BodyMix::MALE, BodyMix::FEMALE] {
            let mut m = Tract3d::new(body.body());
            let mut f = Tract3dFrame::default();
            for shift in [0., 0.017, 0.034] {
                let a = HumanArtic { larynx: shift, ..NEUTRAL };
                m.compute(&a, &ShapeParams::default(), &mut f).unwrap();
                assert!(model_measure(&m, &f).is_ok(), "{} {}", body.body().name, shift);
            }
        }
    }
    #[test]
    fn identical_area_functions_have_zero_distance_and_halving_gives_one() {
        let s: Vec<f64> = (0..50).map(|i| i as f64 * 0.25).collect();
        let a: Vec<f64> = (0..50).map(|i| 1. + i as f64 * 0.1).collect();
        assert!(distance(&s, &a, &s, &a) < 1e-12);
        let half: Vec<f64> = a.iter().map(|x| x * 0.5).collect();
        assert!((distance(&s, &a, &s, &half) - 1.).abs() < 1e-9);
    }
    #[test]
    fn median_of_even_count_is_the_mean_of_the_middle_pair() {
        assert_eq!(median(vec![4., 1., 3., 2.]), Some(2.5));
        assert_eq!(median(vec![f64::NAN, 2.]), Some(2.));
        assert_eq!(median(vec![]), None);
    }
}
