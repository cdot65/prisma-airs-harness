//! Fixed studio lighting per brand color, adapted to the terminal's background.

use crate::color::is_light;

/// The exact brand cyan from `airs_onboarding::logo::CYAN`; the resting pose settles near it.
const AIRS_CYAN: [f64; 3] = [0.0, 192.0, 232.0];
/// Palo Alto Networks orange, which the mark takes on as it morphs mid-spin.
const PALO_ALTO_ORANGE: [f64; 3] = [250.0, 88.0, 45.0];

#[derive(Clone, Copy)]
pub(super) struct Lighting {
    pub(super) background: [f64; 3],
    pub(super) key: [f64; 3],
    pub(super) shadow: [f64; 3],
    pub(super) fill: [f64; 3],
    pub(super) rim: [f64; 3],
    pub(super) highlight: [f64; 3],
}

impl Lighting {
    pub(super) fn airs(bg: (u8, u8, u8)) -> Self {
        let background = [bg.0.into(), bg.1.into(), bg.2.into()];
        if is_light(bg) {
            // Light themes keep the flat brand color and only deepen the shaded faces.
            Self {
                background,
                key: AIRS_CYAN,
                shadow: [0.0, 118.0, 148.0],
                fill: AIRS_CYAN,
                rim: AIRS_CYAN,
                highlight: [120.0, 224.0, 246.0],
            }
        } else {
            Self {
                background,
                key: [56.0, 206.0, 242.0],
                shadow: [4.0, 58.0, 90.0],
                fill: [0.0, 126.0, 168.0],
                rim: [160.0, 240.0, 255.0],
                highlight: [240.0, 252.0, 255.0],
            }
        }
    }

    pub(super) fn palo_alto(bg: (u8, u8, u8)) -> Self {
        let background = [bg.0.into(), bg.1.into(), bg.2.into()];
        if is_light(bg) {
            Self {
                background,
                key: PALO_ALTO_ORANGE,
                shadow: [168.0, 52.0, 24.0],
                fill: PALO_ALTO_ORANGE,
                rim: PALO_ALTO_ORANGE,
                highlight: [255.0, 150.0, 110.0],
            }
        } else {
            Self {
                background,
                key: [252.0, 104.0, 60.0],
                shadow: [92.0, 30.0, 16.0],
                fill: [184.0, 62.0, 30.0],
                rim: [255.0, 176.0, 136.0],
                highlight: [255.0, 240.0, 226.0],
            }
        }
    }

    /// Blend two palettes so the color follows the shape through the morph.
    pub(super) fn mix(from: &Self, to: &Self, t: f64) -> Self {
        let lerp = |a: [f64; 3], b: [f64; 3]| std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t);
        Self {
            background: from.background,
            key: lerp(from.key, to.key),
            shadow: lerp(from.shadow, to.shadow),
            fill: lerp(from.fill, to.fill),
            rim: lerp(from.rim, to.rim),
            highlight: lerp(from.highlight, to.highlight),
        }
    }

    pub(super) fn shade(&self, [nx, ny, nz]: [f64; 3], depth: f64, grain: f64) -> u32 {
        let diffuse = (nx * -0.410 + ny * -0.564 + nz * 0.718 + grain)
            .clamp(/*min*/ 0.0, /*max*/ 1.0);
        let bounce = (nx * 0.55 + ny * 0.2 - nz * 0.35).clamp(/*min*/ 0.0, /*max*/ 1.0)
            * (1.0 - diffuse)
            * 0.38;
        let edge = (1.0 - nz.clamp(/*min*/ -1.0, /*max*/ 1.0).abs()).powf(/*n*/ 2.4)
            * (nx * 0.85 - ny * 0.38).clamp(/*min*/ 0.0, /*max*/ 1.0)
            * 0.82;
        let gloss = (nx * -0.220 + ny * -0.302 + nz * 0.928)
            .clamp(/*min*/ 0.0, /*max*/ 1.0)
            .powi(/*n*/ 18)
            * 0.96;
        let base = (1.0 - bounce) * (1.0 - edge) * (1.0 - gloss);
        let gain = (0.90 + depth * 0.18).clamp(/*min*/ 0.68, /*max*/ 1.0);
        let rgb: [u8; 3] = std::array::from_fn(|i| {
            let value = self.shadow[i] * (1.0 - diffuse) * base
                + self.key[i] * diffuse * base
                + self.fill[i] * bounce * (1.0 - edge) * (1.0 - gloss)
                + self.rim[i] * edge * (1.0 - gloss)
                + self.highlight[i] * gloss;
            (self.background[i] + (value - self.background[i]) * gain).round() as u8
        });
        u32::from_be_bytes([0, rgb[0], rgb[1], rgb[2]])
    }
}
