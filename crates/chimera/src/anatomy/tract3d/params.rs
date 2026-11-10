//! The 3D-only parameters: one table drives the struct, its JSON names, the lab's controls
//! (via `SPECS`) and the clamping of offsets, so ranges cannot drift apart.
use super::super::human::HumanArtic;
use crate::math::smoothstep;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ParamSpec {
    pub key: &'static str,
    pub element: &'static str,
    pub unit: &'static str,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub doc: &'static str,
}

macro_rules! shape_params {
    ($($field:ident, $key:literal, $element:literal, $unit:literal, $min:expr, $max:expr, $default:expr, $doc:literal;)*) => {
        /// 3D shape parameters (cm unless noted). In JSON (camelCase) they are offsets from
        /// `derive()`; unknown names are refused.
        #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase", default, deny_unknown_fields)]
        pub struct ShapeParams {
            $(pub $field: f64,)*
        }
        impl Default for ShapeParams {
            fn default() -> Self {
                Self { $($field: $default,)* }
            }
        }
        pub const SPECS: &[ParamSpec] = &[
            $(ParamSpec { key: $key, element: $element, unit: $unit, min: $min, max: $max, default: $default, doc: $doc },)*
        ];
        impl ShapeParams {
            /// `self` (derived values) plus offsets measured from the defaults, clamped to range.
            pub fn with_offsets(self, offsets: &Self) -> Result<Self, String> {
                let d = Self::default();
                let mut out = self;
                $(
                    if !offsets.$field.is_finite() {
                        return Err(concat!($key, " must be finite").into());
                    }
                    out.$field = (out.$field + offsets.$field - d.$field).clamp($min, $max);
                )*
                Ok(out)
            }
            pub fn get(&self, key: &str) -> Option<f64> {
                match key {
                    $($key => Some(self.$field),)*
                    _ => None,
                }
            }
            pub fn set(&mut self, key: &str, value: f64) -> bool {
                match key {
                    $($key => { self.$field = value; true })*
                    _ => false,
                }
            }
        }
    };
}

