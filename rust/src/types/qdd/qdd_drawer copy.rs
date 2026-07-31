use itertools::Itertools;
use oxidd_core::DiagramRules;
use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::hash::Hash;
use std::io::Cursor;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::u32;
use web_sys::console::log;

use crate::configuration::configuration::Configuration;
use crate::configuration::configuration_object::AbstractConfigurationObject;
use crate::configuration::configuration_object::Abstractable;
use crate::configuration::configuration_object::ConfigObjectGetter;
use crate::configuration::observe_configuration::after_configuration_change;
use crate::configuration::observe_configuration::on_configuration_change;
use crate::configuration::types::button_config::ButtonConfig;
use crate::configuration::types::choice_config::Choice;
use crate::configuration::types::choice_config::ChoiceConfig;
use crate::configuration::types::composite_config;
use crate::configuration::types::composite_config::CompositeConfig;
use crate::configuration::types::container_config::ContainerConfig;
use crate::configuration::types::container_config::ContainerStyle;
use crate::configuration::types::int_config::IntConfig;
use crate::configuration::types::label_config::LabelConfig;
use crate::configuration::types::label_config::LabelKind;
use crate::configuration::types::location_config::Location;
use crate::configuration::types::location_config::LocationConfig;
use crate::configuration::types::panel_config::OpenSide;
use crate::configuration::types::panel_config::PanelConfig;
use crate::configuration::types::text_config::TextConfig;
use crate::configuration::types::text_output_config::TextOutputConfig;
use crate::traits::Diagram;
use crate::traits::DiagramSection;
use crate::traits::DiagramSectionDrawer;
use crate::types::util::drawing::layouts::layer_orderings::edge_layer_ordering::EdgeLayerOrdering;
use crate::types::util::drawing::renderers::latex_renderer::latex_headers;
use crate::types::util::drawing::renderers::webgl_renderer::LayerRenderingColorConfig;
use crate::types::util::drawing::renderers::webgl_renderer::WebglLayerStyle;
use crate::types::util::graph_structure::graph_manipulators::child_edge_adjuster::ChildEdgeAdjuster;
use crate::types::util::graph_structure::graph_manipulators::edge_to_adjuster::EdgeToAdjuster;
use crate::types::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceGroups;
use crate::types::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceRemainder;
use crate::types::util::graph_structure::graph_manipulators::reachability_adjuster::ReachabilityAdjuster;
use crate::types::util::graph_structure::graph_manipulators::reachability_adjuster::ReachabilityLabel;
use crate::types::util::graph_structure::oxidd_graph_structure::NodeType;
use crate::util::color::Color;
use crate::util::color::TransparentColor;
use crate::util::dummy_bdd::DummyBDDEdge;
use crate::util::dummy_bdd::DummyBDDFunction;
use crate::util::dummy_bdd::DummyBDDManager;
use crate::util::dummy_bdd::DummyBDDManagerRef;
use crate::util::dummy_bdd::DummyBDDNode;
use crate::util::free_id_manager::FreeIdManager;
use crate::util::logging::console;
use crate::util::rc_refcell::MutRcRefCell;
use crate::util::rectangle::Rectangle;
use crate::util::transition::Interpolatable;
use crate::wasm_interface::NodeGroupID;
use crate::wasm_interface::NodeID;
use crate::wasm_interface::StepData;
use crate::wasm_interface::TargetID;
use crate::wasm_interface::TargetIDType;
use oxidd::bdd::BDDFunction;
use oxidd::util::Borrowed;
use oxidd::BooleanFunction;
use oxidd::Edge;
use oxidd::Function;
use oxidd::InnerNode;
use oxidd::{Manager, ManagerRef};
use oxidd_core::HasApplyCache;
use oxidd_core::HasLevel;
use oxidd_core::Node;
use oxidd_core::{util::DropWith, Tag};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use super::super::util::drawing::diagram_layout::LayerStyle;
use super::super::util::drawing::diagram_layout::NodeStyle;
use super::super::util::drawing::drawer::Drawer;
use super::super::util::drawing::layout_rules::LayoutRules;
use super::super::util::drawing::layouts::layer_group_sorting::average_group_alignment::AverageGroupAlignment;
use super::super::util::drawing::layouts::layer_group_sorting::ordering_group_alignment::OrderingGroupAlignment;
use super::super::util::drawing::layouts::layer_orderings::combinators::sequence_ordering::SequenceOrdering;
use super::super::util::drawing::layouts::layer_orderings::pseudo_random_layer_ordering::PseudoRandomLayerOrdering;
use super::super::util::drawing::layouts::layer_orderings::random_layer_ordering::RandomLayerOrdering;
use super::super::util::drawing::layouts::layer_orderings::sugiyama_ordering::SugiyamaOrdering;
use super::super::util::drawing::layouts::layer_positionings::brandes_kopf_positioning::BrandesKopfPositioning;
use super::super::util::drawing::layouts::layer_positionings::brandes_kopf_positioning_corrected::BrandesKopfPositioningCorrected;
use super::super::util::drawing::layouts::layer_positionings::dummy_layer_positioning::DummyLayerPositioning;
use super::super::util::drawing::layouts::layered_layout::LayeredLayout;
use super::super::util::drawing::layouts::layered_layout_traits::WidthLabel;
use super::super::util::drawing::layouts::random_test_layout::RandomTestLayout;
use super::super::util::drawing::layouts::sugiyama_lib_layout::SugiyamaLibLayout;
use super::super::util::drawing::layouts::toggle_layout::IndexedSelect;
use super::super::util::drawing::layouts::toggle_layout::ToggleLayout;
use super::super::util::drawing::layouts::toggle_layout::ToggleLayoutUnit;
use super::super::util::drawing::layouts::transition::transition_layout::TransitionLayout;
use super::super::util::drawing::renderer::Renderer;
use super::super::util::drawing::renderers::latex_renderer::LatexLayerStyle;
use super::super::util::drawing::renderers::latex_renderer::LatexNodeStyle;
use super::super::util::drawing::renderers::latex_renderer::LatexRenderer;
use super::super::util::drawing::renderers::util::Font::Font;
use super::super::util::drawing::renderers::webgl::edge_renderer::EdgeRenderingType;
use super::super::util::drawing::renderers::webgl::node_renderer::NodeRenderingColorConfig;
use super::super::util::drawing::renderers::webgl_renderer::WebglNodeStyle;
use super::super::util::drawing::renderers::webgl_renderer::WebglRenderer;
use super::super::util::graph_structure::graph_manipulators::group_presence_adjuster::GroupPresenceAdjuster;
use super::super::util::graph_structure::graph_manipulators::label_adjusters::group_label_adjuster::GroupLabelAdjuster;
use super::super::util::graph_structure::graph_manipulators::node_presence_adjuster::NodePresenceAdjuster;
use super::super::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceLabel;
use super::super::util::graph_structure::graph_manipulators::pointer_node_adjuster::PointerLabel;
use super::super::util::graph_structure::graph_manipulators::pointer_node_adjuster::PointerNodeAdjuster;
use super::super::util::graph_structure::graph_manipulators::rc_graph::RCGraph;
use super::super::util::graph_structure::graph_manipulators::terminal_level_adjuster::TerminalLevelAdjuster;
use super::super::util::graph_structure::graph_structure::{DrawTag, EdgeType, GraphStructure};
use super::super::util::graph_structure::grouped_graph_structure::GroupedGraphStructure;
use super::super::util::graph_structure::oxidd_graph_structure::NodeLabel;
use super::super::util::graph_structure::oxidd_graph_structure::OxiddGraphStructure;
use super::super::util::group_manager::GroupManager;
use super::super::util::storage::state_storage::Serializable;
use super::super::util::storage::state_storage::StateStorage;

