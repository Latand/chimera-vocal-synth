//! Local-only evidence figures and the markdown table. Figures carry the dataset citation;
//! they are written to the output directory, never into the repository.
use super::sheet::Svg;
use super::*;
use std::fmt::Write as _;

const DVTD: &str = "DVTD, doi:10.1038/s41597-020-00597-w (CC0 data)";

fn mask(svg: &mut Svg, m: &volume::Slice, x: f64, y: f64, size: f64, color: &str) {
    let n = volume::MASK;
    let scale = size / n as f64;
    for row in 0..n {
        let mut col = 0;
        while col < n {
            if m.mask[row * n + col] == 0 {
                col += 1;
                continue;
            }
            let start = col;
            while col < n && m.mask[row * n + col] != 0 {
                col += 1;
            }
            svg.rect(x + start as f64 * scale, y + (n - 1 - row) as f64 * scale, (col - start) as f64 * scale, scale + 0.05, color);
        }
    }
}
/// The voxel plane through a point, lateral coordinate fixed: (x right, y up), glottis-aligned.
/// The tract's front (lips) is drawn to the right.
fn midsagittal(svg: &mut Svg, vol: &Volume, origin: [f64; 3], front: [f64; 3], ox: f64, oy: f64, scale: f64, color: &str) {
    let z = ((origin[2] - vol.lo[2]) / vol.h).round().clamp(0., (vol.dims[2] - 1) as f64) as usize;
    let flip = if front[0] < origin[0] { -1. } else { 1. };
    for y in 0..vol.dims[1] {
        let mut x = 0;
        while x < vol.dims[0] {
            if vol.vox[vol.index(x, y, z)] == 0 {
                x += 1;
                continue;
            }
            let first = x;
            while x < vol.dims[0] && vol.vox[vol.index(x, y, z)] != 0 {
                x += 1;
            }
            let (a, b) = (vol.lo[0] + first as f64 * vol.h - origin[0], vol.lo[0] + x as f64 * vol.h - origin[0]);
            let (a, b) = (flip * a, flip * b);
            let yy = oy - (vol.lo[1] + y as f64 * vol.h - origin[1]) * scale;
            svg.rect(ox + a.min(b) * scale, yy, (b - a).abs() * scale, vol.h * scale + 0.05, color);
        }
    }
}

pub fn reference(r: &Measurement, path: &Path) -> Result<(), String> {
    let mut svg = Svg::new(1400, 260, &format!("MRI reference perpendicular cuts, cm² and channel count · {DVTD}"));
    for j in 0..7 {
        let i = ((r.slices.len() - 1) as f64 * [0.15, 0.3, 0.45, 0.6, 0.75, 0.9, 0.95][j]).round() as usize;
        let q = &r.slices[i];
        mask(&mut svg, q, 20. + j as f64 * 195., 48., 180., "#e3edf3");
        svg.text(20. + j as f64 * 195., 245., &format!("{:.2} cm · {:.3} cm² · {} ch", q.s, q.area, q.channels));
    }
    fs::write(path, svg.end()).map_err(|e| e.to_string())
}

