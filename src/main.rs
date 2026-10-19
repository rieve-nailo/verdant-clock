use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{Local, Timelike};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};

const SETTINGS_PANEL_WIDTH: f32 = 270.0;
const SETTINGS_EDGE_HOTSPOT: f32 = 12.0;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 680.0])
            .with_min_inner_size([720.0, 500.0])
            .with_title("Verdant Clock")
            .with_transparent(true),
        ..Default::default()
    };

    eframe::run_native(
        "Verdant Clock",
        options,
        Box::new(|creation_context| Ok(Box::new(ClockApp::new(creation_context)))),
    )
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct PersistedSettings {
    face_color: [u8; 4],
    hand_color: [u8; 4],
    marker_color: [u8; 4],
    numeral_color: [u8; 4],
    accent_color: [u8; 4],
    face_opacity: u8,
    element_opacity: u8,
    window_opacity: u8,
    always_on_top: bool,
    menu_auto_hide: bool,
    hide_title_bar: bool,
    background_path: Option<PathBuf>,
}

impl Default for PersistedSettings {
    fn default() -> Self {
        Self {
            face_color: [13, 34, 28, 255],
            hand_color: [191, 244, 205, 255],
            marker_color: [86, 190, 122, 255],
            numeral_color: [172, 219, 185, 255],
            accent_color: [91, 235, 139, 255],
            face_opacity: 92,
            element_opacity: 100,
            window_opacity: 100,
            always_on_top: false,
            menu_auto_hide: true,
            hide_title_bar: false,
            background_path: None,
        }
    }
}

struct ClockApp {
    face_color: Color32,
    hand_color: Color32,
    marker_color: Color32,
    numeral_color: Color32,
    accent_color: Color32,
    face_opacity: u8,
    element_opacity: u8,
    window_opacity: u8,
    settings_visible: bool,
    always_on_top: bool,
    menu_auto_hide: bool,
    hide_title_bar: bool,
    background: Option<egui::TextureHandle>,
    background_path: Option<PathBuf>,
    background_error: Option<String>,
}

impl Default for ClockApp {
    fn default() -> Self {
        Self {
            face_color: Color32::from_rgb(13, 34, 28),
            hand_color: Color32::from_rgb(191, 244, 205),
            marker_color: Color32::from_rgb(86, 190, 122),
            numeral_color: Color32::from_rgb(172, 219, 185),
            accent_color: Color32::from_rgb(91, 235, 139),
            face_opacity: 92,
            element_opacity: 100,
            window_opacity: 100,
            settings_visible: false,
            always_on_top: false,
            menu_auto_hide: true,
            hide_title_bar: false,
            background: None,
            background_path: None,
            background_error: None,
        }
    }
}

impl ClockApp {
    fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        let settings = creation_context
            .storage
            .and_then(|storage| eframe::get_value::<PersistedSettings>(storage, "settings"))
            .unwrap_or_default();

        let mut app = Self {
            face_color: Color32::from_rgba_unmultiplied(
                settings.face_color[0],
                settings.face_color[1],
                settings.face_color[2],
                settings.face_color[3],
            ),
            hand_color: Color32::from_rgba_unmultiplied(
                settings.hand_color[0],
                settings.hand_color[1],
                settings.hand_color[2],
                settings.hand_color[3],
            ),
            marker_color: Color32::from_rgba_unmultiplied(
                settings.marker_color[0],
                settings.marker_color[1],
                settings.marker_color[2],
                settings.marker_color[3],
            ),
            numeral_color: Color32::from_rgba_unmultiplied(
                settings.numeral_color[0],
                settings.numeral_color[1],
                settings.numeral_color[2],
                settings.numeral_color[3],
            ),
            accent_color: Color32::from_rgba_unmultiplied(
                settings.accent_color[0],
                settings.accent_color[1],
                settings.accent_color[2],
                settings.accent_color[3],
            ),
            face_opacity: settings.face_opacity,
            element_opacity: settings.element_opacity,
            window_opacity: settings.window_opacity,
            settings_visible: false,
            always_on_top: settings.always_on_top,
            menu_auto_hide: settings.menu_auto_hide,
            hide_title_bar: settings.hide_title_bar,
            background: None,
            background_path: None,
            background_error: None,
        };

        if let Some(path) = settings.background_path {
            app.background_path = Some(path.clone());
            app.load_background(path, &creation_context.egui_ctx, false);
        }

        app
    }
}

