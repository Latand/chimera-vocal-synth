//! One parasagittal section of the tract at lateral offset z (cm): air polygons and tissue
//! polygons in the x–y plane. Air is the envelope minus the union of tissue, plus the extra air
//! polygons (laryngeal tube and side pockets), which override tissue.
use super::*;

/// Air polygon slots. Slot 0 is the envelope; every other slot is "extra" air.
pub const AIR_ENVELOPE: usize = 0;
pub const AIR_TUBE: usize = 1;
pub const AIR_PIRIFORM: [usize; 2] = [2, 3];
pub const AIR_VALLECULA: [usize; 2] = [4, 5];
pub const N_AIR: usize = 6;
/// Tissue polygon slots (union).
pub const T_TONGUE: usize = 0;
pub const T_EPIGLOTTIS: usize = 1;
pub const T_UPPER_FRONT: usize = 2;
pub const T_LOWER_FRONT: usize = 3;
pub const T_UPPER_TEETH: usize = 4;
pub const T_LOWER_TEETH: usize = 5;
pub const T_PHARYNX: [usize; 2] = [6, 7];
pub const T_FAUCES: usize = 8;
pub const T_CHEEK: [usize; 2] = [9, 10];
pub const T_ARY_FOLD: usize = 11;
pub const N_TISSUE: usize = 12;

#[derive(Clone, Debug)]
pub struct Lamina {
    pub z: f64,
    pub air: Vec<Vec<Vec2>>,
    pub tissue: Vec<Vec<Vec2>>,
    /// bounding boxes [min x, min y, max x, max y], same order as `air` then `tissue`
    pub bbox: [[f64; 4]; N_AIR + N_TISSUE],
}
impl Lamina {
    pub fn new(z: f64) -> Self {
        Self {
            z,
            air: (0..N_AIR).map(|_| Vec::with_capacity(160)).collect(),
            tissue: (0..N_TISSUE).map(|_| Vec::with_capacity(160)).collect(),
            bbox: [[0.; 4]; N_AIR + N_TISSUE],
        }
    }
    pub fn clear(&mut self) {
        for p in self.air.iter_mut().chain(&mut self.tissue) {
            p.clear();
        }
    }
    /// Recompute the bounding boxes after the polygons changed.
    pub fn bound(&mut self) {
        for (i, p) in self.air.iter().chain(&self.tissue).enumerate() {
            let mut b = [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
            for q in p {
                b[0] = b[0].min(q.x);
                b[1] = b[1].min(q.y);
                b[2] = b[2].max(q.x);
                b[3] = b[3].max(q.y);
            }
            self.bbox[i] = b;
        }
    }
    /// Copy another lamina's polygons (the mirror image at −z when the shape is symmetric).
    pub fn copy_from(&mut self, o: &Lamina) {
        for (a, b) in self.air.iter_mut().zip(&o.air) {
            a.clear();
            a.extend_from_slice(b);
        }
        for (a, b) in self.tissue.iter_mut().zip(&o.tissue) {
            a.clear();
            a.extend_from_slice(b);
        }
        self.bbox = o.bbox;
    }
}
/// Signed polygon area (positive counter-clockwise).
pub fn area(poly: &[Vec2]) -> f64 {
    let mut s = 0.;
    for i in 0..poly.len() {
        let a = poly[i];
        let b = poly[(i + 1) % poly.len()];
        s += a.x * b.y - b.x * a.y;
    }
    0.5 * s
}
/// Superellipse profile: 1 at w = 0, 0 at |w| ≥ 1, `(1 − |w|^p)^(1/p)` between.
#[inline]
pub fn superellipse(w: f64, p: f64) -> f64 {
    let w = w.abs();
    if w >= 1. {
        0.
    } else {
        pow(1. - pow(w, p), 1. / p)
    }
}
/// C1 monotone Hermite interpolation of the five anatomical tongue stations.
pub fn station(values: [f64; 5], t: f64) -> f64 {
    let xs = [0., 0.3, 0.55, 0.8, 1.];
    let t = t.clamp(0., 1.);
    let i = (0..4).find(|&i| t <= xs[i + 1]).unwrap_or(3);
    let mut ds = [0.; 4];
    for j in 0..4 {
        ds[j] = (values[j + 1] - values[j]) / (xs[j + 1] - xs[j]);
    }
    let slope = |j: usize| -> f64 {
        if j == 0 {
            ds[0]
        } else if j == 4 {
            ds[3]
        } else if ds[j - 1] * ds[j] <= 0. {
            0.
        } else {
            2. * ds[j - 1] * ds[j] / (ds[j - 1] + ds[j])
        }
    };
    let h = xs[i + 1] - xs[i];
    let u = (t - xs[i]) / h;
    (2. * u * u * u - 3. * u * u + 1.) * values[i]
        + (u * u * u - 2. * u * u + u) * h * slope(i)
        + (-2. * u * u * u + 3. * u * u) * values[i + 1]
        + (u * u * u - u * u) * h * slope(i + 1)
}
