use chimera::anatomy::geom::v;
use chimera::anatomy::human::{ArticKey, BodyMix, BODY_CHILD, BODY_FEMALE, BODY_MALE, NEUTRAL};
use chimera::anatomy::tract3d::{presets, Cut, ShapeParams, Tract3d, Tract3dFrame, SPECS};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
struct Counting;
thread_local! {static ALLOCS: Cell<usize> = const {Cell::new(0)};}
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.with(|n| n.set(n.get() + 1));
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        ALLOCS.with(|n| n.set(n.get() + 1));
        unsafe { System.realloc(p, l, n) }
    }
}
#[global_allocator]
static A: Counting = Counting;
#[test]
fn warm_compute_allocates_nothing_for_all_presets() {
    let all = presets(BodyMix::MALE);
    let mut m = Tract3d::new(BODY_MALE);
    let mut f = Tract3dFrame::default();
    for p in &all {
        m.compute(&p.artic, &p.shape, &mut f).unwrap();
    }
    let n = ALLOCS.with(Cell::get);
    for p in &all {
        m.compute(&p.artic, &p.shape, &mut f).unwrap();
    }
    assert_eq!(ALLOCS.with(Cell::get), n);
}
#[test]
fn presets_are_finite_symmetric_and_body_scaled() {
    for (body, mix) in [
        (BODY_MALE, BodyMix::MALE),
        (BODY_FEMALE, BodyMix::FEMALE),
        (BODY_CHILD, BodyMix::CHILD),
    ] {
        let mut m = Tract3d::new(body);
        let mut f = Tract3dFrame::default();
        for p in presets(mix) {
            m.compute(&p.artic, &p.shape, &mut f).unwrap();
            let af = &f.area_fn;
            assert!(
                af.length > 8. && af.length < 30.,
                "{} {}",
                p.name,
                af.length
            );
            assert!(af.volume.is_finite() && af.volume > 0.);
            assert!(af.area.iter().all(|a| a.is_finite() && *a >= 0.));
            let meshes = m.meshes();
            assert_eq!(meshes.len(), 8);
            for mesh in meshes {
                assert!(mesh.positions.iter().all(|x| x.is_finite()));
                assert!(mesh
                    .indices
                    .iter()
                    .all(|&i| (i as usize) * 3 < mesh.positions.len()));
            }
        }
    }
}
#[test]
fn nonfinite_inputs_are_rejected_and_unknown_parameters_fail() {
    let mut m = Tract3d::new(BODY_MALE);
    let mut f = Tract3dFrame::default();
    for key in ArticKey::ALL {
        let mut a = NEUTRAL;
        a.set(key, f64::NAN);
        assert!(m.compute(&a, &ShapeParams::default(), &mut f).is_err());
    }
    assert!(serde_json::from_str::<ShapeParams>(r#"{"typo":1}"#).is_err());
    let mut p = ShapeParams::default();
    p.side_root = f64::INFINITY;
    assert!(m.compute(&NEUTRAL, &p, &mut f).is_err());
}
#[test]
fn shape_extremes_and_all_articulators_have_valid_cuts() {
    let mut m = Tract3d::new(BODY_MALE);
    let mut f = Tract3dFrame::default();
    for spec in SPECS {
        for value in [spec.min, spec.max] {
            let p: ShapeParams =
                serde_json::from_value(serde_json::json!({spec.key:value})).unwrap();
            m.compute(&NEUTRAL, &p, &mut f).unwrap();
            assert!(
                f.area_fn.area.iter().all(|x| x.is_finite() && *x >= 0.),
                "{}",
                spec.key
            );
        }
    }
    for key in ArticKey::ALL {
        let spec = chimera::control::articulators::artic_spec(key);
        for value in [spec.min, spec.max] {
            let mut a = NEUTRAL;
            a.set(key, value);
            m.compute(&a, &ShapeParams::default(), &mut f).unwrap();
            assert!(f.area_fn.volume.is_finite());
        }
    }
}
#[test]
fn cut_planes_and_air_use_the_same_laminae() {
    let mut m = Tract3d::new(BODY_MALE);
    let mut f = Tract3dFrame::default();
    m.compute(&NEUTRAL, &ShapeParams::default(), &mut f)
        .unwrap();
    let mut c = Cut::default();
    m.cut(v(0., 1.), v(1., 0.), &mut c);
    assert!(c.area > 0.);
    for row in &c.rows {
        for q in row.intervals.iter().take(row.count) {
            let x = c.origin.x + c.u.x * (q.lo + q.hi) * 0.5;
            let y = c.origin.y + c.u.y * (q.lo + q.hi) * 0.5;
            assert!(m.air(x, y, row.z));
        }
    }
}

#[test]
fn small_articulator_steps_do_not_open_large_area_jumps() {
    let mut m = Tract3d::new(BODY_MALE);
    let mut f = Tract3dFrame::default();
    for key in [
        ArticKey::TbX,
        ArticKey::TbY,
        ArticKey::TtY,
        ArticKey::Jaw,
        ArticKey::Lateral,
    ] {
        let mut previous: Vec<f64> = Vec::new();
        for i in 0..51 {
            let mut a = NEUTRAL;
            a.set(key, NEUTRAL.get(key) + (i as f64 - 25.) * 0.002);
            m.compute(&a, &ShapeParams::default(), &mut f).unwrap();
            if previous.len() == f.area_fn.weighted_area.len() {
                for (j, (a, b)) in previous.iter().zip(&f.area_fn.weighted_area).enumerate() {
                    assert!(
                        (a - b).abs() < 0.5,
                        "{} step {} slice {} s={} area {} -> {}, regions {:?}",
                        key.name(),
                        i,
                        j,
                        f.area_fn.s[j],
                        a,
                        b,
                        f.slices[j].regions
                    );
                }
            }
            previous.clone_from(&f.area_fn.weighted_area);
        }
    }
}

#[test]
fn neutral_tongue_midline_is_the_existing_contour() {
    let mut m = Tract3d::new(BODY_MALE);
    let mut f = Tract3dFrame::default();
    m.compute(&NEUTRAL, &ShapeParams::default(), &mut f)
        .unwrap();
    let mid = m.midsagittal();
    assert_eq!(
        mid.iter().find(|x| x.0 == "tongue").unwrap().1,
        m.midline.tongue
    );
    for c in f.slices.iter().take(f.area_fn.s.len()) {
        for i in 0..c.rows.len() / 2 {
            let a = &c.rows[i];
            let b = &c.rows[c.rows.len() - 1 - i];
            assert_eq!(a.count, b.count);
            for j in 0..a.count {
                assert!((a.intervals[j].lo - b.intervals[j].lo).abs() < 1e-10);
                assert!((a.intervals[j].hi - b.intervals[j].hi).abs() < 1e-10);
            }
        }
    }
}
