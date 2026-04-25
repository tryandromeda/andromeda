// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use cosmic_text::Buffer;

use crate::ext::canvas::font_system::{FontDescriptor, FontManager};

/// Metrics for measured text.
#[derive(Debug, Clone)]
pub struct TextMetrics {
    pub width: f32,
    pub actual_bounding_box_left: f32,
    pub actual_bounding_box_right: f32,
    pub font_bounding_box_ascent: f32,
    pub font_bounding_box_descent: f32,
    pub actual_bounding_box_ascent: f32,
    pub actual_bounding_box_descent: f32,
    pub em_height_ascent: f32,
    pub em_height_descent: f32,
    pub hanging_baseline: f32,
    pub alphabetic_baseline: f32,
    pub ideographic_baseline: f32,
}

impl TextMetrics {
    /// Measure text using cosmic-text layout, pulling ascent and descent from
    /// the laid-out run rather than guessing from the font size.
    pub fn measure(
        text: &str,
        font_manager: &mut FontManager,
        font_descriptor: &FontDescriptor,
    ) -> Result<Self, String> {
        let mut buffer = Buffer::new(
            &mut font_manager.font_system,
            cosmic_text::Metrics::new(font_descriptor.size, font_descriptor.size * 1.2),
        );

        let attrs = cosmic_text::Attrs::new()
            .family(cosmic_text::Family::Name(&font_descriptor.family))
            .weight(cosmic_text::Weight(font_descriptor.weight))
            .style(font_descriptor.style);

        buffer.set_text(
            &mut font_manager.font_system,
            text,
            &attrs,
            cosmic_text::Shaping::Advanced,
            None,
        );

        buffer.shape_until_scroll(&mut font_manager.font_system, false);

        let mut advance_width: f32 = 0.0;
        let mut ascent: f32 = 0.0;
        let mut descent: f32 = 0.0;
        let mut ink_left: f32 = 0.0;
        let mut ink_right: f32 = 0.0;
        let mut ink_top_above: f32 = 0.0; // distance above baseline (positive)
        let mut ink_bottom_below: f32 = 0.0; // distance below baseline (positive)
        let mut baseline_y: Option<f32> = None;
        let mut has_glyph = false;

        for run in buffer.layout_runs() {
            if baseline_y.is_none() {
                baseline_y = Some(run.line_y);
            }
            ascent = ascent.max(run.line_y - run.line_top);
            descent = descent.max(run.line_top + run.line_height - run.line_y);
            advance_width = advance_width.max(run.line_w);

            for glyph in run.glyphs.iter() {
                has_glyph = true;
                let glyph_right = glyph.x + glyph.w;
                ink_left = ink_left.min(glyph.x);
                ink_right = ink_right.max(glyph_right);
                // Vertical ink extent from font metrics as a first approximation
                ink_top_above = ink_top_above.max(run.line_y - run.line_top);
                ink_bottom_below =
                    ink_bottom_below.max(run.line_top + run.line_height - run.line_y);
            }
        }

        if !has_glyph {
            return Ok(Self {
                width: 0.0,
                actual_bounding_box_left: 0.0,
                actual_bounding_box_right: 0.0,
                font_bounding_box_ascent: ascent,
                font_bounding_box_descent: descent,
                actual_bounding_box_ascent: 0.0,
                actual_bounding_box_descent: 0.0,
                em_height_ascent: ascent,
                em_height_descent: descent,
                hanging_baseline: ascent * 0.8,
                alphabetic_baseline: 0.0,
                ideographic_baseline: -descent,
            });
        }

        let _ = baseline_y;

        Ok(Self {
            width: advance_width,
            actual_bounding_box_left: (-ink_left).max(0.0),
            actual_bounding_box_right: ink_right,
            font_bounding_box_ascent: ascent,
            font_bounding_box_descent: descent,
            actual_bounding_box_ascent: ink_top_above,
            actual_bounding_box_descent: ink_bottom_below,
            em_height_ascent: ascent,
            em_height_descent: descent,
            hanging_baseline: ascent * 0.8,
            alphabetic_baseline: 0.0,
            ideographic_baseline: -descent,
        })
    }
}
