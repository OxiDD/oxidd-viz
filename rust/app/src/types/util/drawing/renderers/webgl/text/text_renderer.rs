use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
    num::NonZeroUsize,
    rc::Rc,
    slice,
};

use i_float::f64_point::F64Point;
use i_overlay::core::{
    fill_rule::FillRule, float_overlay::FloatOverlay, overlay::ShapeType, overlay_rule::OverlayRule,
};
use itertools::Itertools;
use lru::LruCache;
use swash::{
    proxy::CharmapProxy,
    scale::{outline::Outline, Render, ScaleContext, Scaler, Source},
    shape::{ShapeContext, Shaper},
    zeno::{Command, PathData, Vector},
    Charmap, FontRef, GlyphId,
};
use web_sys::WebGl2RenderingContext;

use crate::{
    types::util::drawing::renderers::{
        util::Font::Font,
        webgl::{
            text::triangulate::triangulate,
            util::{
                render_texture::{RenderTarget, RenderTexture},
                vertex_renderer::VertexRenderer,
            },
        },
    },
    util::{
        color::Color, logging::console, matrix4::Matrix4, point::Point, rectangle::Rectangle,
        transition::Transition,
    },
};

pub struct TextRenderer {
    vertex_renderer: VertexRenderer,
    char_renderer: VertexRenderer,
    atlases: LruCache<i32, Atlas>,
    // char_atlases: Vec<RenderTexture>,
    // char_atlas_poses: HashMap<GlyphId, (Rectangle, Point, usize)>,
    settings: TextRendererSettings,
    cur_scale_index: i32,
    cur_text: Vec<Text>,
    screen_height: f32,

    // Font helpers
    font: Rc<Font>,
    _char_scaler_context: Box<ScaleContext>,
    char_scaler: Scaler<'static>,
}

struct Atlas {
    pub textures: Vec<RenderTexture>,
    pub positions: HashMap<GlyphId, (Rectangle, Point, usize)>,
}

#[derive(Clone)]
pub struct Text {
    pub text: String,
    pub position: Transition<Point>,
    pub exists: Transition<f32>, // A number between 0 and 1 of whether this node is visible (0-1)
}

/**
 * TODO:
 * - Distribute characters over multiple textures when scaling becomes too large
 */
impl TextRenderer {
    pub fn new(
        context: &WebGl2RenderingContext,
        font: Rc<Font>,
        settings: TextRendererSettings,
        screen_height: usize,
    ) -> TextRenderer {
        let vertex_renderer = VertexRenderer::new(
            context,
            include_str!("text_renderer.vert"),
            include_str!("text_renderer.frag"),
        )
        .unwrap();
        let char_renderer = VertexRenderer::new(
            context,
            include_str!("char_renderer.vert"),
            include_str!("char_renderer.frag"),
        )
        .unwrap();

        // Every time unsafe is used, we make sure we own the data in this struct first, and don't leak any data outside

        // let charmap = CharmapProxy::from_font(&font).materialize(&font);

        let (scaler_context, scaler) =
            create_scaler(screen_height, (*font).as_ref().clone(), &settings);
        TextRenderer {
            vertex_renderer,
            char_renderer,
            atlases: LruCache::new(NonZeroUsize::new(settings.scale_cache_size as usize).unwrap()),
            settings,
            cur_scale_index: -20,
            cur_text: Vec::new(),
            screen_height: screen_height as f32,

            _char_scaler_context: scaler_context,
            char_scaler: scaler,
            font,
        }
    }

    // Gets font size
    pub fn get_text_size(&self) -> f32 {
        self.font.text_size()
    }

    fn get_cur_atlas(&mut self) -> &mut Atlas {
        self.atlases.get_mut(&self.cur_scale_index).unwrap()
    }
    fn get_char_scale(&self) -> f32 {
        self.get_scale_from_index(self.cur_scale_index)
    }
    fn get_draw_scale(&self) -> f32 {
        self.font.text_size()
    }
    fn get_atlas_resolution(&self) -> f32 {
        self.screen_height * self.settings.resolution * self.settings.scale_factor_group_size
    }
    fn get_scale_index(&self, scale: f32) -> i32 {
        (scale.log2() / self.settings.scale_factor_group_size.log2()).floor() as i32
    }
    fn get_scale_from_index(&self, index: i32) -> f32 {
        self.settings.scale_factor_group_size.powi(index)
    }

