//! Geometric checks of the tongue shapes and side cavities the research lists (validation plan
//! T-features and T-anatomy). Each check measures the model and compares with a line taken
//! from the DVTD measurements or a cited number; the CLI tabulates them and the tests hold them.
use super::lamina::Lamina;
use super::*;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub preset: String,
    pub feature: &'static str,
    pub measured: String,
    pub line: &'static str,
    pub pass: bool,
    /// how far the measurement is from its line (0 when it passes), for tuning presets
    #[serde(skip)]
    pub slack: f64,
}

/// Depth (cm) of the main region's lower outline at the midline below the higher of its two
/// shoulders, the highest points of that outline 0.4–1.0 cm to either side (inside the inner
/// 90 % of the region): > 0 grooved, < 0 domed (metric M4).
pub fn groove_index(c: &Cut) -> Option<f64> {
    let main = c.regions.iter().position(|r| r.min_z <= 0. && r.max_z >= 0.)?;
    let r = &c.regions[main];
    let reach = (0.45 * (r.max_z - r.min_z)).min(1.0);
    let (mut l0, mut left, mut right, mut best) = (f64::NAN, f64::NEG_INFINITY, f64::NEG_INFINITY, f64::INFINITY);
    for row in &c.rows {
        let lo = row.intervals[..row.count].iter().filter(|q| q.component == main).map(|q| q.lo).fold(f64::INFINITY, f64::min);
        if !lo.is_finite() || row.z.abs() > reach {
            continue;
        }
        if row.z.abs() < best {
            best = row.z.abs();
            l0 = lo;
        }
        if row.z <= -0.4 {
            left = left.max(lo);
        } else if row.z >= 0.4 {
            right = right.max(lo);
        }
    }
    let shoulder = left.max(right);
    (l0.is_finite() && shoulder.is_finite()).then(|| shoulder - l0)
}

/// Concavity of the tongue's own surface at dorsal point i: the highest rise (cm, along the
/// surface normal) of the surface 0.3–1.0 cm to either side above its midline point.
pub fn surface_groove(p: &Parts, i: usize) -> f64 {
    let c = p.dorsal_at(i, 0.);
    let reach = (0.6 * p.dw[i]).min(1.0 * p.map.k);
    let mut best = f64::NEG_INFINITY;
    for j in 0..=8 {
        let z = lerp(0.3 * p.map.k, reach.max(0.3 * p.map.k), j as f64 / 8.);
        for side in [-1., 1.] {
            let q = p.dorsal_at(i, side * z);
            best = best.max((q.x - c.x) * p.dn[i].x + (q.y - c.y) * p.dn[i].y);
        }
    }
    best
}
/// The largest surface concavity over the dorsal points with t in [lo, hi].
fn max_surface_groove(p: &Parts, lo: f64, hi: f64) -> f64 {
    (0..p.dorsal.len()).filter(|&i| p.dt[i] >= lo && p.dt[i] <= hi).map(|i| surface_groove(p, i)).fold(f64::NEG_INFINITY, f64::max)
}

fn cut_at(f: &Tract3dFrame, lo: f64, hi: f64, key: impl Fn(&Cut) -> f64) -> Option<&Cut> {
    let l = f.area_fn.length;
    f.cuts().iter().filter(|c| c.s / l >= lo && c.s / l <= hi).min_by(|a, b| key(a).total_cmp(&key(b)))
}

impl Tract3d {
    /// Air along a line in one lamina (the midsagittal one is `lamina_count() / 2`).
    pub fn line_air(&self, lamina: usize, o: Vec2, u: Vec2, out: &mut Row) {
        cut::intervals(&self.laminae[lamina], o, u, out);
    }
    pub(super) fn lamina(&self, i: usize) -> &Lamina {
        &self.laminae[i]
    }
}

/// All feature checks for the presets of one body.
pub fn check_presets(body: super::super::human::BodyMix) -> Vec<Check> {
    let mut m = Tract3d::new(body.body());
    let mut f = Tract3dFrame::default();
    let mut out = Vec::new();
    let mut s_at = 0.9;
    let all = presets(body);
    // /ʃ/ is judged against where /s/ narrows
    if let Some(p) = all.iter().find(|p| p.name == "s") {
        if m.compute(&p.artic, &p.shape, &mut f).is_ok() {
            if let Some(c) = cut_at(&f, 0.8, 0.97, |c| c.area) {
                s_at = c.s / f.area_fn.length;
            }
        }
    }
    for p in &all {
        if m.compute(&p.artic, &p.shape, &mut f).is_ok() {
            check_one(&m, &f, &p.name, &p.shape, s_at, body.body().scale, &mut out);
        }
    }
    out
}

