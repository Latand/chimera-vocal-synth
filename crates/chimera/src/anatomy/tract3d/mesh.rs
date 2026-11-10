//! Display surfaces, sampled from the same lateral rules the laminae use (`parts.rs`), so the
//! views show the geometry the cuts measure. Each element has a vertex count fixed by the body,
//! so the browser updates positions in place. `indices` are triangles in `groups` by material;
//! `lines` are segment pairs (a sparse grid for the outline style).
//!
//! What is display only, beyond the cut geometry: the single crowns of the teeth (the laminae
//! hold each arch as one row between the same lingual and buccal edges, from the same occlusal
//! surface), the gums over the roots, and the floor of the mouth under the tongue's margins.
use super::parts::{self, Arch, Parts, CUSP};
use super::*;

#[derive(Clone, Debug, Default, Serialize)]
pub struct MeshGroup {
    pub material: &'static str,
    /// first index and index count in `indices`
    pub start: u32,
    pub count: u32,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ElementMesh {
    pub element: &'static str,
    pub positions: Vec<f32>,
    pub indices: Vec<u32>,
    pub lines: Vec<u32>,
    pub groups: Vec<MeshGroup>,
}
impl ElementMesh {
    fn new(element: &'static str) -> Self {
        Self { element, ..Default::default() }
    }
    /// Triangles added from here on belong to `material`.
    fn material(&mut self, material: &'static str) {
        self.close();
        self.groups.push(MeshGroup { material, start: self.indices.len() as u32, count: 0 });
    }
    fn close(&mut self) {
        let n = self.indices.len() as u32;
        if let Some(g) = self.groups.last_mut() {
            g.count = n - g.start;
        }
    }
    fn point(&mut self, p: [f64; 3]) -> u32 {
        self.positions.extend_from_slice(&[p[0] as f32, p[1] as f32, p[2] as f32]);
        (self.positions.len() / 3 - 1) as u32
    }
    /// A rows × cols grid of points; lines along every `rs`-th row and every `cs`-th column
    /// (0: none), and the border when either is set.
    fn grid(&mut self, rows: usize, cols: usize, rs: usize, cs: usize, mut f: impl FnMut(usize, usize) -> [f64; 3]) {
        let base = (self.positions.len() / 3) as u32;
        for r in 0..rows {
            for c in 0..cols {
                let p = f(r, c);
                self.positions.extend_from_slice(&[p[0] as f32, p[1] as f32, p[2] as f32]);
            }
        }
        let id = |r: usize, c: usize| base + (r * cols + c) as u32;
        for r in 0..rows - 1 {
            for c in 0..cols - 1 {
                self.indices.extend_from_slice(&[id(r, c), id(r + 1, c), id(r, c + 1), id(r, c + 1), id(r + 1, c), id(r + 1, c + 1)]);
            }
        }
        if rs > 0 {
            for r in 0..rows {
                if r % rs == 0 || r + 1 == rows {
                    for c in 0..cols - 1 {
                        self.lines.extend_from_slice(&[id(r, c), id(r, c + 1)]);
                    }
                }
            }
        }
        if cs > 0 {
            for c in 0..cols {
                if c % cs == 0 || c + 1 == cols {
                    for r in 0..rows - 1 {
                        self.lines.extend_from_slice(&[id(r, c), id(r + 1, c)]);
                    }
                }
            }
        }
    }
    /// A closed tube of `rings` rings of `segs` points, capped at its last ring by a fan to
    /// `tip`; lines round the first and last ring.
    fn capped(&mut self, rings: usize, segs: usize, mut f: impl FnMut(usize, usize) -> [f64; 3], tip: [f64; 3]) {
        let base = (self.positions.len() / 3) as u32;
        for r in 0..rings {
            for s in 0..segs {
                self.point(f(r, s));
            }
        }
        let t = self.point(tip);
        let id = |r: usize, s: usize| base + (r * segs + s % segs) as u32;
        for r in 0..rings - 1 {
            for s in 0..segs {
                self.indices.extend_from_slice(&[id(r, s), id(r + 1, s), id(r, s + 1), id(r, s + 1), id(r + 1, s), id(r + 1, s + 1)]);
            }
        }
        for s in 0..segs {
            self.indices.extend_from_slice(&[id(rings - 1, s), t, id(rings - 1, s + 1)]);
            self.lines.extend_from_slice(&[id(0, s), id(0, s + 1), id(rings - 1, s), id(rings - 1, s + 1)]);
        }
    }
}

fn p3(q: Vec2, z: f64) -> [f64; 3] {
    [q.x, q.y, z]
}
/// point along a polyline by parameter t ∈ [0, 1] over its vertex index
fn along(poly: &[Vec2], t: f64) -> (usize, f64) {
    let x = t.clamp(0., 1.) * (poly.len() - 1) as f64;
    let i = (x as usize).min(poly.len() - 2);
    (i, x - i as f64)
}

/// Crown widths along each arch from the midline back, template cm: central and lateral incisor,
/// canine, two premolars, two molars (typical adult crowns; scaled to fill the arch).
const UPPER_CROWNS: [f64; 7] = [0.86, 0.66, 0.76, 0.70, 0.66, 1.00, 0.95];
const LOWER_CROWNS: [f64; 7] = [0.54, 0.59, 0.69, 0.70, 0.71, 1.10, 1.03];
pub const TEETH_PER_SIDE: usize = 7;
/// The pharyngeal ring's grid: rows from the laryngeal inlet to the nasopharynx's roof, columns across.
pub const RING_ROWS: usize = 30;
pub const RING_COLS: usize = 21;
/// How much of the soft tube's half-width the tongue's own body is drawn over.
const TONGUE_IN_TUBE: f64 = 0.72;
const TOOTH_RINGS: usize = 7;
const TOOTH_SEGS: usize = 14;
const ARC_SAMPLES: usize = 96;

/// A point of an arch's tooth row: `u` the depth fraction from the incisors, `b` across the row
/// (0 lingual edge … 1 buccal edge; outside that range it continues along the normal, in
/// template cm per unit tooth thickness). Returns template (x, lateral).
fn arch_point(arch: &Arch, u: f64, b: f64) -> (f64, f64) {
    let ((x, a), (nx, na)) = arch.frame(u);
    let t = Arch::tooth(u) * b;
    (x + t * nx, a + t * na)
}

/// The depth fractions where each tooth of one side begins and ends: the crowns' widths laid
/// along the row's middle, scaled to its length.
fn tooth_spans(arch: &Arch, widths: &[f64; TEETH_PER_SIDE]) -> [(f64, f64); TEETH_PER_SIDE] {
    // length along the row's middle, from the midline back
    let mut cum = [0.; ARC_SAMPLES + 1];
    let mut prev = (arch_point(arch, 0., 0.5).0, 0.);
    for i in 1..=ARC_SAMPLES {
        let q = arch_point(arch, i as f64 / ARC_SAMPLES as f64, 0.5);
        cum[i] = cum[i - 1] + sqrt((q.0 - prev.0) * (q.0 - prev.0) + (q.1 - prev.1) * (q.1 - prev.1));
        prev = q;
    }
    let total = cum[ARC_SAMPLES];
    let sum: f64 = widths.iter().sum();
    let at = |len: f64| -> f64 {
        let i = (1..=ARC_SAMPLES).find(|&i| cum[i] >= len).unwrap_or(ARC_SAMPLES);
        let f = ((len - cum[i - 1]) / (cum[i] - cum[i - 1]).max(1e-12)).clamp(0., 1.);
        (i as f64 - 1. + f) / ARC_SAMPLES as f64
    };
    let mut out = [(0., 0.); TEETH_PER_SIDE];
    let mut len = 0.;
    for (j, w) in widths.iter().enumerate() {
        let next = len + w * total / sum;
        out[j] = (at(len), at(next.min(total)));
        len = next;
    }
    out
}
/// How much of the row's thickness a crown is drawn with: all of it (the rows the laminae hold
/// are as thin at the incisors as the crowns are).
fn drawn(_u: f64) -> f64 {
    1.
}
/// crown height above the gum (template cm) at depth fraction u, as the palate's gum line has it
fn crown(u: f64) -> f64 {
    lerp(1.0, 0.75, smoothstep(0., 0.5, u))
}

/// One crown: rings from the neck (in the gum) to the biting edge, closed by a cap. `kind`:
/// 0 incisor (a blade), 1 canine (a point), 2 premolar or molar (a block with a rounded top).
/// The crown follows the arch: its lingual and buccal faces lie on the row's own curves, so
/// neighbours' faces run on from one to the next (crowns as straight blocks of each tooth's own
/// thickness stepped outwards from tooth to tooth: a staircase along the molars).
#[allow(clippy::too_many_arguments)]
fn tooth(m: &mut ElementMesh, p: &Parts, arch: &Arch, upper: bool, side: f64, span: (f64, f64), kind: usize) {
    let k = p.map.k;
    // the depth fraction a share f of the way along the tooth, by length along the row's middle
    // (the arch runs almost sideways at the incisors, where the depth fraction is no measure of length)
    const N: usize = 6;
    let mut len = [0.; N + 1];
    let mid = |u: f64| arch_point(arch, u, 0.5 * drawn(u));
    let mut prev = mid(span.0);
    for i in 1..=N {
        let q = mid(lerp(span.0, span.1, i as f64 / N as f64));
        len[i] = len[i - 1] + sqrt((q.0 - prev.0) * (q.0 - prev.0) + (q.1 - prev.1) * (q.1 - prev.1));
        prev = q;
    }
    let along = |f: f64| -> f64 {
        let want = f.clamp(0., 1.) * len[N];
        let i = (1..=N).find(|&i| len[i] >= want).unwrap_or(N);
        let g = ((want - len[i - 1]) / (len[i] - len[i - 1]).max(1e-12)).clamp(0., 1.);
        lerp(span.0, span.1, (i as f64 - 1. + g) / N as f64)
    };
    // v: 0 at the biting edge … 1 at the neck, then 1.25 inside the gum
    let levels = [0.03, 0.1, 0.24, 0.45, 0.7, 0.95, 1.25];
    let shape = |v: f64| -> (f64, f64, f64) {
        // (half-width along the row, half-thickness across it, shift towards the lip), as fractions
        let neck = 1. - 0.1 * smoothstep(0.65, 1.1, v);
        match kind {
            0 => (1.0 * (1. - 0.08 * (1. - smoothstep(0., 0.12, v))) * neck, lerp(0.3, 0.95, smoothstep(0., 0.75, v)) * neck.max(0.9), 0.3 * (1. - smoothstep(0., 0.7, v))),
            1 => (lerp(0.5, 1.0, smoothstep(0., 0.35, v)) * neck, lerp(0.4, 0.95, smoothstep(0., 0.55, v)) * neck, 0.15 * (1. - smoothstep(0., 0.6, v))),
            _ => (lerp(0.84, 1.0, smoothstep(0., 0.2, v)) * neck, lerp(0.76, 0.94, smoothstep(0., 0.22, v)) * neck, 0.),
        }
    };
    // a point of the crown: `a` along the row (−1 mesial … 1 distal), `b` across it (−1 the
    // lingual face … 1 the buccal one)
    // the row's frame at nine places along the tooth, once (the arch's curve is two powers a
    // point; every vertex asking for its own made the teeth three quarters of the meshes' time):
    // (lingual edge x, lateral; normal x, lateral; thickness; crown height)
    const FRAMES: usize = 9;
    // (the arch's own curve begins 1.4 mm off the midline; the first crown begins on it)
    let off = if span.0 <= 0. { arch_point(arch, 0., 0.).1 } else { 0. };
    let mut frames = [[0.; 6]; FRAMES];
    for (i, frame) in frames.iter_mut().enumerate() {
        let f = i as f64 / (FRAMES - 1) as f64;
        let u = along(f).max(0.);
        let ((x, lat), (nx, na)) = arch.frame(u);
        *frame = [x, lat - off * (1. - f), nx, na, Arch::tooth(u) * drawn(u), crown(u)];
    }
    // a point of the crown: `a` along the row (−1 mesial … 1 distal), `b` across it (−1 the
    // lingual face … 1 the buccal one)
    let at = |a: f64, b: f64, v: f64| -> [f64; 3] {
        let q = (0.5 * (a + 1.)).clamp(0., 1.) * (FRAMES - 1) as f64;
        let i = (q as usize).min(FRAMES - 2);
        let t = q - i as f64;
        let g = |j: usize| lerp(frames[i][j], frames[i + 1][j], t);
        let across = 0.5 * g(4) * (1. + b);
        let (xt, lat, h) = (g(0) + across * g(2), g(1) + across * g(3), g(5));
        let q = if upper { p.upper_occlusal(xt, v * (h + CUSP) - CUSP) } else { p.lower_occlusal(xt, v * h) };
        [q.x, q.y, side * lat.max(0.) * k]
    };
    let ring = |r: usize, s: usize| -> [f64; 3] {
        // rings run neck → edge so the cap closes the biting end
        let v = levels[TOOTH_RINGS - 1 - r];
        let (wa, wb, shift) = shape(v);
        let th = s as f64 / TOOTH_SEGS as f64 * 2. * PI;
        // a rounded rectangle: a superellipse of exponent 3.2
        let (c, sn) = (cos(th), sin(th));
        let e = 2. / 3.2;
        let (ca, sb) = (c.signum() * pow(c.abs(), e), sn.signum() * pow(sn.abs(), e));
        // cusps of the back teeth: the corners stand a little higher than the middle of each side
        let cusp = if kind == 2 { 0.035 * (1. - cos(2. * th).abs()) * (1. - smoothstep(0., 0.3, v)) } else { 0. };
        // (a hair's gap between neighbours)
        at(0.97 * wa * ca, (wb * sb + shift).clamp(-1., 1.2), v - cusp)
    };
    let (_, _, shift) = shape(0.);
    let tip = at(0., shift, if kind == 2 { 0.09 } else { 0.015 });
    m.capped(TOOTH_RINGS, TOOTH_SEGS, ring, tip);
}

/// The gum of an arch, both sides in one surface: a band over the row from the lingual side over
/// the necks to the buccal side, its margin higher between the teeth than at their middles.
/// (One grid per side met the other at the midline with normals of its own: behind the upper
/// incisors the two halves were lit as a dark rectangle beside a pale one.)
fn gum(m: &mut ElementMesh, p: &Parts, arch: &Arch, upper: bool, spans: &[(f64, f64); TEETH_PER_SIDE]) {
    const PER: usize = 6;
    let k = p.map.k;
    let half = TEETH_PER_SIDE * PER;
    // across the band: (position across the row, height as crown fractions from the edge, extra cm)
    // (positions across the drawn crown: 0 its lingual face, 1 its buccal face)
    let cols: [(f64, f64, f64); 8] = [(-0.42, 1.0, 0.5), (-0.1, 1.0, 0.12), (0.06, 0.84, 0.0), (0.3, 0.96, 0.0), (0.7, 0.96, 0.0), (0.94, 0.84, 0.0), (1.08, 1.0, 0.14), (1.14, 1.0, 0.7)];
    m.grid(2 * half + 1, cols.len(), 0, 0, |r, c| {
        // rows run from the left side's last molar through the midline to the right side's
        let (side, r) = if r < half { (-1., half - r) } else { (1., r - half) };
        let j = (r / PER).min(TEETH_PER_SIDE - 1);
        let f = if r == half { 1. } else { (r % PER) as f64 / PER as f64 };
        let u = lerp(spans[j].0, spans[j].1, f).max(0.004);
        let (b, lift, extra) = cols[c];
        // (the lower gum's skirts are short: the chin's skin lies close under the lower incisors'
        // roots, and a longer skirt came out through it when the jaw was down)
        let extra = if upper { extra } else { 0.45 * extra };
        // the margin (columns 2 and 5) dips at each tooth's middle and rises to a papilla between
        let margin = if c == 2 || c == 5 { -0.13 * sin(PI * f) } else { 0. };
        let (xt, mut lat) = arch_point(arch, u, b * drawn(u));
        if r == 0 {
            lat = 0.;
        }
        let h = crown(u);
        let v = lift + margin;
        let q = if upper { p.upper_occlusal(xt, v * (h + CUSP) - CUSP + extra) } else { p.lower_occlusal(xt, v * h + extra) };
        [q.x, q.y, side * lat.max(0.) * k]
    });
}

pub fn meshes(p: &Parts) -> Vec<ElementMesh> {
    let k = p.map.k;
    let s = &p.shape;
    let mut out: Vec<ElementMesh> = ELEMENTS.iter().map(|e| ElementMesh::new(e)).collect();

    // ---- tongue: a closed body. Its dorsal surface over (t, w); the underside of the tip lobe;
    // its belly (from each margin down to the core inside the tongue and across under it, back
    // to the frenulum) and the end of its root; and the floor of the mouth inside the lower arch
    // (carried by the jaw). The dorsal surface and the lobe are the laminae's; the belly lies
    // inside the tongue's tissue and bounds no air.
    {
        let m = &mut out[0];
        m.material("tongue");
        let n = p.dorsal.len();
        let (rows, cols) = (64, 27);
        let top = |x: f64, z: f64| -> Vec2 {
            let (i, f) = along(&p.dorsal, x);
            let lobe = p.lobe_factor(z);
            lerp_v(p.tongue_at(i, z, lobe), p.tongue_at(i + 1, z, lobe), f)
        };
        // the tongue's own half-width; in the soft tube no more than `TONGUE_IN_TUBE` of the
        // tube's (beyond it the same surface is the pharynx's side wall, drawn with the pharynx)
        let width = |x: f64| -> f64 {
            let (i, f) = along(&p.dorsal, x);
            let at = |j: usize| lerp(p.dw[j].min(TONGUE_IN_TUBE * p.dtube[j]), p.dw[j], p.dloaf[j]);
            lerp(at(i), at((i + 1).min(n - 1)), f)
        };
        m.grid(rows, cols, 4, 3, |r, c| {
            let x = r as f64 / (rows - 1) as f64;
            let z = (c as f64 / (cols - 1) as f64 * 2. - 1.) * width(x) * 0.999;
            p3(top(x, z), z)
        });
        let tipw = p.dw[n - 1];
        let bladew = p.dw[p.lobe_start];
        let nu = p.under.len();
        m.grid(nu + 1, cols, 2, 3, |r, c| {
            let width = lerp(tipw, bladew, r as f64 / nu as f64);
            let z = (c as f64 / (cols - 1) as f64 * 2. - 1.) * width * 0.999;
            let lobe = p.lobe_factor(z);
            let q = if r == 0 { p.tongue_at(n - 1, z, lobe) } else if r == nu { p.under[nu - 1] } else { p.compress(p.under[r - 1], lobe) };
            p3(q, z)
        });
        // the belly, from the root to the blade's start (where the lobe's underside takes over):
        // the tongue's underside, straight from the root's foot (the valleculae's floor) to the
        // frenulum and a little fuller along the middle. (A belly that followed the dorsum at
        // the tongue's thickness made the body a curved slab, hollow underneath.)
        let body = p.lobe_start as f64 / (n - 1) as f64;
        let (foot, fren) = (p.root[0], p.under[nu - 1]);
        const BELLY: usize = 11;
        let belly = |x: f64, c: usize| -> [f64; 3] {
            if c == 0 || c == BELLY - 1 {
                let z = if c == 0 { -1. } else { 1. } * width(x) * 0.999;
                return p3(top(x, z), z);
            }
            let w = (c as f64 - (BELLY - 1) as f64 / 2.) / ((BELLY - 3) as f64 / 2.);
            let q = lerp_v(foot, fren, x / body);
            p3(v(q.x, q.y - 0.35 * k * (1. - w * w) * sin(PI * x / body)), w * width(x) * 0.97)
        };
        m.grid(40, BELLY, 0, 0, |r, c| belly(body * r as f64 / 39., c));
        // the root's end: both loops close on a point inside it
        let centre = p3(lerp_v(p.dorsal[0], foot, 0.5), 0.);
        m.grid(2, cols, 0, 0, |r, c| if r == 1 { centre } else { p3(top(0., (c as f64 / (cols - 1) as f64 * 2. - 1.) * width(0.) * 0.999), (c as f64 / (cols - 1) as f64 * 2. - 1.) * width(0.) * 0.999) });
        m.grid(2, BELLY, 0, 0, |r, c| if r == 1 { centre } else { belly(0., c) });
        m.material("floor");
        let arch = &p.lower;
        m.grid(16, 11, 0, 0, |r, c| {
            let u = lerp(0.03, 1.15, r as f64 / 15.);
            let w = c as f64 / 10. * 2. - 1.;
            let (xt, lat) = arch_point(arch, u.min(1.), -0.05);
            let xt = xt - (u - 1.).max(0.) * arch.depth;
            // a shallow trough: at the gum line beside the teeth, lower along the middle
            let q = p.lower_occlusal(xt, crown(u.min(1.)) * 0.92 + 0.5 * (1. - w * w));
            [q.x, q.y, w * lat * k]
        });
    }
    // ---- hard palate over the arch's inner width
    {
        let m = &mut out[1];
        m.material("palate");
        let np = p.palate.len();
        m.grid(np, 23, 3, 2, |r, c| {
            let a = (p.upper.inner_at(p.palate_xt[r]) * k).max(0.05 * k);
            let z = (c as f64 / 22. * 2. - 1.) * a;
            p3(p.palate_at(r, z), z)
        });
    }
    // ---- soft palate: a body with its own thickness (the oral surface the laminae hold, the
    // nasal one the engine's outline: no air of the oral tract lies above it), its sides coming
    // down as the faucial arches, and the uvula's lobe hanging from it
    {
        let m = &mut out[2];
        m.material("velum");
        let nv = p.velum.len();
        let rows = 2 * nv - 1;
        const ORAL: usize = 11;
        const NASAL: usize = 7;
        let section = |r: usize, c: usize| -> [f64; 3] {
            let (i, f) = (r / 2, (r % 2) as f64 * 0.5);
            let j = (i + 1).min(nv - 1);
            let half = p.velum_width_at(lerp(p.velum_f[i], p.velum_f[j], f)) * 0.999;
            let oral = |z: f64| lerp_v(p.velum_at(i, z), p.velum_at(j, z), f);
            // (raised, the engine's flap runs on into the posterior wall: the soft palate lies against it)
            let pressed = |q: Vec2, z: f64| v(q.x.max(p.wall_point(q.y, z).x + 0.02 * k), q.y);
            // (the last column closes the section on the first)
            let c = c % (ORAL + NASAL);
            if c < ORAL {
                let z = (c as f64 / (ORAL - 1) as f64 * 2. - 1.) * half;
                return p3(pressed(oral(z), z), z);
            }
            // back over the nasal surface, rounded into both edges
            let w = 1. - (c - ORAL + 1) as f64 / (NASAL + 1) as f64 * 2.;
            let lift = sub(lerp_v(p.velum_nasal[i], p.velum_nasal[j], f), lerp_v(p.velum[i], p.velum[j], f));
            p3(pressed(add(oral(w * half), scale(lift, sqrt(1. - w * w))), w * half), w * half)
        };
        m.grid(rows, ORAL + NASAL + 1, 2, 0, section);
        // its tip's end
        let end = lerp_v(p.velum[0], p.velum_nasal[0], 0.5);
        let end = p3(v(end.x.max(p.wall_x(end.y) + 0.02 * k), end.y), 0.);
        m.grid(2, ORAL + NASAL + 1, 0, 0, |r, c| if r == 1 { end } else { section(0, c) });
        // the arches: from the soft palate's own edge out to the tube's side, where the roof has
        // come down on the tongue
        let from = (p.uvula_fraction() + 0.25).min(0.95);
        for side in [-1., 1.] {
            m.grid(10, 8, 0, 0, |r, c| {
                let f = lerp(from, 1., r as f64 / 9.);
                let j = (0..nv - 1).find(|&j| p.velum_f[j + 1] >= f).unwrap_or(nv - 2);
                let u = ((f - p.velum_f[j]) / (p.velum_f[j + 1] - p.velum_f[j]).max(1e-9)).clamp(0., 1.);
                let z = side * lerp(p.velum_width_at(f) * 0.999, lerp(p.vtube[j], p.vtube[j + 1], u), c as f64 / 7.);
                p3(lerp_v(p.velum_at(j, z), p.velum_at(j + 1, z), u), z)
            });
        }
        // the uvula's lobe, lamina by lamina
        let half = p.uvula.4;
        m.grid(9, parts::UVULA_SAMPLES + 3, 0, 0, |r, c| {
            let z = (r as f64 / 8. * 2. - 1.) * half * 0.9995;
            p3(p.uvula_outline(z).map_or(p.uvula.0, |o| o[c]), z)
        });
    }
    // ---- posterior and lateral pharyngeal wall, from the laryngeal inlet up over the
    // nasopharynx's roof: flat behind, turning forward at its corners (the laminae's ring)
    {
        let m = &mut out[3];
        m.material("pharynx");
        let y0 = p.bottom;
        let y1 = p.full_wall[p.full_wall.len() - 1].y;
        m.grid(RING_ROWS, RING_COLS, 2, 2, |r, c| {
            // (rows closer together towards the roof, where the wall turns forward)
            let t = r as f64 / (RING_ROWS - 1) as f64;
            let y = lerp(y0, y1, 1. - pow(1. - t, 1.7));
            // (columns closer together towards the ring's sides, where it turns forward)
            let w = 0.985 * sin(0.5 * PI * (c as f64 / (RING_COLS - 1) as f64 * 2. - 1.));
            let z = w * p.pharynx_width(y);
            p3(p.wall_point(y, z), z)
        });
        // the tube's side walls in front of the ring's corners: the hypopharynx's front wall
        // beside the epiglottis' leaf (the folds from its edges back to the tube's sides,
        // `Parts::epiglottis_at`), and above it the tongue's surface beyond the tongue's own
        // body, out to where it has come back to the posterior wall (`Parts::tongue_at`)
        let ewid = s.epiglottis_width * k;
        let n = p.dorsal.len();
        // (the stretch of the tongue that faces the wall: up to its last point whose sides come out)
        let soft = (0..n).rev().find(|&i| p.dclose[i] > 0.02 * k).unwrap_or(0) as f64 / (n - 1) as f64;
        for side in [-1., 1.] {
            m.grid(8, 9, 0, 0, |r, c| {
                let z = side * lerp(ewid, 0.99 * p.pharynx_width(p.bottom), c as f64 / 8.);
                p3(p.epiglottis_at(p.epiglottis_top(z) * r as f64 / 7., z), z)
            });
            m.grid(16, 7, 0, 0, |r, c| {
                let (i, f) = along(&p.dorsal, soft * r as f64 / 15.);
                let j = (i + 1).min(n - 1);
                let (from, to) = (lerp(p.dw[i].min(TONGUE_IN_TUBE * p.dtube[i]), p.dw[j].min(TONGUE_IN_TUBE * p.dtube[j]), f), lerp(p.dtube[i], p.dtube[j], f));
                let z = side * lerp(from * 0.999, to.max(from), c as f64 / 6.);
                let lobe = p.lobe_factor(z);
                p3(lerp_v(p.tongue_at(i, z, lobe), p.tongue_at(j, z, lobe), f), z)
            });
        }
    }
    // ---- larynx: laryngeal tube, piriform fossae, valleculae, the epiglottis' leaf
    {
        let m = &mut out[4];
        m.material("larynx");
        let nt = parts::TUBE_SAMPLES + 1;
        m.grid(nt, 25, 4, 3, |r, c| {
            let (x, a, b) = p.tube[r];
            let y = p.glottis.y + p.epi_len * r as f64 / (nt - 1) as f64;
            let th = c as f64 / 24. * 2. * PI;
            [x + a * cos(th), y, b * sin(th)]
        });
        for side in [-1., 1.] {
            // a fossa is the half-ellipsoid the laminae cut (`Parts::piriform_section`), drawn
            // from its apex up to the hypopharynx floor it opens into
            let (cx, cz, rx, rz) = p.piriform(side);
            let (top, height) = p.piriform_top();
            let floor = crate::math::asin(((p.bottom - top) / height).clamp(-1., 0.));
            m.grid(11, 17, 2, 2, |r, c| {
                let la = lerp(-0.5 * PI, floor, r as f64 / 10.);
                let lo = c as f64 / 16. * 2. * PI;
                [cx + rx * cos(la) * cos(lo), top + height * sin(la), cz + rz * cos(la) * sin(lo)]
            });
            // a vallecula: its floor at the leaf, then the tongue's root up to the lid, across
            // the trough (`Parts::vallecula_span`, `root_x`, `epiglottis_front_x`)
            let (vz, vr) = p.vallecula(side);
            m.grid(9, 13, 2, 2, |r, c| {
                let z = vz + vr * (r as f64 / 8. * 2. - 1.) * 0.98;
                let top = p.epiglottis_tip(z).y;
                let (floor, top) = p.vallecula_span(z).unwrap_or((top, top));
                let lid = p.dorsal_at(0, z).y;
                if c == 0 {
                    return [p.epiglottis_front_x(floor, z), floor, z];
                }
                let y = lerp(floor, lid, (c - 1) as f64 / 11.);
                [p.root_x(y, z).max(p.epiglottis_front_x(y.min(top), z)), y, z]
            });
        }
        // the hypopharynx's floor round the laryngeal tube, with the aryepiglottic folds standing
        // on it and the fossae opening in it (`Parts::floor_height`): the tube, the folds and
        // the fossae are one surface
        {
            let (cx, a, b) = p.tube_at((p.bottom - p.glottis.y) / p.epi_len);
            // (each spoke's two ends once: the reach is a search)
            const SPOKES: usize = 37;
            let mut spokes = [(0., 0., 0., 0.); SPOKES];
            for (c, spoke) in spokes.iter_mut().enumerate() {
                let th = c as f64 / (SPOKES - 1) as f64 * 2. * PI;
                let (dx, dz) = (cos(th), sin(th));
                let rim = 1. / sqrt((dx / a) * (dx / a) + (dz / b) * (dz / b));
                *spoke = (dx, dz, rim, p.floor_reach(cx, dx, dz).max(rim));
            }
            m.grid(7, SPOKES, 0, 0, |r, c| {
                let (dx, dz, rim, reach) = spokes[c];
                let t = lerp(rim, reach, r as f64 / 6.);
                let (x, z) = (cx + t * dx, t * dz);
                [x, if r == 0 { p.bottom } else { p.floor_height(x, z) }, z]
            });
        }
        // what lies between the tube and each fossa below the floor (the aryepiglottic fold's
        // foot): a web from the tube's side to the pouch's, level by level. It bounds no air;
        // without it the pouches hung beside the tube as if detached
        let ewid = s.epiglottis_width * k;
        for side in [-1., 1.] {
            let (top, height) = p.piriform_top();
            m.grid(9, 3, 0, 0, |r, c| {
                let y = lerp(top - 0.98 * height, p.bottom, r as f64 / 8.);
                let (tx, _, tb) = p.tube_at((y - p.glottis.y) / p.epi_len);
                let (fx, fz, _, rz) = p.piriform_section(side, y).unwrap_or((p.piriform(side).0, p.piriform(side).1, 0., 0.));
                let u = c as f64 / 2.;
                [lerp(tx, fx, u), y, lerp(side * tb, fz - side * rz, u)]
            });
        }
        // the leaf: its laryngeal surface from its foot to its tip, then its lingual surface back
        // down (the laminae's plate, `Parts::epiglottis_at` and its thickness): broad and round
        // above, on a stalk a third as wide (`Parts::epiglottis_foot`)
        m.material("epiglottis");
        let th = p.epiglottis_thickness();
        m.grid(14, 13, 2, 3, |r, c| {
            let z = (c as f64 / 12. * 2. - 1.) * ewid * 0.999;
            let top = p.epiglottis_top(z);
            let foot = p.epiglottis_foot(z).min(top);
            if r < 7 {
                p3(p.epiglottis_at(lerp(foot, top, r as f64 / 6.), z), z)
            } else {
                p3(add(p.epiglottis_at(lerp(foot, top, (13 - r) as f64 / 6.), z), th), z)
            }
        });
    }
    // ---- lips: their inner surfaces, closing beyond the mouth's corners
    {
        let m = &mut out[5];
        m.material("lip");
        let wl = p.lip_w + 0.6 * k;
        let nu = p.upper_lip.len();
        m.grid(nu, 33, 2, 2, |r, c| {
            let z = (c as f64 / 32. * 2. - 1.) * wl;
            p3(p.upper_lip_at(r, z, p.front_shift(&p.upper, z)), z)
        });
        let nl = p.lower_front.len() - 3;
        m.grid(nl, 33, 2, 2, |r, c| {
            let z = (c as f64 / 32. * 2. - 1.) * wl;
            p3(p.lower_lip_at(r + 3, z, p.front_shift(&p.lower, z)), z)
        });
    }
    // ---- teeth along both arches, with their gums
    {
        let m = &mut out[6];
        let spans = [tooth_spans(&p.upper, &UPPER_CROWNS), tooth_spans(&p.lower, &LOWER_CROWNS)];
        m.material("enamel");
        for (a, (arch, upper)) in [(&p.upper, true), (&p.lower, false)].into_iter().enumerate() {
            for side in [-1., 1.] {
                for (j, span) in spans[a].iter().enumerate() {
                    tooth(m, p, arch, upper, side, *span, if j < 2 { 0 } else if j == 2 { 1 } else { 2 });
                }
            }
        }
        m.material("gum");
        for (a, (arch, upper)) in [(&p.upper, true), (&p.lower, false)].into_iter().enumerate() {
            gum(m, p, arch, upper, &spans[a]);
        }
    }
    // ---- cheeks: the inner surface beside the teeth, from behind the mouth's corners back past
    // the last molars: on the upper row's buccal face above, on the lower row's below (the jaw
    // carries it, as in the laminae), turning in over the gums at its upper and lower folds
    {
        let m = &mut out[7];
        m.material("cheek");
        let gap = s.buccal_gap + 0.6 * s.cheek_puff;
        for side in [-1., 1.] {
            m.grid(16, 11, 2, 2, |r, c| {
                let u = lerp(0.24, 1.22, r as f64 / 15.);
                let behind = (u - 1.).max(0.);
                let v = c as f64 / 10. * 2. - 1.;
                let point = |arch: &Arch, upper: bool, reach: f64| -> (Vec2, f64) {
                    let (x, a) = arch_point(arch, u.min(1.), 1.);
                    let x = x - behind * arch.depth + gap * 0.3;
                    (if upper { p.upper_occlusal(x, reach) } else { p.lower_occlusal(x, reach) }, a)
                };
                // the middle fifth spans the rows; above and below it the cheek runs 1.3 cm up
                // and down the gums
                let ((q, a), fold) = if v > 0.2 {
                    let t = (v - 0.2) / 0.8;
                    (point(&p.upper, true, 1.3 * t), t)
                } else if v < -0.2 {
                    let t = (-v - 0.2) / 0.8;
                    (point(&p.lower, false, 1.3 * t), t)
                } else {
                    let ((hi, ah), (lo, al)) = (point(&p.upper, true, 0.), point(&p.lower, false, 0.));
                    let t = (v + 0.2) / 0.4;
                    ((lerp_v(lo, hi, t), lerp(al, ah, t)), 0.)
                };
                // at its folds the cheek turns in onto the gum; behind the last molar it draws in towards the faucial arch
                let lateral = (a + gap + 0.06 - 0.3 * pow(fold, 3.)) * (1. - 1.6 * behind);
                [q.x, q.y, side * lateral * k]
            });
        }
    }
    for m in &mut out {
        m.close();
    }
    out
}