impl eframe::App for ClockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(33));

        let pointer_x = ctx
            .input(|input| input.pointer.hover_pos())
            .map(|position| position.x);
        let popup_open = ctx.memory(|memory| memory.any_popup_open());
        self.settings_visible = should_show_settings(
            self.menu_auto_hide,
            self.settings_visible,
            pointer_x,
            popup_open,
        );
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(if self.always_on_top {
            egui::WindowLevel::AlwaysOnTop
        } else {
            egui::WindowLevel::Normal
        }));
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(!self.hide_title_bar));

        if self.settings_visible {
            egui::SidePanel::left("clock_settings")
                .resizable(false)
                .exact_width(SETTINGS_PANEL_WIDTH)
                .frame(egui::Frame::default().fill(Color32::from_rgba_unmultiplied(
                    12,
                    25,
                    21,
                    opacity_alpha(self.window_opacity),
                )))
                .show(ctx, |ui| self.show_settings(ui));
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(Color32::from_rgba_unmultiplied(
                8,
                17,
                15,
                opacity_alpha(self.window_opacity),
            )))
            .show(ctx, |ui| self.show_clock(ui));
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(
            storage,
            "settings",
            &PersistedSettings {
                face_color: self.face_color.to_array(),
                hand_color: self.hand_color.to_array(),
                marker_color: self.marker_color.to_array(),
                numeral_color: self.numeral_color.to_array(),
                accent_color: self.accent_color.to_array(),
                face_opacity: self.face_opacity,
                element_opacity: self.element_opacity,
                window_opacity: self.window_opacity,
                always_on_top: self.always_on_top,
                menu_auto_hide: self.menu_auto_hide,
                hide_title_bar: self.hide_title_bar,
                background_path: self.background_path.clone(),
            },
        );
    }
}

