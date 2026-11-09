//! Parameter metadata is shared by Rust, the browser controls and the fitter.
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
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ShapeParams {
    pub side_root: f64,
    pub side_back: f64,
    pub side_dorsum: f64,
    pub side_blade: f64,
    pub side_tip: f64,
    pub groove_root: f64,
    pub groove_back: f64,
    pub groove_dorsum: f64,
    pub groove_blade: f64,
    pub groove_tip: f64,
    pub groove_width: f64,
    pub groove_taper: f64,
    pub side_asym: f64,
    pub tongue_width: f64,
    pub tip_width: f64,
    pub edge_round: f64,
    pub vault_depth: f64,
    pub arch_width: f64,
    pub vault_shape: f64,
    pub arch_taper: f64,
    pub velum_width: f64,
    pub uvula_length: f64,
    pub uvula_width: f64,
    pub pharynx_width_naso: f64,
    pub pharynx_width_oro: f64,
    pub pharynx_width_hypo: f64,
    pub pharynx_shape: f64,
    pub lateral_squeeze: f64,
    pub epilarynx_width: f64,
    pub epiglottis_width: f64,
    pub epiglottis_curl: f64,
    pub piriform_depth: f64,
    pub piriform_width: f64,
    pub vallecula_depth: f64,
    pub vallecula_width: f64,
    pub lip_corner_shape: f64,
    pub lip_thickness: f64,
    pub lower_arch_width: f64,
    pub tooth_gap: f64,
    pub buccal_gap: f64,
    pub cheek_puff: f64,
}
impl Default for ShapeParams {
    fn default() -> Self {
        Self {
            side_root: 0.0,
            side_back: 0.0,
            side_dorsum: 0.0,
            side_blade: 0.0,
            side_tip: 0.0,
            groove_root: 0.0,
            groove_back: 0.0,
            groove_dorsum: 0.0,
            groove_blade: 0.0,
            groove_tip: 0.0,
            groove_width: 0.8,
            groove_taper: 0.0,
            side_asym: 0.0,
            tongue_width: 1.0,
            tip_width: 0.55,
            edge_round: 2.0,
            vault_depth: 1.4,
            arch_width: 1.9,
            vault_shape: 2.5,
            arch_taper: 0.6,
            velum_width: 2.6,
            uvula_length: 1.0,
            uvula_width: 0.8,
            pharynx_width_naso: 1.5,
            pharynx_width_oro: 1.6,
            pharynx_width_hypo: 1.5,
            pharynx_shape: 2.2,
            lateral_squeeze: 0.0,
            epilarynx_width: 0.6,
            epiglottis_width: 1.0,
            epiglottis_curl: 0.4,
            piriform_depth: 1.8,
            piriform_width: 0.6,
            vallecula_depth: 0.8,
            vallecula_width: 0.7,
            lip_corner_shape: 2.0,
            lip_thickness: 1.0,
            lower_arch_width: 1.8,
            tooth_gap: 0.05,
            buccal_gap: 0.1,
            cheek_puff: 0.0,
        }
    }
}
pub const SPECS: &[ParamSpec] = &[
    ParamSpec {
        key: "sideRoot",
        element: "tongue",
        unit: "cm",
        min: -1.4,
        max: 1.4,
        default: 0.0,
        doc: "Root margin elevation",
    },
    ParamSpec {
        key: "sideBack",
        element: "tongue",
        unit: "cm",
        min: -1.4,
        max: 1.4,
        default: 0.0,
        doc: "Back margin elevation",
    },
    ParamSpec {
        key: "sideDorsum",
        element: "tongue",
        unit: "cm",
        min: -1.4,
        max: 1.4,
        default: 0.0,
        doc: "Dorsum margin elevation",
    },
    ParamSpec {
        key: "sideBlade",
        element: "tongue",
        unit: "cm",
        min: -1.4,
        max: 1.4,
        default: 0.0,
        doc: "Blade margin elevation",
    },
    ParamSpec {
        key: "sideTip",
        element: "tongue",
        unit: "cm",
        min: -1.4,
        max: 1.4,
        default: 0.0,
        doc: "Tip margin elevation",
    },
    ParamSpec {
        key: "grooveRoot",
        element: "tongue",
        unit: "cm",
        min: 0.0,
        max: 0.8,
        default: 0.0,
        doc: "Root groove depth",
    },
    ParamSpec {
        key: "grooveBack",
        element: "tongue",
        unit: "cm",
        min: 0.0,
        max: 0.8,
        default: 0.0,
        doc: "Back groove depth",
    },
    ParamSpec {
        key: "grooveDorsum",
        element: "tongue",
        unit: "cm",
        min: 0.0,
        max: 0.8,
        default: 0.0,
        doc: "Dorsum groove depth",
    },
    ParamSpec {
        key: "grooveBlade",
        element: "tongue",
        unit: "cm",
        min: 0.0,
        max: 0.8,
        default: 0.0,
        doc: "Blade groove depth",
    },
    ParamSpec {
        key: "grooveTip",
        element: "tongue",
        unit: "cm",
        min: 0.0,
        max: 0.8,
        default: 0.0,
        doc: "Tip groove depth",
    },
    ParamSpec {
        key: "grooveWidth",
        element: "tongue",
        unit: "cm",
        min: 0.3,
        max: 2.0,
        default: 0.8,
        doc: "Full width at half depth at blade",
    },
    ParamSpec {
        key: "grooveTaper",
        element: "tongue",
        unit: "",
        min: -1.0,
        max: 1.0,
        default: 0.0,
        doc: "Positive narrows groove towards tip",
    },
    ParamSpec {
        key: "sideAsym",
        element: "tongue",
        unit: "",
        min: -1.0,
        max: 1.0,
        default: 0.0,
        doc: "Right versus left margin elevation",
    },
    ParamSpec {
        key: "tongueWidth",
        element: "tongue",
        unit: "×",
        min: 0.8,
        max: 1.2,
        default: 1.0,
        doc: "Tongue half-width multiplier",
    },
    ParamSpec {
        key: "tipWidth",
        element: "tongue",
        unit: "cm",
        min: 0.3,
        max: 1.5,
        default: 0.55,
        doc: "Tip half-width",
    },
    ParamSpec {
        key: "edgeRound",
        element: "tongue",
        unit: "",
        min: 1.5,
        max: 4.0,
        default: 2.0,
        doc: "Margin superellipse exponent",
    },
    ParamSpec {
        key: "vaultDepth",
        element: "palate",
        unit: "cm",
        min: 0.5,
        max: 2.5,
        default: 1.4,
        doc: "Roof fall from midline to arch",
    },
    ParamSpec {
        key: "archWidth",
        element: "palate",
        unit: "cm",
        min: 1.4,
        max: 2.6,
        default: 1.9,
        doc: "Upper molar arch half-width",
    },
    ParamSpec {
        key: "vaultShape",
        element: "palate",
        unit: "",
        min: 1.5,
        max: 6.0,
        default: 2.5,
        doc: "Palate dome exponent",
    },
    ParamSpec {
        key: "archTaper",
        element: "palate",
        unit: "",
        min: 0.0,
        max: 1.0,
        default: 0.6,
        doc: "Arch narrowing towards incisors",
    },
    ParamSpec {
        key: "velumWidth",
        element: "velum",
        unit: "cm",
        min: 1.5,
        max: 3.5,
        default: 2.6,
        doc: "Velum root full width",
    },
    ParamSpec {
        key: "uvulaLength",
        element: "velum",
        unit: "cm",
        min: 0.5,
        max: 1.5,
        default: 1.0,
        doc: "Uvula length",
    },
    ParamSpec {
        key: "uvulaWidth",
        element: "velum",
        unit: "cm",
        min: 0.4,
        max: 1.2,
        default: 0.8,
        doc: "Uvula full width",
    },
    ParamSpec {
        key: "pharynxWidthNaso",
        element: "pharynx",
        unit: "cm",
        min: 0.8,
        max: 2.2,
        default: 1.5,
        doc: "Upper pharynx half-width",
    },
    ParamSpec {
        key: "pharynxWidthOro",
        element: "pharynx",
        unit: "cm",
        min: 0.8,
        max: 2.4,
        default: 1.6,
        doc: "Oropharynx half-width",
    },
    ParamSpec {
        key: "pharynxWidthHypo",
        element: "pharynx",
        unit: "cm",
        min: 0.8,
        max: 2.4,
        default: 1.5,
        doc: "Hypopharynx half-width",
    },
    ParamSpec {
        key: "pharynxShape",
        element: "pharynx",
        unit: "",
        min: 1.5,
        max: 4.0,
        default: 2.2,
        doc: "Lateral wall superellipse exponent",
    },
    ParamSpec {
        key: "lateralSqueeze",
        element: "pharynx",
        unit: "",
        min: 0.0,
        max: 1.0,
        default: 0.0,
        doc: "Medial lateral-wall squeeze",
    },
    ParamSpec {
        key: "epilarynxWidth",
        element: "larynx",
        unit: "cm",
        min: 0.3,
        max: 1.2,
        default: 0.6,
        doc: "Epilarynx half-width",
    },
    ParamSpec {
        key: "epiglottisWidth",
        element: "larynx",
        unit: "cm",
        min: 0.4,
        max: 1.6,
        default: 1.0,
        doc: "Epiglottis half-width",
    },
    ParamSpec {
        key: "epiglottisCurl",
        element: "larynx",
        unit: "",
        min: 0.0,
        max: 1.0,
        default: 0.4,
        doc: "Epiglottis lateral curvature",
    },
    ParamSpec {
        key: "piriformDepth",
        element: "larynx",
        unit: "cm",
        min: 1.6,
        max: 2.0,
        default: 1.8,
        doc: "Blind fossa depth",
    },
    ParamSpec {
        key: "piriformWidth",
        element: "larynx",
        unit: "cm",
        min: 0.3,
        max: 1.0,
        default: 0.6,
        doc: "Fossa lateral radius",
    },
    ParamSpec {
        key: "valleculaDepth",
        element: "larynx",
        unit: "cm",
        min: 0.2,
        max: 1.4,
        default: 0.8,
        doc: "Vallecula depth",
    },
    ParamSpec {
        key: "valleculaWidth",
        element: "larynx",
        unit: "cm",
        min: 0.2,
        max: 1.0,
        default: 0.7,
        doc: "Vallecula lateral radius",
    },
    ParamSpec {
        key: "lipCornerShape",
        element: "lips",
        unit: "",
        min: 1.0,
        max: 4.0,
        default: 2.0,
        doc: "Lip opening superellipse exponent",
    },
    ParamSpec {
        key: "lipThickness",
        element: "lips",
        unit: "cm",
        min: 0.6,
        max: 1.6,
        default: 1.0,
        doc: "Lip front-to-back depth",
    },
    ParamSpec {
        key: "lowerArchWidth",
        element: "teethJaw",
        unit: "cm",
        min: 1.3,
        max: 2.5,
        default: 1.8,
        doc: "Lower molar arch half-width",
    },
    ParamSpec {
        key: "toothGap",
        element: "teethJaw",
        unit: "cm",
        min: 0.0,
        max: 0.15,
        default: 0.05,
        doc: "Gap between arch blocks",
    },
    ParamSpec {
        key: "buccalGap",
        element: "cheeks",
        unit: "cm",
        min: 0.0,
        max: 0.6,
        default: 0.1,
        doc: "Vestibule gap outside dental arch",
    },
    ParamSpec {
        key: "cheekPuff",
        element: "cheeks",
        unit: "",
        min: 0.0,
        max: 1.0,
        default: 0.0,
        doc: "Cheek inflation",
    },
];
impl ShapeParams {
    /// JSON shape values are offsets from derive(), including size parameters.
    pub fn with_offsets(self, offsets: &Self) -> Result<Self, String> {
        let d = Self::default();
        let mut out = self;
        if !offsets.side_root.is_finite() {
            return Err("sideRoot must be finite".into());
        }
        out.side_root = (out.side_root + offsets.side_root - d.side_root).clamp(-1.4, 1.4);
        if !offsets.side_back.is_finite() {
            return Err("sideBack must be finite".into());
        }
        out.side_back = (out.side_back + offsets.side_back - d.side_back).clamp(-1.4, 1.4);
        if !offsets.side_dorsum.is_finite() {
            return Err("sideDorsum must be finite".into());
        }
        out.side_dorsum = (out.side_dorsum + offsets.side_dorsum - d.side_dorsum).clamp(-1.4, 1.4);
        if !offsets.side_blade.is_finite() {
            return Err("sideBlade must be finite".into());
        }
        out.side_blade = (out.side_blade + offsets.side_blade - d.side_blade).clamp(-1.4, 1.4);
        if !offsets.side_tip.is_finite() {
            return Err("sideTip must be finite".into());
        }
        out.side_tip = (out.side_tip + offsets.side_tip - d.side_tip).clamp(-1.4, 1.4);
        if !offsets.groove_root.is_finite() {
            return Err("grooveRoot must be finite".into());
        }
        out.groove_root = (out.groove_root + offsets.groove_root - d.groove_root).clamp(0.0, 0.8);
        if !offsets.groove_back.is_finite() {
            return Err("grooveBack must be finite".into());
        }
        out.groove_back = (out.groove_back + offsets.groove_back - d.groove_back).clamp(0.0, 0.8);
        if !offsets.groove_dorsum.is_finite() {
            return Err("grooveDorsum must be finite".into());
        }
        out.groove_dorsum =
            (out.groove_dorsum + offsets.groove_dorsum - d.groove_dorsum).clamp(0.0, 0.8);
        if !offsets.groove_blade.is_finite() {
            return Err("grooveBlade must be finite".into());
        }
        out.groove_blade =
            (out.groove_blade + offsets.groove_blade - d.groove_blade).clamp(0.0, 0.8);
        if !offsets.groove_tip.is_finite() {
            return Err("grooveTip must be finite".into());
        }
        out.groove_tip = (out.groove_tip + offsets.groove_tip - d.groove_tip).clamp(0.0, 0.8);
        if !offsets.groove_width.is_finite() {
            return Err("grooveWidth must be finite".into());
        }
        out.groove_width =
            (out.groove_width + offsets.groove_width - d.groove_width).clamp(0.3, 2.0);
        if !offsets.groove_taper.is_finite() {
            return Err("grooveTaper must be finite".into());
        }
        out.groove_taper =
            (out.groove_taper + offsets.groove_taper - d.groove_taper).clamp(-1.0, 1.0);
        if !offsets.side_asym.is_finite() {
            return Err("sideAsym must be finite".into());
        }
        out.side_asym = (out.side_asym + offsets.side_asym - d.side_asym).clamp(-1.0, 1.0);
        if !offsets.tongue_width.is_finite() {
            return Err("tongueWidth must be finite".into());
        }
        out.tongue_width =
            (out.tongue_width + offsets.tongue_width - d.tongue_width).clamp(0.8, 1.2);
        if !offsets.tip_width.is_finite() {
            return Err("tipWidth must be finite".into());
        }
        out.tip_width = (out.tip_width + offsets.tip_width - d.tip_width).clamp(0.3, 1.5);
        if !offsets.edge_round.is_finite() {
            return Err("edgeRound must be finite".into());
        }
        out.edge_round = (out.edge_round + offsets.edge_round - d.edge_round).clamp(1.5, 4.0);
        if !offsets.vault_depth.is_finite() {
            return Err("vaultDepth must be finite".into());
        }
        out.vault_depth = (out.vault_depth + offsets.vault_depth - d.vault_depth).clamp(0.5, 2.5);
        if !offsets.arch_width.is_finite() {
            return Err("archWidth must be finite".into());
        }
        out.arch_width = (out.arch_width + offsets.arch_width - d.arch_width).clamp(1.4, 2.6);
        if !offsets.vault_shape.is_finite() {
            return Err("vaultShape must be finite".into());
        }
        out.vault_shape = (out.vault_shape + offsets.vault_shape - d.vault_shape).clamp(1.5, 6.0);
        if !offsets.arch_taper.is_finite() {
            return Err("archTaper must be finite".into());
        }
        out.arch_taper = (out.arch_taper + offsets.arch_taper - d.arch_taper).clamp(0.0, 1.0);
        if !offsets.velum_width.is_finite() {
            return Err("velumWidth must be finite".into());
        }
        out.velum_width = (out.velum_width + offsets.velum_width - d.velum_width).clamp(1.5, 3.5);
        if !offsets.uvula_length.is_finite() {
            return Err("uvulaLength must be finite".into());
        }
        out.uvula_length =
            (out.uvula_length + offsets.uvula_length - d.uvula_length).clamp(0.5, 1.5);
        if !offsets.uvula_width.is_finite() {
            return Err("uvulaWidth must be finite".into());
        }
        out.uvula_width = (out.uvula_width + offsets.uvula_width - d.uvula_width).clamp(0.4, 1.2);
        if !offsets.pharynx_width_naso.is_finite() {
            return Err("pharynxWidthNaso must be finite".into());
        }
        out.pharynx_width_naso = (out.pharynx_width_naso + offsets.pharynx_width_naso
            - d.pharynx_width_naso)
            .clamp(0.8, 2.2);
        if !offsets.pharynx_width_oro.is_finite() {
            return Err("pharynxWidthOro must be finite".into());
        }
        out.pharynx_width_oro = (out.pharynx_width_oro + offsets.pharynx_width_oro
            - d.pharynx_width_oro)
            .clamp(0.8, 2.4);
        if !offsets.pharynx_width_hypo.is_finite() {
            return Err("pharynxWidthHypo must be finite".into());
        }
        out.pharynx_width_hypo = (out.pharynx_width_hypo + offsets.pharynx_width_hypo
            - d.pharynx_width_hypo)
            .clamp(0.8, 2.4);
        if !offsets.pharynx_shape.is_finite() {
            return Err("pharynxShape must be finite".into());
        }
        out.pharynx_shape =
            (out.pharynx_shape + offsets.pharynx_shape - d.pharynx_shape).clamp(1.5, 4.0);
        if !offsets.lateral_squeeze.is_finite() {
            return Err("lateralSqueeze must be finite".into());
        }
        out.lateral_squeeze =
            (out.lateral_squeeze + offsets.lateral_squeeze - d.lateral_squeeze).clamp(0.0, 1.0);
        if !offsets.epilarynx_width.is_finite() {
            return Err("epilarynxWidth must be finite".into());
        }
        out.epilarynx_width =
            (out.epilarynx_width + offsets.epilarynx_width - d.epilarynx_width).clamp(0.3, 1.2);
        if !offsets.epiglottis_width.is_finite() {
            return Err("epiglottisWidth must be finite".into());
        }
        out.epiglottis_width =
            (out.epiglottis_width + offsets.epiglottis_width - d.epiglottis_width).clamp(0.4, 1.6);
        if !offsets.epiglottis_curl.is_finite() {
            return Err("epiglottisCurl must be finite".into());
        }
        out.epiglottis_curl =
            (out.epiglottis_curl + offsets.epiglottis_curl - d.epiglottis_curl).clamp(0.0, 1.0);
        if !offsets.piriform_depth.is_finite() {
            return Err("piriformDepth must be finite".into());
        }
        out.piriform_depth =
            (out.piriform_depth + offsets.piriform_depth - d.piriform_depth).clamp(1.6, 2.0);
        if !offsets.piriform_width.is_finite() {
            return Err("piriformWidth must be finite".into());
        }
        out.piriform_width =
            (out.piriform_width + offsets.piriform_width - d.piriform_width).clamp(0.3, 1.0);
        if !offsets.vallecula_depth.is_finite() {
            return Err("valleculaDepth must be finite".into());
        }
        out.vallecula_depth =
            (out.vallecula_depth + offsets.vallecula_depth - d.vallecula_depth).clamp(0.2, 1.4);
        if !offsets.vallecula_width.is_finite() {
            return Err("valleculaWidth must be finite".into());
        }
        out.vallecula_width =
            (out.vallecula_width + offsets.vallecula_width - d.vallecula_width).clamp(0.2, 1.0);
        if !offsets.lip_corner_shape.is_finite() {
            return Err("lipCornerShape must be finite".into());
        }
        out.lip_corner_shape =
            (out.lip_corner_shape + offsets.lip_corner_shape - d.lip_corner_shape).clamp(1.0, 4.0);
        if !offsets.lip_thickness.is_finite() {
            return Err("lipThickness must be finite".into());
        }
        out.lip_thickness =
            (out.lip_thickness + offsets.lip_thickness - d.lip_thickness).clamp(0.6, 1.6);
        if !offsets.lower_arch_width.is_finite() {
            return Err("lowerArchWidth must be finite".into());
        }
        out.lower_arch_width =
            (out.lower_arch_width + offsets.lower_arch_width - d.lower_arch_width).clamp(1.3, 2.5);
        if !offsets.tooth_gap.is_finite() {
            return Err("toothGap must be finite".into());
        }
        out.tooth_gap = (out.tooth_gap + offsets.tooth_gap - d.tooth_gap).clamp(0.0, 0.15);
        if !offsets.buccal_gap.is_finite() {
            return Err("buccalGap must be finite".into());
        }
        out.buccal_gap = (out.buccal_gap + offsets.buccal_gap - d.buccal_gap).clamp(0.0, 0.6);
        if !offsets.cheek_puff.is_finite() {
            return Err("cheekPuff must be finite".into());
        }
        out.cheek_puff = (out.cheek_puff + offsets.cheek_puff - d.cheek_puff).clamp(0.0, 1.0);
        Ok(out)
    }
}
