use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

use crate::capture::{ScreenFrame, WindowInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShapeKind {
    Rectangle,
    Ellipse,
    Lasso,
    Polygon,
    Fullscreen,
    Window,
}

impl ShapeKind {
    pub fn label(self) -> &'static str {
        match self {
            ShapeKind::Rectangle => "矩形",
            ShapeKind::Ellipse => "圆形/椭圆",
            ShapeKind::Lasso => "Lasso",
            ShapeKind::Polygon => "多边形",
            ShapeKind::Fullscreen => "全屏",
            ShapeKind::Window => "窗口识别",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectionShape {
    pub kind: ShapeKind,
    pub start: egui::Pos2,
    pub current: egui::Pos2,
    pub points: Vec<egui::Pos2>,
    pub closed: bool,
    pub window: Option<WindowInfo>,
}

impl SelectionShape {
    pub fn new(kind: ShapeKind, pos: egui::Pos2) -> Self {
        Self {
            kind,
            start: pos,
            current: pos,
            points: if matches!(kind, ShapeKind::Lasso | ShapeKind::Polygon) { vec![pos] } else { Vec::new() },
            closed: false,
            window: None,
        }
    }

    pub fn fullscreen(rect: egui::Rect) -> Self {
        Self {
            kind: ShapeKind::Fullscreen,
            start: rect.min,
            current: rect.max,
            points: Vec::new(),
            closed: true,
            window: None,
        }
    }

    pub fn from_window(window: WindowInfo) -> Self {
        let rect = window.rect();
        Self {
            kind: ShapeKind::Window,
            start: rect.min,
            current: rect.max,
            points: Vec::new(),
            closed: true,
            window: Some(window),
        }
    }

    pub fn update_drag(&mut self, pos: egui::Pos2, shift: bool) {
        match self.kind {
            ShapeKind::Rectangle | ShapeKind::Ellipse => {
                self.current = if shift { lock_square(self.start, pos) } else { pos };
            }
            ShapeKind::Lasso => {
                self.current = pos;
                if self.points.last().map(|last| last.distance(pos) > 2.0).unwrap_or(true) {
                    self.points.push(pos);
                }
            }
            _ => self.current = pos,
        }
    }

    pub fn add_polygon_point(&mut self, pos: egui::Pos2) {
        if self.kind == ShapeKind::Polygon && !self.closed {
            self.points.push(pos);
            self.current = pos;
        }
    }

    pub fn close(&mut self) {
        self.closed = true;
        if self.kind == ShapeKind::Lasso && self.points.len() > 2 {
            if let Some(first) = self.points.first().copied() {
                self.points.push(first);
            }
        }
    }

    pub fn is_ready(&self, min_px: f32) -> bool {
        match self.kind {
            ShapeKind::Rectangle | ShapeKind::Ellipse | ShapeKind::Fullscreen | ShapeKind::Window => {
                self.rect().width() >= min_px && self.rect().height() >= min_px
            }
            ShapeKind::Lasso => self.points.len() > 8 && self.bounds().width() >= min_px && self.bounds().height() >= min_px,
            ShapeKind::Polygon => self.closed && self.points.len() >= 3 && self.bounds().width() >= min_px && self.bounds().height() >= min_px,
        }
    }

    pub fn rect(&self) -> egui::Rect {
        egui::Rect::from_two_pos(self.start, self.current).intersect(egui::Rect::EVERYTHING)
    }

    pub fn bounds(&self) -> egui::Rect {
        match self.kind {
            ShapeKind::Lasso | ShapeKind::Polygon => bounds_of_points(&self.points).unwrap_or_else(|| self.rect()),
            _ => self.rect(),
        }
    }

    pub fn draw(&self, painter: &egui::Painter, stroke: egui::Stroke, fill: Option<egui::Color32>) {
        match self.kind {
            ShapeKind::Rectangle | ShapeKind::Fullscreen | ShapeKind::Window => {
                if let Some(fill) = fill {
                    painter.rect_filled(self.rect(), 0.0, fill);
                }
                painter.rect_stroke(self.rect(), 2.0, stroke, egui::StrokeKind::Outside);
            }
            ShapeKind::Ellipse => {
                let rect = self.rect();
                if let Some(fill) = fill {
                    painter.add(egui::Shape::ellipse_filled(rect.center(), rect.size() / 2.0, fill));
                }
                painter.add(egui::Shape::ellipse_stroke(rect.center(), rect.size() / 2.0, stroke));
            }
            ShapeKind::Lasso | ShapeKind::Polygon => {
                if self.points.len() >= 2 {
                    painter.add(egui::Shape::line(self.points.clone(), stroke));
                    if self.closed && self.points.len() >= 3 {
                        painter.line_segment([*self.points.last().unwrap(), self.points[0]], stroke);
                    }
                }
            }
        }
    }

    pub fn size_label(&self) -> String {
        let rect = self.bounds();
        format!("{} x {}", rect.width().round() as i32, rect.height().round() as i32)
    }
}

fn lock_square(start: egui::Pos2, pos: egui::Pos2) -> egui::Pos2 {
    let delta = pos - start;
    let side = delta.x.abs().min(delta.y.abs());
    egui::pos2(start.x + side * delta.x.signum(), start.y + side * delta.y.signum())
}

pub fn bounds_of_points(points: &[egui::Pos2]) -> Option<egui::Rect> {
    let first = points.first().copied()?;
    let mut min = first;
    let mut max = first;
    for p in points.iter().copied() {
        min.x = min.x.min(p.x);
        min.y = min.y.min(p.y);
        max.x = max.x.max(p.x);
        max.y = max.y.max(p.y);
    }
    Some(egui::Rect::from_min_max(min, max))
}

#[derive(Debug, Clone)]
struct ImageSelection {
    kind: ShapeKind,
    rect: egui::Rect,
    points: Vec<egui::Pos2>,
}

pub fn crop_selection(frame: &ScreenFrame, selection: &SelectionShape, display_rect: egui::Rect) -> anyhow::Result<RgbaImage> {
    let image_selection = to_image_selection(frame, selection, display_rect);
    crop_image_selection(&frame.image, &image_selection)
}

fn to_image_selection(frame: &ScreenFrame, selection: &SelectionShape, display_rect: egui::Rect) -> ImageSelection {
    let to_image_pos = |p: egui::Pos2| -> egui::Pos2 {
        let x = ((p.x - display_rect.left()) / display_rect.width() * frame.width as f32)
            .clamp(0.0, frame.width.saturating_sub(1) as f32);
        let y = ((p.y - display_rect.top()) / display_rect.height() * frame.height as f32)
            .clamp(0.0, frame.height.saturating_sub(1) as f32);
        egui::pos2(x, y)
    };

    match selection.kind {
        ShapeKind::Lasso | ShapeKind::Polygon => {
            let points: Vec<_> = selection.points.iter().copied().map(to_image_pos).collect();
            let rect = bounds_of_points(&points).unwrap_or_else(|| egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(frame.width as f32, frame.height as f32)));
            ImageSelection { kind: selection.kind, rect, points }
        }
        _ => ImageSelection { kind: selection.kind, rect: rect_to_image(frame, selection.rect(), display_rect), points: Vec::new() },
    }
}

fn rect_to_image(frame: &ScreenFrame, rect: egui::Rect, display_rect: egui::Rect) -> egui::Rect {
    let sx = frame.width as f32 / display_rect.width().max(1.0);
    let sy = frame.height as f32 / display_rect.height().max(1.0);
    let min = egui::pos2((rect.left() - display_rect.left()) * sx, (rect.top() - display_rect.top()) * sy);
    let max = egui::pos2((rect.right() - display_rect.left()) * sx, (rect.bottom() - display_rect.top()) * sy);
    let bounds = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(frame.width as f32, frame.height as f32));
    egui::Rect::from_min_max(min, max).intersect(bounds)
}

fn crop_image_selection(source: &RgbaImage, selection: &ImageSelection) -> anyhow::Result<RgbaImage> {
    let bounds = selection.rect;
    let left = bounds.left().floor().max(0.0) as u32;
    let top = bounds.top().floor().max(0.0) as u32;
    let right = bounds.right().ceil().min(source.width() as f32) as u32;
    let bottom = bounds.bottom().ceil().min(source.height() as f32) as u32;
    let width = right.saturating_sub(left).max(1);
    let height = bottom.saturating_sub(top).max(1);

    if matches!(selection.kind, ShapeKind::Rectangle | ShapeKind::Fullscreen | ShapeKind::Window) {
        return Ok(image::imageops::crop_imm(source, left, top, width, height).to_image());
    }

    let mut out = RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let px = left + x;
            let py = top + y;
            let inside = match selection.kind {
                ShapeKind::Ellipse => point_in_ellipse(egui::pos2(px as f32 + 0.5, py as f32 + 0.5), selection.rect),
                ShapeKind::Lasso | ShapeKind::Polygon => point_in_polygon(egui::pos2(px as f32 + 0.5, py as f32 + 0.5), &selection.points),
                _ => true,
            };
            if inside {
                out.put_pixel(x, y, *source.get_pixel(px, py));
            } else {
                out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            }
        }
    }
    Ok(out)
}

