use std::{collections::HashMap, rc::Rc};

use oxidd_core::Tag;
use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader,
    WebGlVertexArrayObject,
};

use crate::{
    types::util::{
        drawing::{
            diagram_layout::{DiagramLayout, LayerStyle, NodeStyle},
            layout_rules::LayoutRules,
            renderer::{GroupSelection, Renderer},
        },
        graph_structure::graph_structure::{DrawTag, EdgeType},
    },
    util::{
        color::{Color, TransparentColor},
        logging::console,
        point::Point,
        transformation::Transformation,
        transition::{Interpolatable, Transition},
    },
    wasm_interface::NodeGroupID,
};

use super::{
    util::Font::Font,
    webgl::{
        edge_renderer::{Edge, EdgeRenderer, EdgeRenderingType},
        layers::{
            layer_bg_renderer::LayerBgRenderer,
            layer_lines_renderer::LayerLinesRenderer,
            layer_renderer::{Layer, LayerRenderer},
        },
        node_renderer::{Node, NodeRenderer, NodeRenderingColorConfig, TextRenderingConfig},
        text::text_renderer::{Text, TextRenderer, TextRendererSettings},
        util::render_texture::{RenderTarget, ScreenTexture},
    },
};

/// A simple renderer that uses webgl to draw nodes and edges
pub struct WebglRenderer<T: DrawTag> {
    webgl_context: WebGl2RenderingContext,
    node_renderer: NodeRenderer,
    edge_renderer: EdgeRenderer,
    layer_renderer: LayerRenderer,
    edge_type_ids: HashMap<EdgeType<T>, usize>,
    screen_texture: ScreenTexture,
}

