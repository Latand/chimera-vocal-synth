//! `chimera tract3d acoustics <dvtd-root> <validate-out> <out-dir>`: the engine's 1D acoustics
//! against the volume-velocity transfer functions DVTD measured on 3D prints of the same tracts.
//!
//! Each print is rigid, closed at the glottis and measured in room air (345 m/s, the dataset's
//! paper); the engine computes in body-temperature air (`C_SOUND`). So the engine's curve for the
//! MRI area function (from `validate`'s `NN-name-reference.json`) is read at f·C_SOUND/345, which
//! is exact for the lossless part. Nothing from the dataset is written outside `out-dir`.
use super::sheet::Svg;
use super::*;
use chimera::airway::frequency::transfer_function;
use chimera::airway::network::{AirwayNetwork, Area, TubeSpec};
use chimera::physics::C_SOUND;
use serde::Serialize;
use std::fmt::Write as _;

/// Speed of sound of the print measurements (22 °C).
const C_PRINT: f64 = 34500.;
/// Frequency grid of every comparison (Hz).
const F_LO: f64 = 60.;
const F_HI: f64 = 4000.;
const DF: f64 = 1.0;

pub struct Curve {
    pub freqs: Vec<f64>,
    pub db: Vec<f64>,
}

fn grid() -> Vec<f64> {
    let n = ((F_HI - F_LO) / DF) as usize + 1;
    (0..n).map(|i| F_LO + i as f64 * DF).collect()
}

/// A measured `XX-vvtf-measured.txt` (freq, magnitude, phase), smoothed over ±3 Hz (the
/// sweep's ripple) and resampled to the comparison grid, in dB.
fn read_vvtf(path: &Path) -> Result<Curve, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut f = Vec::new();
    let mut m = Vec::new();
    for line in text.lines() {
        let mut it = line.split_whitespace().map(str::parse::<f64>);
        if let (Some(Ok(a)), Some(Ok(b))) = (it.next(), it.next()) {
            f.push(a);
            m.push(b.max(1e-9));
        }
    }
    if f.len() < 100 {
        return Err(format!("{}: too few rows", path.display()));
    }
    let df = f[1] - f[0];
    let half = (3. / df).round() as usize;
    let freqs = grid();
    let db = freqs
        .iter()
        .map(|&x| {
            let i = ((x - f[0]) / df).round() as usize;
            let (lo, hi) = (i.saturating_sub(half), (i + half).min(f.len() - 1));
            let mean = m[lo..=hi].iter().map(|v| v * v).sum::<f64>() / (hi - lo + 1) as f64;
            10. * mean.log10()
        })
        .collect();
    Ok(Curve { freqs, db })
}

/// How the engine's tube is set up for one comparison.
#[derive(Clone, Copy)]
pub struct Physics {
    pub name: &'static str,
    /// evanescent-mode inertance at abrupt area changes (see `AirwayNetwork::discontinuity`)
    pub discontinuity: bool,
    /// thermal boundary-layer loss (see `AirwayNetwork::thermal`)
    pub thermal: bool,
}