fn point_in_ellipse(p: egui::Pos2, rect: egui::Rect) -> bool {
    let rx = (rect.width() / 2.0).max(1.0);
    let ry = (rect.height() / 2.0).max(1.0);
    let c = rect.center();
    let dx = (p.x - c.x) / rx;
    let dy = (p.y - c.y) / ry;
    dx * dx + dy * dy <= 1.0
}

fn point_in_polygon(p: egui::Pos2, points: &[egui::Pos2]) -> bool {
    if points.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = points.len() - 1;
    for i in 0..points.len() {
        let pi = points[i];
        let pj = points[j];
        let denom = pj.y - pi.y;
        if denom.abs() > f32::EPSILON
            && ((pi.y > p.y) != (pj.y > p.y))
            && (p.x < (pj.x - pi.x) * (p.y - pi.y) / denom + pi.x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

pub fn draw_mask_with_hole(painter: &egui::Painter, screen: egui::Rect, selection: Option<&SelectionShape>, mask: egui::Color32) {
    if let Some(selection) = selection {
        match selection.kind {
            ShapeKind::Rectangle | ShapeKind::Fullscreen | ShapeKind::Window | ShapeKind::Ellipse => {
                let hole = selection.bounds().intersect(screen);
                painter.rect_filled(egui::Rect::from_min_max(screen.min, egui::pos2(screen.max.x, hole.top())), 0.0, mask);
                painter.rect_filled(egui::Rect::from_min_max(egui::pos2(screen.min.x, hole.bottom()), screen.max), 0.0, mask);
                painter.rect_filled(egui::Rect::from_min_max(egui::pos2(screen.min.x, hole.top()), egui::pos2(hole.left(), hole.bottom())), 0.0, mask);
                painter.rect_filled(egui::Rect::from_min_max(egui::pos2(hole.right(), hole.top()), egui::pos2(screen.max.x, hole.bottom())), 0.0, mask);
            }
            ShapeKind::Lasso | ShapeKind::Polygon => painter.rect_filled(screen, 0.0, mask.gamma_multiply(0.72)),
        }
    } else {
        painter.rect_filled(screen, 0.0, mask);
    }
}