impl<T: DrawTag> WebglRenderer<T> {
    pub fn new(
        context: WebGl2RenderingContext,
        screen_texture: ScreenTexture,
        edge_types: HashMap<EdgeType<T>, EdgeRenderingType>,
        node_colors: NodeRenderingColorConfig,
        layer_colors: LayerRenderingColorConfig,
        font: Rc<Font>,
        // TODO: add text configuration?
    ) -> Result<WebglRenderer<T>, JsValue> {
        let (edge_type_ids, edge_rendering_types): (
            HashMap<EdgeType<T>, usize>,
            Vec<EdgeRenderingType>,
        ) = edge_types
            .iter()
            .enumerate()
            .map(|(index, (edge_type, edge_rendering))| {
                ((edge_type.clone(), index), edge_rendering.clone())
            })
            .unzip();

        let screen_height = screen_texture.get_size().1;
        let font_settings = TextRendererSettings::new()
            .resolution(2.0)
            .sample_distance(35.)
            .scale_factor_group_size(3.0)
            .scale_cache_size(10)
            .max_scale(1.5)
            .color(node_colors.text);

        // context.enable(WebGl2RenderingContext::DEPTH_TEST);
        context.enable(WebGl2RenderingContext::BLEND);
        context.blend_func(
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        Ok(WebglRenderer {
            node_renderer: NodeRenderer::new(
                &context,
                node_colors,
                TextRenderingConfig {
                    screen_height,
                    font: font.clone(),
                    font_settings: font_settings.clone(),
                },
            ),
            edge_renderer: EdgeRenderer::new(&context, edge_rendering_types),
            layer_renderer: LayerRenderer::new(
                &context,
                LayerBgRenderer::new(&context, layer_colors.background1, layer_colors.background2),
                // LayerLinesRenderer::new(&context),
                screen_height,
                font,
                font_settings.color(layer_colors.text),
            ),
            webgl_context: context,
            screen_texture,
            edge_type_ids,
        })
    }
    pub fn from_canvas(
        canvas: HtmlCanvasElement,
        edge_types: HashMap<EdgeType<T>, EdgeRenderingType>,
        node_colors: NodeRenderingColorConfig,
        layer_colors: LayerRenderingColorConfig,
        font: Rc<Font>,
    ) -> Result<WebglRenderer<T>, JsValue> {
        let context = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .unwrap();
        let c = layer_colors.background1;
        WebglRenderer::new(
            context,
            ScreenTexture::new(
                canvas.width() as usize,
                canvas.height() as usize,
                (c.0, c.1, c.2, c.3),
            ),
            edge_types,
            node_colors,
            layer_colors,
            font,
        )
    }
}

impl<L: LayoutRules> Renderer<L> for WebglRenderer<L::T>
where
    L::NS: WebglNodeStyle,
    L::LS: WebglLayerStyle,
{
    fn set_transform(&mut self, transform: Transformation) {
        let height = transform.height as usize;
        // if self.screen_texture.get_size().1 != height {
        //     self.layer_renderer
        //         .set_screen_height(&self.webgl_context, height);
        // }
        console::log!(
            "{} {} {} {} {}",
            transform.position.x,
            transform.position.y,
            transform.width,
            transform.height,
            transform.scale
        );

        self.screen_texture
            .set_size(transform.width as usize, height);
        let matrix = transform.get_matrix();
        self.node_renderer
            .set_transform_and_screen_height(&self.webgl_context, &matrix, height);
        self.edge_renderer
            .set_transform(&self.webgl_context, &matrix);
        self.layer_renderer
            .set_transform_and_screen_height(&self.webgl_context, &matrix, height);
    }
    fn update_layout(&mut self, layout: &DiagramLayout<L::T, L::NS, L::LS>) {
        self.node_renderer.set_nodes(
            &self.webgl_context,
            &layout
                .groups
                .iter()
                .map(|(id, group)| {
                    let style = &group.style;
                    // console::log!("pos: {}, {}", group.position, group.size * 0.5);
                    Node {
                        ID: *id,
                        center_position: &group.position
                            + &Transition {
                                new: Point {
                                    y: 0.5 * group.size.new.y,
                                    x: 0.,
                                },
                                old: Point {
                                    y: 0.5 * group.size.old.y,
                                    x: 0.,
                                },
                                ..group.size
                            },
                        size: group.size,
                        label: style.new.get_label().clone(),
                        exists: group.exists,
                        color: Transition {
                            old_time: style.old_time,
                            duration: style.duration,
                            old: style.old.get_color(),
                            new: style.new.get_color(),
                        },
                        outline_color: Transition {
                            old_time: style.old_time,
                            duration: style.duration,
                            old: style.old.get_outline_color(),
                            new: style.new.get_outline_color(),
                        },
                    }
                })
                .collect(),
        );
        let edge_type_ids = self.edge_type_ids.clone();
        self.edge_renderer.set_edges(
            &self.webgl_context,
            &layout
                .groups
                .iter()
                .flat_map(|(&id, group)| {
                    let start = group.position;
                    let edge_type_ids = &edge_type_ids;
                    group.edges.iter().filter_map(move |(edge_data, edge)| {
                        Some(Edge {
                            start: &start + &edge.start_offset,
                            start_node: id,
                            points: edge.points.iter().map(|point| point.point).collect(),
                            end: &layout.groups.get(&edge_data.to)?.position + &edge.end_offset,
                            end_node: edge_data.to,
                            edge_type: *edge_type_ids.get(&edge_data.edge_type)?,
                            shift: edge.curve_offset,
                            exists: edge.exists,
                        })
                    })
                })
                .collect(),
        );
        self.layer_renderer.set_layers(
            &self.webgl_context,
            &layout
                .layers
                .iter()
                .map(|layer| Layer {
                    top: layer.top,
                    bottom: layer.bottom,
                    label: layer.style.new.get_label(),
                    index: layer.index,
                    exists: layer.exists,
                })
                .collect(),
        );
    }

    fn select_groups(&mut self, selection: GroupSelection, old_selection: GroupSelection) {
        self.node_renderer
            .update_selection(&self.webgl_context, &selection, &old_selection);
        self.edge_renderer
            .update_selection(&self.webgl_context, &selection, &old_selection);
    }
    fn render(&mut self, time: u32) {
        self.screen_texture.clear(&self.webgl_context);
        self.layer_renderer.render(&self.webgl_context, time);
        self.edge_renderer.render(&self.webgl_context, time);
        self.node_renderer.render(&self.webgl_context, time);
    }
}

pub struct LayerRenderingColorConfig {
    pub background1: TransparentColor,
    pub background2: TransparentColor,
    pub text: Color,
}

impl<T: DrawTag> Drop for WebglRenderer<T> {
    fn drop(&mut self) {
        self.node_renderer.dispose(&self.webgl_context);
        self.edge_renderer.dispose(&self.webgl_context);
        self.layer_renderer.dispose(&self.webgl_context);
    }
}

pub trait WebglNodeStyle: NodeStyle {
    fn get_color(&self) -> Color;
    fn get_outline_color(&self) -> TransparentColor;
    fn get_label(&self) -> Option<String>;
}
pub trait WebglLayerStyle: LayerStyle {
    fn get_label(&self) -> String;
}
