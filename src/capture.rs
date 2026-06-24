use image::RgbaImage;
use xcap::{Monitor, Window};

#[derive(Debug, Clone)]
pub struct ScreenFrame {
    pub image: RgbaImage,
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: u32,
    pub height: u32,
    pub monitor_name: String,
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub index: usize,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub minimized: bool,
    pub maximized: bool,
}

impl WindowInfo {
    pub fn rect(&self) -> egui::Rect {
        egui::Rect::from_min_size(
            egui::pos2(self.x as f32, self.y as f32),
            egui::vec2(self.width as f32, self.height as f32),
        )
    }

    pub fn contains_screen_pos(&self, pos: egui::Pos2) -> bool {
        self.rect().contains(pos)
    }
}

pub fn capture_primary() -> anyhow::Result<ScreenFrame> {
    let monitors = Monitor::all()?;
    let monitor = monitors
        .into_iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .ok_or_else(|| anyhow::anyhow!("No primary monitor found"))?;
    capture_monitor(monitor)
}

pub fn capture_monitor_at(x: i32, y: i32) -> anyhow::Result<ScreenFrame> {
    let monitor = Monitor::from_point(x, y)?;
    capture_monitor(monitor)
}

pub fn capture_all_monitors() -> anyhow::Result<Vec<ScreenFrame>> {
    let mut frames = Vec::new();
    for monitor in Monitor::all()? {
        frames.push(capture_monitor(monitor)?);
    }
    Ok(frames)
}

fn capture_monitor(monitor: Monitor) -> anyhow::Result<ScreenFrame> {
    let image = monitor.capture_image()?;
    let width = image.width();
    let height = image.height();
    let origin_x = monitor.x().unwrap_or(0);
    let origin_y = monitor.y().unwrap_or(0);
    let monitor_name = monitor.friendly_name().unwrap_or_else(|_| "Monitor".to_owned());
    Ok(ScreenFrame { image, origin_x, origin_y, width, height, monitor_name })
}

pub fn capture_region(x: i32, y: i32, width: u32, height: u32) -> anyhow::Result<ScreenFrame> {
    let monitor = Monitor::from_point(x, y)?;
    let image = monitor.capture_region(x, y, width.max(1), height.max(1))?;
    Ok(ScreenFrame {
        image,
        origin_x: x,
        origin_y: y,
        width,
        height,
        monitor_name: monitor.friendly_name().unwrap_or_else(|_| "Region".to_owned()),
    })
}

pub fn list_windows() -> anyhow::Result<Vec<WindowInfo>> {
    let mut out = Vec::new();
    for (index, window) in Window::all()?.into_iter().enumerate() {
        let minimized = window.is_minimized().unwrap_or(false);
        let title = window.title().unwrap_or_default();
        if title.trim().is_empty() {
            continue;
        }
        out.push(WindowInfo {
            index,
            title,
            x: window.x().unwrap_or(0),
            y: window.y().unwrap_or(0),
            width: window.width().unwrap_or(0),
            height: window.height().unwrap_or(0),
            minimized,
            maximized: window.is_maximized().unwrap_or(false),
        });
    }
    Ok(out)
}

pub fn window_at(pos: egui::Pos2, windows: &[WindowInfo]) -> Option<WindowInfo> {
    windows
        .iter()
        .filter(|window| !window.minimized && window.width > 16 && window.height > 16)
        .filter(|window| window.contains_screen_pos(pos))
        .min_by_key(|window| window.width as u64 * window.height as u64)
        .cloned()
}

pub fn capture_window(info: &WindowInfo) -> anyhow::Result<RgbaImage> {
    let windows = Window::all()?;
    let window = windows
        .into_iter()
        .nth(info.index)
        .ok_or_else(|| anyhow::anyhow!("Window vanished before capture: {}", info.title))?;
    if window.is_minimized().unwrap_or(false) {
        anyhow::bail!("Cannot capture a minimized window");
    }
    Ok(window.capture_image()?)
}
