//! Display surfaces, sampled from the same lateral rules the laminae use (`parts.rs`), so the
//! lab shows the geometry it cuts. Each element has a vertex count fixed by the body, so the
//! browser can update positions in place. `indices` are triangles (filled black to hide what
//! is behind), `lines` are segment pairs (the drawn grid).
use super::parts::{self, Arch, Parts};
use super::*;

#[derive(Clone, Debug, Default, Serialize)]
pub struct ElementMesh {
    pub element: &'static str,
    pub positions: Vec<f32>,
    pub indices: Vec<u32>,
    pub lines: Vec<u32>,
}
impl ElementMesh {
    fn new(element: &'static str) -> Self {
        Self { element, ..Default::default() }
    }
    /// A rows × cols grid of points; lines along every `rs`-th row and every `cs`-th column
    /// (and the border).
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
        for r in 0..rows {
            if r % rs == 0 || r + 1 == rows {
                for c in 0..cols - 1 {
                    self.lines.extend_from_slice(&[id(r, c), id(r, c + 1)]);
                }
            }
        }
        for c in 0..cols {
            if c % cs == 0 || c + 1 == cols {
                for r in 0..rows - 1 {
                    self.lines.extend_from_slice(&[id(r, c), id(r + 1, c)]);
                }
            }
        }
    }
    /// A box from 8 corners (bottom four then top four, each in ring order).
    fn cuboid(&mut self, c: [[f64; 3]; 8]) {
        let base = (self.positions.len() / 3) as u32;
        for p in c {
            self.positions.extend_from_slice(&[p[0] as f32, p[1] as f32, p[2] as f32]);
        }
        for i in 0..4u32 {
            let j = (i + 1) % 4;
            self.lines.extend_from_slice(&[base + i, base + j, base + 4 + i, base + 4 + j, base + i, base + 4 + i]);
            self.indices.extend_from_slice(&[base + i, base + j, base + 4 + i, base + 4 + i, base + j, base + 4 + j]);
        }
        self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        self.indices.extend_from_slice(&[base + 4, base + 5, base + 6, base + 4, base + 6, base + 7]);
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

pub fn meshes(p: &Parts) -> Vec<ElementMesh> {
    let k = p.map.k;
    let s = &p.shape;
    let mut out: Vec<ElementMesh> = ELEMENTS.iter().map(|e| ElementMesh::new(e)).collect();

    // ---- tongue: dorsal surface over (t, w), then the underside of the tip lobe
    {
        let m = &mut out[0];
        let n = p.dorsal.len();
        let (rows, cols) = (44, 19);
        m.grid(rows, cols, 3, 3, |r, c| {
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
    }
    // ---- hard palate over the arch's inner width
    {
        let m = &mut out[1];
        let np = p.palate.len();
        m.grid(np, 15, 3, 2, |r, c| {
            let a = (p.upper.inner_at(p.palate_xt[r]) * k).max(0.05 * k);
            let z = (c as f64 / 14. * 2. - 1.) * a;
            p3(p.palate_at(r, z), z)
        });
    }
    // ---- velum and uvula: the uvula a narrow lobe at the middle of the free margin
    {
        let m = &mut out[2];
        let nv = p.velum.len();
        m.grid(nv, 11, 1, 2, |r, c| {
            let z = (c as f64 / 10. * 2. - 1.) * p.velum_width_at(p.velum_f[r]) * 0.999;
            p3(p.velum_at(r, z), z)
        });
    }
    // ---- posterior and lateral pharyngeal wall, from the laryngeal inlet to the port
    {
        let m = &mut out[3];
        let nw = p.wall.len();
        let y0 = p.bottom;
        let y1 = p.wall[nw - 1].y;
        let at = |y: f64| -> (f64, f64) {
            for i in 0..nw - 1 {
                let (a, b) = (p.wall[i], p.wall[i + 1]);
                if (a.y <= y && y <= b.y) || i + 2 == nw {
                    let f = ((y - a.y) / (b.y - a.y).max(1e-9)).clamp(0., 1.);
                    return (lerp(a.x, b.x, f), lerp(p.wall_gap[i], p.wall_gap[i + 1], f));
                }
            }
            (p.wall[0].x, p.wall_gap[0])
        };
        m.grid(18, 15, 2, 2, |r, c| {
            let y = lerp(y0, y1, r as f64 / 17.);
            let (x, gap) = at(y);
            let width = p.pharynx_width(y);
            let w = c as f64 / 14. * 2. - 1.;
            [x + gap.min(3. * k) * (1. - superellipse(w, s.pharynx_shape)), y, w * width]
        });
    }
    // ---- larynx: laryngeal tube, epiglottis, piriform fossae, valleculae
    {
        let m = &mut out[4];
        let ew = s.epilarynx_width * k;
        let nt = parts::TUBE_SAMPLES + 1;
        m.grid(nt, 17, 4, 2, |r, c| {
            let y = p.epi_len * r as f64 / (nt - 1) as f64;
            let rx = p.tube[r] / (PI * ew);
            let th = c as f64 / 16. * 2. * PI;
            [p.glottis.x + rx * cos(th), p.glottis.y + y, ew * sin(th)]
        });
        let ewid = s.epiglottis_width * k;
        m.grid(2, 9, 1, 2, |r, c| {
            let z = (c as f64 / 8. * 2. - 1.) * ewid;
            p3(add(p.epi[r], v(s.epiglottis_curl * z * z / k, 0.)), z)
        });
        for side in [-1., 1.] {
            let (cz, rz) = p.piriform(side);
            let depth = s.piriform_depth * k;
            let c0 = add(p.glottis, v(0., depth * 0.65));
            m.grid(9, 13, 2, 2, |r, c| {
                let la = (r as f64 / 8. - 0.5) * PI;
                let lo = c as f64 / 12. * 2. * PI;
                [c0.x + 1.1 * k * cos(la) * cos(lo), c0.y + depth * 0.5 * sin(la), cz + rz * cos(la) * sin(lo)]
            });
            let vw = s.vallecula_width * k;
            let vc = add(p.vallecula, v(-0.2 * k, 0.3 * k));
            m.grid(7, 9, 2, 2, |r, c| {
                let la = (r as f64 / 6. - 0.5) * PI;
                let lo = c as f64 / 8. * 2. * PI;
                [vc.x + 0.3 * k * cos(la) * cos(lo), vc.y + 0.5 * s.vallecula_depth * k * sin(la), side * (0.15 * k + vw) + vw * cos(la) * sin(lo)]
            });
        }
    }
    // ---- lips: their inner surfaces, closing beyond the mouth's corners
    {
        let m = &mut out[5];
        let wl = p.lip_w + 0.6 * k;
        let nu = p.upper_lip.len();
        m.grid(nu, 21, 2, 2, |r, c| {
            let z = (c as f64 / 20. * 2. - 1.) * wl;
            p3(p.upper_lip_at(r, z, p.front_shift(&p.upper, z)), z)
        });
        let nl = p.lower_front.len() - 3;
        m.grid(nl, 21, 2, 2, |r, c| {
            let z = (c as f64 / 20. * 2. - 1.) * wl;
            p3(p.lower_lip_at(r + 3, z, p.front_shift(&p.lower, z)), z)
        });
    }
    // ---- teeth along both arches, and the mandible
    {
        let m = &mut out[6];
        for (arch, upper) in [(&p.upper, true), (&p.lower, false)] {
            for side in [-1., 1.] {
                for t in 0..8 {
                    let (u0, u1) = (t as f64 / 8. + 0.006, (t + 1) as f64 / 8. - 0.006);
                    let corner = |u: f64, outer: bool, gum: bool| -> [f64; 3] {
                        let x = arch.incisor_x - arch.depth * u.max(1e-3);
                        let a = arch.width * pow(u.max(1e-3), arch.e);
                        let da = arch.width * arch.e * pow(u.max(1e-3), arch.e - 1.);
                        let nn = norm(v(da, arch.depth));
                        let th = if outer { Arch::tooth(u) } else { 0. };
                        let (xx, aa) = (x + th * nn.x, a + th * nn.y);
                        let crown = lerp(1.0, 0.75, smoothstep(0., 0.5, u));
                        let q = if upper {
                            p.upper_occlusal(xx, if gum { crown } else { 0. })
                        } else {
                            p.lower_occlusal(xx, if gum { crown } else { 0. })
                        };
                        [q.x, q.y, side * aa * k]
                    };
                    m.cuboid([
                        corner(u0, false, false),
                        corner(u1, false, false),
                        corner(u1, true, false),
                        corner(u0, true, false),
                        corner(u0, false, true),
                        corner(u1, false, true),
                        corner(u1, true, true),
                        corner(u0, true, true),
                    ]);
                }
            }
        }
        // the mandible's body on each side: its outline as a band 4 mm thick, narrowing to the chin
        let w = (p.lower.outer_max() + 0.1) * k;
        let jaw = [(7.25, 6.75), (7.4, 5.4), (6.4, 4.6), (3.5, 4.4), (1.2, 5.2), (-0.6, 7.4), (-1.9, 9.6)];
        for side in [-1., 1.] {
            m.grid(2, jaw.len(), 1, 1, |r, c| {
                let (x, y) = jaw[c];
                let q = p.jaw(x, y);
                let narrow = 1. - 0.75 * smoothstep(3.5, 7.4, x);
                [q.x, q.y, side * (w * narrow + r as f64 * 0.4 * k)]
            });
        }
    }
    // ---- cheeks: the inner surface beside the teeth, behind the mouth's corners
    {
        let m = &mut out[7];
        let gap = s.buccal_gap + 0.6 * s.cheek_puff;
        for side in [-1., 1.] {
            m.grid(12, 3, 1, 1, |r, c| {
                let u = lerp(0.35, 1., r as f64 / 11.);
                let i = ((u * (parts::ARCH_SAMPLES - 1) as f64).round() as usize).min(parts::ARCH_SAMPLES - 1);
                let (x, a) = p.upper.outer[i];
                let y = [1.1, 0., -1.1][c];
                let q = p.upper_occlusal(x + gap * 0.3, y);
                [q.x, q.y, side * (a + gap) * k]
            });
        }
    }
    out
}
