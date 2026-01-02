// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use cosmic_text::Buffer;

use crate::ext::canvas::font_system::{FontDescriptor, FontManager};

/// Metrics for measured text
#[derive(Debug, Clone)]
pub struct TextMetrics {
    /// Width of the text
    pub width: f32,
    /// Distance to the left of the alignment point
    pub actual_bounding_box_left: f32,
    /// Distance to the right of the alignment point
    pub actual_bounding_box_right: f32,
    /// Distance above the alphabetic baseline
    pub font_bounding_box_ascent: f32,
    /// Distance below the alphabetic baseline
    pub font_bounding_box_descent: f32,
    /// Actual distance above the alphabetic baseline
    pub actual_bounding_box_ascent: f32,
    /// Actual distance below the alphabetic baseline
    pub actual_bounding_box_descent: f32,
    /// Em height ascent
    pub em_height_ascent: f32,
    /// Em height descent
    pub em_height_descent: f32,
    /// Hanging baseline position
    pub hanging_baseline: f32,
    /// Alphabetic baseline position
    pub alphabetic_baseline: f32,
    /// Ideographic baseline position
    pub ideographic_baseline: f32,
}

impl TextMetrics {
    /// Measure text using cosmic-text layout
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

        let mut width = 0.0f32;
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                width = width.max(glyph.x + glyph.w);

                min_x = min_x.min(glyph.x);
                max_x = max_x.max(glyph.x + glyph.w);
                min_y = min_y.min(glyph.y - glyph.font_size);
                max_y = max_y.max(glyph.y);
            }
        }

        if min_x >= max_x {
            return Ok(Self {
                width: 0.0,
                actual_bounding_box_left: 0.0,
                actual_bounding_box_right: 0.0,
                font_bounding_box_ascent: 0.0,
                font_bounding_box_descent: 0.0,
                actual_bounding_box_ascent: 0.0,
                actual_bounding_box_descent: 0.0,
                em_height_ascent: 0.0,
                em_height_descent: 0.0,
                hanging_baseline: 0.0,
                alphabetic_baseline: 0.0,
                ideographic_baseline: 0.0,
            });
        }

        let size = font_descriptor.size;
        let ascent = size * 0.8;
        let descent = size * 0.2;

        Ok(Self {
            width,
            actual_bounding_box_left: min_x.abs(),
            actual_bounding_box_right: max_x,
            font_bounding_box_ascent: ascent,
            font_bounding_box_descent: descent,
            actual_bounding_box_ascent: min_y.abs(),
            actual_bounding_box_descent: max_y,
            em_height_ascent: size,
            em_height_descent: 0.0,
            hanging_baseline: size * 0.6,
            alphabetic_baseline: 0.0,
            ideographic_baseline: -descent,
        })
    }
}