/// |U_lips / U_glottis| of an area function (closed glottis end, radiating lips, rigid walls),
/// in dB on the comparison grid, read at the print's speed of sound.
pub fn engine_curve(s: &[f64], a: &[f64], physics: Physics) -> Curve {
    let length = *s.last().unwrap();
    let n = ((length / 0.25).round() as usize).max(16);
    let areas: Vec<f64> = (0..n).map(|i| interp(s, a, length * (i as f64 + 0.5) / n as f64).max(0.02)).collect();
    let mut net = AirwayNetwork::new(96000.);
    net.discontinuity = physics.discontinuity;
    net.thermal = physics.thermal;
    // the human airway's split: the back half with the pharynx's perimeter factor, the front half
    // with the oral cavity's (`voice::human_airway`), so losses scale as in the voice
    let half = n / 2;
        let back = net.add_tube(TubeSpec::new("back", half, length * half as f64 / n as f64, Area::Profile(areas[..half].to_vec())).perimeter(1.25));
    let front = net.add_tube(TubeSpec::new("front", n - half, length * (n - half) as f64 / n as f64, Area::Profile(areas[half..].to_vec())).perimeter(1.5));
    net.connect(back.last_node(), front.first_node());
    net.add_radiation(front.last_node());
    net.finalize();
    let t = back;
    let freqs = grid();
    let at: Vec<f64> = freqs.iter().map(|f| f * C_SOUND / C_PRINT).collect();
    let tf = transfer_function(&net.linear_elements(), t.first_node(), &at, &[]);
    Curve { db: tf.mag.iter().map(|m| 20. * m.max(1e-12).log10()).collect(), freqs }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Peak {
    pub freq: f64,
    pub db: f64,
    /// −3 dB bandwidth (Hz), NaN when a side never falls 3 dB
    pub bw: f64,
}

/// Peaks above 100 Hz (the sweep's band starts there) standing at least 4 dB above the lower of
/// the two minima that separate them from a higher neighbour (prominence), refined by a parabola.
pub fn peaks(c: &Curve) -> Vec<Peak> {
    let d = &c.db;
    let n = d.len();
    let mut out = Vec::new();
    for i in 1..n - 1 {
        if c.freqs[i] < 100. || !(d[i] > d[i - 1] && d[i] >= d[i + 1]) {
            continue;
        }
        let side = |step: isize| {
            let mut j = i as isize;
            let mut lo = d[i];
            while j + step >= 0 && ((j + step) as usize) < n {
                j += step;
                let v = d[j as usize];
                if v > d[i] {
                    break;
                }
                lo = lo.min(v);
            }
            lo
        };
        let prominence = d[i] - side(-1).max(side(1));
        if prominence < 4. {
            continue;
        }
        let (a, b, cc) = (d[i - 1], d[i], d[i + 1]);
        let den = a - 2. * b + cc;
        let off = if den.abs() > 1e-12 { 0.5 * (a - cc) / den } else { 0. };
        let peak_db = b - 0.25 * (a - cc) * off;
        let (mut lo, mut hi) = (i, i);
        while lo > 0 && d[lo] > peak_db - 3. {
            lo -= 1;
        }
        while hi < n - 1 && d[hi] > peak_db - 3. {
            hi += 1;
        }
        let bw = if d[lo] <= peak_db - 3. && d[hi] <= peak_db - 3. { c.freqs[hi] - c.freqs[lo] } else { f64::NAN };
        out.push(Peak { freq: c.freqs[i] + off * DF, db: peak_db, bw });
    }
    out
}

/// RMS of the dB difference over [lo, hi] Hz.
fn band_rms(a: &Curve, b: &Curve, lo: f64, hi: f64) -> f64 {
    let (mut s, mut n) = (0., 0);
    for i in 0..a.freqs.len() {
        if a.freqs[i] >= lo && a.freqs[i] <= hi {
            s += (a.db[i] - b.db[i]).powi(2);
            n += 1;
        }
    }
    (s / n.max(1) as f64).sqrt()
}

#[derive(Serialize)]
struct Row {
    name: String,
    ipa: &'static str,
    measured: Vec<Peak>,
    engine: Vec<(String, Vec<Peak>, f64)>,
}

fn pct(a: f64, b: f64) -> f64 {
    100. * (a / b - 1.)
}

pub fn median(mut v: Vec<f64>) -> f64 {
    v.retain(|x| x.is_finite());
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 1 { v[n / 2] } else { 0.5 * (v[n / 2 - 1] + v[n / 2]) }
}

pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        return Err("tract3d acoustics <dvtd-root> <validate-out> <out-dir>".into());
    }
    let (root, val, out) = (Path::new(&args[0]), Path::new(&args[1]), Path::new(&args[2]));
    mkdir(out)?;
    let physics = [Physics { name: "before", discontinuity: false, thermal: false }, Physics { name: "after", discontinuity: true, thermal: true }];
    let mut files: Vec<_> = fs::read_dir(val).map_err(|e| e.to_string())?.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.to_string_lossy().ends_with("-reference.json")).collect();
    files.sort();
    let mut rows = Vec::new();
    let mut curves = Vec::new();
    for path in files {
        let stem = path.file_name().unwrap().to_string_lossy().trim_end_matches("-reference.json").to_string();
        let name = stem[3..].to_string();
        let speaker = &name[1..2];
        let number: usize = name[3..5].parse().map_err(|_| format!("{name}: no sound number"))?;
        let v: Value = serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let slices = v["slices"].as_array().ok_or("slices")?;
        let s: Vec<f64> = slices.iter().map(|x| x["s"].as_f64().unwrap_or(0.)).collect();
        let a: Vec<f64> = slices.iter().map(|x| x["area"].as_f64().unwrap_or(0.)).collect();
        let measured = read_vvtf(&root.join(format!("extracted/subject-{speaker}/{name}/{name}-vvtf-measured.txt")))?;
        let mp = peaks(&measured);
        let mut engine = Vec::new();
        let mut ec = Vec::new();
        for p in physics {
            let c = engine_curve(&s, &a, p);
            let rms = band_rms(&measured, &c, 100., 1000.);
            engine.push((p.name.to_string(), peaks(&c), rms));
            ec.push(c);
        }
        curves.push((name.clone(), measured, ec));
        rows.push(Row { name, ipa: IPA[number - 1], measured: mp, engine });
    }
    // per-sound table and summaries
    let mut md = String::from(
        "| Sound | IPA | measured F1, F2 (Hz) | engine before: F1, F2 error % | after: F1, F2 error % | B1 measured / before / after (Hz) | 100–1000 Hz RMS dB before → after |\n|---|---|---|---|---|---|---|\n",
    );
    let mut summary: Vec<Vec<f64>> = vec![Vec::new(); 10];
    for r in &rows {
        let m = &r.measured;
        if m.len() < 2 {
            let _ = writeln!(md, "| {} | /{}/ | (fewer than two peaks) | | | | |", r.name, r.ipa);
            continue;
        }
        let mut cells = Vec::new();
        for (k, (_, e, rms)) in r.engine.iter().enumerate() {
            if e.len() < 2 {
                cells.push("–".to_string());
                continue;
            }
            let (e1, e2) = (pct(e[0].freq, m[0].freq), pct(e[1].freq, m[1].freq));
            summary[4 * k].push(e1);
            summary[4 * k + 1].push(e2);
            summary[4 * k + 2].push(*rms);
            summary[4 * k + 3].push(e[0].bw / m[0].bw);
            cells.push(format!("{e1:+.1}, {e2:+.1}"));
        }
        let bw = |e: &[Peak]| e.first().map(|p| format!("{:.0}", p.bw)).unwrap_or_default();
        let _ = writeln!(
            md,
            "| {} | /{}/ | {:.0}, {:.0} | {} | {} | {} / {} / {} | {:.1} → {:.1} |",
            r.name,
            r.ipa,
            m[0].freq,
            m[1].freq,
            cells[0],
            cells[1],
            bw(m),
            bw(&r.engine[0].1),
            bw(&r.engine[1].1),
            r.engine[0].2,
            r.engine[1].2
        );
    }
    let abs = |v: &Vec<f64>| median(v.iter().map(|x| x.abs()).collect());
    let head = format!(
        "Engine 1D acoustics of the MRI area functions against the transfer functions measured on DVTD's 3D prints (n = {n}).\n\n\
         | Physics | F1 error, median (signed / absolute) % | F2 error, median (signed / absolute) % | B1 engine / measured, median | 100–1000 Hz RMS dB, median |\n|---|---|---|---|---|\n\
         | before | {:+.1} / {:.1} | {:+.1} / {:.1} | {:.2} | {:.2} |\n| after | {:+.1} / {:.1} | {:+.1} / {:.1} | {:.2} | {:.2} |\n\n",
        median(summary[0].clone()),
        abs(&summary[0]),
        median(summary[1].clone()),
        abs(&summary[1]),
        median(summary[3].clone()),
        median(summary[2].clone()),
        median(summary[4].clone()),
        abs(&summary[4]),
        median(summary[5].clone()),
        abs(&summary[5]),
        median(summary[7].clone()),
        median(summary[6].clone()),
        n = summary[0].len()
    );
    println!("{head}");
    fs::write(out.join("acoustics.md"), head + &md).map_err(|e| e.to_string())?;
    write(&out.join("acoustics.json"), &rows)?;
    // a grid of curves, 0–2.5 kHz: measured (white), before (blue), after (amber)
    let (cols, w, h) = (6usize, 240., 150.);
    let rows_n = curves.len().div_ceil(cols);
    let mut svg = Svg::new(cols * 250 + 20, rows_n * 175 + 60, "|U_lips/U_glottis| 0–2.5 kHz, dB: DVTD 3D-print measurement (white), engine 1D before (blue) and after (amber) · DVTD, doi:10.1038/s41597-020-00597-w (CC0 data)");
    for (i, (name, m, e)) in curves.iter().enumerate() {
        let (x0, y0) = (20. + (i % cols) as f64 * 250., 50. + (i / cols) as f64 * 175.);
        svg.frame(x0, y0, w, h);
        svg.small(x0 + 4., y0 + 14., name);
        let map = |f: f64, db: f64| (x0 + f / 2500. * w, y0 + h - ((db + 30.) / 70.).clamp(0., 1.) * h);
        for (c, color, width) in [(m, "#e3edf3", 1.4), (&e[0], "#4fa3d9", 1.0), (&e[1], "#f2b33d", 1.0)] {
            let pts: Vec<_> = c.freqs.iter().zip(&c.db).filter(|(f, _)| **f <= 2500.).map(|(f, d)| map(*f, *d)).collect();
            svg.line(&pts, color, width, false);
        }
    }
    fs::write(out.join("acoustics.svg"), svg.end()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_uniform_tube_resonates_at_odd_quarter_wavelengths_at_the_print_speed() {
        // 17.5 cm, 4 cm²: c/4L = 493 Hz at 345 m/s, plus the radiation end correction
        let c = engine_curve(&[0., 17.5], &[4., 4.], Physics { name: "t", discontinuity: false, thermal: false });
        let p = peaks(&c);
        let ideal = C_PRINT / (4. * (17.5 + 0.85 * (4f64 / std::f64::consts::PI).sqrt()));
        assert!((p[0].freq / ideal - 1.).abs() < 0.02, "{} vs {ideal}", p[0].freq);
        assert!((p[1].freq / (3. * ideal) - 1.).abs() < 0.03, "{} vs {}", p[1].freq, 3. * ideal);
    }

    #[test]
    fn peaks_ignore_ripple_below_the_prominence() {
        let freqs = grid();
        let db = freqs.iter().map(|f| -((f - 500.) / 80.).powi(2) + 0.5 * (f / 7.).sin()).collect();
        let p = peaks(&Curve { freqs, db });
        // the ±0.5 dB ripple (44 Hz period) makes no second peak; on the flat top it may move the
        // one peak by up to half a period
        assert_eq!(p.len(), 1);
        assert!((p[0].freq - 500.).abs() < 25., "{}", p[0].freq);
    }
}

