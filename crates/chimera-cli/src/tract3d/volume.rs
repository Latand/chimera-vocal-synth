//! Common voxel/centreline/cut pipeline for imported MRI surfaces and parametric geometry.
use chimera::anatomy::tract3d::{Row, Tract3d};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::path::Path;
pub type P = [f64; 3];
/// Pixels per side of a cut mask (±4 cm at 0.25 mm).
pub const MASK: usize = 321;
pub fn add(a: P, b: P) -> P {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub fn mul(a: P, s: f64) -> P {
    [a[0] * s, a[1] * s, a[2] * s]
}
pub fn norm(a: P) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
pub struct Volume {
    pub lo: P,
    pub dims: [usize; 3],
    pub h: f64,
    pub vox: Vec<u8>,
}
impl Volume {
    pub fn new(lo: P, hi: P, h: f64) -> Self {
        let dims = std::array::from_fn(|i| ((hi[i] - lo[i]) / h).ceil() as usize + 1);
        Self {
            lo,
            dims,
            h,
            vox: vec![0; dims.iter().product()],
        }
    }
    pub fn index(&self, x: usize, y: usize, z: usize) -> usize {
        (x * self.dims[1] + y) * self.dims[2] + z
    }
    pub fn pos(&self, i: usize) -> [usize; 3] {
        let z = i % self.dims[2];
        let j = i / self.dims[2];
        [j / self.dims[1], j % self.dims[1], z]
    }

    pub fn at(&self, p: [isize; 3]) -> u8 {
        if (0..3).any(|a| p[a] < 0 || p[a] >= self.dims[a] as isize) {
            0
        } else {
            self.vox[self.index(p[0] as usize, p[1] as usize, p[2] as usize)]
        }
    }
    pub fn sample(&self, p: P) -> bool {
        let q: P = std::array::from_fn(|i| (p[i] - self.lo[i]) / self.h);
        let b: [isize; 3] = q.map(|x| x.floor() as isize);
        let w: P = std::array::from_fn(|i| q[i] - b[i] as f64);
        let mut sum = 0.;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    sum += self.at([b[0] + i, b[1] + j, b[2] + k]) as f64
                        * if i == 0 { 1. - w[0] } else { w[0] }
                        * if j == 0 { 1. - w[1] } else { w[1] }
                        * if k == 0 { 1. - w[2] } else { w[2] };
                }
            }
        }
        sum > 0.5
    }
    pub fn fill_holes(&mut self) {
        let mut seen = vec![false; self.vox.len()];
        let mut q = VecDeque::new();
        q.push_back(0);
        seen[0] = true;
        while let Some(i) = q.pop_front() {
            let p = self.pos(i);
            for a in 0..3 {
                for d in [-1, 1] {
                    let mut v = p.map(|x| x as isize);
                    v[a] += d;
                    if (0..3).any(|a| v[a] < 0 || v[a] >= self.dims[a] as isize) {
                        continue;
                    }
                    let j = self.index(v[0] as usize, v[1] as usize, v[2] as usize);
                    if !seen[j] && self.vox[j] == 0 {
                        seen[j] = true;
                        q.push_back(j);
                    }
                }
            }
        }
        for (i, v) in self.vox.iter_mut().enumerate() {
            if !seen[i] {
                *v = 1;
            }
        }
    }
}
pub fn stl(path: &Path) -> Result<Vec<[P; 3]>, String> {
    let b = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    let convert = |p: P| [-p[1] / 10., p[2] / 10., p[0] / 10.];
    if b.starts_with(b"solid") && String::from_utf8_lossy(&b[..b.len().min(400)]).contains("facet")
    {
        let text = String::from_utf8_lossy(&b);
        let mut points = Vec::new();
        for line in text.lines() {
            let v: Vec<_> = line.split_whitespace().collect();
            if v.first() == Some(&"vertex") {
                if v.len() != 4 {
                    return Err("invalid STL vertex".into());
                }
                let p = [v[1], v[2], v[3]].map(|s| s.parse::<f64>());
                if p.iter().any(|x| x.is_err()) {
                    return Err("invalid STL number".into());
                }
                points.push(convert(p.map(Result::unwrap)));
            }
        }
        for p in points.chunks_exact(3) {
            out.push([p[0], p[1], p[2]]);
        }
    } else {
        if b.len() < 84 {
            return Err("truncated STL".into());
        }
        let n = u32::from_le_bytes(b[80..84].try_into().unwrap()) as usize;
        if n > 5_000_000 || 84 + n * 50 != b.len() {
            return Err("invalid STL length".into());
        }
        for t in 0..n {
            let mut tri = [[0.; 3]; 3];
            for j in 0..3 {
                for a in 0..3 {
                    let i = 84 + t * 50 + 12 + j * 12 + a * 4;
                    tri[j][a] = f32::from_le_bytes(b[i..i + 4].try_into().unwrap()) as f64;
                }
                tri[j] = convert(tri[j]);
            }
            out.push(tri);
        }
    }
    if out.is_empty() || out.iter().flatten().flatten().any(|x| !x.is_finite()) {
        return Err("empty or nonfinite STL".into());
    }
    Ok(out)
}
pub fn openings(ts: &[[P; 3]]) -> Result<[P; 2], String> {
    let mut vertices = Vec::<P>::new();
    let mut ids = HashMap::new();
    let mut edges = HashMap::<(usize, usize), usize>::new();
    for t in ts {
        let mut v = [0usize; 3];
        for j in 0..3 {
            let key = t[j].map(|x| (x * 100000.).round() as i64);
            v[j] = *ids.entry(key).or_insert_with(|| {
                let id = vertices.len();
                vertices.push(t[j]);
                id
            });
        }
        for j in 0..3 {
            let (a, b) = (v[j], v[(j + 1) % 3]);
            *edges.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    let mut neighbors = vec![Vec::new(); vertices.len()];
    for ((a, b), n) in edges {
        if n == 1 {
            neighbors[a].push(b);
            neighbors[b].push(a);
        }
    }
    let mut seen = vec![false; vertices.len()];
    let mut loops = Vec::new();
    for i in 0..vertices.len() {
        if seen[i] || neighbors[i].is_empty() {
            continue;
        }
        let mut stack = vec![i];
        seen[i] = true;
        let mut points = Vec::new();
        while let Some(j) = stack.pop() {
            points.push(vertices[j]);
            for &k in &neighbors[j] {
                if !seen[k] {
                    seen[k] = true;
                    stack.push(k);
                }
            }
        }
        if points.len() >= 8 {
            let cen = mul(
                points.iter().copied().fold([0.; 3], add),
                1. / points.len() as f64,
            );
            loops.push((points.len(), cen));
        }
    }
    loops.sort_by_key(|x| std::cmp::Reverse(x.0));
    if loops.len() < 2 {
        return Err("expected two boundary loops in open STL".into());
    }
    let mut out = [loops[0].1, loops[1].1];
    if out[0][1] > out[1][1] {
        out.swap(0, 1);
    }
    Ok(out)
}
pub fn voxelize(ts: &[[P; 3]], h: f64) -> Volume {
    let lo = std::array::from_fn(|a| {
        ts.iter()
            .flat_map(|t| t.iter())
            .map(|p| p[a])
            .fold(f64::INFINITY, f64::min)
            - 3. * h
    });
    let hi = std::array::from_fn(|a| {
        ts.iter()
            .flat_map(|t| t.iter())
            .map(|p| p[a])
            .fold(f64::NEG_INFINITY, f64::max)
            + 3. * h
    });
    let mut vol = Volume::new(lo, hi, h);
    let mut crosses = vec![Vec::<f64>::new(); vol.dims[0] * vol.dims[1]];
    for &[a, b, c] in ts {
        let d = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
        if d.abs() < 1e-12 {
            continue;
        }
        let i0 = (((a[0].min(b[0]).min(c[0]) - lo[0]) / h).ceil() as usize).min(vol.dims[0] - 1);
        let i1 = (((a[0].max(b[0]).max(c[0]) - lo[0]) / h).floor() as usize).min(vol.dims[0] - 1);
        let j0 = (((a[1].min(b[1]).min(c[1]) - lo[1]) / h).ceil() as usize).min(vol.dims[1] - 1);
        let j1 = (((a[1].max(b[1]).max(c[1]) - lo[1]) / h).floor() as usize).min(vol.dims[1] - 1);
        for i in i0..=i1 {
            for j in j0..=j1 {
                let x = lo[0] + i as f64 * h + 1e-8;
                let y = lo[1] + j as f64 * h + 2.3e-8;
                let l1 = ((b[1] - c[1]) * (x - c[0]) + (c[0] - b[0]) * (y - c[1])) / d;
                let l2 = ((c[1] - a[1]) * (x - c[0]) + (a[0] - c[0]) * (y - c[1])) / d;
                let l3 = 1. - l1 - l2;
                if l1 >= 0. && l2 >= 0. && l3 >= 0. {
                    crosses[i * vol.dims[1] + j].push(l1 * a[2] + l2 * b[2] + l3 * c[2]);
                }
            }
        }
    }
    for i in 0..vol.dims[0] {
        for j in 0..vol.dims[1] {
            let xs = &mut crosses[i * vol.dims[1] + j];
            xs.sort_by(f64::total_cmp);
            for k in 0..vol.dims[2] {
                let z = lo[2] + k as f64 * h;
                let n = xs.partition_point(|v| *v < z);
                let id = vol.index(i, j, k);
                vol.vox[id] = (n % 2) as u8;
            }
        }
    }
    vol.fill_holes();
    vol
}
pub fn model(m: &Tract3d) -> Volume {
    let k = m.tract.body.scale;
    let mut vol = Volume::new(
        [-5. * k, -3. * k, -3.3 * k],
        [12. * k, 15. * k, 3.3 * k],
        0.05,
    );
    let mut row = Row::default();
    for j in 0..vol.dims[1] {
        let y = vol.lo[1] + j as f64 * vol.h;
        for z in 0..vol.dims[2] {
            let zz = vol.lo[2] + z as f64 * vol.h;
            m.air_intervals(y, zz, &mut row);
            for r in row.intervals.iter().take(row.count) {
                let lo = ((r.lo - vol.lo[0]) / vol.h).ceil().max(0.) as usize;
                let hi = ((r.hi - vol.lo[0]) / vol.h).floor().max(0.) as usize;
                for x in lo.min(vol.dims[0])..=hi.min(vol.dims[0] - 1) {
                    let id = vol.index(x, j, z);
                    vol.vox[id] = 1;
                }
            }
        }
    }
    vol
}
#[derive(Clone, Copy)]
struct Node(f64, usize);
impl PartialEq for Node {
    fn eq(&self, o: &Self) -> bool {
        self.0 == o.0 && self.1 == o.1
    }
}
impl Eq for Node {}
impl PartialOrd for Node {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Node {
    fn cmp(&self, o: &Self) -> Ordering {
        o.0.total_cmp(&self.0).then_with(|| o.1.cmp(&self.1))
    }
}
pub fn centreline(vol: &Volume, ends: [P; 2]) -> Result<Vec<P>, String> {
    let dims = vol.dims.map(|n| n.div_ceil(2));
    let n: usize = dims.iter().product();
    let idx = |p: [usize; 3]| (p[0] * dims[1] + p[1]) * dims[2] + p[2];
    let pos = |i: usize| [i / (dims[1] * dims[2]), i / dims[2] % dims[1], i % dims[2]];
    let mut air = vec![false; n];
    let mut distance = vec![1e9f64; n];
    for i in 0..n {
        let p = pos(i);
        air[i] = vol.at(p.map(|x| 2 * x as isize)) != 0;
        if !air[i] {
            distance[i] = 0.;
        }
    }
    // Exact separable squared Euclidean distance transform, matching scipy's EDT.
    for axis in 0..3 {
        let mut f = vec![0.; dims[axis]];
        let mut d = vec![0.; dims[axis]];
        let mut vv = vec![0usize; dims[axis]];
        let mut zz = vec![0.; dims[axis] + 1];
        for i in 0..n {
            let p = pos(i);
            if p[axis] != 0 {
                continue;
            }
            for j in 0..dims[axis] {
                let mut q = p;
                q[axis] = j;
                f[j] = distance[idx(q)];
            }
            let mut k = 0;
            vv[0] = 0;
            zz[0] = f64::NEG_INFINITY;
            zz[1] = f64::INFINITY;
            for q in 1..f.len() {
                let mut s;
                loop {
                    let v = vv[k];
                    s = ((f[q] + (q * q) as f64) - (f[v] + (v * v) as f64)) / (2. * (q - v) as f64);
                    if s > zz[k] || k == 0 {
                        break;
                    }
                    k -= 1;
                }
                k += 1;
                vv[k] = q;
                zz[k] = s;
                zz[k + 1] = f64::INFINITY;
            }
            k = 0;
            for q in 0..f.len() {
                while zz[k + 1] < (q as f64) {
                    k += 1;
                }
                d[q] = (q as f64 - vv[k] as f64).powi(2) + f[vv[k]];
            }
            for j in 0..dims[axis] {
                let mut q = p;
                q[axis] = j;
                distance[idx(q)] = d[j];
            }
        }
    }
    let nearest = |p: P| -> usize {
        (0..n)
            .filter(|&i| air[i])
            .min_by(|&a, &b| {
                let pa = pos(a);
                let pb = pos(b);
                let d = |q: [usize; 3]| {
                    (0..3)
                        .map(|j| (vol.lo[j] + q[j] as f64 * vol.h * 2. - p[j]).powi(2))
                        .sum::<f64>()
                };
                d(pa).total_cmp(&d(pb))
            })
            .unwrap_or(0)
    };
    let inward = |a: P, b: P| add(a, mul(sub(b, a), 0.2 / norm(sub(b, a))));
    let start = nearest(inward(ends[0], ends[1]));
    let end = nearest(inward(ends[1], ends[0]));
    let mut costs = vec![f64::INFINITY; n];
    let mut prev = vec![usize::MAX; n];
    let mut heap = BinaryHeap::new();
    costs[start] = 0.;
    heap.push(Node(0., start));
    while let Some(Node(cost, i)) = heap.pop() {
        if cost > costs[i] {
            continue;
        }
        if i == end {
            break;
        }
        let p = pos(i);
        for dx in -1isize..=1 {
            for dy in -1isize..=1 {
                for dz in -1isize..=1 {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }
                    let q = [p[0] as isize + dx, p[1] as isize + dy, p[2] as isize + dz];
                    if (0..3).any(|a| q[a] < 0 || q[a] >= dims[a] as isize) {
                        continue;
                    }
                    let j = idx(q.map(|x| x as usize));
                    if !air[j] {
                        continue;
                    }
                    let edt = 0.5 * (distance[i].sqrt() + distance[j].sqrt()) * vol.h * 2.;
                    let c = cost
                        + vol.h * 2. * ((dx * dx + dy * dy + dz * dz) as f64).sqrt()
                            / (edt + 0.02).powi(2);
                    if c < costs[j] {
                        costs[j] = c;
                        prev[j] = i;
                        heap.push(Node(c, j));
                    }
                }
            }
        }
    }
    if !costs[end].is_finite() {
        return Err("no connected lumen path between openings".into());
    }
    let mut path = vec![end];
    while *path.last().unwrap() != start {
        path.push(prev[*path.last().unwrap()]);
    }
    path.reverse();
    let mut pts = vec![inward(ends[0], ends[1])];
    pts.extend(path.iter().map(|&i| {
        let p = pos(i);
        std::array::from_fn(|a| vol.lo[a] + p[a] as f64 * vol.h * 2.)
    }));
    pts.push(inward(ends[1], ends[0]));
    for _ in 0..30 {
        let old = pts.clone();
        for i in 1..pts.len() - 1 {
            pts[i] = add(mul(old[i], 0.5), mul(add(old[i - 1], old[i + 1]), 0.25));
        }
    }
    pts.insert(0, ends[0]);
    pts.push(ends[1]);
    let z = 0.5 * (ends[0][2] + ends[1][2]);
    for p in &mut pts {
        p[2] = z;
    }
    Ok(resample(&pts, 0.25))
}
pub fn resample(points: &[P], step: f64) -> Vec<P> {
    let mut s = vec![0.];
    for i in 1..points.len() {
        s.push(s[i - 1] + norm(sub(points[i], points[i - 1])));
    }
    let mut out = Vec::new();
    let mut j = 1;
    for i in 0..=(*s.last().unwrap() / step).floor() as usize {
        let t = i as f64 * step;
        while j < s.len() - 1 && s[j] < t {
            j += 1;
        }
        out.push(add(
            points[j - 1],
            mul(
                sub(points[j], points[j - 1]),
                (t - s[j - 1]) / (s[j] - s[j - 1]).max(1e-12),
            ),
        ));
    }
    out
}
#[derive(Clone, Debug, Serialize)]
pub struct Slice {
    pub s: f64,
    pub area: f64,
    pub width: f64,
    pub height: f64,
    pub channels: usize,
    pub origin: P,
    pub u: P,
    #[serde(skip)]
    pub mask: Vec<u8>,
}
#[derive(Serialize)]
pub struct Measurement {
    pub length: f64,
    pub volume: f64,
    pub slices: Vec<Slice>,
}
pub fn measure(vol: &Volume, line: &[P]) -> Measurement {
    let mean = mul(
        line.iter().copied().fold([0.; 3], add),
        1. / line.len() as f64,
    );
    let slices = line
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            let t = sub(line[(i + 1).min(line.len() - 1)], line[i.saturating_sub(1)]);
            let t = mul(t, 1. / norm(t).max(1e-9));
            let mut u = [-t[1], t[0], 0.];
            let outward = sub(p, mean);
            if u[0] * outward[0] + u[1] * outward[1] < 0. {
                u = mul(u, -1.);
            }
            let n = 321;
            let res = 0.025;
            let mut mask = vec![0u8; n * n];
            for a in 0..n {
                for b in 0..n {
                    let q = add(
                        add(p, mul(u, (a as f64 - 160.) * res)),
                        [0., 0., (b as f64 - 160.) * res],
                    );
                    mask[a * n + b] = u8::from(vol.sample(q));
                }
            }
            let mut lab = vec![0usize; n * n];
            let mut regions = vec![false];
            let mut id = 0;
            for j in 0..mask.len() {
                if mask[j] == 0 || lab[j] != 0 {
                    continue;
                }
                id += 1;
                regions.push(false);
                let mut q = VecDeque::from([j]);
                lab[j] = id;
                while let Some(k) = q.pop_front() {
                    let (a, b) = (k / n, k % n);
                    if (a as f64 - 160.).abs() * res <= 0.6 && (b as f64 - 160.).abs() * res <= 3. {
                        regions[id] = true;
                    }
                    for x in [
                        if a > 0 { Some(k - n) } else { None },
                        if a + 1 < n { Some(k + n) } else { None },
                        if b > 0 { Some(k - 1) } else { None },
                        if b + 1 < n { Some(k + 1) } else { None },
                    ]
                    .into_iter()
                    .flatten()
                    {
                        if mask[x] != 0 && lab[x] == 0 {
                            lab[x] = id;
                            q.push_back(x);
                        }
                    }
                }
            }
            let (mut count, mut alo, mut ahi, mut blo, mut bhi) = (0usize, n, 0, n, 0);
            for j in 0..mask.len() {
                mask[j] = u8::from(lab[j] != 0 && regions[lab[j]]);
                if mask[j] != 0 {
                    count += 1;
                    alo = alo.min(j / n);
                    ahi = ahi.max(j / n);
                    blo = blo.min(j % n);
                    bhi = bhi.max(j % n);
                }
            }
            Slice {
                s: i as f64 * 0.25,
                area: count as f64 * res * res,
                width: if count > 0 {
                    (bhi - blo + 1) as f64 * res
                } else {
                    0.
                },
                height: if count > 0 {
                    (ahi - alo + 1) as f64 * res
                } else {
                    0.
                },
                channels: regions.iter().filter(|&&x| x).count(),
                origin: p,
                u,
                mask,
            }
        })
        .collect();
    Measurement {
        length: (line.len() - 1) as f64 * 0.25,
        volume: vol.vox.iter().map(|&x| x as f64).sum::<f64>() * vol.h.powi(3),
        slices,
    }
}
