// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use cosmic_text::{Buffer, Placement, SwashCache};

use crate::ext::canvas::font_system::{FontDescriptor, FontManager};

/// Result of rasterizing a line of text.
///
/// The bitmap is sized to its actual ink bounds (so descenders are never
/// clipped). `baseline_y` is the row inside the bitmap where the alphabetic
/// baseline sits, and `x_offset` is the column where the pen origin (x = 0 in
/// glyph coordinates) sits. Callers use these to land the baseline at a
/// specific canvas coordinate regardless of which glyphs happen to appear.
pub struct RenderedText {
    pub bitmap: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// Distance from bitmap top to the alphabetic baseline, in pixels.
    pub baseline_y: f32,
    /// Distance from bitmap left to the pen origin, in pixels. Usually 0,
    /// but positive when the first glyph's ink extends left of its origin.
    pub x_offset: f32,
    /// Font ascent (em-ascent) in pixels for the dominant run.
    pub ascent: f32,
    /// Font descent (em-descent) in pixels for the dominant run.
    pub descent: f32,
    /// Pen advance width of the text, in pixels. This is what measureText
    /// should report as `width` and what alignment should use.
    pub advance_width: f32,
}

/// Text renderer with glyph caching for efficient rendering
pub struct TextRenderer {
    /// Font system for managing fonts
    font_manager: FontManager,
    /// Swash cache for glyph rasterization
    swash_cache: SwashCache,
    /// LRU cache of rendered text textures
    texture_cache: lru::LruCache<TextCacheKey, CachedTextTexture>,
}

/// Cache key for rendered text
#[derive(Hash, Eq, PartialEq, Clone)]
struct TextCacheKey {
    text: String,
    font_descriptor: FontDescriptor,
    color: [u8; 4],
}

/// Cached texture data for rendered text
struct CachedTextTexture {
    bitmap: Vec<u8>,
    width: u32,
    height: u32,
    baseline_y: f32,
    x_offset: f32,
    ascent: f32,
    descent: f32,
    advance_width: f32,
}

impl TextRenderer {
    /// Create a new text renderer with system fonts loaded
    pub fn new() -> Self {
        Self {
            font_manager: FontManager::new(),
            swash_cache: SwashCache::new(),
            texture_cache: lru::LruCache::new(std::num::NonZeroUsize::new(256).unwrap()),
        }
    }

    /// Expose the font manager for measurement paths that don't need to
    /// rasterize glyphs.
    pub fn font_manager_mut(&mut self) -> &mut FontManager {
        &mut self.font_manager
    }