/// The voice's airway (`HumanAirway`) in one of three states, for `walls`.
#[derive(Clone, Copy, PartialEq)]
enum Walls {
    /// no yielding walls, no soft palate, no thermal loss or discontinuity inertance
    Rigid,
    /// the airway as the engine had it before: every section soft with `WALL_SOFT`
    Before,
    /// the airway as built now: tissue by region, the soft palate's membrane, thermal loss
    After,
}

fn voice_elements(air: &mut chimera::voice::human_airway::HumanAirway, a: &HumanArtic, w: Walls) -> chimera::airway::network::LinearElements {
    use chimera::physics::WALL_SOFT;
    air.apply(a, 0.0);
    if w != Walls::After {
        air.net.discontinuity = false;
        air.net.thermal = false;
        air.net.set_membrane(air.velum_membrane, 0.0, WALL_SOFT);
        let soft = if w == Walls::Rigid { 0.0 } else { 1.0 };
        for t in [air.pharynx, air.oral] {
            for i in 0..t.n {
                air.net.set_node_wall(t.node(i), WALL_SOFT, soft);
            }
        }
    }
    let el = air.net.linear_elements();
    // restore the airway's own state for the next posture
    air.net.discontinuity = true;
    air.net.thermal = true;
    air.net.set_membrane(air.velum_membrane, chimera::voice::human_airway::VELUM_AREA * air.body.scale * air.body.scale, chimera::voice::human_airway::velum_tissue(air.body.scale));
    el
}