// The drawers for QDD and BDD decision diagrams
// Note that we should eventually add reusable helper structure to reduce the perceived complexity of the entries to different diagram visualization implementations
pub struct QDDDiagram<MR: ManagerRef>
where
    for<'id> <<MR as oxidd::ManagerRef>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    manager_ref: MR,
}
impl QDDDiagram<DummyBDDManagerRef> {
    pub fn new() -> QDDDiagram<DummyBDDManagerRef> {
        let manager_ref = DummyBDDManagerRef::from(&DummyBDDManager::new());
        QDDDiagram { manager_ref }
    }
}

impl Diagram for QDDDiagram<DummyBDDManagerRef> {
    fn create_section_from_dddmp(&mut self, dddmp: String) -> Option<Box<dyn DiagramSection>> {
        let (roots, levels, is_bdd) = DummyBDDFunction::from_dddmp(&mut self.manager_ref, &dddmp);
        Some(Box::new(QDDDiagramSection::new(roots, is_bdd, levels)))
    }
    // Other == Buddy
    fn create_section_from_other(
        &mut self,
        data: String,
        vars: Option<String>,
    ) -> Option<Box<dyn DiagramSection>> {
        let (roots, levels, is_bdd) =
            DummyBDDFunction::from_buddy(&mut self.manager_ref, &data, vars.as_deref());
        Some(Box::new(QDDDiagramSection::new(roots, is_bdd, levels)))
    }
    fn create_section_from_ids(
        &self,
        sources: &[(oxidd::NodeID, &Box<dyn DiagramSection>)],
    ) -> Option<Box<dyn DiagramSection>> {
        let mut levels = Vec::new();
        let roots = sources
            .iter()
            .map(|&(id, section)| {
                let root_edge = DummyBDDEdge::new(Arc::new(id), self.manager_ref.clone());
                levels = section.get_level_labels();
                (DummyBDDFunction(root_edge), section.get_node_labels(id))
            })
            .collect_vec();
        let is_bdd = sources.iter().all(|&(_, section)| section.get_meta() == 1);
        Some(Box::new(QDDDiagramSection::new(roots, is_bdd, levels)))
    }
}

pub struct QDDDiagramSection<F: Function>
where
    for<'id> <<F as oxidd::Function>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    roots: Vec<(F, Vec<String>)>,
    labels: HashMap<NodeID, Vec<String>>,
    levels: Vec<String>,
    is_bdd: bool,
}

impl<F: Function> QDDDiagramSection<F>
where
    for<'id> <<F as oxidd::Function>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    fn new(roots: Vec<(F, Vec<String>)>, is_bdd: bool, levels: Vec<String>) -> Self {
        let s = QDDDiagramSection {
            labels: roots
                .iter()
                .map(|(f, names)| {
                    (
                        f.with_manager_shared(|_, edge| edge.node_id()),
                        names.clone(),
                    )
                })
                .collect(),
            roots,
            is_bdd,
            levels,
        };
        console::log!(
            "init {}",
            s.labels
                .iter()
                .map(|(id, names)| format!("{}:[{}]", id, names.iter().join(",")))
                .join(", ")
        );
        s
    }
}

