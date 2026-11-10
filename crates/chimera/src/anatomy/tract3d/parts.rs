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
//!   tongue (a tubular neighbourhood: fibres do not cross);
//! - the tip lobe shortens by an affine compression towards its base line (a homeomorphism);
//! - teeth, lips, cheeks, faucial pillars and the pharyngeal ring are separate tissue polygons,
//!   united by counting, so they never need to meet another outline exactly;
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
/// lateral offsets (template cm) over which the tooth rows rise out of the roof/floor: the
/// incisors at the midline are the engine's own outlines, the rows join them from the side
const BAND_FADE: (f64, f64) = (0.25, 0.45);
/// template x behind which the oral cavity ends (faucial pillars) and the pharynx begins
const FAUCES_X: f64 = 1.0;
/// template x of the back of the upper dental arch
const ARCH_DEPTH_BACK: f64 = UPPER_INCISOR_X - ARCH_DEPTH;
/// template height below which the faucial pillars have run into the tongue's sides
const FAUCES_Y: f64 = 6.2;
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
    /// crown thickness across the arch (template cm): incisors thin, molars thick
    pub fn tooth(u: f64) -> f64 {
        lerp(0.7, 1.0, smoothstep(0., 0.5, u))
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
}

pub struct Parts {
    pub map: Map,
    pub body: HumanBody,
    pub shape: ShapeParams,
    pub artic: HumanArtic,
    /// posterior pharyngeal wall, from below `bottom` up to the port window
    pub wall: Vec<Vec2>,
    /// the pharyngeal ring's sample heights and the midline wall's x at each
    pub ring_y: Vec<f64>,
    pub ring_x: Vec<f64>,
    /// x of the faucial pillars' back face
    pub fauces_x: f64,
    /// x the pharynx's side walls reach at its half-width, at the mouth's height (the back of
    /// the dental arches) and low in the throat (just in front of the epiglottis' foot)
    pub ring_front: f64,
    pub ring_front_low: f64,
    /// velum oral surface, uvula tip → root, and each point's arc-length fraction from the tip
    pub velum: Vec<Vec2>,
    pub velum_f: Vec<f64>,
    pub velum_len: f64,
    pub velum_top: f64,
    pub velum_bottom: f64,
    /// hard palate, behind the velum root → upper incisor edge
    pub palate: Vec<Vec2>,
    pub palate_drop: Vec<f64>,
    pub palate_xt: Vec<f64>,
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
            ring_y: Vec::with_capacity(RING_SAMPLES + 1),
            ring_x: Vec::with_capacity(RING_SAMPLES + 1),
            fauces_x: 1.,
            ring_front: 2.,
            ring_front_low: 1.,
            velum: Vec::with_capacity(16),
            velum_f: Vec::with_capacity(16),
            velum_len: 1.,
            velum_top: 0.,
            velum_bottom: 0.,
            palate: Vec::with_capacity(40),
            palate_drop: Vec::with_capacity(40),
            palate_xt: Vec::with_capacity(40),
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
        self.velum.clear();
        self.velum.extend_from_slice(&f.outer[no - 51..no - 40]);
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
        self.fauces_x = m.p(FAUCES_X, 8.).x;
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
        for q in &self.palate {
            let xt = m.un_p(*q).x;
            let crown = lerp(1.0, 0.75, smoothstep(0., 0.5, (UPPER_INCISOR_X - xt) / ARCH_DEPTH));
            let gum = m.p(xt, occlusal(xt) + crown).y;
            self.palate_drop.push((q.y - gum).clamp(0., s.vault_depth * k));
            self.palate_xt.push(xt);
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
        self.lobe_p = self.dorsal[self.lobe_start];
        let fren = *self.under.last().unwrap();
        let tip = self.dorsal[n - 1];
        let mut a_dir = norm(rot(sub(fren, self.lobe_p)));
        if dot(a_dir, sub(tip, self.lobe_p)) < 0. {
            a_dir = scale(a_dir, -1.);
        }
        self.lobe_a = a_dir;
        self.lobe_u = dot(sub(tip, self.lobe_p), a_dir).max(1e-3);

        // the piriform fossae: beside the laryngeal tube's rim, beyond the aryepiglottic fold,
        // each between the ring behind it and the front wall of the hypopharynx at its own
        // lateral offset (the ring at rest: a squeeze of the pharynx does not move them)
        let zc = self.fold_span().1 + 0.03 * k + s.piriform_width * k;
        let back = self.ring_x_at(self.bottom, zc, false) + 0.04 * k;
        // (its front runs 1 mm under the front wall's foot)
        let front = self.epiglottis_at(0., zc).x + 0.1 * k;
        self.pir = (0.5 * (front + back), zc, (0.5 * (front - back)).max(0.15 * k));
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

    /// The posterior wall's midline x at height y (the wall is clipped at the port window: above
    /// it, and below its first point, the nearest point's x).
    pub fn wall_x(&self, y: f64) -> f64 {
        let n = self.wall.len();
        if y <= self.wall[0].y {
            return self.wall[0].x;
        }
        for i in 1..n {
            let (a, b) = (self.wall[i - 1], self.wall[i]);
            if y <= b.y {
                return lerp(a.x, b.x, ((y - a.y) / (b.y - a.y).max(1e-9)).clamp(0., 1.));
            }
        }
        self.wall[n - 1].x
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
        ) * (if squeezed { 1. - 0.7 * s.lateral_squeeze } else { 1. })
            * self.map.k
    }
    /// the ring's x at height y and lateral offset z, from the midline wall's x there
    fn ring_x_at(&self, y: f64, z: f64, squeezed: bool) -> f64 {
        let x0 = self.wall_x(y);
        let w = (z.abs() / self.ring_width(y, squeezed)).min(1.);
        // (the hypopharynx lies flat on the spine between the thyroid's plates: squarer)
        let square = 1.2 * (1. - smoothstep(2., 6., self.map.un_p(v(0., y)).y));
        lerp(x0, self.ring_reach(y).max(x0), 1. - superellipse(w, self.shape.pharynx_shape + square))
    }
    /// How far forward the side walls come at height y: to the back teeth beside the mouth, to
    /// the thyroid's plates in front of the epiglottis' foot low in the throat.
    fn ring_reach(&self, y: f64) -> f64 {
        lerp(self.ring_front_low, self.ring_front, smoothstep(3., 6.5, self.map.un_p(v(0., y)).y))
    }
    /// faucial isthmus half-width (cm): the pillars stand just inside the velum's edges
    pub fn fauces_width(&self) -> f64 {
        0.45 * self.shape.velum_width * self.map.k
    }

