//! Funções matemáticas de geometria, projeção, zoom ancorado e limites de escala.

use eframe::egui::{Pos2, Rect, Vec2};

pub fn fit_scale(image_px: Vec2, canvas: Rect) -> f32 {
    let available = (canvas.size() - Vec2::splat(32.0)).max(Vec2::splat(32.0));
    let width = image_px.x.max(1.0);
    let height = image_px.y.max(1.0);
    (available.x / width).min(available.y / height).max(0.000_1)
}

pub fn scale_limits(pixels_per_point: f32) -> (f32, f32) {
    let pixels_per_point = pixels_per_point.max(0.5);
    (0.02 / pixels_per_point, 32.0 / pixels_per_point)
}

pub fn image_rect(canvas: Rect, image_px: Vec2, scale: f32, offset: Vec2) -> Rect {
    Rect::from_center_size(canvas.center() + offset, image_px * scale)
}

pub fn fit_rect(bounds: Rect, image: Vec2) -> Rect {
    let scale = (bounds.width() / image.x.max(1.0))
        .min(bounds.height() / image.y.max(1.0))
        .max(0.000_1);
    Rect::from_center_size(bounds.center(), image * scale)
}

pub fn zoom_about(
    canvas: Rect,
    image_px: Vec2,
    scale: f32,
    offset: Vec2,
    anchor: Pos2,
    factor: f32,
    min_scale: f32,
    max_scale: f32,
) -> (f32, Vec2) {
    let safe_scale = scale.max(0.000_1);
    let new_scale = (safe_scale * factor).clamp(min_scale, max_scale);
    let old = image_rect(canvas, image_px, safe_scale, offset);
    let point = if old.contains(anchor) {
        anchor
    } else {
        old.center()
    };
    let relative = (point - old.min) / safe_scale;
    let new_min = point - relative * new_scale;
    let new_center = new_min + image_px * new_scale * 0.5;
    (new_scale, new_center - canvas.center())
}

pub fn clamp_offset(canvas: Rect, image_px: Vec2, scale: f32, offset: Vec2) -> Vec2 {
    let rect = image_rect(canvas, image_px, scale, offset);
    let margin = 48.0;
    let mut delta = Vec2::ZERO;
    if rect.right() < canvas.left() + margin {
        delta.x = canvas.left() + margin - rect.right();
    } else if rect.left() > canvas.right() - margin {
        delta.x = canvas.right() - margin - rect.left();
    }
    if rect.bottom() < canvas.top() + margin {
        delta.y = canvas.top() + margin - rect.bottom();
    } else if rect.top() > canvas.bottom() - margin {
        delta.y = canvas.bottom() - margin - rect.top();
    }
    offset + delta
}
