//! A one-page drawing of a model posture: laminae, perpendicular cuts, planes and the area
//! function. Made from the engine's own geometry only (no reference data).
use super::*;
use chimera::anatomy::geom::v;
use chimera::anatomy::tract3d::{Cut, Row};
use std::fmt::Write as _;

pub struct Svg {
    pub s: String,
}
impl Svg {
    pub fn new(w: usize, h: usize, title: &str) -> Self {
        let mut s = String::new();
        let _ = write!(
            s,
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}"><rect width="100%" height="100%" fill="#05090d"/><g font-family="sans-serif" fill="#cfe3ef" font-size="13"><text x="16" y="24" font-size="15">{}</text>"##,
            esc(title)
        );
        Self { s }
    }
    pub fn text(&mut self, x: f64, y: f64, t: &str) {
        let _ = write!(self.s, r#"<text x="{x:.1}" y="{y:.1}">{}</text>"#, esc(t));
    }
    pub fn small(&mut self, x: f64, y: f64, t: &str) {
        let _ = write!(self.s, r##"<text x="{x:.1}" y="{y:.1}" font-size="11" fill="#8fb0c2">{}</text>"##, esc(t));
    }
    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, fill: &str) {
        let _ = write!(self.s, r#"<rect x="{x:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}" fill="{fill}"/>"#);
    }
    pub fn frame(&mut self, x: f64, y: f64, w: f64, h: f64) {
        let _ = write!(self.s, r##"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="none" stroke="#1d3a4c"/>"##);
    }
    pub fn line(&mut self, pts: &[(f64, f64)], stroke: &str, width: f64, close: bool) {
        if pts.len() < 2 {
            return;
        }
        let d: Vec<String> = pts.iter().map(|(x, y)| format!("{x:.2},{y:.2}")).collect();
        let tag = if close { "polygon" } else { "polyline" };
        let _ = write!(self.s, r#"<{tag} points="{}" fill="none" stroke="{stroke}" stroke-width="{width}"/>"#, d.join(" "));
    }
    pub fn poly(&mut self, pts: &[(f64, f64)], fill: &str) {
        let d: Vec<String> = pts.iter().map(|(x, y)| format!("{x:.2},{y:.2}")).collect();
        let _ = write!(self.s, r#"<polygon points="{}" fill="{fill}"/>"#, d.join(" "));
    }
    pub fn end(mut self) -> String {
        self.s.push_str("</g></svg>");
        self.s
    }
}
pub fn esc(t: &str) -> String {
    t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Air of a cut as filled trapezoids between neighbouring rows (no stair steps). `map` takes
/// (z, a) in cm to the page.
pub fn cut_fill(svg: &mut Svg, c: &Cut, fill: &str, map: &dyn Fn(f64, f64) -> (f64, f64)) {
    let rows: &[Row] = &c.rows;
    for k in 0..rows.len() {
        let r = &rows[k];
        let dz = if k + 1 < rows.len() { rows[k + 1].z - r.z } else { r.z - rows[k - 1].z };
        for q in &r.intervals[..r.count] {
            // join to the overlapping interval of the next row, else a half-step cap
            let next = rows.get(k + 1).and_then(|n| n.intervals[..n.count].iter().find(|p| p.lo <= q.hi && q.lo <= p.hi));
            let prev = k.checked_sub(1).and_then(|i| {
                let n = &rows[i];
                n.intervals[..n.count].iter().find(|p| p.lo <= q.hi && q.lo <= p.hi)
            });
            match next {
                Some(p) => svg.poly(&[map(r.z, q.lo), map(r.z, q.hi), map(r.z + dz, p.hi), map(r.z + dz, p.lo)], fill),
                None => svg.poly(&[map(r.z, q.lo), map(r.z, q.hi), map(r.z + 0.5 * dz, q.hi), map(r.z + 0.5 * dz, q.lo)], fill),
            }
            if prev.is_none() {
                svg.poly(&[map(r.z - 0.5 * dz, q.lo), map(r.z - 0.5 * dz, q.hi), map(r.z, q.hi), map(r.z, q.lo)], fill);
            }
        }
    }
}

/// Air of a lamina, rasterised by horizontal lines (x, y in cm; `map` to the page).
pub fn lamina_fill(svg: &mut Svg, m: &Tract3d, z: f64, y0: f64, y1: f64, step: f64, fill: &str, map: &dyn Fn(f64, f64) -> (f64, f64)) {
    let mut row = Row::default();
    let mut y = y0;
    while y <= y1 {
        m.air_intervals(y, z, &mut row);
        for q in &row.intervals[..row.count] {
            let (a, b) = (map(q.lo, y + step * 0.5), map(q.hi, y - step * 0.5));
            svg.rect(a.0, a.1, (b.0 - a.0).max(0.3), (b.1 - a.1).abs() + 0.3, fill);
        }
        y += step;
    }
}

pub const COLORS: [&str; 8] = ["#f077a9", "#68b6e8", "#9dbbf6", "#65a6ce", "#8bdfbc", "#efb778", "#e0d2a9", "#7195bd"];

pub fn sheet(m: &Tract3d, f: &Tract3dFrame, title: &str) -> String {
    let k = m.tract.body.scale;
    let mut svg = Svg::new(1600, 1240, title);
    let g = f.centreline[0];
    // ---- midsagittal and three parasagittal laminae
    let panels = [(0., 16., 50., 560., 34.), (0.6, 600., 50., 320., 20.), (1.2, 930., 50., 320., 20.), (1.8, 1260., 50., 320., 20.)];
    for (z, px, py, w, sc) in panels {
        let z = z * k;
        svg.frame(px, py, w, w * 0.92);
        svg.text(px + 8., py + 18., &format!("lamina z = {z:.1} cm"));
        let ox = px + w * 0.32;
        let oy = py + w * 0.86;
        let map = move |x: f64, y: f64| (ox + (x - g.x) * sc, oy - (y - g.y) * sc);
        lamina_fill(&mut svg, m, z, g.y - 0.5, g.y + 13. * k, 0.04, "#ffc573", &map);
        let i = (z / (chimera::anatomy::tract3d::Z_STEP * k)).round() as isize + (m.lamina_count() / 2) as isize;
        let (_, tissue) = m.lamina_polygons(i.clamp(0, m.lamina_count() as isize - 1) as usize);
        for (j, poly) in tissue.iter().enumerate() {
            if poly.len() > 2 {
                let pts: Vec<_> = poly.iter().map(|q| map(q.x, q.y)).filter(|p| p.0 > px && p.0 < px + w && p.1 > py && p.1 < py + w * 0.92).collect();
                svg.line(&pts, COLORS[j % COLORS.len()], 0.8, false);
            }
        }
        if z == 0. {
            let pts: Vec<_> = f.centreline.iter().map(|q| map(q.x, q.y)).collect();
            svg.line(&pts, "#ff5fa2", 1., false);
            for (n, c) in f.cuts().iter().enumerate() {
                if n % 8 == 0 {
                    let a = map(c.origin.x - c.u.x * 1.2, c.origin.y - c.u.y * 1.2);
                    let b = map(c.origin.x + c.u.x * 1.2, c.origin.y + c.u.y * 1.2);
                    svg.line(&[a, b], "#ff5fa2", 0.6, false);
                    svg.small(b.0 + 2., b.1, &format!("{:.0}", c.s));
                }
            }
        }
    }
    // ---- perpendicular cuts at fractions of the length
    let fr = [0.04, 0.1, 0.17, 0.25, 0.33, 0.42, 0.5, 0.58, 0.66, 0.74, 0.82, 0.88, 0.94, 0.98];
    let n = f.cuts().len();
    let (cw, sc) = (108., 15.);
    for (j, &t) in fr.iter().enumerate() {
        let c = &f.cuts()[((n - 1) as f64 * t).round() as usize];
        let px = 16. + (j % 14) as f64 * (cw + 4.);
        let py = 590.;
        svg.frame(px, py, cw, cw);
        let (cx, cy) = (px + cw / 2., py + cw / 2.);
        svg.line(&[(px, cy), (px + cw, cy)], "#132632", 0.5, false);
        svg.line(&[(cx, py), (cx, py + cw)], "#132632", 0.5, false);
        cut_fill(&mut svg, c, "#ffc573", &|z, a| (cx + z * sc, cy - a * sc));
        svg.small(px + 2., py + cw + 13., &format!("s {:.1} · {:.2} cm²", c.s, c.area));
        svg.small(px + 2., py + cw + 26., &format!("{} ch · {:.1}×{:.1}", c.channels, c.width, c.height));
    }
    svg.small(16., 584., "Perpendicular cuts (lateral →, towards the outer wall ↑), 15 px/cm; amber: air in the band");
    // ---- area function
    {
        let (px, py, w, h) = (16., 760., 1000., 300.);
        svg.frame(px, py, w, h);
        let af = &f.area_fn;
        let max = af.area.iter().cloned().fold(1f64, f64::max);
        let x = |s: f64| px + 40. + s / af.length * (w - 60.);
        let y = |a: f64| py + h - 30. - a / max * (h - 60.);
        let pts: Vec<_> = af.s.iter().zip(&af.area).map(|(s, a)| (x(*s), y(*a))).collect();
        svg.line(&pts, "#ffc573", 2., false);
        for (i, &c) in af.channels.iter().enumerate() {
            if c > 1 {
                svg.line(&[(x(af.s[i]), y(af.area[i]) - 3.), (x(af.s[i]), y(af.area[i]) + 3.)], "#68b6e8", 3., false);
            }
        }
        svg.text(px + 8., py + 18., &format!("area function: {:.2} cm, {:.1} cm³, min {:.3} cm² at {:.1} cm; blue ticks: several channels", af.length, af.volume, af.min_area, af.min_s));
        svg.small(px + 4., py + 40., &format!("{max:.1} cm²"));
        svg.small(px + 40., py + h - 10., "glottis");
        svg.small(px + w - 60., py + h - 10., "lips");
    }
    // ---- coronal and axial planes through the oral cavity
    let mut c = Cut::default();
    for (j, (label, o, u)) in [("coronal x = 4.5", v(4.5 * k, 8. * k), v(0., 1.)), ("axial y = 8.3", v(4. * k, 8.3 * k), v(1., 0.)), ("coronal x = 0 (pharynx)", v(0., 5. * k), v(0., 1.))].iter().enumerate() {
        m.cut_plane(*o, *u, &mut c);
        let (px, py, w) = (1030. + j as f64 * 188., 760., 182.);
        svg.frame(px, py, w, 300.);
        svg.small(px + 4., py + 14., label);
        let (cx, cy) = (px + w / 2., py + 150.);
        cut_fill(&mut svg, &c, "#8bdfbc", &|z, a| (cx + z * 20., cy - a * 20.));
        svg.small(px + 4., py + 294., &format!("{:.2} cm², {} regions", c.area, c.channels));
    }
    // ---- side cavities
    let mut y = 1090.;
    for cav in &f.cavities {
        svg.small(16., y, &format!("{} {}: {:.2} cm³, depth {:.2} cm, neck {:.2} cm², at s = {:.1} cm", cav.kind, if cav.side < 0 { "left" } else { "right" }, cav.volume, cav.depth, cav.neck_area, cav.attachment));
        y += 15.;
    }
    svg.end()
}

/// One cut, large, with every tissue polygon's span along each lamina's line drawn in its own
/// colour under the air (for finding which element bounds the air).
pub fn cut_detail(m: &Tract3d, c: &Cut, title: &str) -> String {
    let mut svg = Svg::new(1200, 900, title);
    let (cx, cy, sc) = (600., 450., 60.);
    let names = ["tongue", "epiglottis", "upper lip", "lower lip", "upper teeth", "lower teeth", "pharynx wall", "pharynx wall", "fauces", "cheek", "cheek"];
    for i in 0..m.lamina_count() {
        let (_, tissue) = m.lamina_polygons(i);
        let z = m.lamina_z(i);
        for (j, poly) in tissue.iter().enumerate() {
            if poly.len() < 3 {
                continue;
            }
            // crossings of this polygon with the cut's line
            let mut ts: Vec<f64> = Vec::new();
            for e in 0..poly.len() {
                let a = poly[e];
                let b = poly[(e + 1) % poly.len()];
                let (ax, ay) = (a.x - c.origin.x, a.y - c.origin.y);
                let (bx, by) = (b.x - c.origin.x, b.y - c.origin.y);
                let sa = c.u.x * ay - c.u.y * ax;
                let sb = c.u.x * by - c.u.y * bx;
                if (sa >= 0.) == (sb >= 0.) {
                    continue;
                }
                let den = c.u.x * (by - ay) - c.u.y * (bx - ax);
                ts.push((ax * (by - ay) - ay * (bx - ax)) / den);
            }
            ts.sort_by(f64::total_cmp);
            for p in ts.chunks(2) {
                if p.len() == 2 {
                    let (lo, hi) = (p[0].max(-6.), p[1].min(6.));
                    if hi > lo {
                        svg.rect(cx + z * sc - 2., cy - hi * sc, 4., (hi - lo) * sc, COLORS[j % COLORS.len()]);
                    }
                }
            }
        }
    }
    cut_fill(&mut svg, c, "#ffffffcc", &|z, a| (cx + z * sc, cy - a * sc));
    for (j, n) in names.iter().enumerate().take(9) {
        svg.small(20., 50. + j as f64 * 16., n);
        svg.rect(100., 41. + j as f64 * 16., 20., 10., COLORS[j % COLORS.len()]);
    }
    svg.small(20., 880., &format!("s {:.2} cm, {:.3} cm², {} channels; 60 px/cm; white: air kept in the band", c.s, c.area, c.channels));
    svg.end()
}
