//! Named postures: the engine's calibrated targets for a sound (so `derive` supplies the 3D
//! shape the articulators imply) plus 3D offsets for the shapes the research lists
//! (docs/tract3d/model-design.md §4.1). The lab's preset chips and the feature checks use these.
use super::super::human::{BodyMix, NEUTRAL};
use super::*;
use crate::phonetics::{inventory, planner};

#[derive(Clone, Debug, Serialize)]
pub struct Preset {
    pub name: String,
    pub label: &'static str,
    pub artic: HumanArtic,
    pub shape: ShapeParams,
}

pub const PRESET_NAMES: [(&str, &str); 15] = [
    ("flat", "flat tongue"),
    ("a", "open vowel /a/"),
    ("i", "close front /i/"),
    ("u", "close back rounded /u/"),
    ("ə", "neutral /ə/"),
    ("s", "grooved /s/"),
    ("ʃ", "domed /ʃ/"),
    ("l", "lateral /l/"),
    ("f", "labiodental /f/"),
    ("ç", "palatal /ç/"),
    ("x", "velar /x/"),
    ("funnel", "funnel groove"),
    ("cupped", "cupped tongue"),
    ("bunched", "bunched /ɹ/"),
    ("retroflex", "retroflex /ɹ/"),
];

/// The engine's target posture for a vowel or consonant symbol (vowel table, then the
/// consonant inventory's targets on top), with the tip resting where the engine rests it.
pub fn posture(symbol: &str, body: BodyMix) -> HumanArtic {
    let mut a = planner::vowel_artic(symbol, body).map_or(NEUTRAL, |p| p.apply_to(&NEUTRAL));
    let tip_set = inventory::phoneme(symbol, body).is_some_and(|c| c.targets.get(ArticKey::TtX).is_some() || c.targets.get(ArticKey::TtY).is_some());
    if let Some(c) = inventory::phoneme(symbol, body) {
        a = c.targets.apply_to(&a);
    }
    if !tip_set {
        let tip = super::super::human::rest_tip(a.jaw, a.tb_x, a.tb_y);
        a.tt_x = tip.x;
        a.tt_y = tip.y;
    }
    a
}

pub fn presets(body: BodyMix) -> Vec<Preset> {
    let mut out = Vec::new();
    for (name, label) in PRESET_NAMES {
        let mut a = posture(name, body);
        let mut p = ShapeParams::default();
        match name {
            "flat" => {
                a = posture("ə", body);
                // cancel what derive adds, and square the margins
                p.edge_round = 4.;
            }
            "s" => {
                // jaw nearly closed, tip forward under the ridge, margins braced on the teeth:
                // the air leaves along a narrow midline groove (tuned with `chimera tract3d tune`)
                a.jaw = 0.14;
                a.tt_x = 6.98;
                a.tt_y = 9.25;
                p.groove_blade = 0.1;
                p.groove_tip = 0.11;
                p.groove_width = 0.5;
                p.side_blade = 0.3;
                p.side_tip = 0.3;
            }
            "ʃ" => {
                // blade domed close under the palate's vault: a long thin arched channel
                a.tb_x = 3.06;
                a.tb_y = 7.6;
                a.tt_x = 5.76;
                a.tt_y = 10.15;
                a.jaw = 0.31;
                p.side_back = -0.06;
                p.side_blade = -0.23;
                p.side_tip = -0.25;
            }
            "l" => {
                // tip sealed on the ridge, margins lowered: air passes on both sides
                a.tt_y = 9.8;
                p.side_asym = 0.15;
            }
            "funnel" => {
                // a groove deepening and narrowing from the back towards the tip
                a = posture("ə", body);
                a.tb_y += 0.4;
                p.groove_root = 0.;
                p.groove_back = 0.1;
                p.groove_dorsum = 0.2;
                p.groove_blade = 0.3;
                p.groove_tip = 0.38;
                p.groove_width = 0.9;
                p.groove_taper = 0.8;
            }
            "cupped" => {
                a = posture("ə", body);
                p.side_back = 0.4;
                p.side_dorsum = 0.7;
                p.side_blade = 0.6;
            }
            "bunched" => {
                // dorsum bunched under the palate, tip low, root drawn back (concave posterior)
                a = posture("ə", body);
                a.tb_x = 2.6;
                a.tb_y = 8.0;
                a.root = -0.7;
                let tip = super::super::human::rest_tip(a.jaw, a.tb_x, a.tb_y);
                a.tt_x = tip.x;
                a.tt_y = tip.y;
                a.lip_prot = 0.5;
                a.lip_width = 0.3;
                p.side_back = 0.4;
                p.side_dorsum = 0.3;
                p.groove_root = 0.2;
                p.groove_back = 0.2;
                p.side_blade = -0.3;
            }
            "retroflex" => {
                a = posture("ɹ", body);
                a.tb_y = 7.17;
                a.tt_x = 5.8;
                a.tt_y = 10.;
                p.tip_width = 0.4;
                p.side_blade = -0.3;
            }
            _ => {}
        }
        out.push(Preset { name: name.into(), label, artic: a, shape: p });
    }
    out
}
