#![warn(clippy::pedantic, clippy::nursery)]

pub mod operations;

/// A pixel is a single point in a pixel buffer with an RGBA color value.
pub struct Pixel {
    pub x: i32,
    pub y: i32,
    pub color: [u8; 4],
}

/// A stamp is a collection of pixels that represent a brush shape.
pub struct Stamp {
    pub pixels: Vec<Pixel>,
}

pub struct BrushBaseSettings {
    pub id: String,
    pub radius: f32,
    pub spacing: f32,
    pub opacity: f32,
}

pub enum Brush {
    SoftCircle {
        hardness: f32,
        base: BrushBaseSettings,
    },
}

impl Default for Brush {
    fn default() -> Self {
        Self::SoftCircle {
            hardness: 0.1,
            base: BrushBaseSettings {
                id: "soft-circle".to_string(),
                radius: 10.0,
                spacing: 0.1,
                opacity: 1.0,
            },
        }
    }
}

impl Brush {
    /// Gets a stamp for the current brush settings
    #[must_use]
    pub fn compute_stamp(&self, color: [u8; 3]) -> Stamp {
        match self {
            Self::SoftCircle { hardness, base } => {
                soft_circle(base.radius, *hardness, base.opacity, color)
            }
        }
    }

    //==========================================================================
    // accessor methods
    //==========================================================================

    #[must_use]
    pub const fn spacing(&self) -> f32 {
        match self {
            Self::SoftCircle { base, .. } => base.spacing,
        }
    }

    #[must_use]
    pub const fn radius(&self) -> f32 {
        match self {
            Self::SoftCircle { base, .. } => base.radius,
        }
    }

    #[must_use]
    pub const fn opacity(&self) -> f32 {
        match self {
            Self::SoftCircle { base, .. } => base.opacity,
        }
    }

    //==========================================================================
    // builder methods
    //==========================================================================

    #[must_use]
    pub fn with_spacing(self, spacing: f32) -> Self {
        match self {
            Self::SoftCircle { hardness, mut base } => {
                base.spacing = spacing;
                Self::SoftCircle { hardness, base }
            }
        }
    }

    #[must_use]
    pub fn with_radius(self, radius: f32) -> Self {
        match self {
            Self::SoftCircle { hardness, mut base } => {
                base.radius = radius;
                Self::SoftCircle { hardness, base }
            }
        }
    }

    #[must_use]
    pub fn with_opacity(self, opacity: f32) -> Self {
        match self {
            Self::SoftCircle { hardness, mut base } => {
                base.opacity = opacity;
                Self::SoftCircle { hardness, base }
            }
        }
    }
}

/// Generates a soft circle brush stamp which you can use to merge in with a pixel buffer.
/// Generally you wouldn't call this directly and instead would use `PaintOperation::process`.
#[must_use]
pub fn soft_circle(radius: f32, hardness: f32, opacity: f32, color: [u8; 3]) -> Stamp {
    let mut pixels = Vec::new();
    let radius_int = radius.ceil() as i32;

    for y in -radius_int..=radius_int {
        for x in -radius_int..=radius_int {
            let distance = ((x * x + y * y) as f32).sqrt();
            if distance <= radius {
                let normalized_dist = distance / radius;
                let alpha = if normalized_dist < hardness {
                    1.0
                } else {
                    let t = (normalized_dist - hardness) / (1.0 - hardness);
                    (t * t).mul_add(-2.0f32.mul_add(-t, 3.0), 1.0).max(0.0)
                };

                let alpha = (alpha * opacity * 255.0) as u8;
                pixels.push(Pixel {
                    x,
                    y,
                    color: [color[0], color[1], color[2], alpha],
                });
            }
        }
    }

    Stamp { pixels }
}