impl ClockApp {
    fn show_settings(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);
        ui.heading("Verdant Clock");
        ui.label(
            egui::RichText::new("A little time, your way.").color(Color32::from_rgb(133, 171, 145)),
        );
        ui.add_space(20.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label(egui::RichText::new("CLOCK COLORS").strong().size(11.0));
        ui.add_space(7.0);

        color_setting(ui, "Clock face", &mut self.face_color);
        color_setting(ui, "Hands", &mut self.hand_color);
        color_setting(ui, "Hour markers", &mut self.marker_color);
        color_setting(ui, "Numerals", &mut self.numeral_color);
        color_setting(ui, "Second hand", &mut self.accent_color);

        ui.add_space(10.0);
        ui.label("Clock face opacity");
        ui.add(
            egui::Slider::new(&mut self.face_opacity, 0..=100)
                .suffix("%")
                .show_value(true),
        );
        ui.label("Clock element opacity");
        ui.add(
            egui::Slider::new(&mut self.element_opacity, 0..=100)
                .suffix("%")
                .show_value(true),
        );
        ui.label("Window background opacity");
        ui.add(
            egui::Slider::new(&mut self.window_opacity, 0..=100)
                .suffix("%")
                .show_value(true),
        );

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label(egui::RichText::new("BACKGROUND").strong().size(11.0));
        ui.add_space(7.0);
        if ui.button("Choose background image…").clicked() {
            self.choose_background(ui.ctx());
        }
        if let Some(path) = &self.background_path {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("Selected image"),
                )
                .small()
                .color(Color32::from_rgb(151, 185, 160)),
            );
            if ui.small_button("Remove image").clicked() {
                self.background = None;
                self.background_path = None;
                self.background_error = None;
            }
        }
        if let Some(error) = &self.background_error {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(error).small().color(Color32::LIGHT_RED));
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
        ui.checkbox(&mut self.always_on_top, "Keep clock always on top");
        ui.checkbox(&mut self.menu_auto_hide, "Auto-hide settings menu");
        ui.checkbox(&mut self.hide_title_bar, "Hide title bar");
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Runs on Windows, macOS, and Debian Linux")
                    .small()
                    .color(Color32::from_rgb(107, 139, 118)),
            );
        });
    }

    fn choose_background(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Image files", &["png", "jpg", "jpeg", "webp", "bmp"])
            .pick_file()
        else {
            return;
        };

        self.load_background(path, ctx, true);
    }

    fn load_background(&mut self, path: PathBuf, ctx: &egui::Context, copy_to_app_data: bool) {
        let image = match image::open(&path) {
            Ok(image) => image,
            Err(error) => {
                self.background_error = Some(format!("Could not open image: {error}"));
                return;
            }
        };

        let background_path = if copy_to_app_data {
            match copy_background_to_app_data(&path) {
                Ok(path) => path,
                Err(error) => {
                    self.background_error = Some(error);
                    return;
                }
            }
        } else {
            path
        };

        let image = image.to_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
        self.background = Some(ctx.load_texture(
            "clock-background",
            color_image,
            egui::TextureOptions::LINEAR,
        ));
        self.background_path = Some(background_path);
        self.background_error = None;
    }

    fn show_clock(&self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let (response, painter) = ui.allocate_painter(available, Sense::click_and_drag());
        if response.drag_started() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        let bounds = response.rect;
        painter.rect_filled(
            bounds,
            0.0,
            Color32::from_rgba_unmultiplied(8, 17, 15, opacity_alpha(self.window_opacity)),
        );

        let side = bounds.width().min(bounds.height());
        let radius = (side * 0.39).max(80.0);
        let center = bounds.center();

        if let Some(background) = &self.background {
            let image_rect = cover_rect(bounds, background.size_vec2());
            painter.image(
                background.id(),
                image_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                with_opacity(Color32::WHITE, self.window_opacity),
            );
        }

        let face = with_opacity(self.face_color, self.face_opacity);
        painter.circle_filled(center, radius, face);
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(
                2.0_f32,
                with_opacity(
                    self.marker_color,
                    self.element_opacity.saturating_mul(80) / 100,
                ),
            ),
        );

        for hour in 0..60 {
            let angle = hour as f32 * std::f32::consts::TAU / 60.0;
            let is_hour = hour % 5 == 0;
            let outer = point_on_clock(center, radius * 0.91, angle);
            let inner = point_on_clock(center, radius * if is_hour { 0.82 } else { 0.87 }, angle);
            painter.line_segment(
                [inner, outer],
                Stroke::new(
                    if is_hour { 2.5 } else { 1.0_f32 },
                    with_opacity(
                        self.marker_color,
                        if is_hour {
                            self.element_opacity
                        } else {
                            self.element_opacity.saturating_mul(55) / 100
                        },
                    ),
                ),
            );
        }

        for hour in 1..=12 {
            let angle = hour as f32 * std::f32::consts::TAU / 12.0;
            let position = point_on_clock(center, radius * 0.70, angle);
            painter.text(
                position,
                egui::Align2::CENTER_CENTER,
                hour.to_string(),
                egui::FontId::proportional((radius * 0.105).clamp(14.0, 28.0)),
                with_opacity(self.numeral_color, self.element_opacity),
            );
        }

        let now = Local::now();
        let seconds = now.second() as f32 + now.nanosecond() as f32 / 1_000_000_000.0;
        let minutes = now.minute() as f32 + seconds / 60.0;
        let hours = (now.hour() % 12) as f32 + minutes / 60.0;

        draw_hand(
            &painter,
            center,
            radius * 0.48,
            hours * std::f32::consts::TAU / 12.0,
            radius * 0.038,
            with_opacity(self.hand_color, self.element_opacity),
        );
        draw_hand(
            &painter,
            center,
            radius * 0.68,
            minutes * std::f32::consts::TAU / 60.0,
            radius * 0.024,
            with_opacity(self.hand_color, self.element_opacity),
        );
        draw_hand(
            &painter,
            center,
            radius * 0.78,
            seconds * std::f32::consts::TAU / 60.0,
            radius * 0.009,
            with_opacity(self.accent_color, self.element_opacity),
        );
        painter.circle_filled(
            center,
            radius * 0.045,
            with_opacity(self.accent_color, self.element_opacity),
        );
    }
}

fn should_show_settings(
    auto_hide: bool,
    currently_visible: bool,
    pointer_x: Option<f32>,
    popup_open: bool,
) -> bool {
    if !auto_hide || popup_open {
        return true;
    }

    pointer_x.is_some_and(|x| {
        x <= SETTINGS_EDGE_HOTSPOT || (x <= SETTINGS_PANEL_WIDTH && currently_visible)
    })
}

fn opacity_alpha(opacity: u8) -> u8 {
    ((u16::from(opacity) * 255) / 100) as u8
}

fn with_opacity(color: Color32, opacity: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        ((u16::from(color.a()) * u16::from(opacity)) / 100) as u8,
    )
}

fn copy_background_to_app_data(source: &std::path::Path) -> Result<PathBuf, String> {
    let project_dirs = directories::ProjectDirs::from("com", "VerdantClock", "verdant-clock")
        .ok_or_else(|| "Could not determine the application's data directory.".to_owned())?;
    let background_dir = project_dirs.data_local_dir().join("backgrounds");
    copy_background_to_directory(source, &background_dir)
}

