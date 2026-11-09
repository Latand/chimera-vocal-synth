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
/// How much of the row's thickness a crown is drawn with: the row the laminae hold is 7 mm
/// thick at the incisors and reaches into the lip there (the union with the lip's polygon is
/// the same tissue); an incisor's crown is half that, standing on the row's lingual side.
fn drawn(u: f64) -> f64 {
    lerp(0.5, 1., smoothstep(0.08, 0.45, u))
}
/// crown height above the gum (template cm) at depth fraction u, as the palate's gum line has it
fn crown(u: f64) -> f64 {
    lerp(1.0, 0.75, smoothstep(0., 0.5, u))
}

/// One crown: rings from the neck (in the gum) to the biting edge, closed by a cap. `kind`:
/// 0 incisor (a blade), 1 canine (a point), 2 premolar or molar (a block with a rounded top).
#[allow(clippy::too_many_arguments)]
fn tooth(m: &mut ElementMesh, p: &Parts, arch: &Arch, upper: bool, side: f64, span: (f64, f64), kind: usize) {
    let k = p.map.k;
    let uc = 0.5 * (span.0 + span.1);
    let across = 0.5 * drawn(uc);
    // the crown stands between its two ends on the row (the arch runs almost sideways at the
    // incisors, where the depth fraction is no measure of length); the first begins at the midline
    let mut c0 = arch_point(arch, span.0, across);
    if span.0 <= 0. {
        c0.1 = 0.;
    }
    let c1 = arch_point(arch, span.1, across);
    let mid = (0.5 * (c0.0 + c1.0), 0.5 * (c0.1 + c1.1));
    // along the row (mesial → distal) and across it (lingual → buccal), template cm
    let half = 0.5 * sqrt((c1.0 - c0.0) * (c1.0 - c0.0) + (c1.1 - c0.1) * (c1.1 - c0.1)).max(1e-6);
    let t = ((c1.0 - c0.0) / (2. * half), (c1.1 - c0.1) / (2. * half));
    let n = (t.1, -t.0);
    // the buccal side is the one further from the arch's inside
    let (li, bu) = (arch_point(arch, uc, 0.), arch_point(arch, uc, 1.));
    let n = if n.0 * (bu.0 - li.0) + n.1 * (bu.1 - li.1) < 0. { (-n.0, -n.1) } else { n };
    let thick = across * Arch::tooth(uc);
    let h = crown(uc);
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
    let at = |a: f64, b: f64, v: f64| -> [f64; 3] {
        let (xt, lat) = (mid.0 + a * t.0 + b * n.0, mid.1 + a * t.1 + b * n.1);
        let q = if upper { p.upper_occlusal(xt, v * (h + CUSP) - CUSP) } else { p.lower_occlusal(xt, v * h) };
        [q.x, q.y, side * lat * k]
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
        at(half * wa * ca, thick * (wb * sb + shift), v - cusp)
    };
    let (_, _, shift) = shape(0.);
    let tip = at(0., thick * shift, if kind == 2 { 0.09 } else { 0.015 });
    m.capped(TOOTH_RINGS, TOOTH_SEGS, ring, tip);
}

