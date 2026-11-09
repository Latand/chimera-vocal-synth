//! Per-frame anatomy: the engine's midline (`HumanFrame`) split into its parts, plus the lateral
//! rule of every element. The laminae, the display meshes and the side-cavity descriptors all
//! read these rules, so what the lab draws is what the cuts measure.
//!
//! Every lateral rule moves points in a way that cannot fold an outline over itself (the
//! earlier version offset curved outlines by up to 2 cm along their normals, which folded the
//! palate and tongue into self-intersecting polygons and left phantom air strips beside them):
//! - the posterior pharyngeal wall moves only horizontally, the palate and lips only vertically
//!   (both outlines are graphs, so they stay graphs);
//! - the tongue's dorsal surface moves along fibres between itself and a core curve inside the
//!   tongue (a tubular neighbourhood: fibres do not cross);
//! - the tip lobe shortens by an affine compression towards its base line (a homeomorphism);
//! - teeth, lips, cheeks, faucial pillars and lateral pharyngeal walls are separate tissue
//!   polygons, united by counting, so they never need to meet another outline exactly.
use super::lamina::*;
use super::*;
use crate::math::atan2;

/// Samples of each dental arch's outer edge.
pub const ARCH_SAMPLES: usize = 33;
pub const TUBE_SAMPLES: usize = 24;
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
/// vertical overlap of upper and lower cusps (template cm)
const CUSP: f64 = 0.12;

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
    pub wall_gap: Vec<f64>,
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
    /// the engine's epilarynx area profile (cm²), glottis → tube top
    pub tube: [f64; TUBE_SAMPLES + 1],
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
            wall: Vec::with_capacity(32),
            wall_gap: Vec::with_capacity(32),
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
            tube: [0.; TUBE_SAMPLES + 1],
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
        for (j, t) in self.tube.iter_mut().enumerate() {
            *t = tract.epilarynx_area(a, self.epi_len * j as f64 / TUBE_SAMPLES as f64, self.epi_len);
        }
        self.bottom = f.glottis.y + 2.2 * k * self.body.pharynx;

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

        // posterior wall: the depth of its lateral rounding is the engine's own pharyngeal gap
        // at that height (its grid samples behind the velum), interpolated by height. A ray of
        // our own could pass over the top of the tongue and jump to whatever lies behind it.
        self.wall_gap.clear();
        let samples = || f.grid.iter().filter(|g| g.region == 1);
        let (mut lo, mut hi) = (None::<(f64, f64)>, None::<(f64, f64)>);
        for g in samples() {
            if lo.is_none_or(|l| g.outer.y < l.0) {
                lo = Some((g.outer.y, g.d));
            }
            if hi.is_none_or(|h| g.outer.y > h.0) {
                hi = Some((g.outer.y, g.d));
            }
        }
        for q in &self.wall {
            let mut below: Option<(f64, f64)> = None;
            let mut above: Option<(f64, f64)> = None;
            for g in samples() {
                let (y, d) = (g.outer.y, g.d);
                if y <= q.y && below.is_none_or(|b| y > b.0) {
                    below = Some((y, d));
                }
                if y >= q.y && above.is_none_or(|a| y < a.0) {
                    above = Some((y, d));
                }
            }
            let gap = match (below, above) {
                (Some(b), Some(a)) if a.0 > b.0 => lerp(b.1, a.1, (q.y - b.0) / (a.0 - b.0)),
                (Some(b), _) => b.1,
                (_, Some(a)) => a.1,
                _ => lo.or(hi).map_or(2. * k, |x| x.1),
            };
            self.wall_gap.push(gap.clamp(0., 4. * k));
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
        let ph_w = s.pharynx_width_oro.max(s.pharynx_width_hypo);
        let widths = [
            ph_w + 0.3,
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
    }

    pub fn jaw(&self, x: f64, y: f64) -> Vec2 {
        rotate_about(self.map.p(x, y), self.condyle, self.jaw_phi)
    }
    /// half-width (cm) of the pharynx at body height y
    pub fn pharynx_width(&self, y: f64) -> f64 {
        let s = &self.shape;
        let yt = self.map.un_p(v(0., y)).y;
        lerp(
            lerp(s.pharynx_width_hypo, s.pharynx_width_oro, smoothstep(2., 6., yt)),
            s.pharynx_width_naso,
            smoothstep(6., 9., yt),
        ) * (1. - 0.7 * s.lateral_squeeze)
            * self.map.k
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
        // the root is convex across its whole width (it bulges into the pharynx); from the back
        // forward the top is flat across the middle and rounds over its outer 30 % to the core
        let flat = 0.7 * smoothstep(0.12, 0.35, self.dt[i]);
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
    pub fn wall_at(&self, i: usize, z: f64) -> Vec2 {
        let q = self.wall[i];
        let w = (z.abs() / self.pharynx_width(q.y)).min(1.);
        v(q.x + self.wall_gap[i] * (1. - superellipse(w, self.shape.pharynx_shape)), q.y)
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
                let q = self.wall_at(i, z);
                if q.y < self.bottom {
                    continue;
                }
                if !started {
                    started = true;
                    if i > 0 {
                        let p0 = self.wall_at(i - 1, z);
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
            t.push(self.epi[0]);
            t.push(self.epi[1]);
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
            t.push(v(self.epi[0].x, -5. * k));
        }
        // ---- epiglottis plate
        if az < s.epiglottis_width * k {
            let shift = v(s.epiglottis_curl * z * z / k, 0.);
            let base = add(self.epi[0], shift);
            let tip = add(self.epi[1], shift);
            let d = norm(sub(tip, base));
            let th = scale(v(d.y, -d.x), 0.25 * k);
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
        // ---- faucial pillars between the pharynx and the molars
        if az > self.fauces_width() {
            let x0 = self.map.p(FAUCES_X, 8.).x;
            let x1 = self.map.p(self.upper.back(), 8.).x + 0.3 * k;
            l.tissue[T_FAUCES].extend_from_slice(&[v(x0, -5. * k), v(x1, -5. * k), v(x1, 15. * k), v(x0, 15. * k)]);
        }
        // ---- lateral pharyngeal walls: wherever this lamina lies outside the pharynx
        {
            let x0 = -6. * k;
            let x1 = self.map.p(FAUCES_X, 8.).x;
            let (y_lo, y_hi) = (self.glottis.y - 1. * k, 13. * k);
            let n = 64;
            let mut run = 0;
            let mut start = f64::NAN;
            let mut prev = (y_lo, self.pharynx_width(y_lo) - az);
            if prev.1 < 0. {
                start = y_lo;
            }
            for i in 1..=n {
                let y = lerp(y_lo, y_hi, i as f64 / n as f64);
                let d = self.pharynx_width(y) - az;
                let cross = lerp(prev.0, y, prev.1 / (prev.1 - d));
                if prev.1 >= 0. && d < 0. {
                    start = cross;
                } else if prev.1 < 0. && d >= 0. && run < 2 {
                    l.tissue[T_PHARYNX[run]].extend_from_slice(&[v(x0, start), v(x1, start), v(x1, cross), v(x0, cross)]);
                    run += 1;
                    start = f64::NAN;
                }
                prev = (y, d);
            }
            if !start.is_nan() && run < 2 {
                l.tissue[T_PHARYNX[run]].extend_from_slice(&[v(x0, start), v(x1, start), v(x1, y_hi), v(x0, y_hi)]);
            }
        }
        // ---- laryngeal tube: horizontal elliptical sections reproducing the engine's profile
        let ew = s.epilarynx_width * k;
        if az < ew {
            let chord = sqrt((1. - (z / ew) * (z / ew)).max(0.));
            let t = &mut l.air[AIR_TUBE];
            for j in 0..=TUBE_SAMPLES {
                let y = self.epi_len * j as f64 / TUBE_SAMPLES as f64;
                t.push(add(self.glottis, v(-self.tube[j] / (PI * ew) * chord, y)));
            }
            for j in (0..=TUBE_SAMPLES).rev() {
                let y = self.epi_len * j as f64 / TUBE_SAMPLES as f64;
                t.push(add(self.glottis, v(self.tube[j] / (PI * ew) * chord, y)));
            }
        }
        // ---- piriform fossae: blind pockets beside the laryngeal tube, opening upwards into
        // the hypopharynx; valleculae beside the median fold, off the midline only
        for (j, side) in [-1., 1.].iter().enumerate() {
            let (c, rz) = self.piriform(*side);
            let zz = (z - c) / rz;
            let rad = (1. - zz * zz).max(0.);
            if rad > 0. {
                let (centre, rx, ry) = self.piriform_ellipse();
                ellipse(&mut l.air[AIR_PIRIFORM[j]], centre, rx * sqrt(rad), ry * sqrt(rad));
            }
            let vw = s.vallecula_width * k;
            let zz = (az - 0.15 * k - vw) / vw;
            if z * side > 0. && zz.abs() < 1. {
                let r = sqrt(1. - zz * zz);
                ellipse(
                    &mut l.air[AIR_VALLECULA[j]],
                    add(self.vallecula, v(-0.2 * k, 0.3 * k)),
                    0.3 * k * r,
                    0.5 * s.vallecula_depth * k * r,
                );
            }
        }
        l.bound();
    }
    /// the fossa's sagittal section at its widest: centre, AP and vertical radii (cm); its top
    /// rises 1 mm into the hypopharynx, its depth below the hypopharynx floor is `piriformDepth`
    pub fn piriform_ellipse(&self) -> (Vec2, f64, f64) {
        let k = self.map.k;
        let ry = 0.5 * self.shape.piriform_depth * k;
        (v(self.glottis.x - 0.2 * k, self.bottom + 0.1 * k - ry), 1.1 * k, ry + 0.05 * k)
    }
    /// piriform fossa centre (lateral, cm) and lateral radius
    pub fn piriform(&self, side: f64) -> (f64, f64) {
        let s = &self.shape;
        let k = self.map.k;
        (side * (s.epilarynx_width + s.piriform_width * 0.9) * k, s.piriform_width * k)
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
