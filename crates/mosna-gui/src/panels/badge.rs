//! The mark in the top-left corner, saying which implementation is running.
//!
//! Two applications share this working directory, this configuration file and
//! very nearly this interface: one drives the Python analyses, the other a
//! Rust pipeline. They are told apart at a glance by the badge here — a Python
//! mark in this one, a Rust cog in the other — which matters most in the
//! situation where it is easiest to get wrong, with both open side by side and
//! a figure to attribute to one of them.
//!
//! # Why it is painted rather than shipped as an image
//!
//! An image would need a file beside an installed binary, or to be compiled in
//! and decoded at start-up; and at the size this is drawn — one line of the
//! top bar — a bitmap is resampled every time the scale changes. A handful of
//! shapes is sharp at any size, follows the theme's colours where it should,
//! and adds nothing to the build.

use crate::theme;

/// Height of the mark, as a fraction of the row it sits in.
const SIZE: f32 = 22.0;

/// The Python blue and yellow, as the language's own mark uses them.
///
/// Not from the theme: this is somebody else's logo, and a recoloured one is
/// not recognisable, which is the only thing it is for.
const PYTHON_BLUE: egui::Color32 = egui::Color32::from_rgb(0x30, 0x6A, 0x9B);
const PYTHON_YELLOW: egui::Color32 = egui::Color32::from_rgb(0xD8, 0xAE, 0x1F);

/// Draw the badge and its caption.
pub fn show(ui: &mut egui::Ui) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(SIZE, SIZE), egui::Sense::hover());
    paint(ui.painter(), rect);
    response.on_hover_text(
        "MOSNA GUI — the analyses run in Python.\n\
         Steps 1 to 3 are `python -m package.*`; the figures are drawn by mosna_xy.",
    );
    ui.add_space(6.0);
}

/// The Python mark: two interlocking hooks, one in each of its colours.
///
/// Each hook is a bar across the middle and a leg down one side, drawn as two
/// rounded rectangles: the yellow one first, so the blue reads as passing in
/// front of it where they cross — which is the one place a flat drawing has to
/// choose, and the choice the real mark makes on its upper half.
pub fn paint(painter: &egui::Painter, rect: egui::Rect) {
    let at = |x: f32, y: f32| {
        egui::pos2(
            rect.left() + x * rect.width(),
            rect.top() + y * rect.height(),
        )
    };
    let box_of =
        |x0: f32, y0: f32, x1: f32, y1: f32| egui::Rect::from_min_max(at(x0, y0), at(x1, y1));
    let radius = egui::CornerRadius::same((rect.width() * 0.14).round().max(1.0) as u8);

    // The bars stop short of the middle rather than meeting there. What makes
    // the mark recognisable at this size is the step between the two colours
    // across the waist — blue on the left, yellow on the right — and bars that
    // met would leave that band a couple of pixels tall and hide it.
    //
    // The lower hook: the bottom bar, and the leg that climbs the right side.
    painter.rect_filled(box_of(0.09, 0.60, 0.91, 0.95), radius, PYTHON_YELLOW);
    painter.rect_filled(box_of(0.53, 0.28, 0.91, 0.95), radius, PYTHON_YELLOW);

    // The upper hook: the top bar, and the leg that falls down the left side.
    // Drawn last, so it passes in front where the two cross — the one place a
    // flat drawing has to choose, and the choice the real mark makes on its
    // upper half.
    painter.rect_filled(box_of(0.09, 0.05, 0.91, 0.40), radius, PYTHON_BLUE);
    painter.rect_filled(box_of(0.09, 0.05, 0.47, 0.72), radius, PYTHON_BLUE);

    // The eyes. Each sits in its own hook's bar, and is the page colour rather
    // than white so the mark belongs to the interface around it.
    let eye = rect.width() * 0.07;
    painter.circle_filled(at(0.28, 0.21), eye, theme::PANEL);
    painter.circle_filled(at(0.72, 0.79), eye, theme::PANEL);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every shape stays inside the square the badge was given: one that did
    /// not would be painted over the working-directory caption beside it.
    #[test]
    fn the_mark_stays_inside_its_square() {
        let ctx = egui::Context::default();
        let rect = egui::Rect::from_min_size(egui::pos2(10.0, 4.0), egui::vec2(SIZE, SIZE));

        let output = ctx.run_ui(Default::default(), |ui| paint(ui.painter(), rect));

        let painted: Vec<egui::Rect> = output
            .shapes
            .iter()
            .map(|clipped| clipped.shape.visual_bounding_rect())
            .filter(|bounds| bounds.is_finite() && bounds.area() > 0.0)
            .collect();

        assert!(!painted.is_empty(), "the badge painted nothing");
        for bounds in painted {
            assert!(
                rect.expand(1.0).contains_rect(bounds),
                "{bounds:?} escapes {rect:?}"
            );
        }
    }
}
