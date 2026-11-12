//! Per-frame anatomy: the engine's midline (`HumanFrame`) split into its parts, plus the lateral
//! rule of every element. The laminae, the display meshes and the side-cavity descriptors all
//! read these rules, so what the lab draws is what the cuts measure.
//!
//! Every lateral rule moves points in a way that cannot fold an outline over itself (the
//! earlier version offset curved outlines by up to 2 cm along their normals, which folded the
//! palate and tongue into self-intersecting polygons and left phantom air strips beside them):
//! - the palate and the lips move only vertically (graphs stay graphs); the pharynx's posterior
//!   and lateral walls are one tissue polygon per lamina, a graph over height (`ring_at`), so
//!   the envelope's own wall never moves;
//! - the tongue's dorsal surface moves along fibres between itself and a core curve inside the
//!   tongue (a tubular neighbourhood: fibres do not cross), or along the same fibres outwards
//!   to the wall it faces, never beyond it;
//! - the tip lobe shortens by an affine compression towards its base line (a homeomorphism);
//! - teeth, lips, cheeks, the uvula and the pharyngeal ring are separate tissue polygons,
//!   united by counting, so they never need to meet another outline exactly;
//!
//! From the larynx to the back teeth the tract is a tube of soft walls, and its lumen ends
//! sideways in a taper, `closing`: the posterior wall lies flat on the spine and turns forward
//! only at its corners, so in the pharynx it is the front wall (the epiglottis and its folds,
//! the tongue's root and back) whose sides come back to it; under the soft palate it is the
//! roof that comes down to the tongue's sides (the faucial arches). Under the hard palate the
//! tongue is a loaf between the tooth rows.
//! - the laryngeal tube, the piriform fossae and the valleculae are air polygons of their own
//!   (they override tissue), each a smooth solid cut at the lamina's offset.
use super::lamina::*;
use super::*;
use crate::math::atan2;

/// Samples of each dental arch's outer edge.
pub const ARCH_SAMPLES: usize = 33;
pub const TUBE_SAMPLES: usize = 24;
/// Heights at which the pharyngeal ring is sampled, from below the glottis to above the port.
pub const RING_SAMPLES: usize = 56;
/// Points of the tongue root kept below the epiglottis' tip (the valleculae's front wall).
pub const ROOT_SAMPLES: usize = 24;
/// Thickness of the epiglottis' leaf (template cm).
pub const EPIGLOTTIS_THICKNESS: f64 = 0.25;
/// Template constants of the dental arches (template cm, before the oral 0.86 compression).
const UPPER_INCISOR_X: f64 = 7.12;
const LOWER_INCISOR_X: f64 = 6.8;
/// Incisors to the back of the second molars along x.
pub const ARCH_DEPTH: f64 = 5.0;
const OCCLUSAL_Y: f64 = 7.95;
/// rise of the occlusal plane per cm towards the molars (curve of Spee, simplified)
const SPEE: f64 = 0.05;
/// template x of the back of the upper dental arch
const ARCH_DEPTH_BACK: f64 = UPPER_INCISOR_X - ARCH_DEPTH;
/// The ring's outline across: flat behind over two thirds of its width, its corners rounded
/// (a superellipse's exponent).
const RING_SHAPE: f64 = 3.5;
/// Samples along each tooth row's crown in a lamina.
const CROWN_SAMPLES: usize = 12;
/// Points of the uvula's outline in a lamina.
pub const UVULA_SAMPLES: usize = 8;
/// vertical overlap of upper and lower cusps (template cm)
pub const CUSP: f64 = 0.12;

/// Template → body mapping, identical to `HumanVocalTract::p` (checked by a test).
#[derive(Clone, Copy, Debug)]
pub struct Map {
    pub k: f64,
    pub ph: f64,
}
impl Map {
    #[inline]
    pub fn p(&self, x: f64, y: f64) -> Vec2 {
        let xx = if x > 0.5 { 0.5 + (x - 0.5) * 0.86 } else { x };
        let yy = if y < 7.0 { 7.0 - (7.0 - y) * self.ph } else { y };
        v(xx * self.k, yy * self.k)
    }
    #[inline]
    pub fn un_p(&self, q: Vec2) -> Vec2 {
        let mut xx = q.x / self.k;
        let mut yy = q.y / self.k;
        if xx > 0.5 {
            xx = 0.5 + (xx - 0.5) / 0.86;
        }
        if yy < 7.0 {
            yy = 7.0 - (7.0 - yy) / self.ph;
        }
        v(xx, yy)
    }
}

/// One dental arch: inner (lingual) edge half-width `a(x) = width·u^e`, `u` the depth fraction
/// from the incisors, and the outer (buccal) edge one tooth width further out along the normal.
#[derive(Clone, Copy, Debug)]
pub struct Arch {
    pub incisor_x: f64,
    pub depth: f64,
    pub width: f64,
    pub e: f64,
    pub outer: [(f64, f64); ARCH_SAMPLES],
}
impl Arch {
    fn new(incisor_x: f64, depth: f64, width: f64, taper: f64) -> Self {
        let e = 0.28 + 0.5 * taper.clamp(0., 1.);
        let mut outer = [(0., 0.); ARCH_SAMPLES];
        for (i, o) in outer.iter_mut().enumerate() {
            let u = (i as f64 / (ARCH_SAMPLES - 1) as f64).max(1e-3);
            let x = incisor_x - depth * u;
            let a = width * pow(u, e);
            let da = width * e * pow(u, e - 1.);
            let n = norm(v(da, depth));
            let t = Self::tooth(u);
            *o = (x + t * n.x, a + t * n.y);
        }
        outer[0] = (incisor_x + Self::tooth(0.), 0.);
        Self { incisor_x, depth, width, e, outer }
    }
    /// The lingual edge at depth fraction `u` and the unit normal towards the buccal side, both
    /// as template (x, lateral).
    pub fn frame(&self, u: f64) -> ((f64, f64), (f64, f64)) {
        let u = u.max(1e-3);
        let x = self.incisor_x - self.depth * u;
        let a = self.width * pow(u, self.e);
        let da = self.width * self.e * pow(u, self.e - 1.);
        let n = norm(v(da, self.depth));
        ((x, a), (n.x, n.y))
    }
    /// crown thickness across the arch (template cm): the incisors as thin as the engine's own
    /// on the midline (3 mm), the molars 1 cm. (At 7 mm the incisors' rows could not begin at
    /// the midline: they were left out within 2.5 mm of it and rose over the next 2 mm, and a
    /// cut through the front teeth had a square slot between them.)
    pub fn tooth(u: f64) -> f64 {
        lerp(0.3, 1.0, smoothstep(0., 0.45, u))
    }
    pub fn back(&self) -> f64 {
        self.incisor_x - self.depth
    }
    /// inner half-width at template x
    pub fn inner_at(&self, x: f64) -> f64 {
        let u = ((self.incisor_x - x) / self.depth).clamp(0., 1.);
        self.width * pow(u, self.e)
    }
    /// x where the inner edge passes lateral offset `a` (the back of the arch when outside it)
    pub fn x_in(&self, a: f64) -> f64 {
        if a >= self.width {
            self.back()
        } else {
            self.incisor_x - self.depth * pow((a / self.width).max(0.), 1. / self.e)
        }
    }
    /// x where the outer edge passes `a`, from the front; the arch's last sample beyond it
    pub fn x_out(&self, a: f64) -> f64 {
        for i in 0..ARCH_SAMPLES - 1 {
            let (x0, a0) = self.outer[i];
            let (x1, a1) = self.outer[i + 1];
            if (a0 <= a) != (a1 <= a) {
                return lerp(x0, x1, (a - a0) / (a1 - a0));
            }
        }
        let mut best = self.outer[0];
        for o in &self.outer {
            if o.1 > best.1 {
                best = *o;
            }
        }
        best.0
    }
    pub fn outer_max(&self) -> f64 {
        self.outer.iter().map(|o| o.1).fold(0., f64::max)
    }
    /// lateral offset of the outer edge at template x (0 in front of the incisors' row, the
    /// arch's widest behind its back)
    pub fn outer_at(&self, x: f64) -> f64 {
        if x >= self.outer[0].0 {
            return 0.;
        }
        for i in 0..ARCH_SAMPLES - 1 {
            let (x0, a0) = self.outer[i];
            let (x1, a1) = self.outer[i + 1];
            if (x0 >= x) != (x1 >= x) {
                return lerp(a0, a1, (x - x0) / (x1 - x0));
            }
        }
        self.outer_max()
    }
}