#[derive(Clone)]
struct QDDColors {
    edge_true: Color,
    edge_false: Color,
    edge_both: Color,
    node_true: Color,
    node_false: Color,
    node_group: Color,
    node_default: Color,
    node_text: Color,
    node_label: Color,
    layer_background1: Color,
    layer_background2: Color,
    layer_text: Color,
    selection: TransparentColor,
    selection_partial: TransparentColor,
    selection_hover: TransparentColor,
    selection_hover_partial: TransparentColor,
}
impl QDDColors {
    const DARK: QDDColors = QDDColors {
        edge_true: Color(0.631, 0.749, 0.423),
        edge_false: Color(0.835, 0.341, 0.341),
        edge_both: Color(0.6, 0.6, 0.6),
        node_true: Color(0.631, 0.749, 0.423),
        node_false: Color(0.835, 0.341, 0.341),
        node_group: Color(0.45, 0.45, 0.45),
        node_default: Color(0.35, 0.35, 0.35),
        node_text: Color(0.0, 0.0, 0.0),
        node_label: Color(0.5, 0.5, 1.0),
        layer_background1: Color(0.125, 0.125, 0.125),
        layer_background2: Color(0.1875, 0.1875, 0.1875),
        layer_text: Color(1.0, 1.0, 1.0),
        selection: TransparentColor(0.6, 0.0, 1.0, 0.7),
        selection_partial: TransparentColor(0.6, 0.0, 1.0, 0.7),
        selection_hover: TransparentColor(0.0, 0.0, 1.0, 0.3),
        selection_hover_partial: TransparentColor(1.0, 0.0, 0.8, 0.2),
    };

    const LIGHT: QDDColors = QDDColors {
        edge_true: Color(0.2, 1.0, 0.2),
        edge_false: Color(1.0, 0.2, 0.2),
        edge_both: Color(0.6, 0.6, 0.6),
        node_true: Color(0.2, 1.0, 0.2),
        node_false: Color(1.0, 0.2, 0.2),
        node_group: Color(0.45, 0.45, 0.45),
        node_default: Color(0.1, 0.1, 0.1),
        node_text: Color(0.0, 0.0, 0.0),
        node_label: Color(0.5, 0.5, 1.0),
        layer_background1: Color(0.98, 0.98, 0.98),
        layer_background2: Color(0.9, 0.9, 0.9),
        layer_text: Color(0.0, 0.0, 0.0),
        selection: TransparentColor(0.6, 0.0, 1.0, 0.7),
        selection_partial: TransparentColor(0.6, 0.0, 1.0, 0.7),
        selection_hover: TransparentColor(0.0, 0.0, 1.0, 0.3),
        selection_hover_partial: TransparentColor(1.0, 0.0, 0.8, 0.2),
    };
}

impl DiagramSection for QDDDiagramSection<DummyBDDFunction> {
    fn get_level_labels(&self) -> Vec<String> {
        self.levels.clone()
    }
    fn get_node_labels(&self, node: NodeID) -> Vec<String> {
        self.labels.get(&node).cloned().unwrap_or_else(|| vec![])
    }
    fn create_drawer(&self, canvas: HtmlCanvasElement) -> Box<dyn DiagramSectionDrawer> {
        let graph =
            OxiddGraphStructure::new(self.roots.iter().cloned().collect(), self.levels.clone());

        let diagram = QDDDiagramDrawer::new(graph, self.is_bdd, canvas);
        Box::new(diagram)
    }
    fn get_meta(&self) -> i128 {
        self.is_bdd as i128
    }
}

#[derive(Clone)]
pub struct NodeData {
    color: Color,
    border_color: TransparentColor,
    width: f32,
    name: Option<String>,
    is_terminal: Option<usize>,
    is_group: bool,
}

impl Interpolatable for NodeData {
    fn mix(&self, other: &Self, frac: f32) -> Self {
        NodeData {
            color: self.color.mix(&other.color, frac),
            border_color: self.border_color.mix(&other.border_color, frac),
            width: self.width * (1.0 - frac) + other.width * frac,
            name: other.name.clone(),
            is_terminal: other.is_terminal.clone(),
            is_group: other.is_group,
        }
    }
}
impl LatexNodeStyle for NodeData {
    fn is_terminal(&self) -> Option<(String, Option<String>)> {
        self.is_terminal.map(|v| (format!("terminal{}", v), None))
    }

    fn is_group(&self) -> bool {
        self.is_group
    }

    fn get_label(&self) -> Option<String> {
        self.name.clone()
    }
}
impl WebglNodeStyle for NodeData {
    fn get_color(&self) -> Color {
        self.color.clone()
    }

    fn get_outline_color(&self) -> TransparentColor {
        self.border_color.clone()
    }

    fn get_label(&self) -> Option<String> {
        self.name.clone()
    }
}
impl WidthLabel for NodeData {
    fn get_width(&self) -> f32 {
        self.width
    }
}
impl NodeStyle for NodeData {}

