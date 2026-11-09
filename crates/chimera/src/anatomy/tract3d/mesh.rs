//! Surface strips between matching lamina contours. The browser draws these positions directly.
use super::*;
#[derive(Clone, Debug, Serialize)]
pub struct ElementMesh {
    pub element: &'static str,
    pub positions: Vec<f64>,
    pub indices: Vec<u32>,
}
pub fn meshes(laminae: &[Lamina]) -> Vec<ElementMesh> {
    let laminae: Vec<&Lamina> = laminae.iter().step_by(2).collect();
    ELEMENTS
        .iter()
        .enumerate()
        .map(|(e, &element)| {
            let mut m = ElementMesh {
                element,
                positions: Vec::new(),
                indices: Vec::new(),
            };
            // Fixed contour samples per element keep the vertex layout stable across articulation.
            let count = 48;
            for (surface, &owner) in SURFACE_OWNERS.iter().enumerate() {
                if owner != e {
                    continue;
                }
                let base = (m.positions.len() / 3) as u32;
                for l in &laminae {
                    let p = &l.elements[surface];
                    for i in 0..count {
                        let q = if p.is_empty() {
                            v(0., 0.)
                        } else {
                            let t = i as f64 * (p.len() - 1) as f64 / (count - 1) as f64;
                            let j = (t as usize).min(p.len() - 1);
                            lerp_v(p[j], p[(j + 1).min(p.len() - 1)], t - j as f64)
                        };
                        m.positions.extend_from_slice(&[q.x, q.y, l.z]);
                    }
                }
                for z in 0..laminae.len() - 1 {
                    if laminae[z].elements[surface].is_empty()
                        || laminae[z + 1].elements[surface].is_empty()
                    {
                        continue;
                    }
                    for i in 0..count - 1 {
                        let a = base + (z * count + i) as u32;
                        let b = a + count as u32;
                        m.indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
                    }
                }
            }
            m
        })
        .collect()
}
