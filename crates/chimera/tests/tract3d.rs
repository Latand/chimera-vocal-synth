//! The 3D tract's contract: the midline is the engine's, cuts and point queries agree, shapes
//! stay valid at every parameter extreme, nothing allocates once warm, small articulator moves
//! make small area changes, and the presets show the tongue shapes the research lists.
use chimera::anatomy::geom::v;
use chimera::anatomy::human::{ArticKey, BodyMix, HumanVocalTract, BODY_CHILD, BODY_FEMALE, BODY_MALE, NEUTRAL};
use chimera::anatomy::tract3d::{check_presets, presets, Cut, Map, ShapeParams, Tract3d, Tract3dFrame, SPECS};
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
fn template_map_is_the_engines() {
    for body in [BODY_MALE, BODY_FEMALE, BODY_CHILD] {
        let t = HumanVocalTract::new(body);
        let m = Map { k: body.scale, ph: body.pharynx };
        for x in [-2., 0., 0.4, 0.6, 3., 7.5] {
            for y in [-1., 3., 6.9, 7.1, 10.] {
                let (a, b) = (t.p(x, y), m.p(x, y));
                assert!((a.x - b.x).abs() < 1e-12 && (a.y - b.y).abs() < 1e-12);
                let back = m.un_p(a);
                assert!((back.x - x).abs() < 1e-9 && (back.y - y).abs() < 1e-9);
            }
        }
    }
}

#[test]
fn frame_parts_follow_the_engines_outline_layout() {
    // parts.rs reads the outlines by position; a change to how human.rs builds them must fail here
    let mut m = Tract3d::new(BODY_MALE);
    let mut f = Tract3dFrame::default();
    for p in presets(BodyMix::MALE) {
        m.compute(&p.artic, &p.shape, &mut f).unwrap();
        let (mid, parts) = (&m.midline, &m.parts);
        assert_eq!(*parts.dorsal.last().unwrap(), mid.tip, "{}", p.name);
        assert_eq!(parts.lower_front[0], mid.lower_teeth[0], "{}", p.name);
        assert_eq!(parts.lower_front[2], mid.lower_teeth[2], "{}", p.name);
        assert_eq!(parts.upper_lip[0], mid.upper_lip[0], "{}", p.name);
        assert_eq!(*parts.palate.last().unwrap(), mid.upper_teeth[2], "{}", p.name);
        assert_eq!(parts.epi, [mid.epiglottis[0], mid.epiglottis[1]], "{}", p.name);
        let n = mid.tongue.len();
        assert_eq!(*parts.under.last().unwrap(), mid.tongue[n - 3], "frenulum, {}", p.name);
    }
}

#[test]
fn midsagittal_lamina_is_the_engines_outline() {
    // at z = 0 every lateral rule is the identity: the tongue polygon carries the engine's own
    // inner-wall points and the envelope its outer wall, unchanged
    let mut m = Tract3d::new(BODY_MALE);
    let mut f = Tract3dFrame::default();
    for p in presets(BodyMix::MALE).iter().filter(|p| p.shape.groove_root == 0. && p.shape.groove_back == 0. && p.shape.groove_dorsum == 0. && p.shape.groove_blade == 0. && p.shape.groove_tip == 0.) {
        m.compute(&p.artic, &p.shape, &mut f).unwrap();
        if m.shape.groove_blade + m.shape.groove_tip + m.shape.groove_dorsum + m.shape.groove_back + m.shape.groove_root != 0. {
            continue; // derive grooved it
        }
        let mid = m.midsagittal();
        let tongue = &mid.iter().find(|x| x.0 == "tongue").unwrap().1;
        let inner = &m.midline.inner;
        let ni = inner.len();
        assert_eq!(&tongue[..ni - 10], &inner[..ni - 10], "{}", p.name);
        let envelope = &mid.iter().find(|x| x.0 == "envelope").unwrap().1;
        let outer = &m.midline.outer;
        let no = outer.len();
        // the velum and palate (the wall is clipped at the laryngeal inlet)
        let i = envelope.iter().position(|q| *q == outer[no - 51]).expect("velum tip in the envelope");
        assert_eq!(&envelope[i..i + 44], &outer[no - 51..no - 7], "{}", p.name);
    }
}

