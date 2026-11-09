//! Tissue polygons and their complementary lumen at one lateral coordinate (cm).
use super::*;
#[derive(Clone, Debug)]
pub struct Lamina {
    pub z: f64,
    pub air: Vec<Vec<Vec2>>,
    pub tissue: Vec<Vec<Vec2>>,
    pub elements: Vec<Vec<Vec2>>,
}
impl Lamina {
    pub fn new(z: f64) -> Self {
        Self {
            z,
            air: (0..6).map(|_| Vec::with_capacity(256)).collect(),
            tissue: (0..3).map(|_| Vec::with_capacity(256)).collect(),
            elements: (0..15).map(|_| Vec::with_capacity(256)).collect(),
        }
    }
    pub fn clear(&mut self) {
        for p in self
            .air
            .iter_mut()
            .chain(&mut self.tissue)
            .chain(&mut self.elements)
        {
            p.clear();
        }
    }
    pub fn contains(&self, p: Vec2) -> bool {
        (inside(&self.air[0], p) && !self.tissue.iter().any(|q| inside(q, p)))
            || self.air[1..].iter().any(|q| inside(q, p))
    }
}
pub fn inside(poly: &[Vec2], p: Vec2) -> bool {
    let mut on = false;
    for i in 0..poly.len() {
        let a = poly[i];
        let b = poly[(i + 1) % poly.len()];
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            on = !on;
        }
    }
    on
}
pub fn ellipse(out: &mut Vec<Vec2>, center: Vec2, rx: f64, ry: f64) {
    out.clear();
    if rx <= 0. || ry <= 0. {
        return;
    }
    for i in 0..48 {
        let t = i as f64 * 2. * PI / 48.;
        out.push(v(center.x + rx * cos(t), center.y + ry * sin(t)));
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