/// The checks of one named shape on a computed posture; `s_at` is where /s/ narrows (s/L).
pub fn check_one(m: &Tract3d, f: &Tract3dFrame, name: &str, shape: &ShapeParams, s_at: f64, k: f64, out: &mut Vec<Check>) {
    let l = f.area_fn.length;
    let mut push = |feature, measured: String, line, slack: f64| out.push(Check { preset: name.into(), feature, measured, line, pass: slack <= 0., slack: slack.max(0.) });
    let hinge = |x: f64| x.max(0.);
    match name {
        "s" => {
            let c = cut_at(f, 0.8, 0.97, |c| c.area).unwrap();
            let at = c.s / l;
            push(
                "narrow single channel",
                format!("{:.3} cm², {} channel(s) at s/L {:.2}", c.area, c.channels, at),
                "≤ 0.20 cm², 1 channel, s/L 0.85–0.95 (DVTD 0.15/0.12 cm² at 0.90/0.93)",
                hinge(c.area - 0.2) * 5. + (c.channels as f64 - 1.).abs() * 0.3 + hinge(0.85 - at) * 5. + hinge(at - 0.95) * 5.,
            );
            let behind = f.cuts().iter().filter(|q| q.s < c.s && q.s > c.s - 1.5).filter_map(groove_index).fold(f64::NEG_INFINITY, f64::max);
            push("grooved (concave) tongue behind the channel", format!("groove index {behind:.2} cm"), "> 0 (concave lower outline)", if behind.is_finite() { hinge(0.02 - behind) } else { 1. });
        }
        "ʃ" => {
            let c = cut_at(f, 0.6, 0.95, |c| c.area).unwrap();
            let at = c.s / l;
            let thin = f.cuts().iter().filter(|q| q.s < c.s && q.s > c.s - 2.5 && q.height < 0.6 && q.area < 1.5).count();
            push("constriction behind /s/", format!("{:.3} cm² at s/L {at:.2} (/s/ at {s_at:.2})", c.area), "≤ 0.15 cm², further back than /s/ (DVTD 0.10 cm² at 0.79–0.88)", hinge(c.area - 0.15) * 5. + hinge(at - s_at + 0.01) * 5.);
            push("long thin channel behind it", format!("{thin} cuts within 2.5 cm behind, under 0.6 cm high"), "≥ 4 (≥ 1 cm of thin channel)", hinge(4. - thin as f64) * 0.2);
            let g = max_surface_groove(&m.parts, 0.7, 0.97);
            push("domed blade without a groove", format!("largest surface concavity {g:.2} cm over the blade"), "≤ 0.05 cm", hinge(g - 0.05));
        }
        "l" => {
            let c = f.cuts().iter().filter(|c| c.s / l > 0.7 && !c.regions.iter().any(|r| r.min_z <= 0. && r.max_z >= 0.) && c.channels >= 2).max_by(|a, b| a.area.total_cmp(&b.area));
            match c {
                Some(c) => {
                    let mut rs: Vec<&Region> = c.regions.iter().collect();
                    rs.sort_by(|a, b| b.area.total_cmp(&a.area));
                    let (a, b) = (rs[0], rs[1]);
                    let sep = ((a.min_z + a.max_z) - (b.min_z + b.max_z)).abs() * 0.5;
                    let (left, right) = if a.max_z < b.max_z { (a, b) } else { (b, a) };
                    push("two lateral channels at the tip contact", format!("{} channels at s/L {:.2}, {:.2} cm apart", c.channels, c.s / l, sep), "2 channels, 2.0–2.8 cm apart (DVTD)", (c.channels as f64 - 2.).abs() * 0.3 + hinge(2.0 - sep) + hinge(sep - 2.8));
                    push("channel areas follow sideAsym", format!("right/left {:.2}/{:.2} cm² with sideAsym {:+.2}", right.area, left.area, shape.side_asym), "right > left when sideAsym > 0", if (right.area > left.area) == (shape.side_asym > 0.) { 0. } else { 0.1 });
                }
                None => push("two lateral channels at the tip contact", "no cut with the midline closed and two channels".into(), "2 channels, 2.0–2.8 cm apart (DVTD)", 1.),
            }
        }
        "retroflex" => {
            let tip = *m.parts.dorsal.last().unwrap();
            let mut row = Row::default();
            m.line_air(ROWS / 2, v(tip.x - 0.25 * k, tip.y), v(0., -1.), &mut row);
            let under = row.intervals[..row.count].iter().find(|q| q.lo > 0.02 && q.lo < 1.5).map_or(0., |q| q.hi - q.lo);
            push("air under the raised tip (sublingual)", format!("{under:.2} cm of air below the tip in the midsagittal plane"), "≥ 0.3 cm", hinge(0.3 - under));
            let c = cut_at(f, 0.7, 0.92, |c| c.area).unwrap();
            push("palatal constriction at the curled tip", format!("{:.2} cm² at s/L {:.2}", c.area, c.s / l), "≤ 0.8 cm² at s/L 0.7–0.92", hinge(c.area - 0.8));
        }
        "bunched" => {
            let a = &f.area_fn;
            let local = |lo: f64, hi: f64| (0..a.s.len()).filter(|&i| a.s[i] / l >= lo && a.s[i] / l <= hi).min_by(|&x, &y| a.area[x].total_cmp(&a.area[y]));
            let ph = local(0.15, 0.45).unwrap();
            let pal = local(0.5, 0.85).unwrap();
            let between = (ph..pal).map(|i| a.area[i]).fold(0f64, f64::max);
            let front: f64 = (pal..a.s.len()).map(|i| a.area[i] * STEP).sum();
            push("pharyngeal and palatal constrictions", format!("{:.2} cm² at s/L {:.2}, {:.2} cm² at s/L {:.2}; {:.2} cm² between", a.area[ph], a.s[ph] / l, a.area[pal], a.s[pal] / l, between), "both below 60 % of the area between them", hinge(a.area[ph] - 0.6 * between) + hinge(a.area[pal] - 0.6 * between));
            push("large front cavity", format!("{front:.1} cm³ in front of the palatal constriction"), "≥ 10 cm³", hinge(10. - front) * 0.1);
        }
        "flat" => {
            let g = max_surface_groove(&m.parts, 0.2, 0.97);
            push("no concavity along the tongue", format!("largest surface concavity {g:.3} cm"), "≤ 0.05 cm", hinge(g - 0.05));
        }
        "funnel" => {
            let pa = &m.parts;
            let n = pa.dorsal.len();
            let mut rows = Vec::new();
            for t in [0.6, 0.75, 0.9] {
                let i = (0..n).min_by(|&a, &b| (pa.dt[a] - t).abs().total_cmp(&(pa.dt[b] - t).abs())).unwrap();
                let d = pa.dorsal_at(i, pa.dgw[i]);
                let c = pa.dorsal_at(i, 0.);
                let depth = (d.x - c.x) * pa.dn[i].x + (d.y - c.y) * pa.dn[i].y;
                rows.push((t, pa.dgw[i], depth));
            }
            let slack: f64 = rows.windows(2).map(|w| hinge(w[1].1 - w[0].1) + hinge(w[0].2 - w[1].2)).sum();
            push("groove narrows and deepens towards the tip", rows.iter().map(|(t, w, d)| format!("t {t:.2}: {w:.2} wide, {d:.2} deep")).collect::<Vec<_>>().join("; "), "width falls and depth rises from t 0.6 to 0.9", slack);
        }
        "cupped" => {
            let g = max_surface_groove(&m.parts, 0.3, 0.9);
            push("margins raised above the midline", format!("surface rise {g:.2} cm"), "> 0.1 cm", hinge(0.1 - g));
        }
        "ə" => {
            for c in f.cavities.iter().filter(|c| c.kind == "piriform" && c.side > 0) {
                let (lo, hi) = (2.1 * k * k * k, 2.9 * k * k * k);
                push("piriform fossa volume", format!("{:.2} cm³ each", c.volume), "2.1–2.9 cm³ (male; scaled by size³) [R12]", hinge(lo - c.volume) + hinge(c.volume - hi));
                push("piriform fossa depth", format!("{:.2} cm", c.depth), "1.6–2.0 cm (male; scaled by size) [R12]", hinge(1.6 * k - c.depth) + hinge(c.depth - 2.0 * k));
            }
            let mid = m.lamina(ROWS / 2);
            let off = m.lamina(ROWS / 2 + (0.5 / Z_STEP) as usize);
            let at_mid = mid.air[AIR_VALLECULA[0]].len() + mid.air[AIR_VALLECULA[1]].len();
            let at_off = off.air[AIR_VALLECULA[0]].len() + off.air[AIR_VALLECULA[1]].len();
            push("valleculae off the midline only", format!("pocket outline points at z = 0: {at_mid}, at z = 0.5 cm: {at_off}"), "absent at z = 0, present at 0.5 cm", if at_mid == 0 && at_off > 0 { 0. } else { 1. });
        }
        _ => {}
    }
}
