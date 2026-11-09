//! The human tract's losses and soft tissues: the evanescent-mode inertance at abrupt area
//! changes, the thermal boundary layer, tissue walls by region and the soft palate's membrane
//! (docs/tract3d/model-design.md §0, "Acoustics").
use chimera::airway::frequency::{find_peaks, input_impedance, linspace, radiator_transfer, transfer_function};
use chimera::airway::network::{AirwayNetwork, Area, Tube, TubeSpec, discontinuity_inertance};
use chimera::anatomy::human::BODY_MALE;
use chimera::anatomy::tract3d::posture;
use chimera::anatomy::human::BodyMix;
use chimera::math::{PI, sin, sqrt};
use chimera::physics::{RHO, TISSUE_VELUM, WallParams};
use chimera::voice::human_airway::HumanAirway;

/// Two tubes, back then front, closed at the back and radiating at the front.
fn two_tubes(back: f64, front: f64, discontinuity: bool, thermal: bool) -> (AirwayNetwork, Tube) {
    let mut net = AirwayNetwork::new(96000.0);
    net.discontinuity = discontinuity;
    net.thermal = thermal;
    let b = net.add_tube(TubeSpec::new("back", 32, 8.0, Area::Uniform(back)));
    let f = net.add_tube(TubeSpec::new("front", 32, 8.0, Area::Uniform(front)));
    net.connect(b.last_node(), f.first_node());
    net.add_radiation(f.last_node());
    net.finalize();
    (net, b)
}

fn first_peak(net: &AirwayNetwork, inject: usize) -> (f64, f64) {
    let fr = linspace(60.0, 1500.0, 14401);
    let h = transfer_function(&net.linear_elements(), inject, &fr, &[]);
    let p = find_peaks(&fr, &h.mag, 60.0)[0];
    (p.freq, p.bw)
}

#[test]
fn discontinuity_inertance_follows_the_step_formula_and_vanishes_for_gentle_steps() {
    assert_eq!(discontinuity_inertance(1.0, 1.0), 0.0);
    assert_eq!(discontinuity_inertance(1.0, 1.05), 0.0, "a 10 % change of area is a taper, not a step");
    // a narrow neck into a wide cavity: δ → 0.82 r, as the inertance ρδ/(πr²)
    let (s, l) = (sqrt(0.5), sqrt(500.0));
    let r = s / sqrt(PI);
    let expected = RHO * 0.82 * r * (1.0 - 1.35 * (s / l) + 0.31 * (s / l).powi(3)) / (PI * r * r);
    assert!((discontinuity_inertance(s, l) / expected - 1.0).abs() < 1e-12);
    assert_eq!(discontinuity_inertance(s, l), discontinuity_inertance(l, s), "symmetric in its arguments");
}

#[test]
fn a_neck_into_a_cavity_resonates_lower_with_the_step_correction_and_a_uniform_tube_does_not() {
    // the /i/-like two-tube of Birkholz et al. (2022): 5.66 cm² behind, 0.5 cm² in front
    let (plain, b) = two_tubes(5.66, 0.5, false, false);
    let (corrected, _) = two_tubes(5.66, 0.5, true, false);
    let (f0, _) = first_peak(&plain, b.first_node());
    let (f1, _) = first_peak(&corrected, b.first_node());
    // the front tube's effective length grows by δ = 0.82·r·(1 − 1.35α + 0.31α³) ≈ 0.2 cm of 8 cm
    let ratio = f1 / f0;
    assert!(ratio > 0.975 && ratio < 0.997, "F1 {f0:.1} → {f1:.1} Hz");
    let (u0, ub) = two_tubes(3.0, 3.0, false, false);
    let (u1, _) = two_tubes(3.0, 3.0, true, false);
    assert_eq!(first_peak(&u0, ub.first_node()).0, first_peak(&u1, ub.first_node()).0);
}

#[test]
fn thermal_loss_widens_a_rigid_tube_resonance() {
    let (plain, b) = two_tubes(3.0, 3.0, false, false);
    let (warm, _) = two_tubes(3.0, 3.0, false, true);
    let (f0, b0) = first_peak(&plain, b.first_node());
    let (f1, b1) = first_peak(&warm, b.first_node());
    assert!((f1 / f0 - 1.0).abs() < 0.002, "a shunt loss barely moves the resonance");
    // the thermal layer adds about half the viscous loss (Kirchhoff), less the radiation's share
    assert!(b1 > 1.1 * b0 && b1 < 1.8 * b0, "B1 {b0:.2} → {b1:.2} Hz");
}

