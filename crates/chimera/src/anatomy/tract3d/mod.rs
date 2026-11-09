//! Parametric tissue geometry for inspection and future acoustics. No voice calls this module.
//! Units: centimetres; x forward, y superior, z speaker-right. Meshes, cuts and air queries
//! all use the same parasagittal polygons. See docs/tract3d/model-design.md.
mod cut;
mod lamina;
mod mesh;
mod params;
mod presets;
use super::geom::{add, dist, lerp_v, norm, scale, sub, v, Vec2};
use super::human::{ArticKey, HumanArtic, HumanBody, HumanFrame, HumanVocalTract};
use crate::math::{cos, lerp, pow, sin, smoothstep, sqrt, PI};
pub use cut::{Cut, Interval, Region, Row};
use lamina::{ellipse, station, Lamina};
pub use mesh::ElementMesh;
pub use params::{ParamSpec, ShapeParams, SPECS};
pub use presets::{presets, Preset};
use serde::Serialize;
pub const ELEMENTS: [&str; 8] = [
    "tongue", "palate", "velum", "pharynx", "larynx", "lips", "teethJaw", "cheeks",
];
pub const SURFACE_OWNERS: [usize; 15] = [0, 1, 2, 3, 4, 5, 6, 7, 5, 6, 6, 4, 4, 4, 4];
pub const MAX_CUTS: usize = 128;
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SideCavity {
    pub kind: &'static str,
    pub side: i8,
    pub attachment: f64,
    pub volume: f64,
    pub neck_area: f64,
    pub depth: f64,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaFunction {
    pub s: Vec<f64>,
    pub area: Vec<f64>,
    pub weighted_area: Vec<f64>,
    pub perimeter: Vec<f64>,
    pub channels: Vec<usize>,
    pub width: Vec<f64>,
    pub height: Vec<f64>,
    pub length: f64,
    pub volume: f64,
    pub min_area: f64,
    pub min_s: f64,
}
impl Default for AreaFunction {
    fn default() -> Self {
        Self {
            s: Vec::with_capacity(MAX_CUTS),
            area: Vec::with_capacity(MAX_CUTS),
            weighted_area: Vec::with_capacity(MAX_CUTS),
            perimeter: Vec::with_capacity(MAX_CUTS),
            channels: Vec::with_capacity(MAX_CUTS),
            width: Vec::with_capacity(MAX_CUTS),
            height: Vec::with_capacity(MAX_CUTS),
            length: 0.,
            volume: 0.,
            min_area: 0.,
            min_s: 0.,
        }
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tract3dFrame {
    pub centreline: Vec<Vec2>,
    pub slices: Vec<Cut>,
    pub area_fn: AreaFunction,
    pub cavities: Vec<SideCavity>,
    pub shape: ShapeParams,
}
impl Default for Tract3dFrame {
    fn default() -> Self {
        Self {
            centreline: Vec::with_capacity(MAX_CUTS),
            slices: (0..MAX_CUTS).map(|_| Cut::default()).collect(),
            area_fn: AreaFunction::default(),
            cavities: Vec::with_capacity(8),
            shape: ShapeParams::default(),
        }
    }
}
pub struct Tract3d {
    pub tract: HumanVocalTract,
    pub midline: HumanFrame,
    pub shape: ShapeParams,
    pub artic: HumanArtic,
    laminae: Vec<Lamina>,
    path: Vec<Vec2>,
    tmp: Vec<Vec2>,
    cum: Vec<f64>,
}
impl Tract3d {
    pub fn new(body: HumanBody) -> Self {
        Self {
            tract: HumanVocalTract::new(body),
            midline: HumanFrame::with_capacity(),
            shape: ShapeParams::default(),
            artic: super::human::NEUTRAL,
            laminae: (0..cut::ROWS)
                .map(|i| Lamina::new((i as f64 - 32.) * cut::Z_STEP * body.scale))
                .collect(),
            path: Vec::with_capacity(100),
            tmp: Vec::with_capacity(100),
            cum: Vec::with_capacity(100),
        }
    }
    pub fn specs() -> &'static [ParamSpec] {
        SPECS
    }
    pub fn derive(&self, a: &HumanArtic) -> ShapeParams {
        let mut p = ShapeParams::default();
        let high = smoothstep(5.5, 7.4, a.tb_y);
        p.side_back = 0.3 * high;
        p.side_dorsum = 0.35 * high + 0.45 * a.dorsum - 0.24 * a.lateral;
        p.side_blade = -0.8 * a.lateral - 0.25 * a.tt_curl;
        p.side_tip = -0.8 * a.lateral;
        p.side_root = 0.2 * (-a.root).max(0.);
        let ridge = smoothstep(7.8, 9.3, a.tt_y) * (1. - a.lateral) * (1. - a.tt_curl);
        p.groove_blade = 0.25 * ridge;
        p.groove_tip = 0.2 * ridge;
        p.side_blade += 1.2 * ridge;
        p.side_tip += 1.4 * ridge;
        p.tip_width = 0.55 + 0.75 * ridge - 0.2 * a.tt_curl;
        p.lip_corner_shape = 1.5 + 1.5 * (1. - a.lip_width);
        p
    }
    pub fn compute(
        &mut self,
        a: &HumanArtic,
        offsets: &ShapeParams,
        out: &mut Tract3dFrame,
    ) -> Result<(), String> {
        for k in ArticKey::ALL {
            if !a.get(k).is_finite() {
                return Err(format!("{} must be finite", k.name()));
            }
        }
        self.artic = *a;
        self.shape = self.derive(a).with_offsets(offsets)?;
        self.tract.compute(a, &mut self.midline);
        self.build_laminae();
        self.path.clear();
        self.path.push(self.midline.glottis);
        // Include the epilarynx rather than jumping from the glottis to the first grid point.
        for i in 1..6 {
            self.path.push(add(
                self.midline.glottis,
                v(
                    0.,
                    i as f64 * 0.4 * self.tract.body.scale * self.tract.body.pharynx,
                ),
            ));
        }
        self.path.extend(self.midline.grid.iter().map(|g| g.mid));
        for _ in 0..8 {
            self.tmp.clear();
            self.tmp.extend_from_slice(&self.path);
            for i in 1..self.path.len() - 1 {
                self.path[i] = add(
                    scale(self.tmp[i], 0.5),
                    scale(add(self.tmp[i - 1], self.tmp[i + 1]), 0.25),
                );
            }
        }
        self.cum.clear();
        self.cum.push(0.);
        for i in 1..self.path.len() {
            self.cum
                .push(self.cum[i - 1] + dist(self.path[i - 1], self.path[i]));
        }
        let length = *self.cum.last().unwrap();
        let n = ((length / 0.25).ceil() as usize + 1).clamp(2, MAX_CUTS);
        out.centreline.clear();
        let mut j = 1;
        for i in 0..n {
            let s = length * i as f64 / (n - 1) as f64;
            while j < self.cum.len() - 1 && self.cum[j] < s {
                j += 1;
            }
            out.centreline.push(lerp_v(
                self.path[j - 1],
                self.path[j],
                ((s - self.cum[j - 1]) / (self.cum[j] - self.cum[j - 1]).max(1e-12)).clamp(0., 1.),
            ));
        }
        let af = &mut out.area_fn;
        af.s.clear();
        af.area.clear();
        af.weighted_area.clear();
        af.perimeter.clear();
        af.channels.clear();
        af.width.clear();
        af.height.clear();
        af.length = length;
        af.volume = 0.;
        af.min_area = f64::INFINITY;
        af.min_s = 0.;
        for i in 0..n {
            let t = norm(sub(
                out.centreline[(i + 1).min(n - 1)],
                out.centreline[i.saturating_sub(1)],
            ));
            let u = v(-t.y, t.x);
            let c = &mut out.slices[i];
            self.cut(out.centreline[i], u, c);
            c.s = length * i as f64 / (n - 1) as f64;
            c.fold = if i > 0 && i + 1 < n {
                let a = norm(sub(out.centreline[i], out.centreline[i - 1]));
                let b = norm(sub(out.centreline[i + 1], out.centreline[i]));
                dist(a, b) * c.height > 2. * length / (n - 1) as f64
            } else {
                false
            };
            af.s.push(c.s);
            af.area.push(c.area);
            af.weighted_area.push(c.weighted_area);
            af.perimeter.push(c.perimeter);
            af.channels.push(c.channels);
            af.width.push(c.width);
            af.height.push(c.height);
            if i > 0 {
                af.volume += (af.area[i - 1] + c.area) * 0.5 * (c.s - af.s[i - 1]);
            }
            if c.s >= 0.5 && c.area < af.min_area {
                af.min_area = c.area;
                af.min_s = c.s;
            }
        }
        // The output owns a fixed pool of 128 cuts; areaFn.s gives the active count.
        out.shape = self.shape;
        out.cavities.clear();
        let k = self.tract.body.scale;
        let p = &self.shape;
        for side in [-1, 1] {
            out.cavities.push(SideCavity {
                kind: "piriform",
                side,
                attachment: p.piriform_depth * k,
                volume: 4. / 3. * PI * 1.1 * p.piriform_width * p.piriform_depth * 0.5 * k * k * k,
                neck_area: 0.25 * k * k,
                depth: p.piriform_depth * k,
            });
            out.cavities.push(SideCavity {
                kind: "vallecula",
                side,
                attachment: 3.5 * k,
                volume: 4. / 3.
                    * PI
                    * 0.3
                    * p.vallecula_width
                    * p.vallecula_depth
                    * 0.5
                    * k
                    * k
                    * k,
                neck_area: 0.15 * k * k,
                depth: p.vallecula_depth * k,
            });
        }
        Ok(())
    }
    pub fn cut(&self, origin: Vec2, u: Vec2, out: &mut Cut) {
        cut::measure(&self.laminae, origin, u, out);
    }
    /// Whole anatomical plane; unlike centreline cuts it has no band around the origin.
    pub fn cut_plane(&self, origin: Vec2, u: Vec2, out: &mut Cut) {
        cut::measure_plane(&self.laminae, origin, u, out);
    }
    /// Arbitrary point query between adjacent laminae. Endpoints of corresponding intervals
    /// interpolate laterally, keeping thin channels available to the offline voxelizer.
    pub fn air(&self, x: f64, y: f64, z: f64) -> bool {
        let k = self.tract.body.scale;
        let q = z / (cut::Z_STEP * k) + 32.;
        if !(0. ..=64.).contains(&q) {
            return false;
        }
        let i = (q as usize).min(63);
        let w = q - i as f64;
        let mut a = Row::default();
        let mut b = Row::default();
        cut::intervals(&self.laminae[i], v(x, y), v(1., 0.), &mut a);
        cut::intervals(&self.laminae[i + 1], v(x, y), v(1., 0.), &mut b);
        if a.count == b.count {
            for j in 0..a.count {
                if lerp(a.intervals[j].lo, b.intervals[j].lo, w) <= 0.
                    && lerp(a.intervals[j].hi, b.intervals[j].hi, w) >= 0.
                {
                    return true;
                }
            }
            false
        } else {
            if w < 0.5 {
                self.laminae[i].contains(v(x, y))
            } else {
                self.laminae[i + 1].contains(v(x, y))
            }
        }
    }
    /// Scanline query for efficient native voxelization (z snaps to the nearest lamina).
    pub fn air_intervals(&self, y: f64, z: f64, out: &mut Row) {
        let q = z / (cut::Z_STEP * self.tract.body.scale) + 32.;
        if !(0. ..=64.).contains(&q) {
            out.count = 0;
            return;
        }
        let i = (q as usize).min(63);
        let w = q - i as f64;
        let mut a = Row::default();
        let mut b = Row::default();
        cut::intervals(&self.laminae[i], v(0., y), v(1., 0.), &mut a);
        cut::intervals(&self.laminae[i + 1], v(0., y), v(1., 0.), &mut b);
        if a.count == b.count {
            out.count = a.count;
            for j in 0..a.count {
                out.intervals[j] = Interval {
                    lo: lerp(a.intervals[j].lo, b.intervals[j].lo, w),
                    hi: lerp(a.intervals[j].hi, b.intervals[j].hi, w),
                    component: 0,
                };
            }
        } else {
            *out = if w < 0.5 { a } else { b };
        }
    }
    pub fn meshes(&self) -> Vec<ElementMesh> {
        mesh::meshes(&self.laminae)
    }
    pub fn midsagittal(&self) -> Vec<(&'static str, Vec<Vec2>)> {
        SURFACE_OWNERS
            .iter()
            .enumerate()
            .map(|(i, &owner)| (ELEMENTS[owner], self.laminae[32].elements[i].clone()))
            .collect()
    }
    fn build_laminae(&mut self) {
        let f = &self.midline;
        let p = &self.shape;
        let k = self.tract.body.scale;
        let tip = f
            .tongue
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| dist(**a, f.tip).total_cmp(&dist(**b, f.tip)))
            .map(|(i, _)| i)
            .unwrap_or(1)
            .max(1);
        let tongue_point = |q: Vec2, z: f64| -> Vec2 {
            let j = f
                .tongue
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| dist(**a, q).total_cmp(&dist(**b, q)))
                .map(|(j, _)| j)
                .unwrap_or(0);
            if dist(f.tongue[j], q) > 0.08 * k {
                return q;
            }
            let t = (j as f64 / tip as f64).min(1.);
            let width = station([1.5, 2.25, 2.45, 1.6, p.tip_width], t) * p.tongue_width * k;
            let w = (z / width).abs().min(1.);
            let side = station(
                [
                    p.side_root,
                    p.side_back,
                    p.side_dorsum,
                    p.side_blade,
                    p.side_tip,
                ],
                t,
            );
            let groove = station(
                [
                    p.groove_root,
                    p.groove_back,
                    p.groove_dorsum,
                    p.groove_blade,
                    p.groove_tip,
                ],
                t,
            );
            let gw = (p.groove_width * (1. - p.groove_taper * (t - 0.8))).max(0.1) * k;
            let g = if z.abs() < gw {
                0.5 + 0.5 * cos(PI * z / gw)
            } else {
                0.
            };
            let rounded = pow((1. - pow(w, p.edge_round)).max(0.), 1. / p.edge_round);
            let h = side * smoothstep(0.25, 0.8, w) * (1. + p.side_asym * z.signum())
                - groove * g
                - 1.5 * (1. - rounded);
            if j <= tip {
                let d = norm(sub(
                    f.tongue[(j + 1).min(tip)],
                    f.tongue[j.saturating_sub(1)],
                ));
                add(q, scale(v(-d.y, d.x), h * k))
            } else {
                add(q, v(0., 0.5 * (1. - rounded) * k))
            }
        };
        let velum_top = f
            .velum
            .iter()
            .map(|q| q.y)
            .fold(f64::NEG_INFINITY, f64::max);
        let velum_bottom = f.velum.iter().map(|q| q.y).fold(f64::INFINITY, f64::min);
        let velum_point = |q: Vec2, z: f64| {
            let t = ((q.y - velum_bottom) / (velum_top - velum_bottom).max(0.01)).clamp(0., 1.);
            let width = 0.5 * lerp(p.uvula_width, p.velum_width, t) * k;
            let r = sqrt((1. - (z / width) * (z / width)).max(0.));
            add(
                q,
                v(
                    0.,
                    (velum_top - q.y) * (1. - r) - (p.uvula_length - 1.) * (1. - t) * k * r,
                ),
            )
        };
        for l in &mut self.laminae {
            l.clear();
            let z = l.z;
            let az = z.abs() / k;
            // Ceiling and posterior wall, with lateral closure towards the inner tissue.
            for (i, &q) in f.outer.iter().enumerate() {
                let d = norm(sub(
                    f.outer[(i + 1).min(f.outer.len() - 1)],
                    f.outer[i.saturating_sub(1)],
                ));
                let qt = self.tract.un_p(q.x, q.y);
                let width = if qt.x < 1. {
                    lerp(
                        lerp(
                            p.pharynx_width_hypo,
                            p.pharynx_width_oro,
                            smoothstep(2., 6., qt.y),
                        ),
                        p.pharynx_width_naso,
                        smoothstep(6., 9., qt.y),
                    ) * (1. - 0.7 * p.lateral_squeeze)
                } else if qt.x > 7. {
                    0.5 * (1.1 + (self.tract.body.lip_width - 1.1) * self.artic.lip_width)
                } else {
                    p.arch_width * (1. - 0.35 * p.arch_taper * smoothstep(4., 7., qt.x))
                        + p.buccal_gap
                        + 0.5 * p.cheek_puff
                };
                let w = az / width;
                let shift = if qt.x < 1. {
                    let gap = f
                        .grid
                        .iter()
                        .min_by(|a, b| (a.mid.y - q.y).abs().total_cmp(&(b.mid.y - q.y).abs()))
                        .map_or(2., |g| g.d / k);
                    gap * (1.
                        - pow(
                            (1. - pow(w.min(1.), p.pharynx_shape)).max(0.),
                            1. / p.pharynx_shape,
                        ))
                } else if qt.x > 7. {
                    let gap = (self.artic.lip_ap).max(0.);
                    gap * (1.
                        - pow(
                            (1. - pow(w.min(1.), p.lip_corner_shape)).max(0.),
                            1. / p.lip_corner_shape,
                        ))
                } else {
                    p.vault_depth * pow(w.min(1.2), p.vault_shape)
                };
                let shift = shift + if w > 1. { 20. * (w - 1.) } else { 0. };
                let mut point = add(q, scale(v(d.y, -d.x), shift * k));
                if f.velum.iter().any(|&a| dist(a, q) < 0.001 * k) {
                    point = add(point, sub(velum_point(q, z), q));
                }
                l.air[0].push(point);
                let e = if qt.x < 0.3 && qt.y < 7. {
                    3
                } else if qt.x < 2. {
                    2
                } else if qt.x < 6.3 {
                    1
                } else if qt.x < 7.3 {
                    6
                } else {
                    5
                };
                if az <= width && e != 2 && e != 6 {
                    l.elements[e].push(point);
                }
            }
            let first = l.air[0][0];
            let last = *l.air[0].last().unwrap();
            // Keep a finite overlap with the epilarynx: a one-voxel butt joint can
            // disconnect under body scaling or a sub-voxel larynx translation.
            let bottom = f.glottis.y + 2.2 * k * self.tract.body.pharynx;
            l.air[0].push(v(last.x, bottom));
            l.air[0].push(v(first.x, bottom));
            // Clip the pharyngeal envelope at its actual junction with the laryngeal tube.
            // Keeping the low posterior-wall vertices made spurious wedges beside the glottis.
            l.tissue[2].extend_from_slice(&l.air[0]);
            l.air[0].clear();
            for i in 0..l.tissue[2].len() {
                let a = l.tissue[2][i];
                let b = l.tissue[2][(i + 1) % l.tissue[2].len()];
                if a.y >= bottom {
                    l.air[0].push(a);
                }
                if (a.y >= bottom) != (b.y >= bottom) {
                    l.air[0].push(lerp_v(a, b, (bottom - a.y) / (b.y - a.y)));
                }
            }
            l.tissue[2].clear();
            // The cheek wall bounds the oral lumen independently of the roof loft. Without
            // this solid lateral wall, an overclosed roof could fold back and leave phantom
            // wedges beside the incisors; their component ownership jumped as the tongue moved.
            let rear_width = p.arch_width + p.buccal_gap + 0.5 * p.cheek_puff;
            let front_width =
                0.5 * (1.1 + (self.tract.body.lip_width - 1.1) * self.artic.lip_width);
            let width_at = |x: f64| lerp(rear_width, front_width, smoothstep(2., 8.5, x));
            if az > front_width.min(rear_width) {
                let decreasing = rear_width >= front_width;
                let mut lo = 1.;
                let mut hi = 9.;
                for _ in 0..24 {
                    let x = 0.5 * (lo + hi);
                    if (width_at(x) > az) == decreasing {
                        lo = x;
                    } else {
                        hi = x;
                    }
                }
                let x = 0.5 * (lo + hi) * k;
                let (left, right) = if az >= front_width.max(rear_width) {
                    (-5. * k, 15. * k)
                } else if decreasing {
                    (x, 15. * k)
                } else {
                    (-5. * k, x)
                };
                l.tissue[2].extend_from_slice(&[
                    v(left, 3.5 * k),
                    v(right, 3.5 * k),
                    v(right, 15. * k),
                    v(left, 15. * k),
                ]);
                if az < front_width.max(rear_width) {
                    l.elements[7].extend_from_slice(&[v(x, 3.5 * k), v(x, 10. * k)]);
                }
            }
            // Inner tissue is closed well below the envelope; the open dorsal surface keeps
            // real tongue/roof overlap sealed instead of turning self-intersections into air.
            for &q in &f.inner {
                l.tissue[0].push(tongue_point(q, z));
            }
            let first = l.tissue[0][0];
            let last = *l.tissue[0].last().unwrap();
            l.tissue[0].push(v(last.x, -5. * k));
            l.tissue[0].push(v(first.x, -5. * k));
            for &q in &f.tongue {
                if az < 2.5 * p.tongue_width {
                    l.elements[0].push(tongue_point(q, z));
                }
            }
            // Laryngeal lumen: elliptical horizontal cuts reproduce the existing area profile.
            let epi_len = 2.4 * k * self.tract.body.pharynx;
            let ew = p.epilarynx_width * k;
            if az < p.epilarynx_width {
                let chord = sqrt((1. - (z / ew) * (z / ew)).max(0.));
                for j in 0..=24 {
                    let y = epi_len * j as f64 / 24.;
                    let area = self.tract.epilarynx_area(&self.artic, y, epi_len);
                    let rx = area / (PI * ew);
                    l.air[1].push(add(f.glottis, v(-rx * chord, y)));
                }
                for j in (0..=24).rev() {
                    let y = epi_len * j as f64 / 24.;
                    let area = self.tract.epilarynx_area(&self.artic, y, epi_len);
                    let rx = area / (PI * ew);
                    l.air[1].push(add(f.glottis, v(rx * chord, y)));
                }
            }
            l.elements[4].extend_from_slice(&l.air[1]);
            for (j, side) in [-1., 1.].iter().enumerate() {
                let zz = z / k - side * (p.epilarynx_width + p.piriform_width * 0.9);
                let rad = (1. - (zz / p.piriform_width) * (zz / p.piriform_width)).max(0.);
                ellipse(
                    &mut l.air[2 + j],
                    add(f.glottis, v(0., p.piriform_depth * 0.65 * k)),
                    1.1 * k * sqrt(rad),
                    p.piriform_depth * 0.5 * k * sqrt(rad),
                );
            }
            // Open necks connect each blind fossa to the hypopharynx above its blind section.
            if az < p.epilarynx_width + 1.7 * p.piriform_width {
                let y = f.glottis.y + p.piriform_depth * 1.06 * k;
                l.air[5].extend_from_slice(&[
                    v(-0.3 * k, y),
                    v(0.3 * k, y),
                    v(0.3 * k, bottom + 0.2 * k),
                    v(-0.3 * k, bottom + 0.2 * k),
                ]);
            }
            // Valleculae appear only off the median fold, behind the tongue root.
            if az > 0.15 && az < 2. * p.vallecula_width + 0.15 {
                let zz = (az - 0.15 - p.vallecula_width) / p.vallecula_width;
                let r = sqrt((1. - zz * zz).max(0.));
                let epi = f.epiglottis[2];
                ellipse(
                    &mut l.air[4],
                    add(epi, v(-0.2 * k, 0.3 * k)),
                    0.3 * k * r,
                    0.5 * p.vallecula_depth * k * r,
                );
            }
            // Display the tissue faces adjacent to each side cavity as part of the larynx mesh.
            // The epiglottis is an exclusion as well as a visible plate.
            if az < p.epiglottis_width {
                let shift = v(p.epiglottis_curl * z * z / k, 0.);
                let base = add(f.epiglottis[0], shift);
                let tip = add(f.epiglottis[1], shift);
                let d = norm(sub(tip, base));
                let thickness = scale(v(d.y, -d.x), 0.25 * k);
                l.tissue[1].extend_from_slice(&[
                    base,
                    tip,
                    add(tip, thickness),
                    add(base, thickness),
                ]);
                l.elements[11].extend_from_slice(&l.tissue[1]);
            }
            if az < 0.5 * p.velum_width {
                for &q in &f.velum {
                    l.elements[2].push(velum_point(q, z));
                }
            }
            // Jaw-carried dental arch sections and cheeks use the same lateral width limits.
            let arch_drop = smoothstep(0.3, p.lower_arch_width, az) * 3. * k;
            if az < p.arch_width + 0.5 {
                for q in &f.upper_teeth {
                    let phase = ((az + 0.325) / 0.65).fract() * 0.65;
                    let gap =
                        (1. - smoothstep(
                            p.tooth_gap * 0.5,
                            p.tooth_gap * 0.5 + 0.025,
                            phase.min(0.65 - phase),
                        )) * 0.65
                            * k;
                    l.elements[6].push(add(*q, v(-arch_drop, gap)));
                }
            }
            if az < p.lower_arch_width + 0.5 {
                for q in &f.lower_teeth {
                    let phase = ((az + 0.325) / 0.65).fract() * 0.65;
                    let gap =
                        (1. - smoothstep(
                            p.tooth_gap * 0.5,
                            p.tooth_gap * 0.5 + 0.025,
                            phase.min(0.65 - phase),
                        )) * 0.65
                            * k;
                    l.elements[9].push(add(*q, v(-arch_drop, -gap)));
                }
                for q in &f.jaw_bone {
                    l.elements[10].push(add(*q, v(-arch_drop, 0.)));
                }
            }
            let lip_width = 0.5 * (1.1 + (self.tract.body.lip_width - 1.1) * self.artic.lip_width);
            if az < lip_width {
                for q in &f.upper_lip {
                    l.elements[5].push(add(*q, v(-(p.lip_thickness - 1.) * k, 0.)));
                }
                for q in &f.lower_lip {
                    l.elements[8].push(add(*q, v(-(p.lip_thickness - 1.) * k, 0.)));
                }
            }
            l.elements[12].extend_from_slice(&l.air[2]);
            l.elements[13].extend_from_slice(&l.air[3]);
            l.elements[14].extend_from_slice(&l.air[4]);
        }
    }
}