/// MRI reference next to the fitted model, both measured by the same voxel pipeline.
pub fn comparison(r: &Measurement, rvol: &Volume, fit: Option<(&Measurement, &Volume)>, default: Option<&Measurement>, title: &str, numbers: &Value, path: &Path) -> Result<(), String> {
    let mut svg = Svg::new(1500, 1560, &format!("{title} · {DVTD}"));
    let f3 = |v: &Value| v.as_f64().map_or("—".into(), |x| format!("{x:.3}"));
    let hz = |v: &Value| v.as_array().map_or("—".into(), |a| a.iter().filter_map(Value::as_f64).map(|x| format!("{x:.0}")).collect::<Vec<_>>().join(" / "));
    svg.text(16., 48., &format!("area functions vs normalised length: white MRI, amber fitted model, blue default (engine targets + derive). d fitted {}, default {}", f3(&numbers["fitD"]), f3(&numbers["defaultD"])));
    svg.small(16., 66., &format!("F1–F3 of the area functions (same 1D solver): MRI {} Hz; fitted {} Hz", hz(&numbers["referencePeaks"]), hz(&numbers["fitPeaks"])));
    let (px, py, w, h) = (50., 80., 1400., 220.);
    svg.line(&[(px, py), (px, py + h), (px + w, py + h)], "#3a5a6c", 1., false);
    let max = r.slices.iter().map(|q| q.area).chain(fit.iter().flat_map(|m| m.0.slices.iter().map(|q| q.area))).chain(default.iter().flat_map(|d| d.slices.iter().map(|q| q.area))).fold(1f64, f64::max);
    for (q, c, wd) in [(Some(r), "#e3edf3", 2.), (fit.map(|f| f.0), "#ffc573", 2.), (default, "#68b6e8", 1.2)] {
        if let Some(q) = q {
            let pts: Vec<_> = q.slices.iter().map(|a| (px + a.s / q.length * w, py + h - a.area / max * (h - 10.))).collect();
            svg.line(&pts, c, wd, false);
        }
    }
    svg.small(14., py + 10., &format!("{max:.1}"));
    svg.small(px, py + h + 14., "glottis 0");
    svg.small(px + w - 40., py + h + 14., "lips 1");
    let fractions = [0.1, 0.25, 0.4, 0.55, 0.7, 0.85, 0.95];
    svg.text(16., 340., "MRI cuts (8 × 8 cm, lateral →, towards the outer wall ↑)");
    svg.text(16., 580., "Fitted model cuts, same pipeline and scale");
    for (j, frac) in fractions.iter().enumerate() {
        let x = 16. + j as f64 * 210.;
        let q = &r.slices[((r.slices.len() - 1) as f64 * frac).round() as usize];
        svg.frame(x, 350., 200., 200.);
        mask(&mut svg, q, x, 350., 200., "#e3edf3");
        svg.small(x, 564., &format!("s/L {frac:.2} · {:.2} cm² · {} ch", q.area, q.channels));
        if let Some((m, _)) = fit {
            let q = &m.slices[((m.slices.len() - 1) as f64 * frac).round() as usize];
            svg.frame(x, 590., 200., 200.);
            mask(&mut svg, q, x, 590., 200., "#ffc573");
            svg.small(x, 804., &format!("{:.2} cm² · {} ch", q.area, q.channels));
        } else {
            svg.small(x, 690., "no connected model path");
        }
    }
    svg.text(16., 838., "Midsagittal planes, glottis-aligned, 15 px/cm: MRI lumen (white) and the fitted model's air (amber)");
    let o = r.slices[0].origin;
    let front = r.slices.last().unwrap().origin;
    midsagittal(&mut svg, rvol, o, front, 300., 1150., 15., "#e3edf3");
    if let Some((m, v)) = fit {
        let o = m.slices[0].origin;
        let front = m.slices.last().unwrap().origin;
        midsagittal(&mut svg, v, o, front, 900., 1150., 15., "#ffc573");
    }
    // the larynx and the lower pharynx cut by cut: every 5 mm from 5 mm above the glottis
    svg.text(16., 1200., "Lower tract, every 5 mm from 0.5 cm above the glottis (8 × 8 cm each): MRI above, the fitted model below");
    for j in 0..10 {
        let x = 16. + j as f64 * 147.;
        let s = 0.5 + 0.5 * j as f64;
        fn at(m: &Measurement, s: f64) -> Option<&volume::Slice> {
            m.slices.iter().min_by(|a, b| (a.s - s).abs().total_cmp(&(b.s - s).abs())).filter(|q| (q.s - s).abs() < 0.2)
        }
        if let Some(q) = at(r, s) {
            svg.frame(x, 1212., 140., 140.);
            mask(&mut svg, q, x, 1212., 140., "#e3edf3");
            svg.small(x, 1366., &format!("{s:.1} cm · {:.2} cm² · {} ch", q.area, q.channels));
        }
        if let Some(q) = fit.and_then(|(m, _)| at(m, s)) {
            svg.frame(x, 1380., 140., 140.);
            mask(&mut svg, q, x, 1380., 140., "#ffc573");
            svg.small(x, 1534., &format!("{:.2} cm² · {} ch", q.area, q.channels));
        }
    }
    fs::write(path, svg.end()).map_err(|e| e.to_string())
}