    /// Tongue dorsal point `i` at lateral offset z, before the tip-lobe compression.
    pub fn dorsal_at(&self, i: usize, z: f64) -> Vec2 {
        let s = &self.shape;
        let q = self.dorsal[i];
        let n = self.dn[i];
        let th = self.dth[i];
        let w = z.abs() / self.dw[i];
        // what faces the pharynx is convex across its whole width (the root and the back of the
        // tongue bulge into it, and the grooves beside them run forward); what faces the palate
        // is flat across the middle and rounds over its outer 30 % to the core
        let flat = 0.7 * smoothstep(0.12, 0.35, self.dt[i]) * smoothstep(-0.9, -0.2, n.x);
        let e = if w <= flat { 1. } else { superellipse((w - flat) / (1. - flat), s.edge_round) };
        // raising (bracing, cupping) spreads from a quarter of the width; lowering (lateral
        // channels) stays at the margins, leaving a midline contact for /l/
        let rise = if self.dside[i] >= 0. { smoothstep(0.25, 0.8, w) } else { smoothstep(0.55, 0.95, w) };
        let side = rise * (1. - smoothstep(0.92, 1., w)) * (1. + s.side_asym * z.signum());
        let gw = self.dgw[i];
        let g = if z.abs() < gw { 0.5 + 0.5 * cos(PI * z / gw) } else { 0. };
        let lam = (e + (self.dside[i] * side - self.dgroove[i] * g) / th).clamp(0., 3.);
        add(q, scale(n, th * (lam - 1.)))
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
        v(q.x, q.y - self.palate_drop[i] * pow(w, self.shape.vault_shape))
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
    /// the velum's point i at lateral offset z: off the midline the free margin recedes towards
    /// the root (the faucial arches), so a lamina meets the velum only from where it is that wide
    pub fn velum_at(&self, i: usize, z: f64) -> Vec2 {
        let az = z.abs();
        let n = self.velum.len();
        let lift = |j: usize| {
            let u = self.uvula_fraction();
            let q = self.velum[j];
            // a longer or shorter uvula moves its tip down or up
            v(q.x, q.y - (self.shape.uvula_length - 1.) * (1. - self.velum_f[j] / u).max(0.) * 0.8 * self.map.k)
        };
        if self.velum_width_at(self.velum_f[i]) >= az {
            return lift(i);
        }
        // the first point from the tip that is wide enough, interpolated
        for j in i + 1..n {
            if self.velum_width_at(self.velum_f[j]) >= az {
                let (w0, w1) = (self.velum_width_at(self.velum_f[j - 1]), self.velum_width_at(self.velum_f[j]));
                return lerp_v(lift(j - 1), lift(j), ((az - w0) / (w1 - w0).max(1e-9)).clamp(0., 1.));
            }
        }
        self.velum[n - 1]
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
        // (the hypopharynx lies flat on the spine between the thyroid's plates: squarer)
        let square = 1.2 * (1. - smoothstep(2., 6., self.map.un_p(v(0., y)).y));
        v(lerp(x0, self.ring_reach(y).max(x0), 1. - superellipse(w, self.shape.pharynx_shape + square)), y)
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
    /// width, its sides curled backwards (`epiglottisCurl`); beyond its edges the folds that
    /// run from it to the side walls come back to the midline's plane.
    pub fn epiglottis_at(&self, f: f64, z: f64) -> Vec2 {
        let k = self.map.k;
        let we = self.shape.epiglottis_width * k;
        let az = z.abs();
        let q = if az <= we { az * az } else { we * we * (1. - smoothstep(we, we + 0.35 * k, az)) };
        // (weights that give the engine's own two points exactly at f = 0 and 1)
        let (a, b) = (self.epi[0], self.epi[1]);
        v(a.x * (1. - f) + b.x * f - self.shape.epiglottis_curl * q / k, a.y * (1. - f) + b.y * f)
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
        if !(0. ..1.).contains(&u) {
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
            let base = self.epiglottis_at(0., z);
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
        // ---- tooth rows beside the incisors
        if at > BAND_FADE.0 {
            let fade = 3. * (1. - smoothstep(BAND_FADE.0, BAND_FADE.1, at));
            for (arch, slot, upper) in [(&self.upper, T_UPPER_TEETH, true), (&self.lower, T_LOWER_TEETH, false)] {
                if at > arch.outer_max() {
                    continue;
                }
                let x0 = arch.x_in(at);
                let x1 = arch.x_out(at).max(x0);
                let t = &mut l.tissue[slot];
                for j in 0..=6 {
                    let xt = lerp(x0, x1, j as f64 / 6.);
                    // upper cusps overlap the lower ones by ~1 mm: a tooth row seals until the jaw opens
                    t.push(if upper { self.upper_occlusal(xt, fade - CUSP) } else { self.lower_occlusal(xt, fade) });
                }
                let (a, b) = (t[0], t[6]);
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
                    let x = self.map.p(self.upper.x_out(at) + gap, 8.).x + slide;
                    l.tissue[T_CHEEK[0]].extend_from_slice(&[v(x, -5. * k), v(15. * k, -5. * k), v(15. * k, 15. * k), v(x, 15. * k)]);
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
        // ---- faucial pillars between the pharynx and the molars, from where they leave the
        // tongue's sides up: each a block whose face towards the midline is rounded (half an
        // ellipse 5 mm deep) and whose foot is rounded too, so the isthmus narrows without a step
        {
            let fw = self.fauces_width();
            let nose = 0.25 * k;
            if az > fw - nose {
                let x0 = self.fauces_x;
                let x1 = self.ring_front;
                let u = ((fw + nose - az) / (2. * nose)).clamp(0., 1.);
                let half = 0.5 * (x1 - x0) * sqrt(1. - u * u);
                let mid = 0.5 * (x0 + x1);
                let foot = self.map.p(0., FAUCES_Y).y;
                let t = &mut l.tissue[T_FAUCES];
                for i in 0..=8 {
                    let th = PI * (1. + i as f64 / 8.);
                    t.push(v(mid + half * cos(th), foot + 0.6 * k * (1. + sin(th))));
                }
                t.push(v(mid + half, 15. * k));
                t.push(v(mid - half, 15. * k));
            }
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
            if zz.abs() < 1. {
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
/// upper occlusal plane height at template x (template cm)
#[inline]
pub fn occlusal(xt: f64) -> f64 {
    OCCLUSAL_Y + SPEE * (UPPER_INCISOR_X - xt).max(0.)
}