shape_params! {
    side_root, "sideRoot", "tongue", "cm", -1.4, 1.4, 0.0, "Root margins: + towards the pharyngeal wall (concave posterior body), − away";
    side_back, "sideBack", "tongue", "cm", -1.4, 1.4, 0.0, "Back margins: + raised (cupped, braced), − lowered";
    side_dorsum, "sideDorsum", "tongue", "cm", -1.4, 1.4, 0.0, "Dorsum margins: + raised against the molars, − lowered";
    side_blade, "sideBlade", "tongue", "cm", -1.4, 1.4, 0.0, "Blade margins: + braced on the teeth, − lowered (lateral channels)";
    side_tip, "sideTip", "tongue", "cm", -1.4, 1.4, 0.0, "Tip margins: + raised, − lowered";
    groove_root, "grooveRoot", "tongue", "cm", 0.0, 0.8, 0.0, "Midline groove depth at the root";
    groove_back, "grooveBack", "tongue", "cm", 0.0, 0.8, 0.0, "Midline groove depth at the back";
    groove_dorsum, "grooveDorsum", "tongue", "cm", 0.0, 0.8, 0.0, "Midline groove depth at the dorsum";
    groove_blade, "grooveBlade", "tongue", "cm", 0.0, 0.8, 0.0, "Midline groove depth at the blade";
    groove_tip, "grooveTip", "tongue", "cm", 0.0, 0.8, 0.0, "Midline groove depth at the tip";
    groove_width, "grooveWidth", "tongue", "cm", 0.3, 2.0, 0.8, "Groove width at half depth, at the blade";
    groove_taper, "grooveTaper", "tongue", "", -1.0, 1.0, 0.0, "Groove narrowing towards the tip (+: a funnel)";
    side_asym, "sideAsym", "tongue", "", -1.0, 1.0, 0.0, "Right minus left margin elevation (unequal lateral channels)";
    tongue_width, "tongueWidth", "tongue", "×", 0.8, 1.2, 1.0, "Tongue width, relative to the template";
    tip_width, "tipWidth", "tongue", "cm", 0.3, 1.5, 0.55, "Half-width at the tip: apical (narrow) or laminal (broad)";
    edge_round, "edgeRound", "tongue", "", 1.5, 4.0, 2.0, "Margin rounding: low rounded, high square";
    vault_depth, "vaultDepth", "palate", "cm", 0.5, 2.5, 1.4, "How far the roof may drop from the midline to the gum line at the molars";
    arch_width, "archWidth", "palate", "cm", 1.4, 2.6, 1.9, "Inner half-width of the upper dental arch at the molars";
    vault_shape, "vaultShape", "palate", "", 1.5, 6.0, 2.5, "Vault profile: low a dome, high flat with steep sides";
    arch_taper, "archTaper", "palate", "", 0.0, 1.0, 0.2, "How the arches narrow towards the incisors: 0 square, 1 V-shaped";
    velum_width, "velumWidth", "velum", "cm", 1.5, 3.5, 2.6, "Soft palate width at its root";
    uvula_length, "uvulaLength", "velum", "cm", 0.5, 1.5, 1.0, "Uvula: length of the lobe hanging from the middle of the soft palate";
    uvula_width, "uvulaWidth", "velum", "cm", 0.4, 1.2, 0.8, "Uvula: width of the lobe";
    pharynx_width_naso, "pharynxWidthNaso", "pharynx", "cm", 0.8, 3.0, 1.7, "Upper pharynx: half-width at which the side walls have come forward to the back teeth (the air ends sooner, where they meet the tongue or the soft palate)";
    pharynx_width_oro, "pharynxWidthOro", "pharynx", "cm", 0.8, 3.0, 1.9, "Oropharynx: half-width of the ring of walls";
    pharynx_width_hypo, "pharynxWidthHypo", "pharynx", "cm", 0.8, 3.0, 2.4, "Hypopharynx: half-width of the ring of walls";
    pharynx_shape, "pharynxShape", "pharynx", "", 2.0, 4.0, 2.2, "The ring's outline: 2 an ellipse, high flat at the back with square corners";
    lateral_squeeze, "lateralSqueeze", "pharynx", "", 0.0, 1.0, 0.0, "Lateral walls drawn inwards (pharyngeal constriction)";
    epilarynx_width, "epilarynxWidth", "larynx", "cm", 0.3, 1.0, 0.6, "Size of the laryngeal tube: its sections' area against the voice's own (0.6: the same), taken up sideways";
    epiglottis_width, "epiglottisWidth", "larynx", "cm", 0.6, 1.4, 1.0, "Half-width of the epiglottis";
    epiglottis_curl, "epiglottisCurl", "larynx", "1/cm", 0.0, 1.0, 0.2, "Backward curl of the epiglottis' sides";
    ary_fold_height, "aryFoldHeight", "larynx", "cm", 0.0, 1.5, 0.3, "How far the aryepiglottic folds stand above the hypopharynx's floor, between the laryngeal inlet and each piriform fossa";
    piriform_depth, "piriformDepth", "larynx", "cm", 1.0, 2.5, 1.8, "Depth of the piriform fossae below the hypopharynx's floor";
    piriform_width, "piriformWidth", "larynx", "cm", 0.3, 1.0, 0.5, "Lateral radius of the piriform fossae at their opening";
    vallecula_depth, "valleculaDepth", "larynx", "cm", 0.3, 1.2, 0.8, "Depth of the valleculae below the epiglottis' tip";
    vallecula_width, "valleculaWidth", "larynx", "cm", 0.3, 1.0, 0.7, "Lateral radius of the valleculae (they end under the epiglottis' edge at the latest)";
    lip_corner_shape, "lipCornerShape", "lips", "", 1.0, 4.0, 2.0, "Mouth opening outline: low a lens (spread), 2 an ellipse, high square";
    lip_thickness, "lipThickness", "lips", "×", 0.6, 1.6, 1.0, "Lip depth front to back";
    lip_round, "lipRound", "lips", "", 0.0, 1.0, 0.0, "Rounding: the corners drawn in beyond what Lip spread gives (a narrower, rounder opening)";
    lower_arch_width, "lowerArchWidth", "teethJaw", "cm", 1.3, 2.5, 1.8, "Inner half-width of the lower dental arch at the molars";
    buccal_gap, "buccalGap", "cheeks", "cm", 0.0, 0.6, 0.0, "Air between the teeth and the cheeks (vestibule)";
    cheek_puff, "cheekPuff", "cheeks", "", 0.0, 1.0, 0.0, "Puffed cheeks: widens the vestibule";
}

