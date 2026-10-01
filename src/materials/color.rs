use std::fmt;
use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color {
            r: srgb_to_linear(r),
            g: srgb_to_linear(g),
            b: srgb_to_linear(b),
        }
    }

    pub fn from_hex(hex: u32) -> Self {
        Self::new(
            ((hex >> 16) & 0xff) as u8,
            ((hex >> 8) & 0xff) as u8,
            (hex & 0xff) as u8,
        )
    }

    pub fn to_hex(&self) -> u32 {
        ((encode_display_channel(tone_map(self.r)) as u32) << 16)
            | ((encode_display_channel(tone_map(self.g)) as u32) << 8)
            | (encode_display_channel(tone_map(self.b)) as u32)
    }

    pub fn to_hex_unmapped(&self) -> u32 {
        ((encode_display_channel(self.r.max(0.0)) as u32) << 16)
            | ((encode_display_channel(self.g.max(0.0)) as u32) << 8)
            | (encode_display_channel(self.b.max(0.0)) as u32)
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, scalar: f32) -> Color {
        Color {
            r: self.r * scalar,
            g: self.g * scalar,
            b: self.b * scalar,
        }
    }
}

fn srgb_to_linear(channel: u8) -> f32 {
    let encoded = channel as f32 / 255.0;
    if encoded <= 0.04045 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}

fn tone_map(linear: f32) -> f32 {
    let positive = linear.max(0.0);
    positive / (1.0 + positive)
}

fn encode_display_channel(linear: f32) -> u8 {
    let linear = linear.clamp(0.0, 1.0);
    let encoded = if linear <= 0.0031308 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round() as u8
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Color(r: {:.3}, g: {:.3}, b: {:.3})",
            self.r, self.g, self.b
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn color_addition_keeps_hdr_values_until_display_conversion() {
        let red = Color::new(255, 0, 0);
        let accumulated = red * 2.0 + red * 2.0;
        let display_red = (accumulated.to_hex() >> 16) & 0xff;

        assert!(accumulated.r > 1.0);
        assert!(display_red > 188);
        assert!(display_red < 255);
    }
}
