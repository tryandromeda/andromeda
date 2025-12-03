// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use cosmic_text::{Buffer, SwashCache};

use crate::ext::canvas::font_system::{FontDescriptor, FontManager};

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

    /// Render text to an RGBA bitmap
    pub fn render_text_to_bitmap(
        &mut self,
        text: &str,
        font_descriptor: &FontDescriptor,
        color: [u8; 4],
    ) -> Result<(Vec<u8>, u32, u32), String> {
        let cache_key = TextCacheKey {
            text: text.to_string(),
            font_descriptor: font_descriptor.clone(),
            color,
        };

        if let Some(cached) = self.texture_cache.get(&cache_key) {
            return Ok((cached.bitmap.clone(), cached.width, cached.height));
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

        let (width, height) = self.calculate_text_bounds(&buffer)?;

        if width == 0 || height == 0 {
            return Ok((Vec::new(), 0, 0));
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                min_x = min_x.min(glyph.x);
                min_y = min_y.min(glyph.y - glyph.font_size);
            }
        }

        let mut bitmap = vec![0u8; (width * height * 4) as usize];

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical = glyph.physical((0.0, 0.0), 1.0);

                let image_opt = self
                    .swash_cache
                    .get_image(&mut self.font_manager.font_system, physical.cache_key);

                if let Some(image) = image_opt {
                    Self::blit_glyph(
                        &mut bitmap,
                        width,
                        height,
                        image,
                        physical.x - min_x as i32,
                        physical.y - min_y as i32,
                        color,
                    );
                }
            }
        }

        self.texture_cache.put(
            cache_key,
            CachedTextTexture {
                bitmap: bitmap.clone(),
                width,
                height,
            },
        );

        Ok((bitmap, width, height))
    }

    /// Calculate the bounding box dimensions for shaped text
    fn calculate_text_bounds(&self, buffer: &Buffer) -> Result<(u32, u32), String> {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                min_x = min_x.min(glyph.x);
                max_x = max_x.max(glyph.x + glyph.w);

                min_y = min_y.min(glyph.y - glyph.font_size);
                max_y = max_y.max(glyph.y);
            }
        }

        if min_x >= max_x || min_y >= max_y {
            return Ok((0, 0));
        }

        let width = (max_x - min_x).ceil() as u32;
        let height = (max_y - min_y).ceil() as u32;

        Ok((width, height))
    }

    /// Blit a glyph image onto the target bitmap with alpha blending
    fn blit_glyph(
        target: &mut [u8],
        target_width: u32,
        target_height: u32,
        image: &cosmic_text::SwashImage,
        x: i32,
        y: i32,
        color: [u8; 4],
    ) {
        let placement = &image.placement;

        for gy in 0..placement.height as i32 {
            for gx in 0..placement.width as i32 {
                let tx = x + gx + placement.left;
                let ty = target_height as i32 - (y + gy - placement.top) - 1;

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

                let alpha_f = final_alpha as f32 / 255.0;
                target[target_idx] = (color[0] as f32 * alpha_f) as u8;
                target[target_idx + 1] = (color[1] as f32 * alpha_f) as u8;
                target[target_idx + 2] = (color[2] as f32 * alpha_f) as u8;
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
