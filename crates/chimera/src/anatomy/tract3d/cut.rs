//! Plane intersections of the same polygons used for display and air queries.
use super::lamina::Lamina;
use super::*;
use serde::ser::{SerializeStruct, Serializer};
pub const MAX_INTERVALS: usize = 16;
/// laminae, at `Z_STEP` × body scale apart, centred on the midline
pub const ROWS: usize = 65;
pub const Z_STEP: f64 = 0.1;
/// Half-height (cm, along u) of the band a region must enter to count, as in the MRI reference.
pub const BAND: f64 = 0.6;
/// Half-width (cm) of that band laterally.
pub const BAND_Z: f64 = 3.0;
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct Interval {
    pub lo: f64,
    pub hi: f64,
    pub component: usize,
}
#[derive(Clone, Debug)]
pub struct Row {
    pub z: f64,
    pub intervals: [Interval; MAX_INTERVALS],
    pub count: usize,
}
impl Default for Row {
    fn default() -> Self {
        Self {
            z: 0.,
            intervals: [Interval::default(); MAX_INTERVALS],
            count: 0,
        }
    }
}
impl Serialize for Row {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("Row", 2)?;
        s.serialize_field("z", &self.z)?;
        s.serialize_field("intervals", &self.intervals[..self.count])?;
        s.end()
    }
}
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Region {
    pub tag: &'static str,
    pub area: f64,
    pub min_z: f64,
    pub max_z: f64,
    pub min_u: f64,
    pub max_u: f64,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cut {
    pub origin: Vec2,
    pub u: Vec2,
    pub s: f64,
    pub area: f64,
    pub weighted_area: f64,
    pub perimeter: f64,
    pub channels: usize,
    pub width: f64,
    pub height: f64,
    pub fold: bool,
    pub rows: Vec<Row>,
    pub regions: Vec<Region>,
}
impl Default for Cut {
    fn default() -> Self {
        Self {
            origin: v(0., 0.),
            u: v(1., 0.),
            s: 0.,
            area: 0.,
            weighted_area: 0.,
            perimeter: 0.,
            channels: 0,
            width: 0.,
            height: 0.,
            fold: false,
            rows: Vec::with_capacity(ROWS),
            regions: Vec::with_capacity(32),
        }
    }
}
#[derive(Clone, Copy, Default)]
struct Event {
    t: f64,
    air: i16,
    tissue: i16,
    extra: i16,
}
/// Polygon parity along a line, union of air polygons minus the tissue union.
pub fn intervals(l: &Lamina, o: Vec2, u: Vec2, row: &mut Row) {
    let mut events = [Event::default(); 1024];
    let mut n = 0;
    let nair = l.air.len();
    for (polys, tissue) in [(&l.air, false), (&l.tissue, true)] {
        for (poly_index, poly) in polys.iter().enumerate() {
            if poly.len() < 3 {
                continue;
            }
            // skip polygons whose box lies wholly on one side of the line
            let b = l.bbox[poly_index + if tissue { nair } else { 0 }];
            let mut pos = false;
            let mut neg = false;
            for c in [v(b[0], b[1]), v(b[2], b[1]), v(b[0], b[3]), v(b[2], b[3])] {
                let side = u.x * (c.y - o.y) - u.y * (c.x - o.x);
                pos |= side >= 0.;
                neg |= side < 0.;
            }
            if !(pos && neg) {
                continue;
            }
            let mut ts = [0.; 256];
            let mut nt = 0;
            for i in 0..poly.len() {
                let a = sub(poly[i], o);
                let b = sub(poly[(i + 1) % poly.len()], o);
                let sa = u.x * a.y - u.y * a.x;
                let sb = u.x * b.y - u.y * b.x;
                if (sa >= 0.) == (sb >= 0.) {
                    continue;
                }
                let den = u.x * (b.y - a.y) - u.y * (b.x - a.x);
                if den.abs() < 1e-14 {
                    continue;
                }
                let t = (a.x * (b.y - a.y) - a.y * (b.x - a.x)) / den;
                if nt < ts.len() {
                    ts[nt] = t;
                    nt += 1;
                }
            }
            ts[..nt].sort_unstable_by(f64::total_cmp);
            for (i, &t) in ts[..nt].iter().enumerate() {
                if n == events.len() {
                    break;
                }
                let d = if i % 2 == 0 { 1 } else { -1 };
                let extra = !tissue && poly_index > 0;
                events[n] = Event {
                    t,
                    air: if tissue || extra { 0 } else { d },
                    tissue: if tissue { d } else { 0 },
                    extra: if extra { d } else { 0 },
                };
                n += 1;
            }
        }
    }
    events[..n].sort_unstable_by(|a, b| a.t.total_cmp(&b.t));
    let (mut air, mut tissue, mut extra) = (0, 0, 0);
    row.count = 0;
    let mut start = 0.;
    let mut open = false;
    for i in 0..n {
        air += events[i].air;
        tissue += events[i].tissue;
        extra += events[i].extra;
        let on = (air > 0 && tissue == 0) || extra > 0;
        if on && !open {
            start = events[i].t;
        }
        if !on && open && events[i].t - start > 1e-10 && row.count < MAX_INTERVALS {
            row.intervals[row.count] = Interval {
                lo: start,
                hi: events[i].t,
                component: 0,
            };
            row.count += 1;
        }
        open = on;
    }
}
fn root(parent: &[usize], mut i: usize) -> usize {
    while parent[i] != i {
        i = parent[i];
    }
    i
}
/// Rows of air along the line o + a·u in every lamina, then regions entering the band
/// (|a| ≤ `band`, |z| ≤ `BAND_Z`); `band` = ∞ keeps every region (whole anatomical planes).
/// The weighted area fades regions out over 3 mm beyond the band, for the future acoustics.
pub fn measure(laminae: &[Lamina], symmetric: bool, o: Vec2, u: Vec2, band: f64, out: &mut Cut) {
    out.origin = o;
    out.u = norm(u);
    out.rows.clear();
    out.regions.clear();
    let mid = laminae.len() / 2;
    for l in laminae {
        out.rows.push(Row { z: l.z, ..Row::default() });
    }
    for (i, l) in laminae.iter().enumerate() {
        if symmetric && i < mid {
            continue;
        }
        intervals(l, o, out.u, &mut out.rows[i]);
    }
    if symmetric {
        for i in 0..mid {
            let (lo, hi) = out.rows.split_at_mut(mid);
            let src = &hi[laminae.len() - 1 - i - mid];
            lo[i].intervals = src.intervals;
            lo[i].count = src.count;
        }
    }
    finish(out, band);
}
/// Smallest region (cm²) a cut keeps.
pub const MIN_REGION: f64 = 0.01;
fn row_dz(rows: &[Row], k: usize) -> f64 {
    if k == 0 {
        (rows[1].z - rows[0].z) * 0.5
    } else if k + 1 == rows.len() {
        (rows[k].z - rows[k - 1].z) * 0.5
    } else {
        (rows[k + 1].z - rows[k - 1].z) * 0.5
    }
}
pub fn finish(out: &mut Cut, band: f64) {
    const N: usize = ROWS * MAX_INTERVALS;
    let mut parents = [0usize; N];
    for (i, p) in parents.iter_mut().enumerate() {
        *p = i;
    }
    let mut near = [false; N];
    let mut nearest = [f64::INFINITY; N];
    for k in 0..out.rows.len() {
        for j in 0..out.rows[k].count {
            let r = out.rows[k].intervals[j];
            let id = k * MAX_INTERVALS + j;
            nearest[id] = r.lo.max(-r.hi).max(0.);
            near[id] = nearest[id] <= band && (band.is_infinite() || out.rows[k].z.abs() <= BAND_Z);
            if k > 0 {
                for b in 0..out.rows[k - 1].count {
                    let q = out.rows[k - 1].intervals[b];
                    if r.lo <= q.hi && q.lo <= r.hi {
                        let a = root(&parents, id);
                        let b = root(&parents, (k - 1) * MAX_INTERVALS + b);
                        if a != b {
                            parents[a] = b;
                        }
                    }
                }
            }
        }
    }
    let mut keep = [false; N];
    for i in 0..out.rows.len() * MAX_INTERVALS {
        let r = root(&parents, i);
        keep[r] |= near[i];
    }
    // Regions under MIN_REGION are one-lamina slivers between stepped surfaces (a tongue margin
    // against a tooth row), far below MRI resolution; they stay in the weighted area only.
    let mut comp = [0f64; N];
    for k in 0..out.rows.len() {
        let dz = row_dz(&out.rows, k);
        for j in 0..out.rows[k].count {
            let q = out.rows[k].intervals[j];
            comp[root(&parents, k * MAX_INTERVALS + j)] += (q.hi - q.lo) * dz;
        }
    }
    for i in 0..N {
        if keep[i] && comp[i] < MIN_REGION {
            keep[i] = false;
        }
    }
    let mut ids = [usize::MAX; N];
    out.area = 0.;
    out.weighted_area = 0.;
    out.perimeter = 0.;
    out.width = 0.;
    out.height = 0.;
    let (mut zlo, mut zhi, mut ulo, mut uhi) = (
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    );
    for k in 0..out.rows.len() {
        let dz = row_dz(&out.rows, k);
        let row = &mut out.rows[k];
        let mut count = 0;
        for j in 0..row.count {
            let mut q = row.intervals[j];
            let rt = root(&parents, k * MAX_INTERVALS + j);
            // the faded area weighs each interval by its own distance from the band: a region's
            // weight taken from its nearest interval jumped when a pocket beside the tongue lost
            // its last overlapping lamina (a yes/no connection), 0.86 cm² at once
            let own = if band.is_infinite() { 1. } else { 1. - smoothstep(band, band + 0.3, nearest[k * MAX_INTERVALS + j]) };
            out.weighted_area += (q.hi - q.lo) * dz * own;
            if !keep[rt] {
                continue;
            }
            let id = if ids[rt] != usize::MAX {
                ids[rt]
            } else {
                let id = out.regions.len();
                ids[rt] = id;
                out.regions.push(Region {
                    tag: "main",
                    area: 0.,
                    min_z: row.z,
                    max_z: row.z,
                    min_u: q.lo,
                    max_u: q.hi,
                });
                id
            };
            q.component = id;
            row.intervals[count] = q;
            count += 1;
            let r = &mut out.regions[id];
            r.area += (q.hi - q.lo) * dz;
            r.min_z = r.min_z.min(row.z);
            r.max_z = r.max_z.max(row.z);
            r.min_u = r.min_u.min(q.lo);
            r.max_u = r.max_u.max(q.hi);
            out.area += (q.hi - q.lo) * dz;
            zlo = zlo.min(row.z);
            zhi = zhi.max(row.z);
            ulo = ulo.min(q.lo);
            uhi = uhi.max(q.hi);
        }
        row.count = count;
    }
    out.channels = out.regions.len();
    if out.channels > 0 {
        out.width = zhi - zlo;
        out.height = uhi - ulo;
        for r in &mut out.regions {
            r.tag = if r.min_z <= 0. && r.max_z >= 0. { "main" } else if r.max_z < 0. { "left" } else { "right" };
        }
    }
    out.perimeter = boundary_length(&out.rows);
}

/// Length of the kept regions' boundary: straight segments between the corresponding ends of
/// overlapping intervals in neighbouring rows, plus the chord that closes a region where an
/// interval begins or ends (its first and last row, and the notch where one interval faces two).
/// (The first version added each row's lateral step to each end's change along u, a staircase
/// whose length does not converge: 28 % too long on a circle.)
fn boundary_length(rows: &[Row]) -> f64 {
    let overlap = |a: &Interval, b: &Interval| a.lo <= b.hi && b.lo <= a.hi;
    let none: &[Interval] = &[];
    let mut length = 0.;
    for k in 0..=rows.len() {
        // `a` the row before the step, `b` the row after it (empty beyond either end)
        let a = if k == 0 { none } else { &rows[k - 1].intervals[..rows[k - 1].count] };
        let b = if k == rows.len() { none } else { &rows[k].intervals[..rows[k].count] };
        let dz = if k == 0 || k == rows.len() { 0. } else { rows[k].z - rows[k - 1].z };
        for r in b {
            let mut first: Option<&Interval> = None;
            let mut last: Option<&Interval> = None;
            for p in a.iter().filter(|p| overlap(p, r)) {
                if let Some(q) = last {
                    // two intervals of the row before meet this one: the notch between them closes
                    length += (p.lo - q.hi).max(0.);
                }
                first = first.or(Some(p));
                last = Some(p);
            }
            match (first, last) {
                (Some(p), Some(q)) => {
                    // an end runs on to the row before only where this interval is the outermost
                    // of that interval's partners; elsewhere it borders a notch (counted below)
                    if b.iter().find(|x| overlap(x, p)).is_some_and(|x| core::ptr::eq(x, r)) {
                        length += sqrt(dz * dz + (r.lo - p.lo) * (r.lo - p.lo));
                    }
                    if b.iter().rev().find(|x| overlap(x, q)).is_some_and(|x| core::ptr::eq(x, r)) {
                        length += sqrt(dz * dz + (r.hi - q.hi) * (r.hi - q.hi));
                    }
                }
                _ => length += r.hi - r.lo,
            }
        }
        for p in a {
            let mut prev: Option<&Interval> = None;
            let mut any = false;
            for r in b.iter().filter(|r| overlap(p, r)) {
                if let Some(q) = prev {
                    length += (r.lo - q.hi).max(0.);
                }
                prev = Some(r);
                any = true;
            }
            if !any {
                length += p.hi - p.lo;
            }
        }
    }
    length
}

#[cfg(test)]
mod tests {
    use super::super::lamina::station;
    use super::*;
    #[test]
    fn circular_tube_area_and_disconnected_channels() {
        let mut ls: Vec<Lamina> = (0..ROWS)
            .map(|i| Lamina::new((i as f64 - 32.) * 0.05))
            .collect();
        for l in &mut ls {
            let h = sqrt((1. - l.z * l.z).max(0.));
            if h > 0. {
                l.air[0] = vec![v(-h, -1.), v(h, -1.), v(h, 1.), v(-h, 1.)];
            }
            l.bound();
        }
        let mut c = Cut::default();
        measure(&ls, false, v(0., 0.), v(1., 0.), BAND, &mut c);
        assert!((c.area - PI).abs() / PI < 0.01, "{}", c.area);
        assert_eq!(c.channels, 1);
        for l in &mut ls {
            l.tissue[0] = vec![v(-2., -1.), v(2., -1.), v(2., 1.), v(-2., 1.)];
            l.bound();
        }
        measure(&ls, false, v(0., 0.), v(1., 0.), BAND, &mut c);
        assert_eq!(c.area, 0.);
        assert_eq!(c.channels, 0);
        for l in &mut ls {
            l.tissue[0].clear();
            if l.z.abs() < 0.3 {
                l.air[0].clear();
            }
            l.bound();
        }
        measure(&ls, false, v(0., 0.), v(1., 0.), BAND, &mut c);
        assert_eq!(c.channels, 2);
    }
    /// laminae 0.05 cm apart whose air at lateral offset z is the interval `at(z)`
    fn solid(at: impl Fn(f64) -> Option<(f64, f64)>) -> Vec<Lamina> {
        let mut ls: Vec<Lamina> = (0..ROWS).map(|i| Lamina::new((i as f64 - 32.) * 0.05)).collect();
        for l in &mut ls {
            if let Some((lo, hi)) = at(l.z) {
                l.air[0] = vec![v(lo, -1.), v(hi, -1.), v(hi, 1.), v(lo, 1.)];
            }
            l.bound();
        }
        ls
    }
    #[test]
    fn perimeter_is_the_boundarys_length_not_a_staircase() {
        // a circle, an ellipse lying along u and one lying across it, and a ring with a hole
        let mut c = Cut::default();
        let circle = solid(|z| (z.abs() < 1.).then(|| (-sqrt(1. - z * z), sqrt(1. - z * z))));
        measure(&circle, false, v(0., 0.), v(1., 0.), f64::INFINITY, &mut c);
        assert!((c.perimeter / (2. * PI) - 1.).abs() < 0.015, "circle: {} vs {}", c.perimeter, 2. * PI);
        // Ramanujan's approximation for an ellipse's circumference
        let ellipse = |a: f64, b: f64| PI * (3. * (a + b) - sqrt((3. * a + b) * (a + 3. * b)));
        for (a, b) in [(0.5, 1.5), (1.5, 0.5)] {
            // half-axis a along u, b across (z)
            let ls = solid(|z| (z.abs() < b).then(|| (-a * sqrt(1. - z * z / (b * b)), a * sqrt(1. - z * z / (b * b)))));
            measure(&ls, false, v(0., 0.), v(1., 0.), f64::INFINITY, &mut c);
            assert!((c.perimeter / ellipse(a, b) - 1.).abs() < 0.02, "ellipse {a}×{b}: {} vs {}", c.perimeter, ellipse(a, b));
        }
        // two discs side by side along u: twice one disc's boundary
        let mut two = solid(|z| (z.abs() < 0.5).then(|| (-1.5 - sqrt(0.25 - z * z), -1.5 + sqrt(0.25 - z * z))));
        for l in &mut two {
            if l.z.abs() < 0.5 {
                let h = sqrt(0.25 - l.z * l.z);
                l.air[1] = vec![v(1.5 - h, -1.), v(1.5 + h, -1.), v(1.5 + h, 1.), v(1.5 - h, 1.)];
            }
            l.bound();
        }
        measure(&two, false, v(0., 0.), v(1., 0.), f64::INFINITY, &mut c);
        assert_eq!(c.channels, 2);
        assert!((c.perimeter / (2. * PI) - 1.).abs() < 0.02, "two discs: {} vs {}", c.perimeter, 2. * PI);
    }
    #[test]
    fn station_interpolation_has_no_overshoot() {
        for values in [[0., 1., -1., 0.5, 0.], [0.; 5], [1.; 5]] {
            for i in 0..1000 {
                let q = station(values, i as f64 / 999.);
                assert!(q >= -1. && q <= 1.);
            }
        }
    }
}
