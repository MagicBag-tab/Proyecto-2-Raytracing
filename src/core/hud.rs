//! Small embedded bitmap font: no font files or UI dependencies.
use super::{
    framebuffer::Framebuffer,
    scene::{DayPhase, Scene},
};

fn glyph(c: char) -> [u16; 9] {
    let rows: [u16; 7] = match c {
        'A' | 'Á' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' | 'É' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'Í' => [2, 4, 14, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 2, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 25, 21, 19, 19, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 27, 17],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '/' => [1, 1, 2, 4, 8, 16, 16],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '✦' => [4, 4, 14, 31, 14, 4, 4],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        '一' => return [0, 0, 0, 0, 511, 0, 0, 0, 0],
        '二' => return [0, 254, 0, 0, 0, 0, 511, 0, 0],
        '三' => return [254, 0, 0, 124, 0, 0, 0, 511, 0],
        '四' => return [511, 325, 325, 325, 325, 313, 257, 257, 511],
        '五' => return [511, 32, 32, 254, 36, 36, 36, 36, 511],
        '六' => return [16, 8, 0, 511, 0, 68, 68, 130, 257],
        '七' => return [32, 32, 39, 504, 32, 32, 33, 33, 31],
        '日' => return [254, 130, 130, 130, 254, 130, 130, 130, 254],
        '目' => return [254, 130, 254, 130, 130, 254, 130, 130, 254],
        _ => [0; 7],
    };
    [
        rows[0], rows[1], rows[2], rows[3], rows[4], rows[5], rows[6], 0, 0,
    ]
}

fn pixel(frame: &mut Framebuffer, x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 && (x as usize) < frame.width && (y as usize) < frame.height {
        frame.buffer[y as usize * frame.width + x as usize] = color;
    }
}

pub fn text(frame: &mut Framebuffer, text: &str, x: i32, y: i32, scale: i32) {
    // Outline first, then foreground, so neighboring pixels cannot erase strokes.
    for outline in [true, false] {
        let mut cursor = x;
        for c in text.chars() {
            let wide = "一二三四五六七日目".contains(c);
            let width = if wide { 9 } else { 5 };
            for (row, bits) in glyph(c).iter().enumerate() {
                for col in 0..width {
                    if bits & (1 << (width - 1 - col)) == 0 {
                        continue;
                    }
                    let border = if outline { 1 } else { 0 };
                    for dy in -border..scale + border {
                        for dx in -border..scale + border {
                            pixel(
                                frame,
                                cursor + col * scale + dx,
                                y + row as i32 * scale + dy,
                                if outline { 0x111018 } else { 0xffdc69 },
                            );
                        }
                    }
                }
            }
            cursor += (width + 1) * scale;
        }
    }
}

pub fn draw(frame: &mut Framebuffer, scene: &Scene) {
    let numeral =
        ['一', '二', '三', '四', '五', '六', '七'][(scene.day_count.clamp(1, 7) - 1) as usize];
    text(frame, &format!("{numeral}日目"), 20, 18, 3);
    text(frame, &format!("DÍA {}", scene.day_count), 21, 53, 2);
    let phase = match scene.day_phase {
        DayPhase::Dawn => "AMANECER",
        DayPhase::Day => "DÍA",
        DayPhase::Sunset => "ATARDECER",
        DayPhase::Night => "NOCHE",
    };
    text(frame, phase, 21, 75, 1);
    text(
        frame,
        &format!("✦ PISTAS {} / 3", scene.game_state.clues_count()),
        frame.width as i32 - 200,
        23,
        2,
    );
    let status = if scene.game_state.secret_room_open {
        "PUERTA ABIERTA"
    } else if scene.game_state.door_unlocked {
        "PUERTA DESBLOQUEADA"
    } else {
        "PUERTA CERRADA"
    };
    text(frame, status, frame.width as i32 - 200, 47, 1);
    let hint = if scene.game_state.secret_room_open {
        "F12 CIERRA / ESPEJO - VIDRIO - RECUERDOS"
    } else if scene.game_state.door_unlocked {
        "PUERTA: CLIC / F12 ACCESO DIRECTO"
    } else {
        match scene.day_count {
            1..=3 => "F5-F11 DÍAS / F1-F4 LUZ / F12 ACCESO DIRECTO",
            4 => "AL ATARDECER ALGO BRILLA DETRÁS DEL CAFÉ",
            5 => "UNA FLOR ROJA JUNTO A LA PUERTA TRASERA",
            6 => "LA NOCHE REVELA UN CUADRO JUNTO AL FAROL",
            _ => "UN PERGAMINO ESPERA EN EL PATIO DE NOCHE",
        }
    };
    text(frame, hint, 20, frame.height as i32 - 25, 1);
}