pub struct Parts {
    pub map: Map,
    pub body: HumanBody,
    pub shape: ShapeParams,
    pub artic: HumanArtic,
    /// posterior pharyngeal wall, from below `bottom` up to the port window
    pub wall: Vec<Vec2>,
    /// the engine's whole posterior wall: it runs on above the port window, over the
    /// nasopharynx's roof (no air of the oral tract lies there)
    pub full_wall: Vec<Vec2>,
    /// what the tongue faces (the wall, the soft palate from its tip, the hard palate) and what
    /// the soft palate faces (the epiglottis, the tongue's back, its underside)
    outer: Vec<Vec2>,
    inner: Vec<Vec2>,
    /// the pharyngeal ring's sample heights and the midline wall's x at each
    pub ring_y: Vec<f64>,
    pub ring_x: Vec<f64>,
    /// x the pharynx's side walls reach at its half-width, at the mouth's height (the back of
    /// the dental arches) and low in the throat (just in front of the epiglottis' foot)
    pub ring_front: f64,
    pub ring_front_low: f64,
    /// velum oral surface, uvula tip → root, and each point's arc-length fraction from the tip
    pub velum: Vec<Vec2>,
    /// its nasal surface, point for point (display: no air of the oral tract lies above it)
    pub velum_nasal: Vec<Vec2>,
    pub velum_f: Vec<f64>,
    pub velum_len: f64,
    pub velum_top: f64,
    pub velum_bottom: f64,
    /// per velum point: how far (cm) its sides come down to the tongue (the faucial arches),
    /// and the tube's half-width there
    pub varch: Vec<f64>,
    pub vtube: Vec<f64>,
    /// the uvula's lobe: where it hangs from the soft palate's oral surface, its axis, its
    /// length, its radius front to back and its half-width (cm)
    pub uvula: (Vec2, Vec2, f64, f64, f64),
    /// hard palate, behind the velum root → upper incisor edge
    pub palate: Vec<Vec2>,
    pub palate_drop: Vec<f64>,
    pub palate_xt: Vec<f64>,
    /// arc length along the hard palate from its back edge, and how far the roof behind the
    /// back teeth comes down with the soft palate's root
    palate_s: Vec<f64>,
    palate_arch: Vec<f64>,
    /// upper lip inner outline, base → edge
    pub upper_lip: Vec<Vec2>,
    pub epi: [Vec2; 2],
    pub vallecula: Vec2,
    /// the tongue root below the epiglottis' tip, from the vallecula's floor up to `dorsal[0]`
    pub root: Vec<Vec2>,
    /// tongue dorsal chain (above the epiglottis tip → tip apex) and its per-point lateral rule
    pub dorsal: Vec<Vec2>,
    pub dn: Vec<Vec2>,
    pub dt: Vec<f64>,
    pub dw: Vec<f64>,
    pub dth: Vec<f64>,
    pub dside: Vec<f64>,
    pub dgroove: Vec<f64>,
    pub dgw: Vec<f64>,
    /// per dorsal point: how far (cm) its sides come out to the wall it faces, how much of its
    /// own rounding-off it keeps (1 under the hard palate, 0 in the soft tube), and the tube's
    /// half-width there
    pub dclose: Vec<f64>,
    pub dloaf: Vec<f64>,
    pub dtube: Vec<f64>,
    /// underside after the tip, ending at the frenulum
    pub under: Vec<Vec2>,
    pub floor: [Vec2; 2],
    /// lower incisors and lower lip inner outline: lt0, lt1, incisor tip, lt3, lip → edge
    pub lower_front: Vec<Vec2>,
    pub glottis: Vec2,
    pub bottom: f64,
    pub epi_len: f64,
    lobe_p: Vec2,
    lobe_a: Vec2,
    lobe_u: f64,
    pub lobe_start: usize,
    condyle: Vec2,
    jaw_phi: f64,
    pub upper: Arch,
    pub lower: Arch,
    pub lip_w: f64,
    ap: f64,
    /// the laryngeal tube's elliptical sections, glottis → tube top: centre x, half-length
    /// front to back, half-width (cm). Their areas are the engine's epilarynx profile with its
    /// steps rounded.
    pub tube: [(f64, f64, f64); TUBE_SAMPLES + 1],
    /// each piriform fossa: centre x, lateral offset of its centre, front-back radius at its
    /// opening (cm); see `piriform_section`
    pir: (f64, f64, f64),
}

fn rot(a: Vec2) -> Vec2 {
    v(-a.y, a.x)
}

impl Parts {
    pub fn new(body: HumanBody) -> Self {
        Self {
            map: Map { k: body.scale, ph: body.pharynx },
            body,
            shape: ShapeParams::default(),
            artic: super::super::human::NEUTRAL,
            wall: Vec::with_capacity(40),
            full_wall: Vec::with_capacity(40),
            outer: Vec::with_capacity(100),
            inner: Vec::with_capacity(100),
            ring_y: Vec::with_capacity(RING_SAMPLES + 1),
            ring_x: Vec::with_capacity(RING_SAMPLES + 1),
            ring_front: 2.,
            ring_front_low: 1.,
            velum: Vec::with_capacity(16),
            velum_nasal: Vec::with_capacity(16),
            velum_f: Vec::with_capacity(16),
            velum_len: 1.,
            velum_top: 0.,
            velum_bottom: 0.,
            varch: Vec::with_capacity(16),
            vtube: Vec::with_capacity(16),
            uvula: (v(0., 0.), v(0., -1.), 0., 0.3, 0.4),
            palate: Vec::with_capacity(40),
            palate_drop: Vec::with_capacity(40),
            palate_xt: Vec::with_capacity(40),
            palate_s: Vec::with_capacity(40),
            palate_arch: Vec::with_capacity(40),
            upper_lip: Vec::with_capacity(8),
            epi: [v(0., 0.); 2],
            vallecula: v(0., 0.),
            root: Vec::with_capacity(ROOT_SAMPLES + 1),
            dorsal: Vec::with_capacity(64),
            dn: Vec::with_capacity(64),
            dt: Vec::with_capacity(64),
            dw: Vec::with_capacity(64),
            dth: Vec::with_capacity(64),
            dside: Vec::with_capacity(64),
            dgroove: Vec::with_capacity(64),
            dgw: Vec::with_capacity(64),
            dclose: Vec::with_capacity(64),
            dloaf: Vec::with_capacity(64),
            dtube: Vec::with_capacity(64),
            under: Vec::with_capacity(8),
            floor: [v(0., 0.); 2],
            lower_front: Vec::with_capacity(12),
            glottis: v(0., 0.),
            bottom: 0.,
            epi_len: 0.,
            lobe_p: v(0., 0.),
            lobe_a: v(1., 0.),
            lobe_u: 1.,
            lobe_start: 0,
            condyle: v(0., 0.),
            jaw_phi: 0.,
            upper: Arch::new(UPPER_INCISOR_X, ARCH_DEPTH, 1.9, 0.6),
            lower: Arch::new(LOWER_INCISOR_X, ARCH_DEPTH - 0.2, 1.8, 0.6),
            lip_w: 1.,
            ap: 0.,
            tube: [(0., 0., 0.); TUBE_SAMPLES + 1],
            pir: (0., 1., 0.3),
        }
    }