#[test]
fn presets_are_finite_and_meshes_keep_their_vertex_counts() {
    for (body, mix) in [(BODY_MALE, BodyMix::MALE), (BODY_FEMALE, BodyMix::FEMALE), (BODY_CHILD, BodyMix::CHILD)] {
        let mut m = Tract3d::new(body);
        let mut f = Tract3dFrame::default();
        let mut counts: Option<Vec<(usize, usize, usize)>> = None;
        for p in presets(mix) {
            m.compute(&p.artic, &p.shape, &mut f).unwrap();
            let af = &f.area_fn;
            assert!(af.length > 8. && af.length < 30., "{} {}", p.name, af.length);
            assert!(af.volume.is_finite() && af.volume > 0.);
            assert!(af.area.iter().all(|a| a.is_finite() && *a >= 0.));
            let meshes = m.meshes();
            assert_eq!(meshes.len(), 8);
            for mesh in &meshes {
                assert!(mesh.positions.iter().all(|x| x.is_finite()), "{} {}", p.name, mesh.element);
                assert!(mesh.indices.iter().chain(&mesh.lines).all(|&i| (i as usize) * 3 < mesh.positions.len()));
            }
            let c: Vec<_> = meshes.iter().map(|x| (x.positions.len(), x.indices.len(), x.lines.len())).collect();
            match &counts {
                None => counts = Some(c),
                Some(prev) => assert_eq!(prev, &c, "{}: the browser updates meshes in place", p.name),
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
fn every_parameter_extreme_and_articulator_extreme_gives_valid_cuts() {
    let mut m = Tract3d::new(BODY_MALE);
    let mut f = Tract3dFrame::default();
    for spec in SPECS {
        for value in [spec.min, spec.max] {
            let p: ShapeParams = serde_json::from_value(serde_json::json!({spec.key: value})).unwrap();
            m.compute(&NEUTRAL, &p, &mut f).unwrap();
            assert!(f.area_fn.area.iter().all(|x| x.is_finite() && *x >= 0.), "{}", spec.key);
            assert!(m.meshes().iter().all(|mesh| mesh.positions.iter().all(|x| x.is_finite())), "{}", spec.key);
        }
    }
    for key in ArticKey::ALL {
        let spec = chimera::control::articulators::artic_spec(key);
        for value in [spec.min, spec.max] {
            let mut a = NEUTRAL;
            a.set(key, value);
            m.compute(&a, &ShapeParams::default(), &mut f).unwrap();
            assert!(f.area_fn.volume.is_finite(), "{}", key.name());
        }
    }
}

#[test]
fn cuts_and_point_queries_read_the_same_air() {
    let mut m = Tract3d::new(BODY_MALE);
    let mut f = Tract3dFrame::default();
    for p in presets(BodyMix::MALE) {
        m.compute(&p.artic, &p.shape, &mut f).unwrap();
        let mut c = Cut::default();
        for (o, u) in [(v(0., 3.), v(1., 0.)), (v(4.5, 8.), v(0., 1.)), (v(3., 7.5), v(0.6, 0.8))] {
            m.cut_plane(o, u, &mut c);
            for row in &c.rows {
                for q in row.intervals.iter().take(row.count) {
                    if q.hi - q.lo < 0.02 {
                        continue;
                    }
                    let a = 0.5 * (q.lo + q.hi);
                    assert!(m.air(c.origin.x + c.u.x * a, c.origin.y + c.u.y * a, row.z), "{} at z {}", p.name, row.z);
                }
            }
        }
    }
}

#[test]
fn symmetric_shapes_cut_symmetric_and_asymmetry_breaks_it() {
    let mut m = Tract3d::new(BODY_MALE);
    let mut f = Tract3dFrame::default();
    m.compute(&NEUTRAL, &ShapeParams::default(), &mut f).unwrap();
    for c in f.cuts() {
        let n = c.rows.len();
        for i in 0..n / 2 {
            let (a, b) = (&c.rows[i], &c.rows[n - 1 - i]);
            assert_eq!(a.count, b.count);
            for j in 0..a.count {
                assert!((a.intervals[j].lo - b.intervals[j].lo).abs() < 1e-10);
            }
        }
    }
    let p = ShapeParams { side_asym: 0.5, side_blade: -0.8, ..ShapeParams::default() };
    m.compute(&NEUTRAL, &p, &mut f).unwrap();
    let differs = f.cuts().iter().any(|c| {
        let n = c.rows.len();
        (0..n / 2).any(|i| c.rows[i].count != c.rows[n - 1 - i].count || (0..c.rows[i].count).any(|j| (c.rows[i].intervals[j].lo - c.rows[n - 1 - i].intervals[j].lo).abs() > 1e-6))
    });
    assert!(differs);
}

#[test]
fn small_articulator_steps_do_not_open_large_area_jumps() {
    let mut m = Tract3d::new(BODY_MALE);
    let mut f = Tract3dFrame::default();
    for p in presets(BodyMix::MALE).into_iter().filter(|p| ["ə", "s", "l", "a", "i"].contains(&p.name.as_str())) {
        for key in [ArticKey::TbX, ArticKey::TbY, ArticKey::TtY, ArticKey::Jaw, ArticKey::Lateral, ArticKey::LipAp, ArticKey::Larynx] {
            let mut previous: Vec<f64> = Vec::new();
            for i in 0..41 {
                let mut a = p.artic;
                a.set(key, p.artic.get(key) + (i as f64 - 20.) * 0.002);
                m.compute(&a, &p.shape, &mut f).unwrap();
                if previous.len() == f.area_fn.weighted_area.len() {
                    for (j, (x, y)) in previous.iter().zip(&f.area_fn.weighted_area).enumerate() {
                        assert!((x - y).abs() < 0.5, "{} {} step {i} cut {j} (s {:.2}): {x:.3} -> {y:.3}", p.name, key.name(), f.area_fn.s[j]);
                    }
                }
                previous.clone_from(&f.area_fn.weighted_area);
            }
        }
    }
}

#[test]
fn the_presets_make_the_tongue_shapes_the_research_lists() {
    // validation plan T-features and T-anatomy, male template
    let checks = check_presets(BodyMix::MALE);
    assert!(checks.len() >= 17);
    let failed: Vec<_> = checks.iter().filter(|c| !c.pass).map(|c| format!("{} / {}: {}", c.preset, c.feature, c.measured)).collect();
    assert!(failed.is_empty(), "{failed:#?}");
}

/// Review finding: the bunched /ɹ/ pressed its dorsum into the palate and shut the tract over
/// 2 cm, and the feature check passed it (a closure is "below 60 %" of anything).
#[test]
fn bunched_r_is_an_open_approximant() {
    let all = presets(BodyMix::MALE);
    let p = all.iter().find(|p| p.name == "bunched").unwrap();
    let mut m = Tract3d::new(BODY_MALE);
    let mut f = Tract3dFrame::default();
    m.compute(&p.artic, &p.shape, &mut f).unwrap();
    for c in f.cuts().iter().filter(|c| c.s >= 0.5) {
        assert!(c.channels >= 1 && c.area >= 0.15, "closed or nearly closed at s = {:.2} cm: {:.3} cm², {} channel(s)", c.s, c.area, c.channels);
    }
    // the check itself refuses the posture that used to pass
    let mut shut = p.artic;
    shut.tb_x = 2.6;
    shut.tb_y = 8.0;
    m.compute(&shut, &p.shape, &mut f).unwrap();
    let mut checks = Vec::new();
    chimera::anatomy::tract3d::check_one(&m, &f, "bunched", &p.shape, 0.9, BODY_MALE.scale, &mut checks);
    let open = checks.iter().find(|c| c.feature == "open along its whole length").expect("the open-path check");
    assert!(!open.pass, "a closed tract passed as a bunched /ɹ/: {}", open.measured);
}

/// Review finding: the lips' opening could not get narrower than 1.1 cm, so rounded vowels were
/// fitted with a wide, thin slit. `lipRound` draws the corners in.
#[test]
fn lip_rounding_narrows_the_opening() {
    let all = presets(BodyMix::MALE);
    let u = all.iter().find(|p| p.name == "u").unwrap();
    let mut m = Tract3d::new(BODY_MALE);
    let mut f = Tract3dFrame::default();
    m.compute(&u.artic, &ShapeParams::default(), &mut f).unwrap();
    let wide = f.cuts().last().unwrap().width;
    let floor = m.parts.lip_w;
    m.compute(&u.artic, &ShapeParams { lip_round: 1., ..ShapeParams::default() }, &mut f).unwrap();
    let narrow = f.cuts().last().unwrap().width;
    assert!((m.parts.lip_w / floor - 0.35).abs() < 1e-9, "{} vs {floor}", m.parts.lip_w);
    assert!(narrow < 0.6 * wide, "the last cut is {narrow} cm wide with the lips rounded, {wide} without");
    // without it nothing changes: the default leaves the engine's own opening
    assert_eq!(ShapeParams::default().lip_round, 0.);
}