pub fn table(table: &Value, path: &Path) -> Result<(), String> {
    let mut s = format!("# Model against MRI ({DVTD})\n\nd: RMS log2 area ratio over normalised length (0.05 cm² floor). Fitted and default d use the common voxel pipeline on both sides.\n\n");
    let f = |v: &Value, scale: f64, digits: usize| v.as_f64().map_or("—".to_string(), |x| format!("{:.*}", digits, x * scale));
    s.push_str("| Speaker | Cases | Median fitted d | Max fitted d | Identified | Median default d | Median F1–F3 error (fitted) | Max | Slicer area error (median, worst) | Slicer channel agreement | Between-speaker d (scaled) |\n|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for g in table["groups"].as_array().unwrap() {
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {}/{} | {} | {} % | {} % | {} %, {} % | {} % | {} |",
            g["body"].as_str().unwrap_or(""),
            g["cases"],
            f(&g["medianFitD"], 1., 3),
            f(&g["maxFitD"], 1., 3),
            g["identified"],
            g["cases"],
            f(&g["medianDefaultD"], 1., 3),
            f(&g["medianFitResonanceErrorPercent"], 1., 1),
            f(&g["maxFitResonanceErrorPercent"], 1., 1),
            f(&g["medianSlicerAreaError"], 100., 1),
            f(&g["worstSlicerAreaError"], 100., 1),
            f(&g["medianSlicerChannelAgreement"], 100., 0),
            f(&g["medianBetweenSpeakerScaledD"], 1., 2),
        );
    }
    // cut shape and the lower pharynx per speaker (this project's measures against the dataset)
    s.push_str("\n| Speaker | Cut IoU, median (between the two speakers) | Cut aspect error, median (between speakers) | Lower pharynx: model's regions within ±1 where MRI splits | Sounds with fitted d ≤ 0.45 | Default d within its line | Sounds with a resonance over 10 % off |\n|---|---:|---:|---:|---:|---:|---:|\n");
    for g in table["groups"].as_array().unwrap() {
        let _ = writeln!(
            s,
            "| {} | {} ({}) | {} ({}) | {}/{} | {}/{} | {}/{} | {} |",
            g["body"].as_str().unwrap_or(""),
            f(&g["medianCutIou"], 1., 2),
            f(&g["medianBetweenSpeakerCutIou"], 1., 2),
            f(&g["medianCutAspectError"], 1., 2),
            f(&g["medianBetweenSpeakerCutAspectError"], 1., 2),
            g["lowerPharynxWithinOne"],
            g["lowerPharynxSplitCuts"],
            g["fitWithin045"],
            g["cases"],
            g["defaultWithinLine"],
            g["cases"],
            g["soundsWithResonanceOver10"],
        );
    }
    // This table goes into the repository's docs: it holds this project's errors only. The
    // reference's own measurements (where its constriction is, how wide, how many channels)
    // stay in table.json, outside git.
    s.push_str("\n| Case | IPA | Default d | Default area against MRI by stretch: larynx / pharynx / mouth / lips (×) | Fitted d | Between speakers (scaled) | Nearest reference | F1/F2/F3 error % (fitted) | Constriction, model against MRI: place (Δ s/L), area (×), channels | Mean cut IoU |\n|---|---|---:|---|---:|---:|---|---|---|---:|\n");
    for r in table["rows"].as_array().unwrap() {
        let peaks = r["fitResonanceErrorPercent"].as_array().map_or("—".into(), |a| a.iter().map(|x| f(x, 1., 0)).collect::<Vec<_>>().join(" / "));
        let (rc, fc) = (&r["referenceConstriction"], &r["fitConstriction"]);
        let con = match (rc["fraction"].as_f64(), fc["fraction"].as_f64(), rc["area"].as_f64(), fc["area"].as_f64(), rc["channels"].as_i64(), fc["channels"].as_i64()) {
            (Some(rp), Some(fp), Some(ra), Some(fa), Some(rn), Some(fnn)) => {
                let ratio = if ra > 0. { format!("×{:.2}", fa / ra) } else { "—".to_string() };
                let channels = if rn == fnn { "equal".to_string() } else { format!("{:+}", fnn - rn) };
                format!("{:+.2}, {ratio}, {channels}", fp - rp)
            }
            _ => "—".to_string(),
        };
        let iou = r["shapeMetrics"].as_array().map(|a| {
            let v: Vec<f64> = a.iter().filter_map(|x| x["iou"].as_f64()).collect();
            v.iter().sum::<f64>() / v.len().max(1) as f64
        });
        // (the geometric mean of the default model's area over the MRI's, per stretch of the tract)
        let regions = r["defaultRegionLog2"].as_array().map_or("—".into(), |a| a.iter().map(|x| x.as_f64().map_or("—".into(), |v| format!("{:.2}", v.exp2()))).collect::<Vec<_>>().join(" / "));
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            r["case"].as_str().unwrap_or(""),
            r["ipa"].as_str().unwrap_or(""),
            f(&r["defaultD"], 1., 3),
            regions,
            f(&r["fitD"], 1., 3),
            f(&r["betweenSpeakerScaledD"], 1., 2),
            if r["identified"] == true { "itself".to_string() } else { r["nearestReference"].as_str().unwrap_or("—").to_string() },
            peaks,
            con,
            iou.map_or("—".into(), |x| format!("{x:.2}")),
        );
    }
    fs::write(path, s).map_err(|e| e.to_string())
}