    /// Render text to an RGBA bitmap.
    pub fn render_text_to_bitmap(
        &mut self,
        text: &str,
        font_descriptor: &FontDescriptor,
        color: [u8; 4],
    ) -> Result<RenderedText, String> {
        let cache_key = TextCacheKey {
            text: text.to_string(),
            font_descriptor: font_descriptor.clone(),
            color,
        };

        if let Some(cached) = self.texture_cache.get(&cache_key) {
            return Ok(RenderedText {
                bitmap: cached.bitmap.clone(),
                width: cached.width,
                height: cached.height,
                baseline_y: cached.baseline_y,
                x_offset: cached.x_offset,
                ascent: cached.ascent,
                descent: cached.descent,
                advance_width: cached.advance_width,
            });
        }

        let mut buffer = Buffer::new(
            &mut self.font_manager.font_system,
            cosmic_text::Metrics::new(font_descriptor.size, font_descriptor.size * 1.2),
        );

        let attrs = cosmic_text::Attrs::new()
            .family(cosmic_text::Family::Name(&font_descriptor.family))
            .weight(cosmic_text::Weight(font_descriptor.weight))
            .style(font_descriptor.style);

        buffer.set_text(
            &mut self.font_manager.font_system,
            text,
            &attrs,
            cosmic_text::Shaping::Advanced,
            None,
        );

        buffer.shape_until_scroll(&mut self.font_manager.font_system, false);

        // Pull em-ascent and em-descent from cosmic-text's laid-out run(s).
        // `line_y` is the baseline Y in the buffer's coordinate space;
        // `line_top` is the top of the line box. The difference is the ascent.
        let mut ascent: f32 = 0.0;
        let mut descent: f32 = 0.0;
        let mut advance_width: f32 = 0.0;
        let mut baseline_buffer_y: Option<f32> = None;
        for run in buffer.layout_runs() {
            if baseline_buffer_y.is_none() {
                baseline_buffer_y = Some(run.line_y);
            }
            ascent = ascent.max(run.line_y - run.line_top);
            descent = descent.max(run.line_top + run.line_height - run.line_y);
            advance_width = advance_width.max(run.line_w);
        }
        let Some(baseline_buffer_y) = baseline_buffer_y else {
            return Ok(RenderedText {
                bitmap: Vec::new(),
                width: 0,
                height: 0,
                baseline_y: 0.0,
                x_offset: 0.0,
                ascent: 0.0,
                descent: 0.0,
                advance_width: 0.0,
            });
        };

        let mut glyph_draws: Vec<(i32, i32, cosmic_text::CacheKey, Placement)> = Vec::new();
        let mut min_x: f32 = 0.0;
        let mut max_x: f32 = advance_width;
        let mut min_y: f32 = baseline_buffer_y - ascent;
        let mut max_y: f32 = baseline_buffer_y + descent;

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical = glyph.physical((0.0, 0.0), 1.0);
                let image_opt = self
                    .swash_cache
                    .get_image(&mut self.font_manager.font_system, physical.cache_key);

                let Some(image) = image_opt else { continue };
                let placement = image.placement;

                let pen_x = physical.x;
                let pen_y = run.line_y.round() as i32 + physical.y;

                let ink_left = (pen_x + placement.left) as f32;
                let ink_top = (pen_y - placement.top) as f32;
                let ink_right = ink_left + placement.width as f32;
                let ink_bottom = ink_top + placement.height as f32;

                min_x = min_x.min(ink_left);
                max_x = max_x.max(ink_right);
                min_y = min_y.min(ink_top);
                max_y = max_y.max(ink_bottom);

                glyph_draws.push((pen_x, pen_y, physical.cache_key, placement));
            }
        }

        let width = (max_x - min_x).ceil().max(1.0) as u32;
        let height = (max_y - min_y).ceil().max(1.0) as u32;
        let x_offset = -min_x;
        let baseline_y = baseline_buffer_y - min_y;

        if glyph_draws.is_empty() {
            return Ok(RenderedText {
                bitmap: Vec::new(),
                width: 0,
                height: 0,
                baseline_y,
                x_offset,
                ascent,
                descent,
                advance_width,
            });
        }

        let mut bitmap = vec![0u8; (width * height * 4) as usize];
        let min_x_i = min_x.floor() as i32;
        let min_y_i = min_y.floor() as i32;

        for (pen_x, pen_y, cache_key, _placement) in &glyph_draws {
            let image_opt = self
                .swash_cache
                .get_image(&mut self.font_manager.font_system, *cache_key);
            let Some(image) = image_opt else { continue };
            let draw_x = pen_x - min_x_i;
            let draw_y = pen_y - min_y_i;
            Self::blit_glyph(&mut bitmap, width, height, image, draw_x, draw_y, color);
        }

        self.texture_cache.put(
            cache_key,
            CachedTextTexture {
                bitmap: bitmap.clone(),
                width,
                height,
                baseline_y,
                x_offset,
                ascent,
                descent,
                advance_width,
            },
        );

        Ok(RenderedText {
            bitmap,
            width,
            height,
            baseline_y,
            x_offset,
            ascent,
            descent,
            advance_width,
        })
    }

    /// Blit a glyph image onto the target bitmap at the given pen (baseline
    /// origin) position, with alpha blending.
    fn blit_glyph(
        target: &mut [u8],
        target_width: u32,
        target_height: u32,
        image: &cosmic_text::SwashImage,
        pen_x: i32,
        pen_y: i32,
        color: [u8; 4],
    ) {
        let placement = &image.placement;

        for gy in 0..placement.height as i32 {
            for gx in 0..placement.width as i32 {
                let tx = pen_x + gx + placement.left;
                let ty = pen_y + gy - placement.top;

                if tx < 0 || ty < 0 || tx >= target_width as i32 || ty >= target_height as i32 {
                    continue;
                }

                let target_idx = ((ty * target_width as i32 + tx) * 4) as usize;
                if target_idx + 3 >= target.len() {
                    continue;
                }

                let glyph_idx = (gy * placement.width as i32 + gx) as usize;
                if glyph_idx >= image.data.len() {
                    continue;
                }

                let alpha = image.data[glyph_idx];
                if alpha == 0 {
                    continue;
                }

                let final_alpha = ((alpha as u16 * color[3] as u16) / 255) as u8;

                target[target_idx] = color[0];
                target[target_idx + 1] = color[1];
                target[target_idx + 2] = color[2];
                target[target_idx + 3] = final_alpha;
            }
        }
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}