/// Tube A (closed, driven) and tube B (radiating), joined only by a membrane.
fn partition(area: f64) -> (AirwayNetwork, Tube, Tube) {
    let mut net = AirwayNetwork::new(96000.0);
    let a = net.add_tube(TubeSpec::new("a", 24, 10.0, Area::Uniform(4.0)));
    let b = net.add_tube(TubeSpec::new("b", 24, 10.0, Area::Uniform(2.0)));
    net.add_radiation(b.last_node());
    net.add_membrane(a.last_node(), b.first_node(), area, TISSUE_VELUM);
    net.finalize();
    (net, a, b)
}

#[test]
fn a_membrane_passes_sound_between_unconnected_tubes_and_the_time_domain_agrees() {
    let f = 400.0;
    let (mut net, a, _) = partition(5.0);
    let h = radiator_transfer(&net.linear_elements(), a.first_node(), &[f], &[])[0][0];
    assert!(h > 1e-3, "sound passes the membrane: |H| = {h}");
    let (none, an, _) = partition(0.0);
    assert_eq!(radiator_transfer(&none.linear_elements(), an.first_node(), &[f], &[])[0][0], 0.0, "no membrane, no path");
    // FDTD: a sine of volume velocity into tube A; the radiated flow's steady amplitude
    net.begin_block(1);
    let fs = net.fs;
    let (mut peak, n) = (0.0f64, (fs * 0.6) as usize);
    for i in 0..n {
        net.ext_flow[a.first_node()] = sin(2.0 * PI * f * i as f64 / fs);
        net.step();
        if i > n / 2 {
            peak = peak.max(net.radiated_flow(0).abs());
        }
    }
    assert!((peak / h - 1.0).abs() < 0.05, "FDTD {peak:.5} against the linear analysis {h:.5}");
}

#[test]
fn the_human_tract_walls_resonate_where_people_measured_them() {
    // through the sealed lips with the glottis closed: a tissue resonance R0 ≈ 20 Hz (impedance
    // minimum) and an anti-resonance R̄0 ≈ 200 Hz (Hanna, Smith & Wolfe 2016)
    let mut air = HumanAirway::new(96000.0, BODY_MALE);
    air.apply(&posture("ə", BodyMix::MALE), 0.0);
    let mut el = air.net.linear_elements();
    let lip = air.oral.last_node();
    el.rad_nodes.retain(|n| *n != lip);
    el.rad_area.truncate(el.rad_nodes.len());
    let fr = linspace(10.0, 400.0, 3901);
    let z: Vec<f64> = input_impedance(&el, lip, &fr, &[air.glottis_port]).iter().map(|(r, i)| (r * r + i * i).sqrt()).collect();
    let arg = |lo: f64, hi: f64, max: bool| {
        let idx = (0..fr.len()).filter(|&i| fr[i] >= lo && fr[i] <= hi);
        let i = if max { idx.max_by(|&a, &b| z[a].total_cmp(&z[b])) } else { idx.min_by(|&a, &b| z[a].total_cmp(&z[b])) };
        fr[i.unwrap()]
    };
    let (r0, rb0) = (arg(10.0, 100.0, false), arg(100.0, 400.0, true));
    assert!((15.0..35.0).contains(&r0), "R0 {r0} Hz");
    assert!((180.0..240.0).contains(&rb0), "R̄0 {rb0} Hz");
}

#[test]
fn high_vowels_reach_the_nose_through_the_closed_soft_palate_and_open_vowels_barely() {
    // transpalatal transmission: with the port shut, the soft palate's vibration still drives the
    // nasal cavity, more for close vowels (small mouth, high oral pressure) than for /a/
    let mut air = HumanAirway::new(96000.0, BODY_MALE);
    let fr = linspace(150.0, 1200.0, 2101);
    let mut at_f1 = |v: &str| {
        air.apply(&posture(v, BodyMix::MALE), 0.0);
        let el = air.net.linear_elements();
        let rad = radiator_transfer(&el, air.pharynx.first_node(), &fr, &[air.glottis_port]);
        let total: Vec<f64> = (0..fr.len()).map(|i| rad[0][i] + rad[1][i]).collect();
        let p = find_peaks(&fr, &total, 150.0)[0];
        let i = fr.iter().position(|f| *f >= p.freq).unwrap();
        20.0 * (rad[1][i] / rad[0][i]).log10()
    };
    // (the model's own numbers: no measurement of this path was found to set a tighter line)
    let (i, a) = (at_f1("i"), at_f1("a"));
    assert!((-35.0..-15.0).contains(&i), "/i/ nose/lips {i:.1} dB");
    assert!(a < i - 20.0, "/a/ {a:.1} dB against /i/ {i:.1} dB");
    // and none of it without the membrane
    air.net.set_membrane(air.velum_membrane, 0.0, WallParams { mass: 1.0, resistance: 1.0, stiffness: 1.0 });
    let el = air.net.linear_elements();
    let rad = radiator_transfer(&el, air.pharynx.first_node(), &fr, &[air.glottis_port]);
    assert!(rad[1].iter().all(|x| *x < 1e-9));
}