    fn update_chars(&mut self, context: &WebGl2RenderingContext, required_chars: Vec<GlyphId>) {
        let atlas = self
            .atlases
            .get_or_insert_mut(self.cur_scale_index, || Atlas {
                textures: Vec::new(),
                positions: HashMap::new(),
            });

        let count = required_chars.len() * 2;
        let chars = required_chars
            .iter()
            .cloned()
            .chain(atlas.positions.keys().cloned())
            .take(count)
            .collect::<Vec<GlyphId>>();

        let char_data = chars
            .iter()
            .filter_map(|&glyph_id| {
                let outline = self.char_scaler.scale_outline(glyph_id)?;
                let bounds = outline.bounds();
                let width = bounds.width() * self.get_char_scale();
                let height = bounds.height() * self.get_char_scale();
                Some((glyph_id, ((width, height), outline)))
            })
            .collect();

        self.position_chars(context, &char_data);
        self.draw_chars(context, &char_data);
    }

    fn position_chars(&mut self, context: &WebGl2RenderingContext, char_data: &CharDataMap) {
        let area = char_data
            .values()
            .fold(0., |sum, ((width, height), _)| sum + width * height);
        let target_width = area.sqrt();
        let max_size = self.settings.max_atlas_size as f32;

        {
            let atlas = self.get_cur_atlas();
            atlas.positions.clear();
            for atlas in &atlas.textures {
                atlas.dispose(context);
            }
            atlas.textures.clear();
        }

        let scale = self.get_char_scale();
        let spacing = self.settings.atlas_spacing;

        let mut texture_id = 0;
        let mut width = 0.;
        let mut row_height = 0.;
        let mut x = 0.;
        let mut y = 0.;
        let finish_texture = |atlas: &mut Atlas, width: f32, height: f32| {
            let width = f32::ceil(width) as usize;
            let height = f32::ceil(height) as usize;
            console::log!("atlas: ({}, {})", width, height);

            atlas
                .textures
                .push(RenderTexture::new(context, width, height, (0.0, 0.0, 0.0, 0.0)).unwrap());
        };

        for char in char_data.keys() {
            let Some(((char_width, char_height), outline)) = char_data.get(char) else {
                continue;
            };
            // Make sure that the character fits here, otherwise go to the next valid position
            let end_x = x + *char_width;
            if end_x > max_size {
                x = 0.;
                y += row_height + spacing;
                row_height = 0.;
            }
            if *char_height > row_height && y + char_height > max_size {
                finish_texture(self.get_cur_atlas(), width, y + row_height);
                x = 0.;
                y = 0.;
                row_height = 0.;
                texture_id += 1;
            }

            // Reserve the space for the character, and update the line height
            let min = outline.bounds().min * scale;
            self.get_cur_atlas().positions.insert(
                *char,
                (
                    Rectangle::new(x, y, *char_width, *char_height),
                    Point { x: min.x, y: min.y },
                    texture_id,
                ),
            );

            if *char_height > row_height {
                row_height = *char_height;
            }

            // Go to the next position based on the target-width
            x += *char_width;
            if x > width {
                width = x;
            }
            x += spacing;
            if x > target_width {
                x = 0.;
                y += row_height + spacing;
                row_height = 0.;
            }
        }

        finish_texture(self.get_cur_atlas(), width, y + row_height);
    }

    fn get_atlas_coord(&mut self, point: Point, index: usize) -> Point {
        let atlas = self.get_cur_atlas();
        let texture_size = atlas.textures.get(index).unwrap().get_size();
        Point {
            x: point.x / (texture_size.0 as f32),
            y: point.y / (texture_size.1 as f32),
        }
    }