/// `chimera tract3d walls <out-dir>`: what the yielding walls and the soft palate do to the
/// voice's airway, per vowel and body, against the human measurements (glottis closed):
/// formant bandwidths 50–90 Hz (men) and 70–90 Hz (women) between 300 Hz and 4 kHz, a tissue
/// resonance R0 ≈ 20 Hz and an anti-resonance near 200 Hz through the sealed lips
/// (Hanna, Smith & Wolfe 2016, doi:10.1121/1.4948754).
pub fn walls(args: &[String]) -> Result<(), String> {
    use chimera::airway::frequency::{find_peaks, input_impedance, radiator_transfer};
    use chimera::anatomy::human::{BODY_FEMALE, BODY_MALE};
    use chimera::voice::human_airway::HumanAirway;
    let out = Path::new(args.first().ok_or("tract3d walls <out-dir>")?);
    mkdir(out)?;
    let states = [(Walls::Rigid, "rigid"), (Walls::Before, "before"), (Walls::After, "after")];
    let freqs: Vec<f64> = (0..3991).map(|i| 10. + i as f64).collect();
    let vowels = ["i", "e", "ɛ", "a", "ɔ", "o", "u", "ə", "ɪ", "ʊ"];
    let mut md = String::new();
    let mut json = Vec::new();
    let mut svg = Svg::new(1500, 940, "The voice's airway, glottis closed: |U_out/U_glottis| (−60…+50 dB, 0–1.5 kHz) rigid (grey), before (blue), after (amber); the nose's share after (green)");
    for (bi, (bname, body)) in [("male", BODY_MALE), ("female", BODY_FEMALE)].into_iter().enumerate() {
        let mut air = HumanAirway::new(96000., body);
        let _ = writeln!(md, "\n### {bname}\n\n| Vowel | F1 rigid / before / after (Hz) | B1 rigid / before / after (Hz) | B2 before / after | B3 before / after | nose / lips at F1, after (dB) |\n|---|---|---|---|---|---|");
        let mut sums: Vec<Vec<f64>> = vec![Vec::new(); 9];
        for (vi, v) in vowels.iter().enumerate() {
            let a = posture(v, if bi == 0 { BodyMix::MALE } else { BodyMix::FEMALE });
            let mut peaks = Vec::new();
            let mut curves = Vec::new();
            let mut nose_db = f64::NAN;
            for (w, _) in states {
                let el = voice_elements(&mut air, &a, w);
                let rad = radiator_transfer(&el, air.pharynx.first_node(), &freqs, &[air.glottis_port]);
                let total: Vec<f64> = (0..freqs.len()).map(|i| rad[0][i] + rad[1][i]).collect();
                let p = find_peaks(&freqs, &total, 150.);
                if w == Walls::After {
                    if let Some(p1) = p.first() {
                        let i = (p1.freq - 10.).round() as usize;
                        nose_db = 20. * (rad[1][i] / rad[0][i].max(1e-12)).log10();
                    }
                    curves.push(rad[1].clone());
                }
                curves.insert(curves.len().saturating_sub(if w == Walls::After { 1 } else { 0 }), total);
                peaks.push(p);
            }
            let f = |s: usize, k: usize, bw: bool| peaks[s].get(k).map(|p| if bw { p.bw } else { p.freq }).unwrap_or(f64::NAN);
            let _ = writeln!(
                md,
                "| /{v}/ | {:.0} / {:.0} / {:.0} | {:.0} / {:.0} / {:.0} | {:.0} / {:.0} | {:.0} / {:.0} | {:.1} |",
                f(0, 0, false), f(1, 0, false), f(2, 0, false), f(0, 0, true), f(1, 0, true), f(2, 0, true), f(1, 1, true), f(2, 1, true), f(1, 2, true), f(2, 2, true), nose_db
            );
            for (i, x) in [f(0, 0, false), f(1, 0, false), f(2, 0, false), f(0, 0, true), f(1, 0, true), f(2, 0, true), f(1, 1, true), f(2, 1, true), nose_db].into_iter().enumerate() {
                sums[i].push(x);
            }
            json.push(json!({ "body": bname, "vowel": v, "peaks": peaks.iter().map(|p| p.iter().take(4).map(|q| [q.freq, q.bw, q.db]).collect::<Vec<_>>()).collect::<Vec<_>>(), "noseOverLipsAtF1Db": nose_db }));
            // panel: 0–1500 Hz
            let (x0, y0, w, h) = (20. + vi as f64 * 147., 60. + bi as f64 * 430., 140., 180.);
            svg.frame(x0, y0, w, h);
            svg.small(x0 + 4., y0 + 14., &format!("{bname} /{v}/"));
            // −60 … +50 dB (the rigid tube's F1 peak stays inside)
            let map = |fr: f64, db: f64| (x0 + fr / 1500. * w, y0 + h - ((db + 60.) / 110.).clamp(0., 1.) * h);
            for (c, color) in curves.iter().zip(["#7d8a93", "#4fa3d9", "#f2b33d", "#5fd38d"]) {
                let pts: Vec<_> = freqs.iter().zip(c).filter(|(fr, _)| **fr <= 1500.).map(|(fr, m)| map(*fr, 20. * m.max(1e-9).log10())).collect();
                svg.line(&pts, color, 1.0, false);
            }
        }
        let _ = writeln!(
            md,
            "| median | {:.0} / {:.0} / {:.0} | {:.0} / {:.0} / {:.0} | {:.0} / {:.0} | | {:.1} |",
            median(sums[0].clone()), median(sums[1].clone()), median(sums[2].clone()), median(sums[3].clone()), median(sums[4].clone()), median(sums[5].clone()), median(sums[6].clone()), median(sums[7].clone()), median(sums[8].clone())
        );
        // through the sealed lips (no radiation there), glottis closed, posture /ə/
        let a = posture("ə", if bi == 0 { BodyMix::MALE } else { BodyMix::FEMALE });
        let _ = writeln!(md, "\nThrough the sealed lips, /ə/: tissue resonance R0 (impedance minimum, 10–100 Hz) and anti-resonance R̄0 (maximum, 100–400 Hz):\n\n| State | R0 (Hz) | R̄0 (Hz) |\n|---|---|---|");
        for (si, (w, name)) in states.into_iter().enumerate() {
            let mut el = voice_elements(&mut air, &a, w);
            let lip = air.oral.last_node();
            el.rad_nodes.retain(|n| *n != lip);
            el.rad_area.truncate(el.rad_nodes.len());
            let z = input_impedance(&el, lip, &freqs, &[air.glottis_port]);
            let mag: Vec<f64> = z.iter().map(|(r, i)| (r * r + i * i).sqrt()).collect();
            let arg = |lo: f64, hi: f64, max: bool| -> f64 {
                let mut best = (f64::NAN, if max { -1. } else { f64::INFINITY });
                for (i, f) in freqs.iter().enumerate() {
                    if *f >= lo && *f <= hi && ((max && mag[i] > best.1) || (!max && mag[i] < best.1)) {
                        best = (*f, mag[i]);
                    }
                }
                best.0
            };
            let (r0, rb0) = if w == Walls::Rigid { (f64::NAN, f64::NAN) } else { (arg(10., 100., false), arg(100., 400., true)) };
            let _ = writeln!(md, "| {name} | {r0:.0} | {rb0:.0} |");
            let (x0, y0, wd, h) = (20. + si as f64 * 300., 300. + bi as f64 * 430., 280., 170.);
            svg.frame(x0, y0, wd, h);
            svg.small(x0 + 4., y0 + 14., &format!("{bname} /ə/ |Z| through the sealed lips, 10 Hz–1 kHz (log), {name}"));
            let pts: Vec<_> = freqs.iter().zip(&mag).filter(|(f, _)| **f <= 1000.).map(|(f, m)| (x0 + (f / 10.).log10() / 2. * wd, y0 + h - ((20. * m.max(1e-9).log10() - 0.) / 80.).clamp(0., 1.) * h)).collect();
            svg.line(&pts, ["#7d8a93", "#4fa3d9", "#f2b33d"][si], 1.2, false);
        }
    }
    println!("{md}");
    fs::write(out.join("walls.md"), md).map_err(|e| e.to_string())?;
    write(&out.join("walls.json"), &json)?;
    fs::write(out.join("walls.svg"), svg.end()).map_err(|e| e.to_string())
}
