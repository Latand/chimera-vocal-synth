//! Parametric 3D vocal tract for inspection and, in phase 2, acoustics. No voice calls this
//! module. Units: centimetres; x forward, y up, z to the speaker's right. The solid is a stack of
//! parasagittal sections (laminae); cuts, air queries and the lab's meshes all read the same
//! per-element rules (`parts.rs`). See docs/tract3d/model-design.md.
mod cut;
mod features;
mod lamina;
mod mesh;
mod params;
mod parts;
mod presets;
use super::geom::{add, dist, lerp_v, norm, rotate_about, scale, sub, v, Vec2};
use super::human::{ArticKey, HumanArtic, HumanBody, HumanFrame, HumanVocalTract};
use crate::math::{cos, lerp, pow, sin, smoothstep, sqrt, PI};
pub use cut::{Cut, Interval, Region, Row, BAND, BAND_Z, MAX_INTERVALS, MIN_REGION, ROWS, Z_STEP};
use lamina::{area as poly_area, Lamina};
pub use lamina::{AIR_PIRIFORM, AIR_VALLECULA};
pub use features::{check_one, check_presets, groove_index, surface_groove, Check};
pub use mesh::{ElementMesh, MeshGroup, RING_COLS, RING_ROWS, TEETH_PER_SIDE};
pub use params::{derive, derive_with, ParamSpec, ShapeParams, DERIVE, SPECS};
pub use parts::{occlusal, Arch, Map, Parts, ARCH_DEPTH, CUSP};
pub use presets::{posture, presets, Preset, PRESET_NAMES};
use serde::Serialize;