/// Constants of `derive`. `chimera tract3d fit-derive` fits them against the DVTD reference
/// (validation-plan.md, T-default); what the fit showed and why these values stand is in
/// docs/tract3d/validation-results.md (the first eight are the starting values; the lips'
/// rounding is what the fitted rounded vowels use; the squeeze is between what the fitted open
/// vowels use and what the median default distance asks for).
pub const DERIVE: [f64; 10] = [0.25, 0.35, 0.3, 0.25, 0.5, 0.9, 0.3, 0.3, 1.0, 0.9];

/// 3D parameters implied by the engine's 17 articulators (design §5), with the default
/// constants.
pub fn derive(a: &HumanArtic) -> ShapeParams {
    derive_with(a, &DERIVE)
}
/// `derive` with explicit constants (for fitting them):
/// [bracing at the back, bracing at the dorsum, sibilant groove, groove at the tip, sibilant
///  side bracing, lateral lowering, curl narrowing, root concavity, lip rounding, pharyngeal
///  squeeze]
pub fn derive_with(a: &HumanArtic, c: &[f64; 10]) -> ShapeParams {
    let mut p = ShapeParams::default();
    let lateral = a.lateral.clamp(0., 1.);
    let curl = a.tt_curl.clamp(0., 1.);
    // a high, fronted body braces its margins against the upper molars (front-raising vowels)
    let high = smoothstep(6.6, 8.2, a.tb_y) * (0.4 + 0.6 * smoothstep(2.4, 3.8, a.tb_x));
    p.side_back = c[0] * high;
    p.side_dorsum = c[1] * high + 0.45 * a.dorsum.clamp(0., 1.) - 0.35 * c[5] * lateral;
    // tip at the alveolar ridge (sibilants): a midline groove between braced margins
    let ridge = smoothstep(8.5, 9.15, a.tt_y) * smoothstep(5.9, 6.3, a.tt_x) * (1. - lateral) * (1. - curl);
    p.groove_blade = c[2] * ridge;
    p.groove_tip = c[3] * ridge;
    p.side_blade = c[4] * ridge - c[5] * lateral - c[6] * curl;
    p.side_tip = c[4] * 0.8 * ridge - c[5] * lateral;
    p.side_root = c[7] * (-a.root).max(0.);
    p.tip_width = 0.55 - 0.2 * curl;
    // spread lips nearly shut meet in a lens with pointed corners; opened, the same lips part in
    // a rounded opening (the lower lip a U; it was a V)
    p.lip_corner_shape = 2.0 - 0.7 * a.lip_width.clamp(0., 1.) + 0.9 * smoothstep(0.5, 1.8, a.lip_ap);
    // rounded lips: protruded and drawn together, the corners come in beyond the engine's own
    // narrowest opening
    // (from the lips' resting protrusion, 0.2, on: lips at rest are not rounded)
    p.lip_round = (c[8] * (1. - a.lip_width.clamp(0., 1.)) * smoothstep(0.2, 0.7, a.lip_prot)).clamp(0., 1.);
    // a low, retracted tongue (open back vowels) comes with the pharynx's side walls drawn in:
    // the constrictors that pull the root back also narrow the ring
    let retracted = ((2.9 - a.tb_x) / 1.4).clamp(0., 1.) * ((7.6 - a.tb_y) / 1.6).clamp(0., 1.) + 0.8 * (-a.root).max(0.);
    p.lateral_squeeze = (c[9] * retracted).clamp(0., 1.);
    p
}