    /// Split the frame's outlines into parts. The index layout follows how `HumanVocalTract`
    /// builds `outer` and `inner`; `tests/tract3d.rs` fails if that layout changes.
    pub fn update(&mut self, tract: &HumanVocalTract, f: &HumanFrame, a: &HumanArtic, s: &ShapeParams) {
        let m = self.map;
        let k = m.k;
        self.shape = *s;
        self.artic = *a;
        let no = f.outer.len();
        let ni = f.inner.len();
        // outer: wall …, velum (11, tip → root), palate[1..] (33), upper lip inner (7)
        self.wall.clear();
        self.wall.extend_from_slice(&f.outer[..no - 51]);
        self.full_wall.clear();
        self.full_wall.extend_from_slice(tract.pharynx_wall());
        self.velum.clear();
        self.velum.extend_from_slice(&f.outer[no - 51..no - 40]);
        // (the frame's outline: the nasal surface from the root to the tip, then the oral one back)
        self.velum_nasal.clear();
        self.velum_nasal.extend(f.velum[..f.velum.len() / 2].iter().rev());
        self.palate.clear();
        self.palate.extend_from_slice(&f.outer[no - 41..no - 7]);
        self.upper_lip.clear();
        self.upper_lip.extend_from_slice(&f.outer[no - 7..]);
        // inner: epiglottis base, tip, dorsal …, tip apex, underside[1..] (5, last = frenulum),
        // floor[1..] (2), lower teeth[1..] (3), lower lip inner (7, first = last tooth point)
        self.epi = [f.inner[0], f.inner[1]];
        self.vallecula = f.epiglottis[2];
        self.dorsal.clear();
        self.dorsal.extend_from_slice(&f.inner[2..=ni - 18]);
        self.under.clear();
        self.under.extend_from_slice(&f.inner[ni - 17..=ni - 13]);
        self.floor = [f.inner[ni - 12], f.inner[ni - 11]];
        self.lower_front.clear();
        self.lower_front.extend_from_slice(&f.inner[ni - 11..ni - 7]);
        self.lower_front.extend_from_slice(&f.inner[ni - 6..]);
        self.glottis = f.glottis;
        self.epi_len = 2.4 * k * self.body.pharynx;
        self.bottom = f.glottis.y + 2.2 * k * self.body.pharynx;
        // the tongue root under the epiglottis' tip: the engine's own root curve from the
        // vallecula's floor up to where the acoustic wall takes it over (`dorsal[0]`)
        self.root.clear();
        let y_root = self.dorsal[0].y;
        for q in f.tongue.iter().take(ROOT_SAMPLES) {
            if q.y >= y_root {
                break;
            }
            self.root.push(*q);
        }
        self.root.push(self.dorsal[0]);

        // the pharyngeal ring's heights, with the midline wall there
        self.ring_front = m.p(ARCH_DEPTH_BACK, 8.).x + 0.3 * k;
        self.ring_front_low = self.epi[0].x + 0.7 * k;
        self.ring_y.clear();
        self.ring_x.clear();
        let (y_lo, y_hi) = (f.glottis.y - 1. * k, 13. * k);
        for i in 0..=RING_SAMPLES {
            let y = lerp(y_lo, y_hi, i as f64 / RING_SAMPLES as f64);
            self.ring_y.push(y);
            self.ring_x.push(self.wall_x(y));
        }

        // the laryngeal tube: elliptical sections whose areas are the engine's epilarynx profile
        // with its steps rounded (two passes of ¼-½-¼ over 1 mm samples), times
        // `epilarynxWidth` / 0.6 (a speaker's tube against the voice's). Each section is as long
        // front to back as the larynx leaves room (the front wall, the epiglottis' stalk, leans
        // back to where the engine's inner wall begins; the back wall keeps the arytenoids'
        // thickness off the pharyngeal wall below the rim) and as wide as its area asks: the
        // ventricle bulges sideways, the ventricular folds draw in, the vestibule widens to its
        // rim. A narrowed tube (twang) is shut from behind.
        let n = TUBE_SAMPLES;
        let mut area = [0.; TUBE_SAMPLES + 1];
        for (j, x) in area.iter_mut().enumerate() {
            *x = tract.epilarynx_area(a, self.epi_len * j as f64 / n as f64, self.epi_len);
        }
        for _ in 0..2 {
            let prev = area;
            for j in 1..n {
                area[j] = 0.25 * prev[j - 1] + 0.5 * prev[j] + 0.25 * prev[j + 1];
            }
        }
        let size = s.epilarynx_width / 0.6;
        // the narrowest a section's half-width may be for its area (cm): what sets its length
        // when the larynx leaves more room than it needs
        let width = |t: f64| 0.6 * k * (0.9 - 0.2 * smoothstep(0.2, 0.38, t) + 0.1 * smoothstep(0.45, 0.7, t) + 0.4 * smoothstep(0.7, 1., t));
        let mut tube = [(0., 0., 0.); TUBE_SAMPLES + 1];
        {
            let room = |t: f64, front: f64| {
                let y = f.glottis.y + self.epi_len * t;
                0.5 * (front - self.wall_x(y) - (0.02 + 0.3 * (1. - smoothstep(0.5, 0.9, t))) * k)
            };
            // at the glottis the tube is centred on the engine's glottis
            let a0 = (area[0] / (PI * width(0.))).min(room(0., f.glottis.x) / 0.75);
            for (j, section) in tube.iter_mut().enumerate() {
                let t = j as f64 / n as f64;
                let front = f.glottis.x + lerp(a0, self.epi[0].x - f.glottis.x, smoothstep(0., 1., t));
                let half = (area[j] / (PI * width(t))).min(room(t, front)).max(0.02 * k);
                *section = (front - half, half, size * area[j] / (PI * half));
            }
        }
        self.tube = tube;

        self.velum_f.clear();
        self.velum_f.push(0.);
        for i in 1..self.velum.len() {
            let d = self.velum_f[i - 1] + dist(self.velum[i - 1], self.velum[i]);
            self.velum_f.push(d);
        }
        self.velum_len = self.velum_f.last().copied().unwrap_or(1.).max(1e-6);
        for f in &mut self.velum_f {
            *f /= self.velum_len;
        }
        self.velum_top = self.velum.iter().map(|q| q.y).fold(f64::NEG_INFINITY, f64::max);
        self.velum_bottom = self.velum.iter().map(|q| q.y).fold(f64::INFINITY, f64::min);

        // jaw rotation about the condyle, read from where the frame put the lower incisor
        self.condyle = m.p(-2.5, 10.5);
        let rest = sub(m.p(6.95, 7.75), self.condyle);
        let now = sub(f.lower_teeth[2], self.condyle);
        self.jaw_phi = atan2(rest.x * now.y - rest.y * now.x, rest.x * now.x + rest.y * now.y);

        self.upper = Arch::new(UPPER_INCISOR_X, ARCH_DEPTH, s.arch_width, s.arch_taper);
        self.lower = Arch::new(LOWER_INCISOR_X, ARCH_DEPTH - 0.2, s.lower_arch_width, s.arch_taper);
        let lw = self.body.lip_width;
        // the engine's opening is at least 1.1 cm wide; rounded lips draw their corners in
        // further (MRI of /u y/ shows openings about 0.5 cm across), which `lipRound` allows
        self.lip_w = 0.5 * (1.1 + (lw - 1.1) * a.lip_width.clamp(0., 1.)) * (1. - 0.65 * s.lip_round) * k;
        let ul_edge = *self.upper_lip.last().unwrap();
        let ll_edge = *self.lower_front.last().unwrap();
        self.ap = ul_edge.y - ll_edge.y;

        // palate: how far the roof may drop towards the arch (down to the gum line)
        self.palate_drop.clear();
        self.palate_xt.clear();
        self.palate_s.clear();
        for (i, q) in self.palate.iter().enumerate() {
            let xt = m.un_p(*q).x;
            let gum = m.p(xt, occlusal(xt) + crown_height(&self.upper, xt)).y;
            self.palate_drop.push((q.y - gum).clamp(0., s.vault_depth * k));
            self.palate_xt.push(xt);
            let before = if i == 0 { 0. } else { self.palate_s[i - 1] + dist(self.palate[i - 1], *q) };
            self.palate_s.push(before);
        }

        // tongue: arc-length parameter t from the vallecula (0) to the tip apex (1)
        let tip_index = f.tongue.len() - 1 - 5 - 2; // under[1..] (5) and floor[1..] (2) follow the apex
        let mut total = 0.;
        for i in 1..=tip_index {
            total += dist(f.tongue[i - 1], f.tongue[i]);
        }
        let n = self.dorsal.len();
        self.dt.clear();
        self.dt.resize(n, 1.);
        let mut rem = 0.;
        for i in (0..n).rev() {
            if i + 1 < n {
                rem += dist(self.dorsal[i], self.dorsal[i + 1]);
            }
            self.dt[i] = (1. - rem / total.max(1e-9)).clamp(0., 1.);
        }
        // (the root's own half-width: it is convex across all of it, so the grooves beside it
        // run forward between it and the pharynx's side walls)
        let widths = [
            1.75 * s.tongue_width,
            2.25 * s.tongue_width,
            2.45 * s.tongue_width,
            1.75 * s.tongue_width,
            s.tip_width,
        ];
        let thick = [1.2, 1.5, 1.5, 0.9, 0.45];
        let sides = [s.side_root, s.side_back, s.side_dorsum, s.side_blade, s.side_tip];
        let grooves = [s.groove_root, s.groove_back, s.groove_dorsum, s.groove_blade, s.groove_tip];
        self.dn.clear();
        self.dw.clear();
        self.dth.clear();
        self.dside.clear();
        self.dgroove.clear();
        self.dgw.clear();
        for i in 0..n {
            let d = sub(self.dorsal[(i + 1).min(n - 1)], self.dorsal[i.saturating_sub(1)]);
            self.dn.push(norm(rot(d)));
            let t = self.dt[i];
            self.dw.push(station(widths, t).max(0.2) * k);
            self.dth.push(station(thick, t) * k);
            self.dside.push(station(sides, t) * k);
            self.dgroove.push(station(grooves, t).max(0.) * k);
            self.dgw.push(s.groove_width * (1. - 2.5 * s.groove_taper * (t - 0.8)).clamp(0.3, 2.) * k);
        }
        // tip lobe: in front of the line from the blade's start (end of the body arc) to the
        // frenulum; it shortens laterally as the tongue narrows towards its tip
        self.lobe_start = n.saturating_sub(7);
        // seen from above the tongue's front is round: it keeps the blade's width and comes in
        // to the tip's over the lobe's last stretch (a quarter of an ellipse; the widths between
        // stations made it a wedge, with air beside it down to the floor of the mouth). A
        // lateral draws its sides in: the wedge again, with the channels beside it.
        {
            let (start, last) = (self.lobe_start, n - 1);
            let (blade, tip) = (self.dw[start], (s.tip_width * k).max(0.2 * k));
            let lateral = a.lateral.clamp(0., 1.);
            for i in start + 1..=last {
                let u = (i - start) as f64 / (last - start) as f64;
                self.dw[i] = lerp(tip + (blade - tip) * sqrt(1. - u * u), self.dw[i], lateral);
            }
        }
        self.lobe_p = self.dorsal[self.lobe_start];
        let fren = *self.under.last().unwrap();
        let tip = self.dorsal[n - 1];
        let mut a_dir = norm(rot(sub(fren, self.lobe_p)));
        if dot(a_dir, sub(tip, self.lobe_p)) < 0. {
            a_dir = scale(a_dir, -1.);
        }
        self.lobe_a = a_dir;
        self.lobe_u = dot(sub(tip, self.lobe_p), a_dir).max(1e-3);

        // ---- the soft tube. Each dorsal point looks along its normal at what it faces: the
        // posterior wall (its sides then come out to it: the lumen ends sideways between a flat
        // wall and a hollowed tongue), the soft palate (whose sides come down to the tongue
        // instead: the tongue stays level across) or the hard palate (a loaf between the teeth,
        // from 1.5 cm in front of the palate's back edge on).
        self.outer.clear();
        self.outer.extend_from_slice(&self.wall);
        self.outer.extend_from_slice(&self.velum);
        self.outer.extend_from_slice(&self.palate[1..]);
        let (nw, nv) = (self.wall.len(), self.velum.len());
        let lower_back = self.lower_occlusal(self.lower.back(), 0.).x;
        self.dclose.clear();
        self.dloaf.clear();
        self.dtube.clear();
        for i in 0..n {
            let (q, nrm) = (self.dorsal[i], self.dn[i]);
            let (mut close, mut loaf, mut y) = (0., 1., q.y);
            if let Some((t, seg, u)) = ray_hit(q, nrm, &self.outer, 6. * k) {
                let (roof, soft) = if seg < nw {
                    (0., 1.)
                } else if seg < nw + nv - 1 {
                    (smoothstep(0.1, 0.5, lerp(self.velum_f[seg - nw], self.velum_f[seg - nw + 1], u)), 1.)
                } else {
                    let j = (seg - (nw + nv - 1)).min(self.palate_s.len() - 2);
                    (1., 1. - smoothstep(0., 1.5 * k, lerp(self.palate_s[j], self.palate_s[j + 1], u)))
                };
                close = soft * (1. - roof) * t;
                loaf = 1. - soft;
                y = q.y + 0.5 * t * nrm.y;
            }
            // behind the last lower molar nothing stands beside the tongue for its margin to
            // round off against: it stays level out to the cheek (rounded off there, it left a
            // gutter 9 mm deep on each side of the tongue's back when the jaw was down)
            loaf *= smoothstep(lower_back - 0.2 * k, lower_back + 1.0 * k, q.x);
            self.dclose.push(close);
            self.dloaf.push(loaf);
            self.dtube.push(self.ring_width(y, true));
        }
        // (two passes of ¼-½-¼ along the chain: a ray that passes the uvula's tip moves from the
        // soft palate to the wall behind it)
        for values in [&mut self.dclose, &mut self.dloaf, &mut self.dtube] {
            for _ in 0..2 {
                let mut prev = values[0];
                for i in 1..n - 1 {
                    let here = values[i];
                    values[i] = 0.25 * prev + 0.5 * here + 0.25 * values[i + 1];
                    prev = here;
                }
            }
        }
        // the soft palate's own side: each of its points looks down at the tongue, and its
        // sides come down that far (the faucial arches). Nothing of that at the uvula's end.
        self.inner.clear();
        self.inner.extend_from_slice(&self.epi);
        self.inner.extend_from_slice(&self.dorsal);
        self.inner.extend_from_slice(&self.under);
        self.varch.clear();
        self.vtube.clear();
        self.varch.resize(nv, 0.);
        self.vtube.resize(nv, 0.);
        // (straight down, so the roof stays a graph over x; from the root back to the tip, a
        // point with no tongue under it taking the gap of the one in front of it)
        let mut gap = 2.5 * k;
        for j in (0..nv).rev() {
            if let Some(h) = ray_hit(self.velum[j], v(0., -1.), &self.inner, 4. * k) {
                gap = h.0;
            }
            self.varch[j] = smoothstep(0.1, 0.5, self.velum_f[j]) * gap;
            self.vtube[j] = self.ring_width(self.velum[j].y - 0.5 * gap, true);
        }
        // behind the back teeth the hard palate's edge comes down with the soft palate's root,
        // less and less up to the last molar (the arch's front slope)
        self.palate_arch.clear();
        let root = self.varch[nv - 1];
        let pns = self.palate_xt[0];
        for i in 0..self.palate.len() {
            self.palate_arch.push(root * (1. - smoothstep(pns, ARCH_DEPTH_BACK + 0.4, self.palate_xt[i])));
        }
        // the uvula: a lobe hanging from the soft palate's oral surface just in front of its
        // tip. Raised, the soft palate lies back against the wall and the lobe hangs down in
        // front of it; lowered, the soft palate hangs itself and the lobe carries on along it.
        {
            let open = (a.velum / 1.2).clamp(0., 1.);
            let at = 0.16 * (1. - open);
            let j = (0..nv - 1).find(|&j| self.velum_f[j + 1] >= at).unwrap_or(0);
            let u = ((at - self.velum_f[j]) / (self.velum_f[j + 1] - self.velum_f[j]).max(1e-9)).clamp(0., 1.);
            let base = lerp_v(self.velum[j], self.velum[j + 1], u);
            let angle = lerp(262., 242., open) * PI / 180.;
            let axis = v(cos(angle), sin(angle));
            let whole = s.uvula_length * k * lerp(0.8, 0.35, open);
            // (squashed where the tongue's back comes up to it)
            let length = ray_hit(add(base, scale(axis, 0.05 * k)), axis, &self.inner, whole + k).map_or(whole, |h| (h.0 + 0.03 * k).clamp(0.35 * whole, whole));
            self.uvula = (base, axis, length, 0.3 * k * (s.uvula_width / 0.8).min(1.2), 0.5 * s.uvula_width * k);
        }

        // the piriform fossae: beside the laryngeal tube's rim, beyond the aryepiglottic fold,
        // each between the ring behind it and the front wall of the hypopharynx at its own
        // lateral offset (the ring at rest: a squeeze of the pharynx does not move them)
        let zc = self.fold_span().1 + 0.03 * k + s.piriform_width * k;
        let back = self.ring_x_at(self.bottom, zc, false) + 0.04 * k;
        // (its front runs 1 mm under the front wall's foot)
        let front = self.front_wall(0., zc, false).x + 0.1 * k;
        // (a pouch wider below its mouth than the lumen above it: 0.85 of the lumen's depth there
        // as its front-to-back radius)
        self.pir = (0.5 * (front + back), zc, (0.85 * (front - back)).max(0.15 * k));
    }