#[derive(Clone)]
pub struct LayerData {
    name: String,
}
impl Interpolatable for LayerData {
    fn mix(&self, other: &Self, frac: f32) -> Self {
        LayerData {
            name: self.name.clone(),
        }
    }
}
impl LayerStyle for LayerData {
    fn squash(layers: Vec<Self>) -> Self {
        LayerData {
            name: layers.into_iter().map(|s| s.name).join(", \n"),
        }
    }
}
impl WebglLayerStyle for LayerData {
    fn get_label(&self) -> String {
        self.name.clone()
    }
}
impl LatexLayerStyle for LayerData {
    fn get_label(&self) -> String {
        self.name.clone()
    }
}

type GroupedGraph =
    GroupPresenceAdjuster<GroupLabelAdjuster<NodeData, LayerData, GroupManager<Graph>>>;
type Graph = RCGraph<TerminalLevelAdjuster<PresenceAdjuster>>;
type PresenceAdjuster = RCGraph<NodePresenceAdjuster<CofactorAdjuster>>;
type CofactorAdjuster = RCGraph<
    ReachabilityAdjuster<
        RCGraph<
            EdgeToAdjuster<
                RCGraph<ChildEdgeAdjuster<PointerNodeAdjuster<TerminalLevelAdjuster<BaseGraph>>>>,
            >,
        >,
    >,
>;
type BaseGraph = OxiddGraphStructure<(), DummyBDDFunction, String>;
type Layout = TransitionLayout<ToggleLayout<Layout1, ToggleLayoutUnit<Layout2>>>;
type Layout1 = LayeredLayout<
    GroupedGraph,
    SequenceOrdering<
        GroupedGraph,
        PseudoRandomLayerOrdering,
        SequenceOrdering<GroupedGraph, EdgeLayerOrdering, SugiyamaOrdering>,
    >,
    OrderingGroupAlignment,
    BrandesKopfPositioningCorrected,
>;
type Layout2 = LayeredLayout<
    GroupedGraph,
    SequenceOrdering<
        GroupedGraph,
        PseudoRandomLayerOrdering,
        SequenceOrdering<GroupedGraph, EdgeLayerOrdering, SugiyamaOrdering>,
    >,
    OrderingGroupAlignment,
    BrandesKopfPositioning,
>;

pub struct QDDDiagramDrawer {
    graph: Graph,
    group_manager: MutRcRefCell<GroupManager<Graph>>,
    presence_adjuster: PresenceAdjuster,
    time: MutRcRefCell<u32>,
    drawer: MutRcRefCell<Drawer<WebglRenderer<()>, Layout, GroupedGraph>>,
    config: Configuration<
        LocationConfig<
            PanelConfig<
                CompositeConfig<(
                    ContainerConfig<
                        CompositeConfig<(
                            LabelConfig<ChoiceConfig<bool>>,
                            LabelConfig<IntConfig>,
                            ButtonConfig,
                            LabelConfig<ChoiceConfig<usize>>,
                        )>,
                    >,
                    ContainerConfig<
                        LabelConfig<
                            CompositeConfig<(
                                LabelConfig<IntConfig>,
                                LabelConfig<IntConfig>,
                                ButtonConfig,
                            )>,
                        >,
                    >,
                    ContainerConfig<
                        LabelConfig<
                            CompositeConfig<(
                                LabelConfig<ChoiceConfig<PresenceRemainder>>,
                                LabelConfig<ChoiceConfig<PresenceRemainder>>,
                                ContainerConfig<LabelConfig<ChoiceConfig<bool>>>,
                            )>,
                        >,
                    >,
                    ContainerConfig<LabelConfig<TextConfig>>,
                    ContainerConfig<
                        LabelConfig<
                            CompositeConfig<(
                                ButtonConfig,
                                TextOutputConfig,
                                LabelConfig<TextOutputConfig>,
                            )>,
                        >,
                    >,
                )>,
            >,
        >,
    >,
}

