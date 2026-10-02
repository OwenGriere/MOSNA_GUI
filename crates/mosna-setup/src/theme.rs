//! The look of both windows: the logo's colours — pastel cells, slate edges,
//! blue nodes — with the logo itself drawn faintly behind the contents.
//!
//! The logo is built into the program rather than read from the MOSNA GUI
//! folder: the uninstaller may run once that folder is gone, and a background
//! that depends on where the program was started is a background that goes
//! missing.

use egui::{Color32, CornerRadius, Stroke, Visuals};

/// The logo, a PNG whatever its extension says.
const LOGO: &[u8] = include_bytes!("../../../assets/logo.ico");

/// The window: the cream of the logo's lightest cells.
pub const BACKGROUND: Color32 = Color32::from_rgb(0xFA, 0xF6, 0xEA);
/// Controls at rest.
pub const SURFACE: Color32 = Color32::from_rgb(0xEE, 0xF1, 0xE4);
/// Controls under the pointer: the logo's pale teal.
pub const SURFACE_HOVER: Color32 = Color32::from_rgb(0xD9, 0xEE, 0xEA);
/// Hairlines.
pub const BORDER: Color32 = Color32::from_rgb(0xB9, 0xC2, 0xB4);
/// What the user types into.
pub const FIELD: Color32 = Color32::from_rgb(0xFF, 0xFE, 0xFA);
/// The console stays dark, as a terminal is: cargo's colours are meant for
/// one. Translucent, so the logo still shows through it.
pub const CONSOLE: Color32 = Color32::from_rgba_premultiplied(0x1E, 0x20, 0x29, 0xEB);

/// The logo's node blue: titles, the main button, the selection.
pub const BLUE: Color32 = Color32::from_rgb(0x1F, 0x6F, 0xAE);
/// A deeper blue, for titles, which are read rather than clicked.
pub const BLUE_DEEP: Color32 = Color32::from_rgb(0x17, 0x52, 0x82);
/// The logo's teal, the fill of a pressed control, under dark text.
pub const TEAL: Color32 = Color32::from_rgb(0x99, 0xCF, 0xCA);

/// Text: the slate of the logo's edges.
pub const TEXT: Color32 = Color32::from_rgb(0x2A, 0x2C, 0x36);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x5E, 0x65, 0x6E);

/// Warnings and errors written in the window itself, on the light background.
const WARNING_TEXT: Color32 = Color32::from_rgb(0x8A, 0x5A, 0x00);
const ERROR_TEXT: Color32 = Color32::from_rgb(0xB3, 0x26, 0x1E);

/// The console's code: the conventional hues, light enough for its dark
/// background.
pub const LOG_ERROR: Color32 = Color32::from_rgb(0xF0, 0x6A, 0x5C);
pub const LOG_WARNING: Color32 = Color32::from_rgb(0xF2, 0xC2, 0x4B);
pub const LOG_INFO: Color32 = Color32::from_rgb(0x7F, 0xB2, 0xE5);
pub const LOG_SUCCESS: Color32 = Color32::from_rgb(0x8B, 0xD0, 0x7A);
/// The start of a stage: the logo's teal.
pub const LOG_STEP: Color32 = Color32::from_rgb(0x8F, 0xD6, 0xCF);
pub const LOG_PLAIN: Color32 = Color32::from_rgb(0xCF, 0xD3, 0xDA);

/// The console's type size.
pub const MONO_SIZE: f32 = 12.5;

/// How much of the logo shows behind the contents.
const LOGO_OPACITY: u8 = 46;

/// Install the palette on a context.
pub fn apply(ctx: &egui::Context) {
    let mut visuals = Visuals::light();
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = SURFACE;
    visuals.extreme_bg_color = FIELD;
    visuals.faint_bg_color = SURFACE;
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.selection.bg_fill = TEAL;
    visuals.selection.stroke = Stroke::new(1.0, BLUE);
    visuals.hyperlink_color = BLUE;
    visuals.warn_fg_color = WARNING_TEXT;
    visuals.error_fg_color = ERROR_TEXT;
    visuals.weak_text_color = Some(TEXT_MUTED);

    let rounding = CornerRadius::same(4);
    let widgets = &mut visuals.widgets;

    widgets.noninteractive.bg_fill = BACKGROUND;
    widgets.noninteractive.weak_bg_fill = BACKGROUND;
    widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    widgets.noninteractive.corner_radius = rounding;

    widgets.inactive.bg_fill = SURFACE;
    widgets.inactive.weak_bg_fill = SURFACE;
    widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    widgets.inactive.corner_radius = rounding;

    widgets.hovered.bg_fill = SURFACE_HOVER;
    widgets.hovered.weak_bg_fill = SURFACE_HOVER;
    widgets.hovered.bg_stroke = Stroke::new(1.0, BLUE);
    widgets.hovered.fg_stroke = Stroke::new(1.5, TEXT);
    widgets.hovered.corner_radius = rounding;

    // Also the colour of strong text, so it stays dark.
    widgets.active.bg_fill = TEAL;
    widgets.active.weak_bg_fill = TEAL;
    widgets.active.bg_stroke = Stroke::new(1.0, BLUE);
    widgets.active.fg_stroke = Stroke::new(1.5, TEXT);
    widgets.active.corner_radius = rounding;

    widgets.open.bg_fill = SURFACE;
    widgets.open.bg_stroke = Stroke::new(1.0, BLUE);
    widgets.open.fg_stroke = Stroke::new(1.0, TEXT);
    widgets.open.corner_radius = rounding;

    // Whatever the system's preference, these windows are light.
    ctx.set_theme(egui::Theme::Light);
    ctx.set_visuals_of(egui::Theme::Light, visuals);
    ctx.all_styles_mut(|style| {
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        if let Some(mono) = style.text_styles.get_mut(&egui::TextStyle::Monospace) {
            mono.size = MONO_SIZE;
        }
    });
}

fn logo_image() -> Option<image::RgbaImage> {
    image::load_from_memory(LOGO)
        .ok()
        .map(|logo| logo.into_rgba8())
}

/// The logo as the window's icon.
pub fn icon() -> Option<egui::IconData> {
    let image = logo_image()?;
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}

/// The logo behind the contents.
pub struct Backdrop {
    texture: Option<egui::TextureHandle>,
}

impl Backdrop {
    pub fn new(ctx: &egui::Context) -> Self {
        let texture = logo_image().map(|image| {
            let size = [image.width() as usize, image.height() as usize];
            let pixels = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
            ctx.load_texture("mosna-logo", pixels, egui::TextureOptions::LINEAR)
        });
        Self { texture }
    }

    /// Draw the logo, faint and centred, over the whole of `ui`. Called before
    /// the contents, so they are drawn over it.
    pub fn paint(&self, ui: &egui::Ui) {
        let Some(texture) = &self.texture else {
            return;
        };
        let area = ui.max_rect();
        let side = area.width().min(area.height()) * 0.92;
        let rect = egui::Rect::from_center_size(area.center(), egui::vec2(side, side));
        ui.painter().image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::from_white_alpha(LOGO_OPACITY),
        );
    }
}

/// The window's title, in gold.
pub fn heading(ui: &mut egui::Ui, text: &str) {
    ui.heading(egui::RichText::new(text).color(BLUE_DEEP).strong());
}

/// The button that does what the window is for: blue, with light text.
pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(text).color(FIELD).strong())
            .fill(BLUE)
            .stroke(Stroke::new(1.0, BLUE_DEEP)),
    )
}