    /// The lateral offsets (cm) between which each aryepiglottic fold stands: from just outside
    /// the laryngeal tube's rim, 3.6 mm across at its foot.
    pub fn fold_span(&self) -> (f64, f64) {
        let k = self.map.k;
        let rim = self.tube[TUBE_SAMPLES].2;
        (rim + 0.02 * k, rim + 0.38 * k)
    }
    /// The top of an aryepiglottic fold at lateral offset z and a share u of the way from the
    /// back wall to the front one: highest at the epiglottis, lower at the arytenoid, rounded
    /// across its thickness. None beside the fold.
    pub fn fold_top(&self, z: f64, u: f64) -> Option<f64> {
        let (lo, hi) = self.fold_span();
        let v = (z.abs() - lo) / (hi - lo);
        if !(0. ..=1.).contains(&v) || self.shape.ary_fold_height <= 0. {
            return None;
        }
        // a ridge: its crest along the middle of its thickness, its flanks down to the floor
        let crest = (0.35 + 0.65 * smoothstep(0., 1., u)) * sqrt(sin(PI * v).max(0.));
        Some(self.bottom + self.shape.ary_fold_height * self.map.k * crest)
    }

    /// The posterior wall's midline x at height y, on the engine's whole wall: it runs on above
    /// the port window and forward over the nasopharynx's roof (below its first point and above
    /// its last, the nearest point's x).
    pub fn wall_x(&self, y: f64) -> f64 {
        let wall = &self.full_wall;
        let n = wall.len();
        if y <= wall[0].y {
            return wall[0].x;
        }
        for i in 1..n {
            let (a, b) = (wall[i - 1], wall[i]);
            if y <= b.y {
                return lerp(a.x, b.x, ((y - a.y) / (b.y - a.y).max(1e-9)).clamp(0., 1.));
            }
        }
        wall[n - 1].x
    }
    /// How much of the gap between the tube's two walls is taken up at a share `w` of its
    /// half-width: nothing on the midline, all of it from the tube's side on (and 3 % more, so
    /// the walls press into each other there). `pharynxShape` is the outline's exponent.
    pub fn closing(&self, w: f64) -> f64 {
        1.03 * (1. - superellipse(w, self.shape.pharynx_shape))
    }

