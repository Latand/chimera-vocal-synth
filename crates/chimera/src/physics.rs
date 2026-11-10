//! Physical constants for airway acoustics, CGS units throughout
//! (cm, g, s; pressure in dyn/cm² = 0.1 Pa; volume velocity in cm³/s).

/// Air density at body temperature, saturated (g/cm³).
pub const RHO: f64 = 1.14e-3;
/// Speed of sound in warm humid air (cm/s).
pub const C_SOUND: f64 = 35000.0;
/// Dynamic viscosity of air (poise = g/(cm·s)).
pub const MU: f64 = 1.86e-4;
/// Kinematic viscosity (cm²/s).
pub const NU: f64 = MU / RHO;
/// ρc² — bulk modulus of air (dyn/cm²).
pub const RHO_C2: f64 = RHO * C_SOUND * C_SOUND;
/// Ratio of specific heats of air.
pub const GAMMA: f64 = 1.4;
/// Thermal diffusivity of air at body temperature, λ/(ρ·c_p) (cm²/s): λ ≈ 2.7·10³ erg/(s·cm·K),
/// c_p ≈ 1.005·10⁷ erg/(g·K).
pub const KAPPA: f64 = 2.7e3 / (RHO * 1.005e7);
/// 1 cm H2O in dyn/cm².
pub const CM_H2O: f64 = 980.665;

/// Soft-tissue wall parameters per unit area (Ishizaka, French & Flanagan 1975, cheek/neck).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallParams {
    /// mass per unit area (g/cm²)
    pub mass: f64,
    /// resistance per unit area (dyn·s/cm³)
    pub resistance: f64,
    /// stiffness per unit area (dyn/cm³)
    pub stiffness: f64,
}

pub const WALL_SOFT: WallParams = WallParams { mass: 1.5, resistance: 1600.0, stiffness: 3.0e5 };
/// Cartilaginous trachea: stiffer and lighter.
pub const WALL_TRACHEA: WallParams = WallParams { mass: 0.6, resistance: 1000.0, stiffness: 1.0e6 };
/// Nasal mucosa: lossy.
pub const WALL_NASAL: WallParams = WallParams { mass: 1.0, resistance: 3000.0, stiffness: 1.0e6 };

/// The human tract's soft tissues per unit area, by region (`voice::human_airway` maps them
/// onto the tube's sections). Masses are tissue density (≈1 g/cm³) times an effective moving
/// thickness; Hanna, Smith & Wolfe (2016, doi:10.1121/1.4948754) measured through the lips of ten
/// subjects a tissue resonance R0 ≈ 20 Hz, a closed-tract anti-resonance near 200 Hz and an
/// effective wall thickness of 1–2 cm, and Ishizaka, French & Flanagan (1975) broad cheek and neck
/// resonances at 30–70 Hz. The stiffnesses put each tissue's own resonance √(k/m)/2π there
/// (`WALL_SOFT`'s 3·10⁵ dyn/cm³ put it at 71 Hz). The resistances are 1.3× the 1000–1600
/// dyn·s/cm³ of Stevens and Flanagan (as quoted by Birkholz et al. 2022), the factor that puts the
/// male vowels' closed-glottis F1 bandwidths inside the 50–90 Hz Hanna et al. measured once the
/// hard palate and the teeth stop yielding (`chimera tract3d walls`). Thicknesses: the tongue moves as a
/// thick mass, the pharyngeal constrictors over loose prevertebral tissue, the cheeks and lips as
/// ~1 cm layers, the laryngeal walls (aryepiglottic folds, mucosa over cartilage) thin and
/// stiffer. These split the tract's measured totals by anatomy; only the totals were measured.
pub const TISSUE_TONGUE: WallParams = WallParams { mass: 2.2, resistance: 2100.0, stiffness: 3.2e4 };
pub const TISSUE_PHARYNX: WallParams = WallParams { mass: 1.8, resistance: 2100.0, stiffness: 2.4e4 };
pub const TISSUE_LARYNX: WallParams = WallParams { mass: 1.0, resistance: 1300.0, stiffness: 5.0e4 };
pub const TISSUE_CHEEK: WallParams = WallParams { mass: 1.8, resistance: 1600.0, stiffness: 9.5e4 };
pub const TISSUE_LIP: WallParams = WallParams { mass: 1.2, resistance: 1300.0, stiffness: 6.3e4 };
/// The soft palate, ~0.9 cm thick, with air on both faces (`Membrane`); its own resonance ≈30 Hz.
pub const TISSUE_VELUM: WallParams = WallParams { mass: 0.9, resistance: 800.0, stiffness: 3.2e4 };
/// Mass of the uvula (g, adult male template): a lobe ≈1 × 0.8 × 0.7 cm of muscle and mucosa
/// hanging from the soft palate's free edge; it rides on the soft palate's motion.
pub const UVULA_MASS: f64 = 0.55;

/// Critical Reynolds number for onset of turbulence noise.
pub const RE_CRIT: f64 = 1800.0;
