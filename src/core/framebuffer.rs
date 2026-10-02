pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    background_color: u32,
    current_color: u32,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buffer: vec![0; width * height],
            background_color: 0x000000,
            current_color: 0xFFFFFF,
        }
    }

    pub fn clear(&mut self) {
        for pixel in self.buffer.iter_mut() {
            *pixel = self.background_color;
        }
    }

    pub fn point(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.buffer[y * self.width + x] = self.current_color;
        }
    }

    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }
}

/// Blend two already-rendered frames so the transition never retraces the scene.
pub fn crossfade(previous: &[u32], next: &[u32], output: &mut [u32], progress: f32) {
    let weight = (progress.clamp(0.0, 1.0) * 256.0).round() as u32;
    for ((pixel, old), new) in output.iter_mut().zip(previous).zip(next) {
        *pixel = 0;
        for shift in [0, 8, 16] {
            let a = (old >> shift) & 255;
            let b = (new >> shift) & 255;
            *pixel |= ((a * (256 - weight) + b * weight) / 256) << shift;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::crossfade;
    #[test]
    fn fade_preserves_endpoints_and_blends_channels() {
        let mut output = [0];
        crossfade(&[0xff0000], &[0x0000ff], &mut output, 0.0);
        assert_eq!(output, [0xff0000]);
        crossfade(&[0xff0000], &[0x0000ff], &mut output, 0.5);
        assert_eq!(output, [0x7f007f]);
        crossfade(&[0xff0000], &[0x0000ff], &mut output, 1.0);
        assert_eq!(output, [0x0000ff]);
    }
}