    pub fn jaw(&self, x: f64, y: f64) -> Vec2 {
        rotate_about(self.map.p(x, y), self.condyle, self.jaw_phi)
    }
    /// the jaw's pivot (cm) and its rotation from rest (rad)
    pub fn condyle(&self) -> Vec2 {
        self.condyle
    }
    pub fn jaw_angle(&self) -> f64 {
        self.jaw_phi
    }
    /// half-width (cm) of the pharyngeal ring at body height y
    pub fn pharynx_width(&self, y: f64) -> f64 {
        self.ring_width(y, true)
    }
    /// the ring's half-width with the side walls drawn in by `lateralSqueeze`, or at rest
    fn ring_width(&self, y: f64, squeezed: bool) -> f64 {
        let s = &self.shape;
        let yt = self.map.un_p(v(0., y)).y;
        lerp(
            lerp(s.pharynx_width_hypo, s.pharynx_width_oro, smoothstep(2., 6., yt)),
            s.pharynx_width_naso,
            smoothstep(6., 9., yt),
        ) * (if squeezed { 1. - 0.5 * s.lateral_squeeze } else { 1. })
            * self.map.k
    }
    /// the ring's x at height y and lateral offset z, from the midline wall's x there
    fn ring_x_at(&self, y: f64, z: f64, squeezed: bool) -> f64 {
        let x0 = self.wall_x(y);
        let w = (z.abs() / self.ring_width(y, squeezed)).min(1.);
        lerp(x0, self.ring_reach(y).max(x0), 1. - superellipse(w, RING_SHAPE))
    }
    /// How far forward the side walls come at height y: to the back teeth beside the mouth, to
    /// the thyroid's plates in front of the epiglottis' foot low in the throat.
    fn ring_reach(&self, y: f64) -> f64 {
        lerp(self.ring_front_low, self.ring_front, smoothstep(3., 6.5, self.map.un_p(v(0., y)).y))
    }
    /// Tongue dorsal point `i` at lateral offset z, before the tip-lobe compression.
    pub fn dorsal_at(&self, i: usize, z: f64) -> Vec2 {
        let s = &self.shape;
        let q = self.dorsal[i];
        let n = self.dn[i];
        let th = self.dth[i];
        let w = z.abs() / self.dw[i];
        // under the hard palate the tongue is a loaf: convex across its whole width, rounding
        // off to its core at the margins (a middle drawn flat gave every cut a straight floor).
        // In the soft tube it keeps its midline's level and its sides come out to the wall
        // (each rule's powers only where the rule acts: the display asks for thousands of points)
        let e = if self.dloaf[i] > 0. { 1. + self.dloaf[i] * (superellipse(w, s.edge_round) - 1.) } else { 1. };
        // raising (bracing, cupping) spreads from a quarter of the width; lowering (lateral
        // channels) stays at the margins, leaving a midline contact for /l/
        let rise = if self.dside[i] >= 0. { smoothstep(0.25, 0.8, w) } else { smoothstep(0.55, 0.95, w) };
        let side = rise * (1. - smoothstep(0.92, 1., w)) * (1. + s.side_asym * z.signum());
        let gw = self.dgw[i];
        let g = if z.abs() < gw { 0.5 + 0.5 * cos(PI * z / gw) } else { 0. };
        let lam = (e + (self.dside[i] * side - self.dgroove[i] * g) / th).clamp(0., 3.);
        let close = if self.dclose[i] > 0. { self.dclose[i] * self.closing(z.abs() / self.dtube[i]) } else { 0. };
        add(q, scale(n, th * (lam - 1.) + close))
    }
    /// how much of the tip lobe remains at lateral offset z (1: all of it)
    pub fn lobe_factor(&self, z: f64) -> f64 {
        let az = z.abs();
        let n = self.dorsal.len();
        if self.dw[n - 1] >= az {
            return 1.;
        }
        for i in (self.lobe_start..n - 1).rev() {
            if self.dw[i] >= az {
                let f = (az - self.dw[i]) / (self.dw[i + 1] - self.dw[i]);
                let q = lerp_v(self.dorsal[i], self.dorsal[i + 1], f.clamp(0., 1.));
                return (dot(sub(q, self.lobe_p), self.lobe_a) / self.lobe_u).clamp(0.03, 1.);
            }
        }
        0.03
    }
    pub fn compress(&self, p: Vec2, factor: f64) -> Vec2 {
        let u = dot(sub(p, self.lobe_p), self.lobe_a);
        if u <= 0. {
            p
        } else {
            sub(p, scale(self.lobe_a, u * (1. - factor)))
        }
    }
    /// tongue surface point (dorsal chain, lobe compressed)
    pub fn tongue_at(&self, i: usize, z: f64, lobe: f64) -> Vec2 {
        let p = self.dorsal_at(i, z);
        if i > self.lobe_start {
            self.compress(p, lobe)
        } else {
            p
        }
    }
    /// roof (palate) point i at lateral offset z
    pub fn palate_at(&self, i: usize, z: f64) -> Vec2 {
        let q = self.palate[i];
        let a = self.upper.inner_at(self.palate_xt[i]) * self.map.k;
        let w = if a < 1e-6 { 1. } else { (z.abs() / a).min(1.) };
        let arch = self.palate_arch[i];
        let arch = if arch > 0. { arch * self.closing(z.abs() / self.vtube[self.vtube.len() - 1]) } else { 0. };
        v(q.x, q.y - self.palate_drop[i] * pow(w, self.shape.vault_shape) - arch)
    }
    /// fraction of the velum (from the tip) that is uvula
    pub fn uvula_fraction(&self) -> f64 {
        (self.shape.uvula_length * self.map.k / self.velum_len).clamp(0.15, 0.6)
    }
    /// half-width (cm) of the soft palate at arc fraction f from the tip: the uvula hangs as a
    /// narrow lobe from the middle of the free margin, which widens fast into the soft palate
    pub fn velum_width_at(&self, f: f64) -> f64 {
        let u = self.uvula_fraction();
        0.5 * lerp(self.shape.uvula_width, self.shape.velum_width, smoothstep(0.85 * u, u + 0.25, f)) * self.map.k
    }
    /// How far sideways the roof reaches at arc fraction f from the tip: the uvula's own width
    /// along the uvula, the soft palate's as its free margin widens, and from there to the root
    /// any width (beyond the soft palate's own edge the roof is the faucial arch).
    pub fn roof_width_at(&self, f: f64) -> f64 {
        if f >= self.uvula_fraction() + 0.25 {
            f64::INFINITY
        } else {
            self.velum_width_at(f)
        }
    }
    /// the velum's point i at lateral offset z: off the midline the free margin recedes towards
    /// the root (the faucial arches), so a lamina meets the velum only from where it is that wide
    pub fn velum_at(&self, i: usize, z: f64) -> Vec2 {
        let az = z.abs();
        let n = self.velum.len();
        let lift = |j: usize| {
            let u = self.uvula_fraction();
            let q = self.velum[j];
            // a longer or shorter uvula moves its tip down or up
            let q = v(q.x, q.y - (self.shape.uvula_length - 1.) * (1. - self.velum_f[j] / u).max(0.) * 0.8 * self.map.k);
            // the faucial arches: the sides come down to the tongue
            v(q.x, q.y - self.varch[j] * self.closing(az / self.vtube[j]))
        };
        if self.roof_width_at(self.velum_f[i]) >= az {
            return lift(i);
        }
        // the first point from the tip that is wide enough, interpolated
        for j in i + 1..n {
            if self.roof_width_at(self.velum_f[j]) >= az {
                let (w0, w1) = (self.roof_width_at(self.velum_f[j - 1]), self.roof_width_at(self.velum_f[j]).min(4. * self.map.k));
                return lerp_v(lift(j - 1), lift(j), ((az - w0) / (w1 - w0).max(1e-9)).clamp(0., 1.));
            }
        }
        lift(n - 1)
    }
    /// The pharyngeal ring at sample height i and lateral offset z: the posterior wall on the
    /// midline, coming forward round the sides until, at the ring's half-width, it has reached
    /// the back teeth (a superellipse of exponent `pharynxShape`). The lumen ends sideways where
    /// this surface meets what stands in front of it: the side of the tongue's root (the
    /// grooves beside it run forward as horns), the epiglottis and its folds, the pillars. A
    /// deep pharynx is therefore also a wide one.
    pub fn ring_at(&self, i: usize, z: f64) -> Vec2 {
        let (y, x0) = (self.ring_y[i], self.ring_x[i]);
        let w = (z.abs() / self.pharynx_width(y)).min(1.);
        v(lerp(x0, self.ring_reach(y).max(x0), 1. - superellipse(w, RING_SHAPE)), y)
    }
    /// The ring at height y and lateral offset z: the point of the lamina's own polyline there
    /// (between `ring_at` of the two samples around y), so a surface drawn through these points
    /// is the surface the cuts meet.
    pub fn wall_point(&self, y: f64, z: f64) -> Vec2 {
        let n = self.ring_y.len();
        let step = (self.ring_y[n - 1] - self.ring_y[0]) / (n - 1) as f64;
        let q = ((y - self.ring_y[0]) / step).clamp(0., (n - 1) as f64);
        let i = (q as usize).min(n - 2);
        let (a, b) = (self.ring_at(i, z), self.ring_at(i + 1, z));
        v(lerp(a.x, b.x, q - i as f64), y)
    }
    /// The lips' meeting line at lateral offset z, for the head's skin to follow: the upper and
    /// the lower lip's edge. Inside the opening they stand `gap` apart, as `lip_close` leaves
    /// them; from the corners out they coincide (the laminae press them 4 mm further into each
    /// other there, to seal).
    pub fn lip_edge(&self, z: f64) -> (Vec2, Vec2) {
        let nu = self.upper_lip.len() - 1;
        let nl = self.lower_front.len() - 1;
        let (u, l) = (self.upper_lip[nu], self.lower_front[nl]);
        // where the closed lips meet: the upper lip carries 35 % of the closing, the lower 65 %
        let meet = 0.65 * u.y + 0.35 * l.y;
        let gap = (u.y - l.y).max(0.) * superellipse(z / self.lip_w, self.shape.lip_corner_shape);
        let xu = self.upper_lip_at(nu, z, self.front_shift(&self.upper, z)).x;
        let xl = self.lower_lip_at(nl, z, self.front_shift(&self.lower, z)).x;
        (v(xu, meet + 0.35 * gap), v(xl, meet - 0.65 * gap))
    }
    /// The front wall of the hypopharynx at lateral offset z, a fraction f of the way from the
    /// engine's epiglottis base to its tip: the epiglottis' laryngeal surface over its own
    /// width, its sides curled backwards (`epiglottisCurl`); it and the folds beyond its edges
    /// come back to the posterior wall at the tube's sides (`closing`).
    pub fn epiglottis_at(&self, f: f64, z: f64) -> Vec2 {
        self.front_wall(f, z, true)
    }
    /// `epiglottis_at` with the pharynx's side walls as `lateralSqueeze` draws them in, or at rest.
    fn front_wall(&self, f: f64, z: f64, squeezed: bool) -> Vec2 {
        let k = self.map.k;
        let we = self.shape.epiglottis_width * k;
        let az = z.abs().min(we);
        // (weights that give the engine's own two points exactly at f = 0 and 1)
        let (a, b) = (self.epi[0], self.epi[1]);
        let (x, y) = (a.x * (1. - f) + b.x * f - self.shape.epiglottis_curl * az * az / k, a.y * (1. - f) + b.y * f);
        // the tube's front wall: its sides come back to the posterior wall
        let back = self.wall_x(y);
        v(x - (x - back).max(0.) * self.closing(z.abs() / self.ring_width(y, squeezed)), y)
    }
    /// How far up from the base (0…1 of the midline's height) the front wall stands at lateral
    /// offset z: the epiglottis is a leaf, highest on the midline and lowest at its edges;
    /// beyond them the wall rises again into the folds that run to the pharynx's sides. The
    /// saddle at each edge is where a vallecula opens into the pharynx beside the leaf.
    pub fn epiglottis_top(&self, z: f64) -> f64 {
        let k = self.map.k;
        let we = self.shape.epiglottis_width * k;
        let az = z.abs();
        if az <= we {
            1. - 0.4 * (az / we) * (az / we)
        } else {
            0.6 + 0.4 * smoothstep(we, we + 0.5 * k, az)
        }
    }
    /// The leaf's tip at lateral offset z (the top of the front wall there).
    pub fn epiglottis_tip(&self, z: f64) -> Vec2 {
        self.epiglottis_at(self.epiglottis_top(z), z)
    }
    /// The leaf's thickness as a vector from its laryngeal to its lingual surface.
    pub fn epiglottis_thickness(&self) -> Vec2 {
        let d = norm(sub(self.epi[1], self.epi[0]));
        scale(v(d.y, -d.x), EPIGLOTTIS_THICKNESS * self.map.k)
    }
    /// How deep (cm) the vallecula is below the leaf's tip at lateral offset z: two troughs,
    /// each ending under the leaf's edge, with the median fold between them a ridge that stands
    /// 55 % of their depth above their floors (the two communicate over it).
    pub fn vallecula_depth_at(&self, z: f64) -> f64 {
        let (c, r) = self.vallecula(1.);
        let w = (z.abs() - c) / r;
        if w >= 1. {
            return 0.;
        }
        let profile = if w >= 0. { sqrt(1. - w * w) } else { 1. - 0.55 * w * w };
        let tip = self.epiglottis_tip(z);
        (self.shape.vallecula_depth * self.map.k * profile).min((tip.y - self.root[0].y).max(0.))
    }
    /// The tongue root's x under the leaf's tip at height y and lateral offset z: the engine's
    /// root curve, moved with the tongue's surface above it.
    pub fn root_x(&self, y: f64, z: f64) -> f64 {
        let shift = self.dorsal_at(0, z).x - self.dorsal[0].x;
        let n = self.root.len();
        for i in 1..n {
            let (a, b) = (self.root[i - 1], self.root[i]);
            if y <= b.y || i + 1 == n {
                return lerp(a.x, b.x, ((y - a.y) / (b.y - a.y).max(1e-9)).clamp(0., 1.)) + shift;
            }
        }
        self.root[0].x + shift
    }
    /// A vallecula's outline in the lamina at z, as (floor height, top height): None where the
    /// lamina has no pocket (the median fold, beyond the leaf's edge).
    pub fn vallecula_span(&self, z: f64) -> Option<(f64, f64)> {
        let depth = self.vallecula_depth_at(z);
        let top = self.epiglottis_tip(z).y;
        (depth > 1e-4 * self.map.k).then(|| (top - depth, top))
    }
    /// The lingual surface of the leaf at height y and lateral offset z.
    pub fn epiglottis_front_x(&self, y: f64, z: f64) -> f64 {
        let (base, tip) = (self.epiglottis_at(0., z), self.epiglottis_tip(z));
        lerp(base.x, tip.x, ((y - base.y) / (tip.y - base.y).max(1e-9)).clamp(0., 1.)) + self.epiglottis_thickness().x
    }
    /// One piriform fossa's horizontal section at height y: centre (x, z) and radii (front to
    /// back, lateral), cm; None above or below it. The fossa is the lower half of an ellipsoid
    /// whose equator lies 1.5 mm above the hypopharynx's floor: a pouch widest at its opening,
    /// `piriformDepth` deep below the floor.
    pub fn piriform_section(&self, side: f64, y: f64) -> Option<(f64, f64, f64, f64)> {
        let (top, height) = self.piriform_top();
        let u = (top - y) / height;
        if !(0. ..1.).contains(&u) || self.shape.piriform_depth < 0.2 {
            return None;
        }
        let g = sqrt(1. - u * u);
        Some((self.pir.0, side * self.pir.1, self.pir.2 * g, self.shape.piriform_width * self.map.k * g))
    }
    /// the fossae's equator height and their height below it (cm)
    pub fn piriform_top(&self) -> (f64, f64) {
        let k = self.map.k;
        (self.bottom + 0.15 * k, self.shape.piriform_depth * k + 0.15 * k)
    }
    /// the laryngeal tube's section at height fraction t (0 glottis … 1 its rim): centre x,
    /// half-length front to back, half-width
    pub fn tube_at(&self, t: f64) -> (f64, f64, f64) {
        let x = t.clamp(0., 1.) * TUBE_SAMPLES as f64;
        let i = (x as usize).min(TUBE_SAMPLES - 1);
        let (a, b, f) = (self.tube[i], self.tube[i + 1], x - i as f64);
        (lerp(a.0, b.0, f), lerp(a.1, b.1, f), lerp(a.2, b.2, f))
    }
    /// how far (cm) the lips and cheek stand behind their midline position at lateral offset z
    pub fn front_shift(&self, arch: &Arch, z: f64) -> f64 {
        let k = self.map.k;
        let at = z.abs() / k;
        let gap = (self.shape.buccal_gap + 0.6 * self.shape.cheek_puff) * smoothstep(0., 0.6, at);
        let x0 = arch.x_out(0.);
        let x = arch.x_out(at.min(arch.outer_max())) + gap;
        (self.map.p(x0, 8.).x - self.map.p(x, 8.).x).max(0.)
    }
    /// vertical closing of the lips at z: (upper lip down, lower lip up), cm
    pub fn lip_close(&self, z: f64) -> (f64, f64) {
        let k = self.map.k;
        let ap = self.ap.max(0.);
        let open = ap * superellipse(z / self.lip_w, self.shape.lip_corner_shape);
        let d = ap - open + 0.4 * k * smoothstep(self.lip_w, self.lip_w + 0.3 * k, z.abs());
        (0.35 * d, 0.65 * d)
    }
    pub fn upper_lip_at(&self, j: usize, z: f64, dx: f64) -> Vec2 {
        let n = self.upper_lip.len() - 1;
        let base = self.upper_lip[0];
        let q = self.upper_lip[j];
        let w = j as f64 / n as f64;
        let (du, _) = self.lip_close(z);
        v(base.x + (q.x - base.x) * self.shape.lip_thickness - dx, q.y - w * du)
    }
    pub fn lower_lip_at(&self, j: usize, z: f64, dx: f64) -> Vec2 {
        // lower_front[3] is the lip's base (last incisor point)
        let n = self.lower_front.len() - 1;
        let q = self.lower_front[j];
        if j <= 3 {
            return v(q.x - dx, q.y);
        }
        let base = self.lower_front[3];
        let w = (j - 3) as f64 / (n - 3) as f64;
        let (_, dl) = self.lip_close(z);
        v(base.x + (q.x - base.x) * self.shape.lip_thickness - dx, q.y + w * dl)
    }
    /// occlusal surface of the upper teeth at template x (body coordinates)
    pub fn upper_occlusal(&self, xt: f64, lift: f64) -> Vec2 {
        self.map.p(xt, occlusal(xt) + lift)
    }
    pub fn lower_occlusal(&self, xt: f64, drop: f64) -> Vec2 {
        let over = 0.2 * (1. - smoothstep(0., 1.5, UPPER_INCISOR_X - xt));
        self.jaw(xt, occlusal(xt) - over - drop)
    }