/// The gum of one side of an arch: a band over the row from the lingual side over the necks to
/// the buccal side, its margin higher between the teeth than at their middles.
fn gum(m: &mut ElementMesh, p: &Parts, arch: &Arch, upper: bool, side: f64, spans: &[(f64, f64); TEETH_PER_SIDE]) {
    const PER: usize = 6;
    let k = p.map.k;
    let rows = TEETH_PER_SIDE * PER + 1;
    // across the band: (position across the row, height as crown fractions from the edge, extra cm)
    // (positions across the drawn crown: 0 its lingual face, 1 its buccal face)
    let cols: [(f64, f64, f64); 8] = [(-0.42, 1.0, 0.5), (-0.1, 1.0, 0.12), (0.06, 0.84, 0.0), (0.3, 0.96, 0.0), (0.7, 0.96, 0.0), (0.94, 0.84, 0.0), (1.08, 1.0, 0.14), (1.14, 1.0, 0.7)];
    m.grid(rows, cols.len(), 0, 0, |r, c| {
        let j = (r / PER).min(TEETH_PER_SIDE - 1);
        let f = if r == rows - 1 { 1. } else { (r % PER) as f64 / PER as f64 };
        let u = lerp(spans[j].0, spans[j].1, f).max(0.004);
        let (b, lift, extra) = cols[c];
        // the margin (columns 2 and 5) dips at each tooth's middle and rises to a papilla between
        let margin = if c == 2 || c == 5 { -0.13 * sin(PI * f) } else { 0. };
        let (xt, mut lat) = arch_point(arch, u, b * drawn(u));
        if r == 0 {
            // the two sides' gums meet at the midline
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

    // ---- tongue: dorsal surface over (t, w), the underside of the tip lobe, and the floor of
    // the mouth inside the lower arch (carried by the jaw)
    {
        let m = &mut out[0];
        m.material("tongue");
        let n = p.dorsal.len();
        let (rows, cols) = (64, 27);
        m.grid(rows, cols, 4, 3, |r, c| {
            let (i, f) = along(&p.dorsal, r as f64 / (rows - 1) as f64);
            let width = lerp(p.dw[i], p.dw[(i + 1).min(n - 1)], f);
            let z = (c as f64 / (cols - 1) as f64 * 2. - 1.) * width * 0.999;
            let lobe = p.lobe_factor(z);
            let q = lerp_v(p.tongue_at(i, z, lobe), p.tongue_at(i + 1, z, lobe), f);
            p3(q, z)
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
    // ---- velum and uvula: the uvula a narrow lobe at the middle of the free margin
    {
        let m = &mut out[2];
        m.material("velum");
        let nv = p.velum.len();
        let rows = 2 * nv - 1;
        m.grid(rows, 17, 2, 2, |r, c| {
            let (i, f) = (r / 2, (r % 2) as f64 * 0.5);
            let j = (i + 1).min(nv - 1);
            let fr = lerp(p.velum_f[i], p.velum_f[j], f);
            let z = (c as f64 / 16. * 2. - 1.) * p.velum_width_at(fr) * 0.999;
            p3(lerp_v(p.velum_at(i, z), p.velum_at(j, z), f), z)
        });
    }
    // ---- posterior and lateral pharyngeal wall, from the laryngeal inlet to the port
    {
        let m = &mut out[3];
        m.material("pharynx");
        let y0 = p.bottom;
        let y1 = p.wall[p.wall.len() - 1].y;
        m.grid(26, 21, 2, 2, |r, c| {
            let y = lerp(y0, y1, r as f64 / 25.);
            let w = c as f64 / 20. * 2. - 1.;
            let z = w * p.pharynx_width(y);
            p3(p.wall_point(y, z), z)
        });
    }
    // ---- larynx: laryngeal tube, epiglottis, piriform fossae, valleculae
    {
        let m = &mut out[4];
        m.material("larynx");
        let ew = s.epilarynx_width * k;
        let nt = parts::TUBE_SAMPLES + 1;
        m.grid(nt, 25, 4, 3, |r, c| {
            let y = p.epi_len * r as f64 / (nt - 1) as f64;
            let rx = p.tube[r] / (PI * ew);
            let th = c as f64 / 24. * 2. * PI;
            [p.glottis.x + rx * cos(th), p.glottis.y + y, ew * sin(th)]
        });
        // the fossae and the valleculae are the ellipsoids the laminae cut (`Parts::piriform*`,
        // `Parts::vallecula*`): the fossa is drawn up to the hypopharynx floor it opens into
        for side in [-1., 1.] {
            let (cz, rz) = p.piriform(side);
            let (centre, rx, ry) = p.piriform_ellipse();
            let top = crate::math::asin(((p.bottom - centre.y) / ry).clamp(-1., 1.));
            m.grid(11, 17, 2, 2, |r, c| {
                let la = lerp(-0.5 * PI, top, r as f64 / 10.);
                let lo = c as f64 / 16. * 2. * PI;
                [centre.x + rx * cos(la) * cos(lo), centre.y + ry * sin(la), cz + rz * cos(la) * sin(lo)]
            });
            let (vc, vrx, vry) = p.vallecula_ellipse();
            let (vz, vrz) = p.vallecula(side);
            m.grid(9, 13, 2, 2, |r, c| {
                let la = (r as f64 / 8. - 0.5) * PI;
                let lo = c as f64 / 12. * 2. * PI;
                [vc.x + vrx * cos(la) * cos(lo), vc.y + vry * sin(la), vz + vrz * cos(la) * sin(lo)]
            });
        }
        m.material("epiglottis");
        let ewid = s.epiglottis_width * k;
        m.grid(7, 13, 2, 3, |r, c| {
            let z = (c as f64 / 12. * 2. - 1.) * ewid * 0.999;
            p3(p.epiglottis_at(r as f64 / 6., z), z)
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
            for side in [-1., 1.] {
                gum(m, p, arch, upper, side, &spans[a]);
            }
        }
    }
    // ---- cheeks: the inner surface beside the teeth, behind the mouth's corners
    {
        let m = &mut out[7];
        m.material("cheek");
        let gap = s.buccal_gap + 0.6 * s.cheek_puff;
        for side in [-1., 1.] {
            m.grid(14, 7, 2, 2, |r, c| {
                let u = lerp(0.3, 1.1, r as f64 / 13.);
                let (x, a) = arch_point(&p.upper, u.min(1.), 1.);
                let x = x - (u - 1.).max(0.) * p.upper.depth;
                // from the upper fold of the vestibule to the lower, bulging out a little between
                let v = c as f64 / 6. * 2. - 1.;
                let q = p.upper_occlusal(x + gap * 0.3, 0.);
                let low = p.lower_occlusal(x + gap * 0.3, 0.);
                let y = if v < 0. { lerp(low.y, low.y - 1.5 * k, -v) } else { lerp(q.y, q.y + 1.5 * k, v) };
                let xx = if v < 0. { low.x } else { q.x };
                [xx, y, side * (a + gap + 0.08 + 0.1 * (1. - v * v)) * k]
            });
        }
    }
    for m in &mut out {
        m.close();
    }
    out
}
