fn brown_black_score(rgb: u32) -> f32 {
    let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
    let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
    let b = (rgb & 0xFF) as f32 / 255.0;

    // --- RGB -> HSV ---
    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;

    let value = max;

    let saturation = if max == 0.0 { 0.0 } else { delta / max };

    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let hue = if hue < 0.0 { hue + 360.0 } else { hue };

    // --- Scoring ---

    // Blackness: darker = higher
    let blackness = 1.0 - value;

    // Brown ≈ dark orange (~20°–40°)
    let target_hue = 30.0;
    let hue_dist = ((hue - target_hue).abs() / 180.0).min(1.0);
    let hue_score = 1.0 - hue_dist;

    // Brown needs some saturation and some darkness
    let brownness = hue_score * saturation * (1.0 - value);

    // Combine (tweak weights as needed)
    0.6 * blackness + 0.4 * brownness
}

pub fn is_nice_color(rgb: u32) -> bool {
    brown_black_score(rgb) < 0.5
}