    /// The uvula's outline in the lamina at lateral offset z: from inside the soft palate down
    /// one side, round its end and back up the other. None beside it.
    pub fn uvula_outline(&self, z: f64) -> Option<[Vec2; UVULA_SAMPLES + 3]> {
        let (base, axis, length, radius, half) = self.uvula;
        let az = z.abs();
        if az >= half || length <= 0. {
            return None;
        }
        let across = sqrt(1. - (az / half) * (az / half));
        let (r, len) = (radius * across, length * (0.6 + 0.4 * across));
        let side = v(-axis.y, axis.x);
        let root = sub(base, scale(axis, 0.3 * self.map.k));
        let centre = add(base, scale(axis, (len - r).max(0.)));
        let mut out = [root; UVULA_SAMPLES + 3];
        out[0] = add(root, scale(side, r));
        for i in 0..=UVULA_SAMPLES {
            let th = PI * i as f64 / UVULA_SAMPLES as f64;
            out[i + 1] = add(centre, add(scale(side, r * cos(th)), scale(axis, r * sin(th))));
        }
        out[UVULA_SAMPLES + 2] = sub(root, scale(side, r));
        Some(out)
    }
    /// How far up from its base (0…1 of the midline's height) the epiglottis' leaf begins at
    /// lateral offset z: a leaf on a stalk, a third of its width at the foot and all of it from
    /// half its height on. (Below the valleculae's floor the leaf lies inside the front wall.)
    pub fn epiglottis_foot(&self, z: f64) -> f64 {
        let w = z.abs() / (self.shape.epiglottis_width * self.map.k);
        0.5 * ((w - 0.33) / 0.67).clamp(0., 1.)
    }
    /// The hypopharynx's floor at (x, z): its level, the aryepiglottic folds standing on it and
    /// the piriform fossae let into it (the same fold tops and pouches the laminae hold).
    pub fn floor_height(&self, x: f64, z: f64) -> f64 {
        let side = if z < 0. { -1. } else { 1. };
        let (top, height) = self.piriform_top();
        let (cx, cz, rx, rz) = self.piriform(side);
        let e = ((x - cx) / rx).powi(2) + ((z - cz) / rz).powi(2);
        if e < 1. {
            let y = top - height * sqrt(1. - e);
            if y < self.bottom {
                return y;
            }
        }
        let (back, front) = (self.wall_point(self.bottom, z).x, self.epiglottis_at(0., z).x);
        let u = ((x - back) / (front - back).max(1e-6)).clamp(0., 1.);
        self.fold_top(z, u).unwrap_or(self.bottom)
    }
    /// How far from (cx, 0) along the unit vector (dx, dz) the hypopharynx's floor reaches: to
    /// the ring behind and beside it, to the front wall in front.
    pub fn floor_reach(&self, cx: f64, dx: f64, dz: f64) -> f64 {
        let inside = |t: f64| {
            let (x, z) = (cx + t * dx, t * dz);
            x >= self.wall_point(self.bottom, z).x && x <= self.epiglottis_at(0., z).x
        };
        let (mut lo, mut hi) = (0., 5. * self.map.k);
        for _ in 0..13 {
            let mid = 0.5 * (lo + hi);
            if inside(mid) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        lo
    }
    /// How far towards its gum line a tooth row's surface has rounded off at template x and
    /// lateral offset `at` (template cm): 0 over the middle of the row and out to its buccal
    /// face (the cheek and the lips lie on that face: rounded off there, each row had a notch
    /// of air between it and them), 1 at its lingual edge and at its back end (behind the last
    /// molar).
    pub fn crown_round(&self, arch: &Arch, xt: f64, at: f64) -> f64 {
        let (inner, outer) = (arch.inner_at(xt), arch.outer_at(xt));
        let across = if outer > inner + 1e-6 { (((at - inner) / (outer - inner)).clamp(0., 1.) * 2. - 1.).min(0.) } else { 0. };
        let end = 1. - ((xt - arch.back()) / 0.45).clamp(0., 1.);
        1. - superellipse(across, 3.) * superellipse(end, 3.)
    }
    /// Fill one lamina. All polygons are rebuilt; buffers keep their capacity.
    pub fn fill(&self, l: &mut Lamina) {
        let k = self.map.k;
        let s = &self.shape;
        let z = l.z;
        let az = z.abs();
        let at = az / k;
        l.clear();
        // ---- envelope: posterior wall ↑, velum, palate, then up and forward around the face
        {
            let env = &mut l.air[AIR_ENVELOPE];
            let mut started = false;
            for i in 0..self.wall.len() {
                let q = self.wall[i];
                if q.y < self.bottom {
                    continue;
                }
                if !started {
                    started = true;
                    if i > 0 {
                        let p0 = self.wall[i - 1];
                        env.push(lerp_v(p0, q, (self.bottom - p0.y) / (q.y - p0.y)));
                    }
                }
                env.push(q);
            }
            for i in 0..self.velum.len() {
                env.push(self.velum_at(i, z));
            }
            for i in 1..self.palate.len() {
                env.push(self.palate_at(i, z));
            }
            // close just in front of the lips (the MRI segmentations end there too)
            let nu = self.upper_lip.len() - 1;
            let nl = self.lower_front.len() - 1;
            let eu = self.upper_lip_at(nu, z, self.front_shift(&self.upper, z));
            let el = self.lower_lip_at(nl, z, self.front_shift(&self.lower, z));
            let front = eu.x.max(el.x) + 0.25 * k;
            let edge = *env.last().unwrap();
            env.push(v(edge.x, 12. * k));
            env.push(v(front.max(edge.x), 12. * k));
            env.push(v(front.max(edge.x), self.bottom));
        }
        // ---- tongue with the floor of the mouth, closed far below
        {
            let lobe = self.lobe_factor(z);
            let t = &mut l.tissue[T_TONGUE];
            let base = self.epiglottis_at(0., z);
            t.push(base);
            t.push(self.epiglottis_tip(z));
            for i in 0..self.dorsal.len() {
                t.push(self.tongue_at(i, z, lobe));
            }
            let nu = self.under.len();
            for (j, &q) in self.under.iter().enumerate() {
                t.push(if j + 1 < nu { self.compress(q, lobe) } else { q });
            }
            t.push(self.floor[0]);
            t.push(self.floor[1]);
            t.push(v(self.floor[1].x, -5. * k));
            t.push(v(base.x, -5. * k));
        }
        // ---- the epiglottis' leaf: free above the valleculae's floor, inside the tongue's
        // polygon below it
        if az < s.epiglottis_width * k {
            let base = self.epiglottis_at(self.epiglottis_foot(z).min(self.epiglottis_top(z)), z);
            let tip = self.epiglottis_tip(z);
            let th = self.epiglottis_thickness();
            l.tissue[T_EPIGLOTTIS].extend_from_slice(&[base, tip, add(tip, th), add(base, th)]);
        }
        // ---- lips (and the face around them), following the dental arches back
        {
            let dx = self.front_shift(&self.upper, z);
            let t = &mut l.tissue[T_UPPER_FRONT];
            let edge = *self.palate.last().unwrap();
            t.push(v(edge.x - dx, edge.y));
            for j in 0..self.upper_lip.len() {
                t.push(self.upper_lip_at(j, z, dx));
            }
            let e = *t.last().unwrap();
            t.push(v(15. * k, e.y));
            t.push(v(15. * k, 15. * k));
            t.push(v(edge.x - dx, 15. * k));
        }
        {
            let dx = self.front_shift(&self.lower, z);
            let t = &mut l.tissue[T_LOWER_FRONT];
            for j in 0..self.lower_front.len() {
                t.push(self.lower_lip_at(j, z, dx));
            }
            let e = *t.last().unwrap();
            t.push(v(15. * k, e.y));
            t.push(v(15. * k, -5. * k));
            let first = t[0];
            t.push(v(first.x, -5. * k));
        }
        // ---- tooth rows (the midline's own lamina holds the engine's incisors; every other one
        // the rows)
        if at > 0.02 {
            for (arch, slot, upper) in [(&self.upper, T_UPPER_TEETH, true), (&self.lower, T_LOWER_TEETH, false)] {
                if at > arch.outer_max() {
                    continue;
                }
                let x0 = arch.x_in(at);
                let x1 = arch.x_out(at).max(x0);
                let t = &mut l.tissue[slot];
                for j in 0..=CROWN_SAMPLES {
                    let xt = lerp(x0, x1, j as f64 / CROWN_SAMPLES as f64);
                    // a crown is round on the row's lingual side and at the row's back end: from
                    // the gum line over its biting surface out to its upright buccal face (a
                    // row with flat faces all round cut as a block)
                    let gum = crown_height(arch, xt) * self.crown_round(arch, xt, at);
                    // upper cusps overlap the lower ones by ~1 mm: a tooth row seals until the jaw opens
                    t.push(if upper { self.upper_occlusal(xt, gum - CUSP * (1. - gum)) } else { self.lower_occlusal(xt, gum) });
                }
                let (a, b) = (t[0], t[CROWN_SAMPLES]);
                let y = if upper { 15. * k } else { -5. * k };
                t.push(v(b.x, y));
                t.push(v(a.x, y));
            }
        }
        // ---- cheeks beyond the mouth's corners; a thin vestibule beside the molars
        {
            let cl = self.lip_w + 0.25 * k;
            if az > self.lip_w {
                let slide = 8. * k * (1. - smoothstep(self.lip_w, cl, az));
                let gap = s.buccal_gap + 0.6 * s.cheek_puff;
                let top = self.upper.outer_max();
                let back = self.map.p(self.upper.back(), 8.).x;
                if at <= top {
                    // the cheek lies on the upper row's buccal face above the mouth and on the
                    // lower row's below it, where the jaw has carried that, and runs straight
                    // from one to the other (held to the upper row alone it left a pocket in
                    // front of the lower teeth whenever the jaw was down)
                    let hi = self.upper_occlusal(self.upper.x_out(at) + gap, 0.);
                    let lo = self.lower_occlusal(self.lower.x_out(at.min(self.lower.outer_max())) + gap, 0.);
                    let down = v(sin(self.jaw_phi), -cos(self.jaw_phi));
                    let x = hi.x + slide;
                    {
                        let t = &mut l.tissue[T_CHEEK[0]];
                        let foot = add(lo, scale(down, 9. * k));
                        t.push(v(foot.x + slide, foot.y));
                        t.push(v(lo.x + slide, lo.y));
                        if hi.y > lo.y + 0.05 * k {
                            for i in 1..6 {
                                let u = i as f64 / 6.;
                                t.push(v(lerp(lo.x, hi.x, smoothstep(0., 1., u)) + slide, lerp(lo.y, hi.y, u)));
                            }
                        }
                        t.push(v(x, hi.y.max(lo.y)));
                        t.push(v(x, 15. * k));
                        t.push(v(15. * k, 15. * k));
                        t.push(v(15. * k, foot.y));
                    }
                } else if at < top + gap {
                    let hi = self.upper_occlusal(self.upper.back() + 1., 1.).y;
                    let lo = self.lower_occlusal(self.lower.back() + 1., 1.).y;
                    l.tissue[T_CHEEK[0]].extend_from_slice(&[v(back, hi), v(15. * k, hi), v(15. * k, 15. * k), v(back, 15. * k)]);
                    l.tissue[T_CHEEK[1]].extend_from_slice(&[v(back, -5. * k), v(15. * k, -5. * k), v(15. * k, lo), v(back, lo)]);
                } else {
                    l.tissue[T_CHEEK[0]].extend_from_slice(&[v(back, -5. * k), v(15. * k, -5. * k), v(15. * k, 15. * k), v(back, 15. * k)]);
                }
            }
        }
        // ---- between the tooth rows the cheek bulges in when the jaw is down (the muscle in it
        // keeps it against the teeth): up to half the rows' thickness at the middle of the gap,
        // nothing where the rows meet. The air between the rows ends in a rounded wedge: against
        // the rows' buccal faces, or half a centimetre inside the mouth's corner where that
        // comes first (the corner's own tissue lies between the rows there; without it the gap
        // ended in a straight edge at the corner).
        {
            let corner = self.lip_w + 0.25 * k;
            if s.buccal_gap + 0.6 * s.cheek_puff == 0. && at <= self.upper.outer_max() && az > self.lip_w - 0.6 * k {
                let (x_in, x_front) = (self.upper.x_in(at), self.upper.x_out(at).max(self.upper.x_in(at)));
                let t = &mut l.tissue[T_CHEEK[1]];
                // (x, the lower row's biting surface, how much of the gap above it is taken up)
                let section = |xt: f64| -> (f64, f64, f64) {
                    let (hi, lo) = (self.upper_occlusal(xt, -CUSP), self.lower_occlusal(xt, 0.));
                    let half = 0.5 * (hi.y - lo.y);
                    let reach = (0.45 * k).min(1.4 * half);
                    let inside = ((self.upper.outer_at(xt) * k).min(corner) - az).max(0.);
                    // (a blunt nose: a pointed one left a thin horn of air beside it)
                    let bulge = if half > 0. && inside < reach { 1. - pow(inside / reach, 2.5) } else { 0. };
                    // rows under 4 mm apart hold no air: the cheek and the tongue's
                    // margin lie in the gap (it cut as a hair-thin wing on each side)
                    let thin = 1. - smoothstep(0.2 * k, 0.4 * k, half);
                    let fill = if half > 0. { 2. * half * lerp(bulge, 1.05, thin) } else { 0. };
                    (self.map.p(xt, 8.).x, lo.y, fill)
                };
                // the bulge stands on the lower row (the cheek and the tongue's margin rise from
                // it), so the air above it thins out in one piece: standing in the middle of
                // the gap it split the air in two
                const N: usize = 8;
                for i in 0..=N {
                    let (px, low, fill) = section(lerp(x_in, x_front, i as f64 / N as f64));
                    t.push(v(px, low + fill));
                }
                let (px, low, fill) = section(x_front);
                t.push(v(px + 0.2 * k, low + fill));
                t.push(v(px + 0.2 * k, low - 0.15 * k));
                for i in (0..=N).rev() {
                    let (px, low, _) = section(lerp(x_in, x_front, i as f64 / N as f64));
                    t.push(v(px, low - 0.15 * k));
                }
            }
        }
        // ---- the uvula: a lobe of round section, from inside the soft palate down its axis
        if let Some(outline) = self.uvula_outline(z) {
            l.tissue[T_UVULA].extend_from_slice(&outline);
        }
        // ---- the pharyngeal ring: everything behind and beside the pharynx at this offset
        {
            let t = &mut l.tissue[T_PHARYNX[0]];
            let n = self.ring_y.len();
            t.push(v(-6. * k, self.ring_y[0]));
            for i in 0..n {
                t.push(self.ring_at(i, z));
            }
            t.push(v(-6. * k, self.ring_y[n - 1]));
        }
        // ---- aryepiglottic folds: a wall on each side of the laryngeal inlet, standing on the
        // hypopharynx's floor between the inlet and the fossa beside it
        if self.fold_top(z, 0.).is_some() {
            let back = self.wall_x(self.bottom) - 0.5 * k;
            let front = self.epiglottis_at(0., z).x + 0.5 * k;
            let t = &mut l.tissue[T_ARY_FOLD];
            t.push(v(back, self.bottom - 0.3 * k));
            for i in 0..=8 {
                let u = i as f64 / 8.;
                t.push(v(lerp(back, front, u), self.fold_top(z, u).unwrap_or(self.bottom)));
            }
            t.push(v(front, self.bottom - 0.3 * k));
        }
        // ---- laryngeal tube: horizontal elliptical sections (`tube`); where the lamina lies
        // beyond a section's half-width the polygon has no width there
        {
            let t = &mut l.air[AIR_TUBE];
            if self.tube.iter().any(|q| az < q.2) {
                let chord = |j: usize| -> (f64, f64, f64) {
                    let (c, a, b) = self.tube[j];
                    let y = self.glottis.y + self.epi_len * j as f64 / TUBE_SAMPLES as f64;
                    (c, if az < b { a * sqrt(1. - (z / b) * (z / b)) } else { 0. }, y)
                };
                for j in 0..=TUBE_SAMPLES {
                    let (c, h, y) = chord(j);
                    t.push(v(c - h, y));
                }
                for j in (0..=TUBE_SAMPLES).rev() {
                    let (c, h, y) = chord(j);
                    t.push(v(c + h, y));
                }
            }
        }
        // ---- piriform fossae: pouches beside and behind the laryngeal tube, opening upwards
        // into the hypopharynx (`piriform_section`); valleculae between the tongue's root and
        // the epiglottis' leaf, beside the median fold
        for (j, side) in [-1., 1.].iter().enumerate() {
            let (cx, cz, rx, rz) = (self.pir.0, side * self.pir.1, self.pir.2, s.piriform_width * k);
            let zz = (z - cz) / rz;
            // (a speaker without fossae to speak of: under 2 mm deep they are left out)
            if zz.abs() < 1. && s.piriform_depth >= 0.2 {
                let r = sqrt(1. - zz * zz);
                let (top, height) = self.piriform_top();
                let out = &mut l.air[AIR_PIRIFORM[j]];
                // the lower half of the ellipsoid's section at this offset, its flat top last
                for i in 0..=32 {
                    let th = PI * (1. + i as f64 / 32.);
                    out.push(v(cx + rx * r * cos(th), top + height * r * sin(th)));
                }
            }
            // (the midline's lamina holds the right side's pocket over the median fold)
            if z * side > 0. || (z == 0. && *side > 0.) {
                if let Some((floor, top)) = self.vallecula_span(z) {
                    let out = &mut l.air[AIR_VALLECULA[j]];
                    let lid = self.dorsal_at(0, z).y;
                    // up the leaf's lingual surface from the floor to its tip, over the lid into
                    // the air above it (air overrides tissue: a sliver of the tongue's polygon
                    // lies along the lid), then down the root to the floor
                    const N: usize = 8;
                    for i in 0..=N {
                        let y = lerp(floor, top, i as f64 / N as f64);
                        out.push(v(self.epiglottis_front_x(y, z), y));
                    }
                    out.push(v(self.epiglottis_front_x(top, z), lid.max(top) + 0.1 * k));
                    out.push(v(self.root_x(lid, z), lid.max(top) + 0.1 * k));
                    for i in 0..=N {
                        let y = lerp(lid, floor, i as f64 / N as f64);
                        // the root never stands behind the leaf: where they would cross, the
                        // pocket has no width
                        out.push(v(self.root_x(y, z).max(self.epiglottis_front_x(y.min(top), z)), y));
                    }
                }
            }
        }
        l.bound();
    }
    /// a vallecula's lateral centre and radius (cm): from the midline outwards,
    /// `valleculaWidth` the radius, ending under the leaf's edge at the latest
    pub fn vallecula(&self, side: f64) -> (f64, f64) {
        let k = self.map.k;
        let outer = (2. * self.shape.vallecula_width * k).min(0.97 * self.shape.epiglottis_width * k).max(0.1 * k);
        (side * 0.5 * outer, 0.5 * outer)
    }
    /// a piriform fossa's centre (x, lateral) and its radii at the opening (front to back,
    /// lateral), cm
    pub fn piriform(&self, side: f64) -> (f64, f64, f64, f64) {
        (self.pir.0, side * self.pir.1, self.pir.2, self.shape.piriform_width * self.map.k)
    }
}

#[inline]
pub fn dot(a: Vec2, b: Vec2) -> f64 {
    a.x * b.x + a.y * b.y
}
/// Where the ray from `o` along the unit vector `d` first meets the polyline, within `reach`:
/// the distance, the segment's index and the share of the segment before the hit.
fn ray_hit(o: Vec2, d: Vec2, line: &[Vec2], reach: f64) -> Option<(f64, usize, f64)> {
    let mut best: Option<(f64, usize, f64)> = None;
    for i in 0..line.len().saturating_sub(1) {
        let (a, e) = (sub(line[i], o), sub(line[i + 1], line[i]));
        let den = d.x * e.y - d.y * e.x;
        if den.abs() < 1e-12 {
            continue;
        }
        let t = (a.x * e.y - a.y * e.x) / den;
        let u = (a.x * d.y - a.y * d.x) / den;
        if t > 1e-6 && t <= reach && (0. ..=1.).contains(&u) && best.is_none_or(|b| t < b.0) {
            best = Some((t, i, u));
        }
    }
    best
}
/// crown height above the gum (template cm) at template x of an arch: incisors 1.0, molars 0.75
pub fn crown_height(arch: &Arch, xt: f64) -> f64 {
    lerp(1.0, 0.75, smoothstep(0., 0.5, (arch.incisor_x - xt) / arch.depth))
}
/// upper occlusal plane height at template x (template cm)
#[inline]
pub fn occlusal(xt: f64) -> f64 {
    OCCLUSAL_Y + SPEE * (UPPER_INCISOR_X - xt).max(0.)
}