impl QDDDiagramDrawer {
    pub fn new(graph: BaseGraph, is_bdd: bool, canvas: HtmlCanvasElement) -> Self {
        let start = js_sys::Date::now();

        let colors = &QDDColors::LIGHT;
        let edge_rendering_type =
            |color: Color, width: f32, dash_solid: f32, dash_transparent: f32| EdgeRenderingType {
                select_color: color.mix_transparent(&colors.selection),
                partial_select_color: color.mix_transparent(&colors.selection_partial),
                hover_color: color.mix_transparent(&colors.selection_hover),
                partial_hover_color: color.mix_transparent(&colors.selection_hover_partial),
                color,
                width,
                dash_solid,
                dash_transparent,
            };
        let font = Rc::new(Font::new(
            include_bytes!("../../../resources/Roboto-Bold.ttf").to_vec(),
            1.0,
        ));
        let renderer = WebglRenderer::from_canvas(
            canvas,
            HashMap::from([
                // True edge
                (
                    EdgeType::new((), 0),
                    edge_rendering_type(
                        colors.edge_true,
                        0.2,
                        1.0,
                        0.0, // No dashing
                    ),
                ),
                // False edge
                (
                    EdgeType::new((), 1),
                    edge_rendering_type(colors.edge_false, 0.2, 0.3, 0.15),
                ),
                // Label edge
                (
                    EdgeType::new((), 2),
                    edge_rendering_type(colors.edge_both, 0.15, 1.0, 0.0),
                ),
            ]),
            NodeRenderingColorConfig {
                select: colors.selection,
                partial_select: colors.selection_partial,
                hover: colors.selection_hover,
                partial_hover: colors.selection_hover_partial,
                text: colors.node_text,
            },
            LayerRenderingColorConfig {
                background1: colors.layer_background1.into(),
                background2: colors.layer_background2.into(),
                text: colors.layer_text,
            },
            font.clone(),
        )
        .unwrap();

        let layout_opt1: Layout1 = LayeredLayout::new(
            // SugiyamaOrdering::new(2, 2),
            SequenceOrdering::new(
                PseudoRandomLayerOrdering::new(2, 0),
                SequenceOrdering::new(EdgeLayerOrdering, SugiyamaOrdering::new(2, 2)),
            ),
            // AverageGroupAlignment,
            OrderingGroupAlignment,
            // BrandesKopfPositioning,
            BrandesKopfPositioningCorrected,
            // DummyLayerPositioning,
            0.3,
        );
        let layout_opt2: Layout2 = LayeredLayout::new(
            // SugiyamaOrdering::new(2, 2),
            SequenceOrdering::new(
                PseudoRandomLayerOrdering::new(2, 0),
                SequenceOrdering::new(EdgeLayerOrdering, SugiyamaOrdering::new(2, 2)),
            ),
            // AverageGroupAlignment,
            OrderingGroupAlignment,
            // BrandesKopfPositioning,
            BrandesKopfPositioning,
            // DummyLayerPositioning,
            0.1,
        );
        let layout = ToggleLayout::new(layout_opt1, ToggleLayoutUnit::new(layout_opt2));
        let layout: Layout = TransitionLayout::new(layout);

        let original_roots = graph.get_roots().clone();
        let base_graph = TerminalLevelAdjuster::new(graph); // Make sure that terminal levels make sense before possibly adding pointers to these terminals
        let pointer_adjuster = PointerNodeAdjuster::new(
            base_graph,
            EdgeType { tag: (), index: 2 },
            true,
            "".to_string(),
        );
        let child_edge_adjuster =
            RCGraph::new(ChildEdgeAdjuster::new(pointer_adjuster, move_shared_edge));
        let edge_to_adjuster = RCGraph::new(EdgeToAdjuster::new(child_edge_adjuster.clone()));
        let reachability_adjuster =
            RCGraph::new(ReachabilityAdjuster::new(edge_to_adjuster.clone()));
        let presence_adjuster: PresenceAdjuster =
            RCGraph::new(NodePresenceAdjuster::new(reachability_adjuster.clone()));
        let modified_graph: Graph =
            RCGraph::new(TerminalLevelAdjuster::new(presence_adjuster.clone()));
        let roots = modified_graph.get_roots();
        let group_manager = MutRcRefCell::new(GroupManager::new(modified_graph.clone()));

        let mut grouped_graph = GroupPresenceAdjuster::new(GroupLabelAdjuster::new_shared(
            group_manager.clone(),
            move |nodes| {
                // TODO: make this adjuster lazy, e.g. don't recompute for the same list of nodes
                let (is_terminal, is_group, mut color) = match (nodes.get(0), nodes.get(1)) {
                    (
                        Some(&PresenceLabel {
                            original_label:
                                ReachabilityLabel {
                                    original_label:
                                        PointerLabel::Node(NodeLabel {
                                            pointers: _,
                                            kind: NodeType::Terminal(ref terminal),
                                        }),
                                    reachable: _,
                                },
                            original_id: _,
                        }),
                        None,
                    ) => {
                        if terminal == "T" || terminal == "B" {
                            (Some(1), false, colors.node_true)
                        } else {
                            (Some(0), false, colors.node_false)
                        }
                    }
                    (
                        Some(&PresenceLabel {
                            original_label:
                                ReachabilityLabel {
                                    original_label: PointerLabel::Pointer(_),
                                    reachable: _,
                                },
                            original_id: _,
                        }),
                        None,
                    ) => (None, false, colors.node_label),
                    (Some(_), None) => (None, false, colors.node_default),
                    _ => (None, true, colors.node_group),
                };
                let name: Option<String> = match (nodes.get(0), nodes.get(1)) {
                    (
                        Some(&PresenceLabel {
                            original_label:
                                ReachabilityLabel {
                                    original_label: PointerLabel::Pointer(ref text),
                                    reachable: _,
                                },
                            original_id: _,
                        }),
                        None,
                    ) => Some(text.clone()),
                    _ => None,
                };
                let reachable = nodes.iter().any(|node| node.original_label.reachable);
                if !reachable {
                    color = color.mix(&Color(1.0, 1.0, 1.0), 0.7);
                }

                NodeData {
                    color,
                    border_color: TransparentColor(0.0, 0.0, 0.0, 0.0),
                    width: 1.
                        + match name {
                            Some(ref text) => font.measure_width(&text),
                            None => 0.,
                        },
                    name,
                    is_terminal,
                    is_group,
                }
            },
            move |layer_label| LayerData {
                name: layer_label.clone(),
            },
        ));
        grouped_graph.hide(0);

        const TOP_MARGIN: f32 = 40.0;
        let composite_config = CompositeConfig::new((
            ContainerConfig::new(
                // Only show these testing options for QDDs
                ContainerStyle::new().hidden(is_bdd),
                CompositeConfig::new((
                    LabelConfig::new(
                        "Move shared",
                        ChoiceConfig::new([
                            Choice::new(true, "enabled"),
                            Choice::new(false, "disabled"),
                        ]),
                    ),
                    LabelConfig::new("Seed", IntConfig::new_min_max(0, Some(0), None)),
                    ButtonConfig::new_labeled("Change seed"),
                    LabelConfig::new(
                        "Layout",
                        ChoiceConfig::new([Choice::new(0, "1"), Choice::new(1, "2")]),
                    ),
                )),
            ),
            ContainerConfig::new(
                ContainerStyle::new().margin_top(if is_bdd { 0.0 } else { TOP_MARGIN }),
                LabelConfig::new_styled(
                    "Node expansion",
                    LabelKind::Category,
                    CompositeConfig::new((
                        LabelConfig::new("Layers", IntConfig::new_min_max(4, Some(1), None)),
                        LabelConfig::new("Max nodes", IntConfig::new_min_max(100, Some(1), None)),
                        ButtonConfig::new_labeled("Expand initial group"),
                    )),
                ),
            ),
            ContainerConfig::new(
                ContainerStyle::new().margin_top(TOP_MARGIN),
                LabelConfig::new_styled(
                    "Terminals",
                    LabelKind::Category,
                    CompositeConfig::new((
                        LabelConfig::new("False visibility", {
                            let mut c = ChoiceConfig::new([
                                Choice::new(PresenceRemainder::Show, "show"),
                                Choice::new(PresenceRemainder::Duplicate, "duplicate"),
                                Choice::new(PresenceRemainder::Hide, "hide"),
                            ]);
                            c.set_index(2).commit();
                            c
                        }),
                        LabelConfig::new("True visibility", {
                            let mut c = ChoiceConfig::new([
                                Choice::new(PresenceRemainder::Show, "show"),
                                Choice::new(PresenceRemainder::Duplicate, "duplicate"),
                                Choice::new(PresenceRemainder::Hide, "hide"),
                            ]);
                            c.set_index(1).commit();
                            c
                        }),
                        // Only show this option for QDDs
                        ContainerConfig::new(
                            ContainerStyle::new().hidden(is_bdd),
                            LabelConfig::new("Shared true visibility", {
                                let mut c = ChoiceConfig::new([
                                    Choice::new(false, "show"),
                                    Choice::new(true, "hide"),
                                ]);
                                c.set_index(1).commit();
                                c
                            }),
                        ),
                    )),
                ),
            ),
            ContainerConfig::new(
                ContainerStyle::new().margin_top(TOP_MARGIN),
                LabelConfig::new_styled(
                    "Cofactor selection",
                    LabelKind::Category,
                    TextConfig::new(vec!["none".into()], "none".into()),
                ),
            ),
            ContainerConfig::new(
                ContainerStyle::new().margin_top(TOP_MARGIN),
                LabelConfig::new_styled(
                    "Latex",
                    LabelKind::Category,
                    CompositeConfig::new((
                        ButtonConfig::new_labeled("Generate"),
                        TextOutputConfig::new(true),
                        LabelConfig::new("Headers", TextOutputConfig::new(false)),
                    )),
                ),
            ),
        ));
        let config = Configuration::new(LocationConfig::new(
            Location::BOTTOM_RIGHT,
            PanelConfig::builder()
                .set_button_icon("Settings")
                .set_button_icon_description("Open visualization settings")
                .set_name("Settings")
                .set_category("visualization-settings")
                .set_open_side(OpenSide::Right)
                .set_open_size(0.3)
                .build(composite_config.clone()),
        ));

        let mut out = QDDDiagramDrawer {
            group_manager,
            presence_adjuster,
            graph: modified_graph,
            time: MutRcRefCell::new(0),
            drawer: MutRcRefCell::new(Drawer::new(
                renderer,
                layout,
                MutRcRefCell::new(grouped_graph),
            )),
            config,
        };

        let (qdd_config, expansion, terminal_config, cofactor_config, latex_config) =
            &*composite_config;
        let (move_shared, seed, change_seed, layout_config) = &***qdd_config;
        let (_max_expand_layers, _max_expand_nodes, expand_all) = &****expansion;
        let (false_visibility, true_visibility, hide_shared_true) = &****terminal_config;
        let (latex_generate, latex_output, latex_header_output) = &****latex_config;

        let drawer = out.drawer.clone();
        let layout_config_copy = layout_config.clone();
        let _ = on_configuration_change(&*layout_config, move || {
            drawer
                .get()
                .get_layout_rules()
                .get_layout_rules()
                .select_layout(layout_config_copy.get());
        });

        let drawer = out.drawer.clone();
        let mut seed_copy = seed.clone();
        change_seed.clone().add_press_listener(move || {
            let new_seed = seed_copy.get() + 1;
            seed_copy.set(new_seed).commit();
        });

        let seed_copy = seed.clone();
        let _ = on_configuration_change(&*seed, move || {
            let mut drawer = drawer.get();
            let p = drawer.get_layout_rules().get_layout_rules();
            p.get_layout_rules1()
                .get_ordering()
                .get_ordering1()
                .set_seed(seed_copy.get() as usize);
            p.get_layout_rules2()
                .get_layout_rules()
                .get_ordering()
                .get_ordering1()
                .set_seed(seed_copy.get() as usize);
        });

        let drawer = out.drawer.clone();
        let mut latex_renderer = LatexRenderer::<Layout>::new();
        let mut output = latex_output.clone();
        latex_generate.clone().add_press_listener(move || {
            latex_renderer.update_layout(&drawer.get().get_current_layout());
            latex_renderer.render(u32::MAX);
            let out = latex_renderer.get_output();
            output.set(out.into()).commit();
        });
        latex_header_output
            .clone()
            .set(latex_headers.to_string())
            .commit();

        let from = out.create_group(vec![TargetID(TargetIDType::NodeGroupID, 0)]);
        // let from = 0;
        for root in roots {
            out.create_group(vec![TargetID(TargetIDType::NodeID, root)]);
        }

        let group_manager = out.group_manager.clone();
        expand_all
            .clone()
            .add_press_listener(move || reveal_all(&group_manager, from, 10_000_000));

        // Connect the config
        let drawer = out.drawer.clone();
        let time = out.time.clone();
        fn set_terminal_presence<const P: usize>(
            presence_adjuster: &PresenceAdjuster,
            target_terminals: [String; P],
            presence: PresenceRemainder,
        ) -> () {
            let mut adjuster = presence_adjuster.get();
            let terminals = adjuster.get_terminals();
            let mut terminals = terminals.iter().filter_map(|&node| {
                match adjuster.get_node_label(node).original_label.original_label {
                    PointerLabel::Node(NodeLabel {
                        pointers: _,
                        kind: NodeType::Terminal(t),
                    }) if target_terminals.iter().any(|terminal| &t == terminal) => Some(node),
                    _ => None,
                }
            });
            let Some(target_terminal) = terminals.next() else {
                return;
            };

            adjuster.set_node_presence(target_terminal, PresenceGroups::remainder(presence));
        }

        let false_presence_adjuster = out.presence_adjuster.clone();
        let false_visibility_copy = false_visibility.clone();
        let _ = on_configuration_change(&*false_visibility, move || {
            set_terminal_presence(
                &false_presence_adjuster,
                ["F".into(), "E".into()],
                false_visibility_copy.get(),
            );
        });
        let true_presence_adjuster = out.presence_adjuster.clone();
        let true_visibility_copy = true_visibility.clone();
        let _ = on_configuration_change(&*true_visibility, move || {
            set_terminal_presence(
                &true_presence_adjuster,
                ["T".into(), "B".into()],
                true_visibility_copy.get(),
            );
        });

        setup_reachability_adjuster(cofactor_config, reachability_adjuster);

        let hide_shared_true_copy = hide_shared_true.clone();
        let _ = on_configuration_change(&*hide_shared_true, move || {
            if hide_shared_true_copy.get() {
                let hide_edges = edge_to_adjuster
                    .get_terminals()
                    .into_iter()
                    .filter_map(|node| match edge_to_adjuster.get_node_label(node) {
                        PointerLabel::Node(NodeLabel {
                            pointers: _,
                            kind: NodeType::Terminal(t),
                        }) if t == "T" => Some((node, EdgeType::new((), 2))),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .into_iter();
                edge_to_adjuster.get().set_remove_to_edges(hide_edges);
            } else {
                edge_to_adjuster
                    .get()
                    .set_remove_to_edges(HashSet::new().into_iter());
            }
        });

        let move_shared_copy = move_shared.clone();
        let _ = on_configuration_change(&*move_shared, move || {
            child_edge_adjuster
                .get()
                .set_enabled(move_shared_copy.get());
        });

        let _ = after_configuration_change(&composite_config, move || {
            drawer.get().layout(*time.get());
        });
        let end = js_sys::Date::now();
        console::log!("Setup took: {}", end - start);

        // Perform initialization
        let start = js_sys::Date::now();
        let max = 10000;
        if out.group_manager.read().get_nodes_of_group(from).len() < max {
            reveal_all(&out.group_manager, from, max);
        }
        let end = js_sys::Date::now();
        console::log!("Revealing took: {}", end - start);

        let start = js_sys::Date::now();
        out.drawer.get().layout(0);
        let end = js_sys::Date::now();
        console::log!("Layout took: {}", end - start);

        out
    }
}

fn reveal_all<G: GraphStructure>(
    group_manager: &MutRcRefCell<GroupManager<G>>,
    from_id: NodeGroupID,
    limit: usize,
) {
    let nodes = {
        let mut gm = group_manager.get();
        if !gm.get_groups().contains_key(&from_id) {
            return;
        }
        let explored_group = gm.create_group(vec![TargetID(TargetIDType::NodeGroupID, from_id)]);
        gm.get_nodes_of_group(explored_group)
    };
    let mut count = 0;
    let mut group_manager = group_manager.get();
    for node_id in nodes.into_iter().rev() {
        // console::log!("{node_id}");
        group_manager.create_group(vec![TargetID(TargetIDType::NodeID, node_id)]);

        count = count + 1;
        if limit > 0 && count >= limit {
            break;
        }
    }
}

impl DiagramSectionDrawer for QDDDiagramDrawer {
    fn render(&mut self, time: u32) -> () {
        *self.time.get() = time;
        self.drawer.get().render(time);
    }

    fn layout(&mut self, time: u32) -> () {
        self.drawer.get().layout(time);
    }

    fn set_transform(&mut self, width: u32, height: u32, x: f32, y: f32, scale: f32) -> () {
        self.drawer.get().set_transform(width, height, x, y, scale);
    }
    fn get_bounding_box(&self) -> crate::wasm_interface::BoundingBox {
        self.drawer.get().get_bounding_box()
    }

    fn set_step(&mut self, step: i32) -> Option<StepData> {
        todo!()
    }

    fn set_group(&mut self, from: Vec<TargetID>, to: NodeGroupID) -> bool {
        self.group_manager.get().set_group(from, to)
    }

    fn create_group(&mut self, from: Vec<TargetID>) -> NodeGroupID {
        // self.config.
        self.group_manager.get().create_group(from)
    }

    fn split_edges(&mut self, nodes: &[NodeID], fully: bool) {
        let (_qdd_config, expansion, _terminal_config, _cofactor_config, _latex_config) =
            &****self.config;
        let (max_expand_layers, max_expand_nodes, _expand_all) = &****expansion;
        self.group_manager.get().split_edges(
            nodes,
            max_expand_layers.get().unsigned_abs(),
            max_expand_nodes.get().unsigned_abs(),
        );
    }

    fn get_nodes(&self, area: Rectangle, max_group_expansion: usize) -> Vec<NodeID> {
        self.drawer.read().get_nodes(area, max_group_expansion)
    }

    fn set_selected_nodes(&mut self, selected_ids: &[NodeID], hovered_ids: &[NodeID]) {
        self.drawer.get().select_nodes(selected_ids, hovered_ids);
    }

    fn local_nodes_to_sources(&self, nodes: &[NodeID]) -> Vec<NodeID> {
        self.graph
            .local_nodes_to_sources(nodes.iter().cloned().collect())
    }

    fn source_nodes_to_local(&self, nodes: &[NodeID]) -> Vec<NodeID> {
        self.graph
            .source_nodes_to_local(nodes.iter().cloned().collect())
    }
    fn serialize_state(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let _ = self.group_manager.read().write(&mut Cursor::new(&mut out));
        out
    }

    fn deserialize_state(&mut self, state: Vec<u8>) -> () {
        let _ = self.group_manager.get().read(&mut Cursor::new(&state));
    }

    fn get_configuration(&self) -> AbstractConfigurationObject {
        self.config.get_abstract()
    }
}

fn move_shared_edge<T: DrawTag + 'static>(
    children: Vec<(EdgeType<T>, NodeID, PointerLabel<NodeLabel<String>>)>,
) -> Option<Vec<(EdgeType<T>, NodeID)>> {
    if children.len() != 3 {
        return None;
    }
    let edges = children
        .into_iter()
        .map(|(edge, to, label)| {
            if let PointerLabel::Node(NodeLabel {
                pointers: _,
                kind: NodeType::Terminal(t),
            }) = label
            {
                (t.clone(), (edge, to))
            } else {
                ("inner".to_string(), (edge, to))
            }
        })
        .collect::<HashMap<_, _>>();
    let Some((to_true_edge, true_node)) = edges.get("T") else {
        return None;
    };
    let Some((to_false_edge, false_node)) = edges.get("F") else {
        return None;
    };
    let Some((to_inner_edge, rest_node)) = edges.get("inner") else {
        return None;
    };

    if to_true_edge.index == 2 {
        return None;
    }

    return Some(vec![
        (to_true_edge.clone(), *rest_node),
        (to_inner_edge.clone(), *true_node),
        (to_false_edge.clone(), *false_node),
    ]);
    // return Some(vec![]);
}

fn setup_reachability_adjuster(
    cofactor_input: &ContainerConfig<LabelConfig<TextConfig>>,
    cofactor_adjuster: CofactorAdjuster,
) {
    let cofactor_input_copy = cofactor_input.clone();
    let _ = on_configuration_change(&*cofactor_input, move || {
        let input = cofactor_input_copy.get();

        let expr = input
            .rsplit_once(':')
            .map(|(_, rhs)| rhs)
            .unwrap_or(input.as_str());

        let named_cofactors = expr
            .replace("&amp;", "&")
            .split(|c: char| c == ',' || c == '&')
            .filter_map(|part| {
                let part = part.trim();
                if part.is_empty() {
                    return None;
                }

                if let Some(var) = part.strip_prefix('!') {
                    Some((var.trim().to_string(), false))
                } else {
                    Some((part.to_string(), true))
                }
            })
            .collect_vec();

        let cofactor_adjuster_inner = cofactor_adjuster.get();
        let known_levels = cofactor_adjuster_inner.get_known_levels();
        drop(cofactor_adjuster_inner);
        let level_names = known_levels
            .into_iter()
            .map(|level| (cofactor_adjuster.get_level_label(level), level))
            .collect::<HashMap<_, _>>();

        let level_cofactors = named_cofactors
            .into_iter()
            .filter_map(|(name, cofactor)| {
                level_names
                    .get(&name)
                    .map(|level| (*level, EdgeType::new((), if cofactor { 1 } else { 0 })))
            })
            .collect_vec();

        cofactor_adjuster
            .get()
            .set_remove_level_edges(level_cofactors.into_iter());
    });
}