/// The model's elements, in the order the meshes are returned.
pub const ELEMENTS: [&str; 8] = [
    "tongue", "palate", "velum", "pharynx", "larynx", "lips", "teethJaw", "cheeks",
];
pub const MAX_CUTS: usize = 128;
/// centreline step (cm)
pub const STEP: f64 = 0.25;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SideCavity {
    pub kind: &'static str,
    pub side: i8,
    /// distance along the centreline where the cavity opens into the tract (cm)
    pub attachment: f64,
    /// volume of the modelled pocket (cm³), integrated over the laminae
    pub volume: f64,
    /// cross-section of its opening into the tract (cm²; the fossae open along the hypopharynx floor)
    pub neck_area: f64,
    /// from the opening to the pocket's lowest point (cm)
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
    /// a fixed pool of cuts; `area_fn.s.len()` of them are in use
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
impl Tract3dFrame {
    /// The cuts in use.
    pub fn cuts(&self) -> &[Cut] {
        &self.slices[..self.area_fn.s.len()]
    }
}

const AIR_NAMES: [&str; lamina::N_AIR] = ["envelope", "tube", "piriform", "piriform", "vallecula", "vallecula"];
const TISSUE_NAMES: [&str; lamina::N_TISSUE] = ["tongue", "epiglottis", "upperLip", "lowerLip", "upperTeeth", "lowerTeeth", "pharynxWall", "pharynxWall", "uvula", "cheek", "cheek", "aryFold"];

#[derive(Clone, Debug, Serialize)]
pub struct NamedPolygon {
    pub name: &'static str,
    pub points: Vec<Vec2>,
}
/// The lips' meeting line from the midline out (z ≥ 0; the lips are symmetric): `upper[i]` and
/// `lower[i]` are the two lips' edges at lateral offset `z[i]`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LipLine {
    /// half-width of the opening (cm)
    pub half_width: f64,
    pub z: Vec<f64>,
    pub upper: Vec<Vec2>,
    pub lower: Vec<Vec2>,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct JawTurn {
    /// the condyle (cm) and the jaw's rotation about it from rest (rad, opening negative)
    pub pivot: Vec2,
    pub angle: f64,
}
#[derive(Clone, Debug, Serialize)]
pub struct Display {
    pub lips: LipLine,
    pub jaw: JawTurn,
    pub glottis: Vec2,
    /// the lamina's air polygons (the first is the envelope; the others are air inside tissue)
    pub air: Vec<NamedPolygon>,
    /// the lamina's tissue polygons
    pub tissue: Vec<NamedPolygon>,
    /// the engine's own midsagittal outlines, for drawing a section's parts
    pub outlines: Vec<NamedPolygon>,
}

pub struct Tract3d {
    pub tract: HumanVocalTract,
    pub midline: HumanFrame,
    pub parts: Parts,
    pub shape: ShapeParams,
    pub artic: HumanArtic,
    laminae: Vec<Lamina>,
    symmetric: bool,
    path: Vec<Vec2>,
    tmp: Vec<Vec2>,
    cum: Vec<f64>,
}

impl Tract3d {
    pub fn new(body: HumanBody) -> Self {
        Self {
            tract: HumanVocalTract::new(body),
            midline: HumanFrame::with_capacity(),
            parts: Parts::new(body),
            shape: ShapeParams::default(),
            artic: super::human::NEUTRAL,
            laminae: (0..ROWS).map(|i| Lamina::new(z_of(i, body.scale))).collect(),
            symmetric: true,
            path: Vec::with_capacity(100),
            tmp: Vec::with_capacity(100),
            cum: Vec::with_capacity(100),
        }
    }
    pub fn specs() -> &'static [ParamSpec] {
        SPECS
    }
    /// 3D parameters implied by the 17 articulators (design §5). The lab's controls and the
    /// presets are offsets from these.
    pub fn derive(&self, a: &HumanArtic) -> ShapeParams {
        params::derive(a)
    }
    /// The midline and every element's rule for a posture, without laminae or cuts: all the
    /// display needs (`meshes`, `display`). `compute` does this first.
    pub fn pose(&mut self, a: &HumanArtic, offsets: &ShapeParams) -> Result<(), String> {
        for key in ArticKey::ALL {
            if !a.get(key).is_finite() {
                return Err(format!("{} must be finite", key.name()));
            }
        }
        self.artic = *a;
        self.shape = self.derive(a).with_offsets(offsets)?;
        self.tract.compute(a, &mut self.midline);
        self.parts.update(&self.tract, &self.midline, a, &self.shape);
        Ok(())
    }
    pub fn compute(&mut self, a: &HumanArtic, offsets: &ShapeParams, out: &mut Tract3dFrame) -> Result<(), String> {
        self.pose(a, offsets)?;
        self.symmetric = self.shape.side_asym == 0.;
        let mid = ROWS / 2;
        for i in mid..ROWS {
            self.parts.fill(&mut self.laminae[i]);
        }
        for i in 0..mid {
            if self.symmetric {
                let (lo, hi) = self.laminae.split_at_mut(mid);
                lo[i].copy_from(&hi[ROWS - 1 - i - mid]);
            } else {
                self.parts.fill(&mut self.laminae[i]);
            }
        }
        self.centreline(out);
        let n = out.centreline.len();
        let length = self.cum.last().copied().unwrap_or(0.);
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
        let step = length / (n - 1) as f64;
        for i in 0..n {
            let t = norm(sub(out.centreline[(i + 1).min(n - 1)], out.centreline[i.saturating_sub(1)]));
            let c = &mut out.slices[i];
            cut::measure(&self.laminae, self.symmetric, out.centreline[i], v(-t.y, t.x), cut::BAND, c);
            c.s = step * i as f64;
            // neighbouring cuts cross inside the airway where the bend's radius is below the
            // cut's own extent along u
            c.fold = i > 0 && i + 1 < n && {
                let a = norm(sub(out.centreline[i], out.centreline[i - 1]));
                let b = norm(sub(out.centreline[i + 1], out.centreline[i]));
                dist(a, b) * 0.5 * c.height > step
            };
            af.s.push(c.s);
            af.area.push(c.area);
            af.weighted_area.push(c.weighted_area);
            af.perimeter.push(c.perimeter);
            af.channels.push(c.channels);
            af.width.push(c.width);
            af.height.push(c.height);
            if i > 0 {
                af.volume += (af.area[i - 1] + c.area) * 0.5 * step;
            }
            if c.s >= 0.5 && c.area < af.min_area {
                af.min_area = c.area;
                af.min_s = c.s;
            }
        }
        out.shape = self.shape;
        self.cavities(out);
        Ok(())
    }
    /// Glottis, five points up the laryngeal tube's own axis, then the engine's grid midpoints;
    /// smoothed with fixed ends and resampled every `STEP`.
    fn centreline(&mut self, out: &mut Tract3dFrame) {
        let f = &self.midline;
        self.path.clear();
        self.path.push(f.glottis);
        let tube = 2.0 * self.tract.body.scale * self.tract.body.pharynx;
        for i in 1..6 {
            let y = tube * i as f64 / 5.;
            self.path.push(v(self.parts.tube_at(y / self.parts.epi_len).0, f.glottis.y + y));
        }
        self.path.extend(f.grid.iter().map(|g| g.mid));
        for _ in 0..8 {
            self.tmp.clear();
            self.tmp.extend_from_slice(&self.path);
            for i in 1..self.path.len() - 1 {
                self.path[i] = add(scale(self.tmp[i], 0.5), scale(add(self.tmp[i - 1], self.tmp[i + 1]), 0.25));
            }
        }
        self.cum.clear();
        self.cum.push(0.);
        for i in 1..self.path.len() {
            self.cum.push(self.cum[i - 1] + dist(self.path[i - 1], self.path[i]));
        }
        let length = *self.cum.last().unwrap();
        let n = ((length / STEP).round() as usize + 1).clamp(2, MAX_CUTS);
        out.centreline.clear();
        let mut j = 1;
        for i in 0..n {
            let s = length * i as f64 / (n - 1) as f64;
            while j < self.cum.len() - 1 && self.cum[j] < s {
                j += 1;
            }
            let t = ((s - self.cum[j - 1]) / (self.cum[j] - self.cum[j - 1]).max(1e-12)).clamp(0., 1.);
            out.centreline.push(lerp_v(self.path[j - 1], self.path[j], t));
        }
    }
    /// Side pockets measured from the laminae themselves (pocket polygons × lamina spacing).
    /// A mirrored lamina holds its pocket in the other side's slot, so sides go by the sign of z.
    fn cavities(&self, out: &mut Tract3dFrame) {
        out.cavities.clear();
        let dz = Z_STEP * self.tract.body.scale;
        // [side][piriform volume, neck area, lowest y, neck top y, vallecula volume, low, high]
        let mut m = [[0., 0., f64::INFINITY, f64::NEG_INFINITY, 0., f64::INFINITY, f64::NEG_INFINITY]; 2];
        for l in &self.laminae {
            let sides: &[(usize, f64)] = if l.z < 0. { &[(0, 1.)] } else if l.z > 0. { &[(1, 1.)] } else { &[(0, 0.5), (1, 0.5)] };
            for &(side, w) in sides {
                let c = &mut m[side];
                for j in 0..2 {
                    let p = &l.air[AIR_PIRIFORM[j]];
                    c[0] += poly_area(p).abs() * dz * w;
                    for q in p {
                        c[2] = c[2].min(q.y);
                    }
                    // the opening: the pocket's chord along the hypopharynx floor
                    if !p.is_empty() {
                        let side = if l.z < 0. { -1. } else if l.z > 0. { 1. } else if j == 0 { -1. } else { 1. };
                        if let Some((_, cz, rx, rz)) = self.parts.piriform_section(side, self.parts.bottom) {
                            let rad = 1. - ((l.z - cz) / rz).powi(2);
                            if rad > 0. {
                                c[1] += 2. * rx * sqrt(rad) * dz * w;
                            }
                        }
                        c[3] = self.parts.bottom;
                    }
                    let q = &l.air[AIR_VALLECULA[j]];
                    c[4] += poly_area(q).abs() * dz * w;
                    if let Some((floor, top)) = (!q.is_empty()).then(|| self.parts.vallecula_span(l.z)).flatten() {
                        // the deepest lamina's floor and the leaf's tip above it
                        if floor < c[5] {
                            c[5] = floor;
                            c[6] = top;
                        }
                    }
                }
            }
        }
        let attach = |y: f64| {
            if !y.is_finite() {
                return 0.;
            }
            let i = out.cuts().iter().position(|c| c.origin.y >= y).unwrap_or(0);
            out.area_fn.s.get(i).copied().unwrap_or(0.)
        };
        let at = [attach(m[0][3]), attach(m[1][3]), attach(m[0][6]), attach(m[1][6])];
        for (j, (side, c)) in [(-1i8, m[0]), (1, m[1])].into_iter().enumerate() {
            out.cavities.push(SideCavity {
                kind: "piriform",
                side,
                attachment: at[j],
                volume: c[0],
                neck_area: c[1],
                depth: if c[3].is_finite() && c[2].is_finite() { c[3] - c[2] } else { 0. },
            });
        }
        for (j, (side, c)) in [(-1i8, m[0]), (1, m[1])].into_iter().enumerate() {
            out.cavities.push(SideCavity {
                kind: "vallecula",
                side,
                attachment: at[2 + j],
                volume: c[4],
                neck_area: 0.,
                depth: if c[6].is_finite() { c[6] - c[5] } else { 0. },
            });
        }
    }
    /// Cut perpendicular to `u` × z through `origin`, keeping the regions in the band around it.
    pub fn cut(&self, origin: Vec2, u: Vec2, out: &mut Cut) {
        cut::measure(&self.laminae, self.symmetric, origin, u, cut::BAND, out);
    }
    /// Whole anatomical plane (coronal: u = ŷ, axial: u = x̂); no band.
    pub fn cut_plane(&self, origin: Vec2, u: Vec2, out: &mut Cut) {
        cut::measure(&self.laminae, self.symmetric, origin, u, f64::INFINITY, out);
    }
    /// Point query between adjacent laminae: corresponding interval ends interpolate in z, so
    /// thin channels survive voxelization between laminae.
    pub fn air(&self, x: f64, y: f64, z: f64) -> bool {
        let mut row = Row::default();
        self.air_intervals(y, z, &mut row);
        row.intervals[..row.count].iter().any(|q| q.lo <= x && x <= q.hi)
    }
    /// Air along the line (·, y, z), as x intervals.
    pub fn air_intervals(&self, y: f64, z: f64, out: &mut Row) {
        let q = z / (Z_STEP * self.tract.body.scale) + (ROWS / 2) as f64;
        out.count = 0;
        out.z = z;
        if !(0. ..=(ROWS - 1) as f64).contains(&q) {
            return;
        }
        let i = (q as usize).min(ROWS - 2);
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
            let r = if w < 0.5 { &a } else { &b };
            out.count = r.count;
            out.intervals = r.intervals;
        }
    }
    /// Display meshes, one per element, with vertex counts fixed by the body.
    pub fn meshes(&self) -> Vec<ElementMesh> {
        mesh::meshes(&self.parts)
    }
    /// What a view draws beside the meshes, for the posture last given to `pose` or `compute`:
    /// the lips' meeting line and the jaw's turn (the head's skin follows both), the lamina at
    /// lateral offset `z` (the cut face of an opened head: the same polygons the cuts read) and
    /// the engine's own midline outlines.
    pub fn display(&self, z: f64) -> Display {
        let k = self.tract.body.scale;
        let reach = self.parts.lip_w + 1.2 * k;
        let n = 25;
        let mut lips = LipLine { half_width: self.parts.lip_w, z: Vec::with_capacity(n), upper: Vec::with_capacity(n), lower: Vec::with_capacity(n) };
        for i in 0..n {
            let zz = reach * i as f64 / (n - 1) as f64;
            let (u, l) = self.parts.lip_edge(zz);
            lips.z.push(zz);
            lips.upper.push(u);
            lips.lower.push(l);
        }
        let mut lamina = Lamina::new(z);
        self.parts.fill(&mut lamina);
        let named = |names: &[&'static str], polys: &[Vec<Vec2>]| -> Vec<NamedPolygon> {
            names.iter().zip(polys).filter(|(_, p)| p.len() >= 3).map(|(name, p)| NamedPolygon { name, points: p.clone() }).collect()
        };
        let f = &self.midline;
        Display {
            lips,
            jaw: JawTurn { pivot: self.parts.condyle(), angle: self.parts.jaw_angle() },
            glottis: f.glottis,
            air: named(&AIR_NAMES, &lamina.air),
            tissue: named(&TISSUE_NAMES, &lamina.tissue),
            outlines: vec![
                NamedPolygon { name: "tongue", points: f.tongue.clone() },
                NamedPolygon { name: "velum", points: f.velum.clone() },
                NamedPolygon { name: "upperTeeth", points: f.upper_teeth.clone() },
                NamedPolygon { name: "lowerTeeth", points: f.lower_teeth.clone() },
                NamedPolygon { name: "jawBone", points: f.jaw_bone.clone() },
                NamedPolygon { name: "nasalCavity", points: f.nasal_cavity.clone() },
            ],
        }
    }
    /// The polygons of the z = 0 lamina, named (the engine's own midsagittal outlines).
    pub fn midsagittal(&self) -> Vec<(&'static str, Vec<Vec2>)> {
        let l = &self.laminae[ROWS / 2];
        AIR_NAMES
            .iter()
            .zip(&l.air)
            .chain(TISSUE_NAMES.iter().zip(&l.tissue))
            .filter(|(_, p)| !p.is_empty())
            .map(|(n, p)| (*n, p.clone()))
            .collect()
    }
    /// The laminae, for exports and tests.
    pub fn lamina_count(&self) -> usize {
        self.laminae.len()
    }
    pub fn lamina_z(&self, i: usize) -> f64 {
        self.laminae[i].z
    }
    pub fn lamina_polygons(&self, i: usize) -> (&[Vec<Vec2>], &[Vec<Vec2>]) {
        (&self.laminae[i].air, &self.laminae[i].tissue)
    }
}

pub fn z_of(i: usize, scale: f64) -> f64 {
    (i as f64 - (ROWS / 2) as f64) * Z_STEP * scale
}