fn copy_background_to_directory(
    source: &std::path::Path,
    background_dir: &std::path::Path,
) -> Result<PathBuf, String> {
    let extension = source
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .filter(|extension| ["bmp", "jpeg", "jpg", "png", "webp"].contains(&extension.as_str()))
        .ok_or_else(|| "The selected image has an unsupported file extension.".to_owned())?;
    fs::create_dir_all(background_dir).map_err(|error| {
        format!(
            "Could not create the local background directory {}: {error}",
            background_dir.display()
        )
    })?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("Could not determine the current time: {error}"))?
        .as_nanos();
    let mut sequence = 0_u32;
    let (destination, mut destination_file) = loop {
        let file_name = format!("background-{timestamp}-{sequence}.{extension}");
        let destination = background_dir.join(file_name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
        {
            Ok(file) => break (destination, file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                sequence = sequence.checked_add(1).ok_or_else(|| {
                    "Could not create a unique local background image name.".to_owned()
                })?;
            }
            Err(error) => {
                return Err(format!(
                    "Could not create a local copy of the background image: {error}"
                ));
            }
        }
    };

    let copy_result = (|| {
        let mut source_file = File::open(source)
            .map_err(|error| format!("Could not read the selected image: {error}"))?;
        io::copy(&mut source_file, &mut destination_file)
            .map_err(|error| format!("Could not copy the selected image: {error}"))?;
        destination_file
            .flush()
            .map_err(|error| format!("Could not finish saving the background image: {error}"))?;
        Ok::<(), String>(())
    })();

    if let Err(error) = copy_result {
        if let Err(cleanup_error) = fs::remove_file(&destination) {
            return Err(format!(
                "{error}; additionally, the incomplete copy at {} could not be removed: {cleanup_error}",
                destination.display()
            ));
        }
        return Err(error);
    }

    Ok(destination)
}

fn color_setting(ui: &mut egui::Ui, label: &str, color: &mut Color32) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.color_edit_button_srgba(color);
        });
    });
}

fn point_on_clock(center: Pos2, radius: f32, angle: f32) -> Pos2 {
    Pos2::new(
        center.x + angle.sin() * radius,
        center.y - angle.cos() * radius,
    )
}

fn cover_rect(bounds: Rect, image_size: Vec2) -> Rect {
    let scale = (bounds.width() / image_size.x).max(bounds.height() / image_size.y);
    let scaled_size = image_size * scale;
    Rect::from_center_size(bounds.center(), scaled_size)
}

fn draw_hand(
    painter: &egui::Painter,
    center: Pos2,
    length: f32,
    angle: f32,
    width: f32,
    color: Color32,
) {
    painter.line_segment(
        [center, point_on_clock(center, length, angle)],
        Stroke::new(width, color),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_points_follow_clockwise_dial_positions() {
        let center = Pos2::new(10.0, 10.0);
        let top = point_on_clock(center, 5.0, 0.0);
        let right = point_on_clock(center, 5.0, std::f32::consts::FRAC_PI_2);

        assert!((top.x - 10.0).abs() < 0.001);
        assert!((top.y - 5.0).abs() < 0.001);
        assert!((right.x - 15.0).abs() < 0.001);
        assert!((right.y - 10.0).abs() < 0.001);
    }

    #[test]
    fn opacity_scales_existing_alpha() {
        assert_eq!(opacity_alpha(0), 0);
        assert_eq!(opacity_alpha(100), 255);
        assert_eq!(
            with_opacity(Color32::from_rgba_unmultiplied(10, 20, 30, 200), 50).a(),
            100
        );
    }

    #[test]
    fn settings_menu_stays_open_while_a_popup_is_open() {
        assert!(should_show_settings(true, true, Some(420.0), true));
        assert!(!should_show_settings(true, true, Some(420.0), false));
        assert!(should_show_settings(false, false, None, false));
    }

    #[test]
    fn background_cover_rect_preserves_aspect_ratio_and_covers_bounds() {
        let bounds = Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 200.0));
        let image_rect = cover_rect(bounds, Vec2::new(400.0, 200.0));

        assert!((image_rect.width() / image_rect.height() - 2.0).abs() < 0.001);
        assert!(image_rect.width() >= bounds.width());
        assert!(image_rect.height() >= bounds.height());
        assert!((image_rect.center().x - bounds.center().x).abs() < 0.001);
        assert!((image_rect.center().y - bounds.center().y).abs() < 0.001);
    }

    #[test]
    fn copied_background_survives_deletion_of_the_original() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_dir = std::env::temp_dir().join(format!("verdant-clock-test-{unique}"));
        let source = test_dir.join("original.png");
        let copy_dir = test_dir.join("app-data");
        fs::create_dir_all(&test_dir).unwrap();
        fs::write(&source, b"image data").unwrap();

        let copy = copy_background_to_directory(&source, &copy_dir).unwrap();
        fs::remove_file(&source).unwrap();

        assert_eq!(fs::read(&copy).unwrap(), b"image data");
        fs::remove_dir_all(test_dir).unwrap();
    }
}
