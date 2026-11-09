//! Named demonstrations combine existing midline articulators with lateral shape offsets.
use super::super::human::{BodyMix, NEUTRAL};
use super::*;
use crate::phonetics::{inventory, planner};
#[derive(Clone, Debug, Serialize)]
pub struct Preset {
    pub name: String,
    pub artic: HumanArtic,
    pub shape: ShapeParams,
}
pub fn presets(body: BodyMix) -> Vec<Preset> {
    let mut out = Vec::new();
    for name in [
        "flat",
        "a",
        "i",
        "u",
        "ə",
        "s",
        "ʃ",
        "l",
        "f",
        "ç",
        "x",
        "funnel",
        "cupped",
        "bunched",
        "retroflex",
    ] {
        let a = planner::vowel_artic(name, body)
            .map(|p| p.apply_to(&NEUTRAL))
            .unwrap_or(NEUTRAL);
        let mut a = a;
        if let Some(c) = inventory::phoneme(name, body) {
            a = c.targets.apply_to(&a);
        }
        let mut p = ShapeParams::default();
        match name {
            "flat" => {}
            "s" => {
                p.groove_blade = 0.3;
                p.groove_tip = 0.25;
                p.groove_width = 0.6;
                p.side_dorsum = 0.3;
            }
            "funnel" => {
                a.tt_y = 8.4;
                p.groove_back = 0.1;
                p.groove_dorsum = 0.25;
                p.groove_blade = 0.4;
                p.groove_tip = 0.5;
                p.groove_taper = 0.7;
                p.side_dorsum = 0.4;
            }
            "cupped" | "ʃ" => {
                p.side_back = 0.3;
                p.side_dorsum = 0.5;
                p.side_blade = 0.4;
                p.groove_blade = 0.15;
            }
            "l" => {
                a.lateral = 1.;
                p.side_asym = 0.15;
            }
            "bunched" => {
                a.tb_y = 6.9;
                a.tb_x = 3.4;
                a.tt_y = 6.6;
                a.root = -0.8;
                p.side_back = 0.6;
                p.side_dorsum = 0.5;
                p.groove_root = 0.2;
                p.groove_back = 0.25;
                p.side_blade = -0.2;
            }
            "retroflex" => {
                a.tt_y = 8.3;
                a.tt_x = 5.4;
                a.tt_curl = 0.9;
                a.root = -0.6;
                p.tip_width = 0.35;
                p.side_blade = -0.3;
            }
            _ => {}
        }
        out.push(Preset {
            name: name.into(),
            artic: a,
            shape: p,
        });
    }
    out
}
