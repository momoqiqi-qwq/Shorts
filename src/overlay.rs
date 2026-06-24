use eframe::egui;
use image::RgbaImage;

use crate::{
    capture::{self, ScreenFrame, WindowInfo},
    config::{AppConfig, ExportFormat, ThemeMode, ToolbarPosition},
    export,
    shape::{self, ShapeKind, SelectionShape},
    theme,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppMode {
    Main,
    Selecting,
}

pub struct ShotrsApp {
    pub config: AppConfig,
    pub selected_shape: ShapeKind,
    mode: AppMode,
    frame: Option<ScreenFrame>,
    texture: Option<egui::TextureHandle>,
    selection: Option<SelectionShape>,
    windows: Vec<WindowInfo>,
    hovered_window: Option<WindowInfo>,
    status: String,
    save_dir_text: String,
}

impl ShotrsApp {
    pub fn new(config: AppConfig) -> Self {
        let save_dir_text = config.export.save_dir.display().to_string();
        Self {
            config,
            selected_shape: ShapeKind::Rectangle,
            mode: AppMode::Main,
            frame: None,
            texture: None,
            selection: None,
            windows: Vec::new(),
            hovered_window: None,
            status: "准备就绪".to_owned(),
            save_dir_text,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        theme::apply_theme(ui.ctx(), &self.config);
        match self.mode {
            AppMode::Main => self.main_ui(ui, frame),
            AppMode::Selecting => self.overlay_ui(ui, frame),
        }
    }

    fn main_ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Decorations(true));
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(420.0, 560.0)));

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Shotrs 截图工具");
                ui.label("小窗口设置，全屏截图时自动切换为遮罩层");
            });
            ui.separator();

            ui.horizontal_wrapped(|ui| {
                for kind in [
                    ShapeKind::Rectangle,
                    ShapeKind::Ellipse,
                    ShapeKind::Lasso,
                    ShapeKind::Polygon,
                    ShapeKind::Fullscreen,
                    ShapeKind::Window,
                ] {
                    let selected = self.selected_shape == kind;
                    if ui.selectable_label(selected, kind.label()).clicked() {
                        self.selected_shape = kind;
                    }
                }
            });

            ui.add_space(4.0);
            if ui.add_sized([ui.available_width(), 36.0], egui::Button::new("开始截图")).clicked() {
                self.start_capture(ui.ctx());
            }
            ui.label(&self.status);

            egui::CollapsingHeader::new("导出设置")
                .default_open(true)
                .show(ui, |ui| self.export_settings_ui(ui));

            egui::CollapsingHeader::new("主题与遮罩")
                .default_open(true)
                .show(ui, |ui| self.theme_settings_ui(ui));

            egui::CollapsingHeader::new("体验优化")
                .default_open(false)
                .show(ui, |ui| self.behavior_settings_ui(ui));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("保存配置").clicked() {
                        self.save_config_status();
                    }
                    if ui.button("打开配置目录").clicked() {
                        if let Some(parent) = AppConfig::path().parent() {
                            let _ = open::that(parent);
                        }
                    }
                });
                ui.small(format!("配置文件：{}", AppConfig::path().display()));
            });
        });
    }

    fn export_settings_ui(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("默认格式")
            .selected_text(format!("{:?}", self.config.export.format))
            .show_ui(ui, |ui| {
                for format in [ExportFormat::Png, ExportFormat::Jpeg, ExportFormat::Webp, ExportFormat::Bmp, ExportFormat::Gif, ExportFormat::Avif] {
                    ui.selectable_value(&mut self.config.export.format, format, format!("{:?}", format));
                }
            });
        ui.add(egui::Slider::new(&mut self.config.export.jpeg_quality, 1..=100).text("JPEG 质量"));
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.config.export.webp_lossless, "WebP 无损");
            ui.add(egui::Slider::new(&mut self.config.export.webp_quality, 1..=100).text("WebP 质量"));
        });
        ui.add(egui::Slider::new(&mut self.config.export.avif_quality, 1.0..=100.0).text("AVIF 质量"));
        ui.add(egui::Slider::new(&mut self.config.export.avif_speed, 1..=10).text("AVIF 速度"));
        ui.checkbox(&mut self.config.export.auto_copy_clipboard, "导出后复制到剪贴板");
        ui.checkbox(&mut self.config.export.auto_save, "自动保存到目录");
        ui.checkbox(&mut self.config.export.open_after_save, "保存后打开文件");
        ui.text_edit_singleline(&mut self.config.export.filename_template)
            .on_hover_text("支持 {timestamp} 和 {ext}，例如 screenshot_{timestamp}.{ext}");
        ui.horizontal(|ui| {
            ui.label("保存目录");
            if ui.text_edit_singleline(&mut self.save_dir_text).lost_focus() {
                self.config.export.save_dir = std::path::PathBuf::from(self.save_dir_text.trim());
            }
        });
    }

    fn theme_settings_ui(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("主题")
            .selected_text(format!("{:?}", self.config.theme))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.config.theme, ThemeMode::Light, "亮色");
                ui.selectable_value(&mut self.config.theme, ThemeMode::Dark, "暗色");
                ui.selectable_value(&mut self.config.theme, ThemeMode::System, "跟随系统");
            });
        theme::color_button(ui, "遮罩 RGBA", &mut self.config.overlay.mask_rgba);
        theme::color_button(ui, "选框颜色", &mut self.config.overlay.stroke_rgba);
        ui.add(egui::Slider::new(&mut self.config.overlay.stroke_width, 1.0..=8.0).text("选框线宽"));
        egui::ComboBox::from_label("工具栏位置")
            .selected_text(format!("{:?}", self.config.overlay.toolbar_position))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.config.overlay.toolbar_position, ToolbarPosition::Top, "顶部");
                ui.selectable_value(&mut self.config.overlay.toolbar_position, ToolbarPosition::Bottom, "底部");
                ui.selectable_value(&mut self.config.overlay.toolbar_position, ToolbarPosition::Left, "左侧");
                ui.selectable_value(&mut self.config.overlay.toolbar_position, ToolbarPosition::Right, "右侧");
                ui.selectable_value(&mut self.config.overlay.toolbar_position, ToolbarPosition::FollowCursor, "跟随鼠标");
            });
        ui.checkbox(&mut self.config.overlay.rainbow_border, "彩虹边框");
        ui.add(egui::Slider::new(&mut self.config.overlay.rainbow_width, 1.0..=10.0).text("彩虹边框宽度"));
        ui.add(egui::Slider::new(&mut self.config.overlay.rainbow_speed, 0.05..=2.0).text("彩虹动画速度"));
    }

    fn behavior_settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.config.behavior.delay_ms, 0..=5000).text("截图前延迟 ms"));
        ui.checkbox(&mut self.config.behavior.freeze_screen_before_select, "先冻结屏幕再选择");
        ui.checkbox(&mut self.config.overlay.show_magnifier, "显示取色放大镜");
        ui.checkbox(&mut self.config.overlay.show_size_label, "显示选区尺寸");
        ui.checkbox(&mut self.config.overlay.snap_to_windows, "窗口悬停高亮与吸附");
        ui.checkbox(&mut self.config.behavior.show_quick_tip, "截图时显示快捷提示");
        ui.add(egui::Slider::new(&mut self.config.behavior.min_selection_px, 1..=32).text("最小选区像素"));
    }

    fn start_capture(&mut self, ctx: &egui::Context) {
        if self.config.behavior.delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.config.behavior.delay_ms));
        }
        match capture::capture_primary() {
            Ok(frame) => {
                let image = egui::ColorImage::from_rgba_unmultiplied(
                    [frame.width as usize, frame.height as usize],
                    frame.image.as_raw(),
                );
                self.texture = Some(ctx.load_texture("shotrs-screen", image, egui::TextureOptions::LINEAR));
                self.windows = capture::list_windows().unwrap_or_default();
                self.hovered_window = None;
                self.selection = None;
                self.frame = Some(frame);
                self.mode = AppMode::Selecting;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                ctx.request_repaint();
            }
            Err(err) => self.status = format!("截图失败：{err}"),
        }
    }

    fn overlay_ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let Some(texture) = self.texture.clone() else { self.cancel_capture(ui.ctx()); return; };
        let Some(screen_frame) = self.frame.clone() else { self.cancel_capture(ui.ctx()); return; };

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                let rect = ui.max_rect();
                let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                let painter = ui.painter_at(rect);
                painter.image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                self.handle_overlay_input(ui, &response, rect, &screen_frame);

                shape::draw_mask_with_hole(&painter, rect, self.selection.as_ref(), self.config.overlay.mask_rgba.to_egui());

                if let Some(window) = &self.hovered_window {
                    let win_rect = egui::Rect::from_min_size(
                        egui::pos2(window.x as f32 - screen_frame.origin_x as f32, window.y as f32 - screen_frame.origin_y as f32),
                        egui::vec2(window.width as f32, window.height as f32),
                    );
                    painter.rect_stroke(win_rect, 3.0, egui::Stroke::new(2.0, egui::Color32::YELLOW), egui::StrokeKind::Outside);
                    painter.text(win_rect.left_top() + egui::vec2(8.0, 8.0), egui::Align2::LEFT_TOP, &window.title, egui::FontId::proportional(14.0), egui::Color32::YELLOW);
                }

                if let Some(selection) = &self.selection {
                    let stroke_color = if self.config.overlay.rainbow_border {
                        theme::rainbow_color(ui.input(|i| i.time), self.config.overlay.rainbow_speed)
                    } else {
                        self.config.overlay.stroke_rgba.to_egui()
                    };
                    let width = if self.config.overlay.rainbow_border { self.config.overlay.rainbow_width } else { self.config.overlay.stroke_width };
                    selection.draw(&painter, egui::Stroke::new(width, stroke_color), None);
                    self.draw_selection_helpers(ui, selection, rect);
                }

                self.draw_toolbar(ui, rect);
            });
    }

    fn handle_overlay_input(&mut self, ui: &egui::Ui, response: &egui::Response, display_rect: egui::Rect, frame: &ScreenFrame) {
        let ctx = ui.ctx();
        let escape = ui.input(|input| input.key_pressed(egui::Key::Escape));
        if escape {
            self.cancel_capture(ctx);
            return;
        }

        let confirm = ui.input(|input| input.key_pressed(egui::Key::Enter));
        if confirm {
            self.confirm_selection(ctx, display_rect);
            return;
        }

        let pointer_pos = response.interact_pointer_pos();
        let shift = ui.input(|input| input.modifiers.shift);

        match self.selected_shape {
            ShapeKind::Fullscreen => {
                if self.selection.is_none() {
                    self.selection = Some(SelectionShape::fullscreen(display_rect));
                }
            }
            ShapeKind::Window => {
                if let Some(pos) = pointer_pos {
                    let screen_pos = egui::pos2(pos.x + frame.origin_x as f32, pos.y + frame.origin_y as f32);
                    self.hovered_window = capture::window_at(screen_pos, &self.windows);
                    if response.clicked() {
                        if let Some(window) = self.hovered_window.clone() {
                            let mut selection = SelectionShape::from_window(window.clone());
                            selection.start -= egui::vec2(frame.origin_x as f32, frame.origin_y as f32);
                            selection.current -= egui::vec2(frame.origin_x as f32, frame.origin_y as f32);
                            self.selection = Some(selection);
                            self.confirm_selection(ctx, display_rect);
                        }
                    }
                }
            }
            ShapeKind::Polygon => {
                if response.double_clicked() {
                    if let Some(selection) = &mut self.selection {
                        selection.close();
                    }
                    self.confirm_selection(ctx, display_rect);
                } else if response.clicked() {
                    if let Some(pos) = pointer_pos {
                        if let Some(selection) = &mut self.selection {
                            selection.add_polygon_point(pos);
                        } else {
                            self.selection = Some(SelectionShape::new(ShapeKind::Polygon, pos));
                        }
                    }
                }
            }
            ShapeKind::Lasso => {
                if response.drag_started() {
                    if let Some(pos) = pointer_pos {
                        self.selection = Some(SelectionShape::new(ShapeKind::Lasso, pos));
                    }
                }
                if response.dragged() {
                    if let Some(pos) = pointer_pos {
                        if let Some(selection) = self.selection.as_mut() {
                            selection.update_drag(pos, shift);
                        }
                    }
                }
                if ui.input(|input| input.pointer.any_released()) {
                    if let Some(selection) = &mut self.selection {
                        selection.close();
                    }
                }
            }
            ShapeKind::Rectangle | ShapeKind::Ellipse => {
                if response.drag_started() {
                    if let Some(pos) = pointer_pos {
                        self.selection = Some(SelectionShape::new(self.selected_shape, pos));
                    }
                }
                if response.dragged() {
                    if let Some(pos) = pointer_pos {
                        if let Some(selection) = self.selection.as_mut() {
                            selection.update_drag(pos, shift);
                        }
                    }
                }
                if ui.input(|input| input.pointer.any_released()) {
                    if let Some(selection) = &mut self.selection {
                        selection.close();
                    }
                }
            }
        }
    }

    fn draw_selection_helpers(&self, ui: &mut egui::Ui, selection: &SelectionShape, display_rect: egui::Rect) {
        if self.config.overlay.show_size_label {
            let bounds = selection.bounds().intersect(display_rect);
            let label_pos = bounds.left_bottom() + egui::vec2(0.0, 8.0);
            ui.painter().text(
                label_pos,
                egui::Align2::LEFT_TOP,
                selection.size_label(),
                egui::FontId::monospace(13.0),
                egui::Color32::WHITE,
            );
        }
        if self.config.overlay.show_magnifier {
            if let Some(pos) = ui.input(|input| input.pointer.hover_pos()) {
                let box_rect = egui::Rect::from_min_size(pos + egui::vec2(18.0, 18.0), egui::vec2(110.0, 52.0));
                ui.painter().rect_filled(box_rect, 8.0, egui::Color32::from_black_alpha(210));
                ui.painter().text(
                    box_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("x {:.0}\ny {:.0}", pos.x, pos.y),
                    egui::FontId::monospace(12.0),
                    egui::Color32::WHITE,
                );
            }
        }
    }

    fn draw_toolbar(&mut self, ui: &mut egui::Ui, display_rect: egui::Rect) {
        let pos = match self.config.overlay.toolbar_position {
            ToolbarPosition::Top => display_rect.center_top() + egui::vec2(-160.0, 18.0),
            ToolbarPosition::Bottom => display_rect.center_bottom() + egui::vec2(-160.0, -64.0),
            ToolbarPosition::Left => display_rect.left_center() + egui::vec2(18.0, -36.0),
            ToolbarPosition::Right => display_rect.right_center() + egui::vec2(-340.0, -36.0),
            ToolbarPosition::FollowCursor => ui.input(|input| input.pointer.hover_pos()).unwrap_or(display_rect.center()) + egui::vec2(18.0, 18.0),
        };

        egui::Area::new("shotrs-toolbar".into())
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("确认 Enter").clicked() {
                            self.confirm_selection(ui.ctx(), display_rect);
                        }
                        if ui.button("取消 Esc").clicked() {
                            self.cancel_capture(ui.ctx());
                        }
                        ui.separator();
                        ui.label(self.selected_shape.label());
                        if self.config.behavior.show_quick_tip {
                            ui.small("Shift 锁正方形 / 双击闭合多边形");
                        }
                    });
                });
            });
    }

    fn confirm_selection(&mut self, ctx: &egui::Context, display_rect: egui::Rect) {
        let Some(selection) = self.selection.clone() else {
            self.status = "没有有效选区".to_owned();
            return;
        };

        if !selection.is_ready(self.config.behavior.min_selection_px as f32) {
            self.status = "选区太小或未闭合".to_owned();
            return;
        }

        let result: anyhow::Result<RgbaImage> = if selection.kind == ShapeKind::Window {
            if let Some(window) = &selection.window {
                capture::capture_window(window).or_else(|_| {
                    self.frame
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("No screen frame"))
                        .and_then(|frame| shape::crop_selection(frame, &selection, display_rect))
                })
            } else {
                anyhow::bail!("No window selected");
            }
        } else {
            self.frame
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("No screen frame"))
                .and_then(|frame| shape::crop_selection(frame, &selection, display_rect))
        };

        match result.and_then(|img| export::export_image(&img, &self.config)) {
            Ok(exported) => {
                let mut parts = Vec::new();
                if let Some(path) = exported.path {
                    parts.push(format!("已保存：{}", path.display()));
                }
                if exported.copied_to_clipboard {
                    parts.push("已复制到剪贴板".to_owned());
                }
                self.status = if parts.is_empty() { "已完成".to_owned() } else { parts.join("；") };
                self.cancel_capture(ctx);
            }
            Err(err) => self.status = format!("导出失败：{err}"),
        }
    }

    fn cancel_capture(&mut self, ctx: &egui::Context) {
        self.mode = AppMode::Main;
        self.frame = None;
        self.texture = None;
        self.selection = None;
        self.hovered_window = None;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(420.0, 560.0)));
        ctx.request_repaint();
    }

    fn save_config_status(&mut self) {
        self.config.export.save_dir = std::path::PathBuf::from(self.save_dir_text.trim());
        match self.config.save() {
            Ok(_) => self.status = "配置已保存".to_owned(),
            Err(err) => self.status = format!("配置保存失败：{err}"),
        }
    }
}