    fn draw_chars(&mut self, context: &WebGl2RenderingContext, char_data: &CharDataMap) {
        let glyphs = {
            let atlas = self.get_cur_atlas();
            char_data
                .iter()
                .filter_map(|(glyph_id, (_, outline))| {
                    atlas
                        .positions
                        .get(glyph_id)
                        .map(|pos| (outline.clone(), pos.clone()))
                })
                .collect_vec()
        };
        let grouped_glyphs = glyphs
            .iter()
            .sorted_by_key(|(_, (_, _, index))| *index)
            .group_by(|(_, (_, _, index))| *index);

        for (index, group) in grouped_glyphs.into_iter() {
            let texture_size = self.get_cur_atlas().textures.get(index).unwrap().get_size();
            let texture_size = Point {
                x: texture_size.0 as f32,
                y: texture_size.1 as f32,
            };

            let scale = self.get_char_scale();
            let distance_per_sample = {
                let display_scale =
                    self.get_draw_scale() * scale.min(self.settings.max_sample_scale);
                // display_scale == 1 when the character size is self.resolution, and gets smaller as the character gets smaller. Hence we can use more pixels per character then, before scaling of the character, however we should not do this linearly or we lose too much detail on small scale, hence we take the sqrt to preserve more detail even at small scale.
                self.settings.sample_distance / display_scale.sqrt()
            };

            self.char_renderer.set_data(
                context,
                "position",
                &group
                    .flat_map(|(outline, (pos, min, _index))| {
                        let offset = pos.pos() - *min;

                        let triangles = triangulate(outline.path().commands(), distance_per_sample);
                        triangles
                            .iter()
                            .flat_map(|point| {
                                vec![
                                    (point.x * scale + offset.x) / texture_size.x,
                                    (point.y * scale + offset.y) / texture_size.y,
                                ]
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<f32>>(),
                2,
            );

            self.char_renderer.send_data(context);

            let texture = self.get_cur_atlas().textures.get(index).unwrap();
            texture.bind_buffer(context);
            texture.clear(context);
            self.char_renderer
                .render(context, WebGl2RenderingContext::TRIANGLES);
        }
    }

    pub fn set_texts(&mut self, context: &WebGl2RenderingContext, texts: &Vec<Text>) {
        self.cur_text = texts.clone();

        if self.screen_height == 0. {
            return;
        }
        for text in texts {
            console::log!(
                "Text: {}, {}, {}",
                text.text,
                text.position.new.x,
                text.position.new.y
            );
        }

        // Obtain the character glyphs and position data, and ensure that these glyphs are on the atlas
        let char_data = texts
            .iter()
            .flat_map(|text| {
                let mut shaper_context = Box::new(ShapeContext::new());
                let mut shaper = shaper_context
                    .builder((*self.font).as_ref().clone())
                    .size(self.font.text_size())
                    .build();

                shaper.add_str(&text.text);
                let mut chars = Vec::new();
                let mut x = 0.;
                shaper.shape_with(|cluster| {
                    for glyph in cluster.glyphs {
                        chars.push((
                            glyph.id,
                            &text.position
                                + &Transition::plain(Point {
                                    x: glyph.x + x,
                                    y: glyph.y,
                                }),
                            text.exists,
                        ));
                        x += glyph.advance;
                    }
                });
                chars
            })
            .collect::<Vec<_>>();
        let mut glyphs = char_data
            .iter()
            .map(|&(glyph_id, _, _)| glyph_id)
            .collect::<HashSet<GlyphId>>();

        let charmap = (*self.font).as_ref().charmap();
        for char in self.settings.default_chars.chars() {
            glyphs.insert(charmap.map(char));
        }

        if let Some(atlas) = self.atlases.get(&self.cur_scale_index) {
            let has_new_glyphs = !glyphs.is_subset(&atlas.positions.keys().cloned().collect());
            if has_new_glyphs {
                self.update_chars(context, glyphs.iter().cloned().collect());
            }
        } else {
            self.update_chars(context, glyphs.iter().cloned().collect());
        }

        // Bind the character data to the shader
        let char_data = {
            let atlas = self.get_cur_atlas();
            char_data
                .iter()
                .filter_map(|(glyph_id, pos, exists)| {
                    atlas
                        .positions
                        .get(glyph_id)
                        .map(|glyph_pos| (glyph_pos.clone(), pos.clone(), exists))
                })
                .collect::<Vec<_>>()
        };

        let make_square = |p: Point, size: Point| {
            let p1 = p;
            let p2 = p + Point { x: 0., y: size.y };
            let p3 = p + size;
            let p4 = p + Point { x: size.x, y: 0. };
            [
                p1.x, p1.y, //
                p2.x, p2.y, //
                p4.x, p4.y, //
                /* */
                p3.x, p3.y, //
                p2.x, p2.y, //
                p4.x, p4.y,
            ]
        };

        let char_to_draw_scale =
            self.get_draw_scale() / self.get_char_scale() / self.get_atlas_resolution();

        let positions_old = char_data
            .iter()
            .flat_map(|((glyph_pos, offset, _), text_pos, _)| {
                make_square(
                    text_pos.old + *offset * char_to_draw_scale,
                    glyph_pos.size() * char_to_draw_scale,
                )
            })
            .collect::<Vec<_>>();

        let positions_new = char_data
            .iter()
            .flat_map(|((glyph_pos, offset, _), text_pos, _)| {
                make_square(
                    text_pos.new + *offset * char_to_draw_scale,
                    glyph_pos.size() * char_to_draw_scale,
                )
            })
            .collect::<Vec<_>>();

        let positions_start_time = char_data
            .iter()
            .flat_map(|(_, text_pos, _)| [text_pos.old_time as f32; 6])
            .collect::<Vec<_>>();
        let positions_duration = char_data
            .iter()
            .flat_map(|(_, text_pos, _)| [text_pos.duration as f32; 6])
            .collect::<Vec<_>>();

        let char_coords = char_data
            .iter()
            .flat_map(|((glyph_pos, _, index), _, _)| {
                make_square(
                    self.get_atlas_coord(glyph_pos.pos(), *index),
                    self.get_atlas_coord(glyph_pos.size(), *index),
                )
            })
            .collect::<Vec<_>>();

        let texture_indices = char_data
            .iter()
            .flat_map(|((_, _, index), _, _)| [*index as f32; 6])
            .collect::<Vec<_>>();

        let exists = char_data
            .iter()
            .flat_map(|(_, _, exists)| [exists.new; 6])
            .collect::<Vec<_>>();

        let exists_old = char_data
            .iter()
            .flat_map(|(_, _, exists)| [exists.old; 6])
            .collect::<Vec<_>>();

        let exists_start_time = char_data
            .iter()
            .flat_map(|(_, _, exists)| [exists.old_time as f32; 6])
            .collect::<Vec<_>>();

        let exists_duration = char_data
            .iter()
            .flat_map(|(_, _, exists)| [exists.duration as f32; 6])
            .collect::<Vec<_>>();

        self.vertex_renderer
            .set_data(context, "positionOld", &positions_old, 2);
        self.vertex_renderer
            .set_data(context, "position", &positions_new, 2);
        self.vertex_renderer
            .set_data(context, "positionStartTime", &positions_start_time, 1);
        self.vertex_renderer
            .set_data(context, "positionDuration", &positions_duration, 1);
        self.vertex_renderer
            .set_data(context, "charCoord", &char_coords, 2);
        self.vertex_renderer
            .set_data(context, "textureIndex", &texture_indices, 1);

        self.vertex_renderer
            .set_data(context, "existsOld", &exists_old, 1);
        self.vertex_renderer.set_data(context, "exists", &exists, 1);
        self.vertex_renderer
            .set_data(context, "existsStartTime", &exists_start_time, 1);
        self.vertex_renderer
            .set_data(context, "existsDuration", &exists_duration, 1);

        self.vertex_renderer.send_data(context);
    }

    pub fn set_transform_and_screen_height(
        &mut self,
        context: &WebGl2RenderingContext,
        transform: &Matrix4,
        screen_height: usize,
    ) {
        let height = screen_height as f32;
        let height_change = self.screen_height != height;
        if height_change {
            self.atlases.clear();
            let (scaler_context, scaler) =
                create_scaler(screen_height, (*self.font).as_ref().clone(), &self.settings);
            self.char_scaler = scaler;
            self._char_scaler_context = scaler_context;
            self.screen_height = height;
        }
        if self.screen_height == 0. {
            return;
        }

        self.vertex_renderer.set_uniform(context, "transform", |u| {
            context.uniform_matrix4fv_with_f32_array(u, true, &transform.0)
        });

        let p1 = transform.mul_vec3((0., 0., 0.));
        let p2 = transform.mul_vec3((0., 1., 0.));

        let exact_scale = Point { x: p1.0, y: p1.1 }
            .distance(&Point { x: p2.0, y: p2.1 })
            .min(self.settings.max_scale);
        let scale_index = self.get_scale_index(exact_scale);
        let cur_index = self.cur_scale_index;

        if cur_index != scale_index || height_change {
            self.cur_scale_index = scale_index;

            let charmap = (*self.font).as_ref().charmap();
            let chars = self
                .cur_text
                .iter()
                .flat_map(|s| s.text.chars())
                .collect::<HashSet<_>>()
                .iter()
                .map(|char| charmap.map(*char))
                .collect::<HashSet<_>>();

            let skip_update = self
                .atlases
                .get(&scale_index)
                .map(|atlas| {
                    atlas
                        .positions
                        .keys()
                        .cloned()
                        .collect::<HashSet<_>>()
                        .is_superset(&chars)
                })
                .unwrap_or(false);
            if !skip_update {
                self.update_chars(context, chars.into_iter().collect());
            }

            self.set_texts(context, &self.cur_text.clone());
        }
    }

    pub fn render(&mut self, context: &WebGl2RenderingContext, time: u32) {
        self.vertex_renderer
            .set_uniform(context, "time", |u| context.uniform1f(u, time as f32));

        if let Some(char_atlas) = self.atlases.get(&self.cur_scale_index) {
            let (r, g, b) = self.settings.rgb_color;
            self.vertex_renderer
                .set_uniform(context, "color", |u| context.uniform3f(u, r, g, b));
            for (index, atlas) in char_atlas.textures.iter().enumerate() {
                self.vertex_renderer
                    .set_uniform(context, "boundTextureIndex", |u| {
                        context.uniform1i(u, index as i32)
                    });
                self.vertex_renderer
                    .set_uniform(context, "characters", |u| {
                        atlas.bind_texture(context, u, 0);
                    });

                self.vertex_renderer
                    .render(context, WebGl2RenderingContext::TRIANGLES);
            }
        }
    }

    pub fn dispose(&mut self, context: &WebGl2RenderingContext) {
        self.vertex_renderer.dispose(context);
        self.char_renderer.dispose(context);
        for (_, char_atlas) in &self.atlases {
            for atlas in &char_atlas.textures {
                atlas.dispose(context);
            }
        }
    }
}

type CharDataMap = HashMap<u16, ((f32, f32), Outline)>;

#[derive(Clone)]
pub struct TextRendererSettings {
    /// The relative resolution to use for the atlas (going for ~2 is good for some anti-aliasing)
    pub resolution: f32,
    /// The base of the range of scales that use the same rendering quality: scale in [scale_step^i..scale_step^{i+1})
    pub scale_factor_group_size: f32,
    /// The sample distance between points expressed in relation to the resolution (i.e. in terms of the distance between pixels if a character is rendered at full resolution)
    pub sample_distance: f32,
    /// The maximum rendering scale to use for the atlas, above which rendering quality won't be increased anymore
    pub max_scale: f32,
    /// The maximum scale after which the number of sample points won't increaase anymore
    pub max_sample_scale: f32,
    /// The spacing that characters have in the atlas
    pub atlas_spacing: f32,
    /// The default characters that should always be included on the atlas
    pub default_chars: String,
    /// The maximum width and height that the atlas may have (to adhere to hardware limitations)
    pub max_atlas_size: u32,
    /// The size of the cache to keep for different scales
    pub scale_cache_size: u8,
    /// The color of the text
    pub rgb_color: (f32, f32, f32),
}

impl TextRendererSettings {
    pub fn new() -> TextRendererSettings {
        TextRendererSettings {
            resolution: 2.,
            scale_factor_group_size: 2.0,
            sample_distance: 25.,
            max_scale: 1.0,
            max_sample_scale: 1.0,
            atlas_spacing: 2.0,
            default_chars: "abcdefghijklmnopqrstuvwxyz_- ".to_string(),
            max_atlas_size: 4096,
            scale_cache_size: 6,
            rgb_color: (0., 0., 0.),
        }
    }
    pub fn resolution(mut self, resolution: f32) -> TextRendererSettings {
        self.resolution = resolution;
        self
    }
    pub fn scale_factor_group_size(mut self, scale_factor_group_size: f32) -> TextRendererSettings {
        self.scale_factor_group_size = scale_factor_group_size;
        self
    }
    pub fn sample_distance(mut self, sample_distance: f32) -> TextRendererSettings {
        self.sample_distance = sample_distance;
        self
    }
    pub fn max_sample_scale(mut self, max_sample_scale: f32) -> TextRendererSettings {
        self.max_sample_scale = max_sample_scale;
        self
    }
    pub fn max_scale(mut self, max_scale: f32) -> TextRendererSettings {
        self.max_scale = max_scale;
        self
    }
    pub fn atlas_spacing(mut self, atlas_spacing: f32) -> TextRendererSettings {
        self.atlas_spacing = atlas_spacing;
        self
    }
    pub fn default_chars(mut self, default_chars: String) -> TextRendererSettings {
        self.default_chars = default_chars;
        self
    }
    pub fn max_atlas_size(mut self, max_atlas_size: u32) -> TextRendererSettings {
        self.max_atlas_size = max_atlas_size;
        self
    }

    pub fn scale_cache_size(mut self, scale_cache_size: u8) -> TextRendererSettings {
        self.scale_cache_size = scale_cache_size;
        self
    }
    pub fn color(mut self, c: Color) -> TextRendererSettings {
        self.rgb_color = (c.0, c.1, c.2);
        self
    }
}

fn create_scaler(
    screen_height: usize,
    font: FontRef<'static>,
    settings: &TextRendererSettings,
) -> (Box<ScaleContext>, Scaler<'static>) {
    let mut scaler_context = Box::new(ScaleContext::new());
    let scaler_context_ref = unsafe {
        std::mem::transmute::<&mut ScaleContext, &'static mut ScaleContext>(scaler_context.as_mut())
    };
    let scaler = scaler_context_ref
        .builder(font)
        .size(screen_height as f32 * settings.resolution * settings.scale_factor_group_size)
        .build();
    (scaler_context, scaler)
}
