use itertools::Itertools;
use std::{collections::HashMap, io::Cursor, rc::Rc, sync::Arc};
use web_sys::HtmlCanvasElement;

use oxidd::{Edge, Function, InnerNode, Manager, ManagerRef, NodeID};
use oxidd_core::{DiagramRules, HasLevel};

use crate::{
    configuration::{
        configuration::Configuration,
        configuration_object::{AbstractConfigurationObject, Abstractable},
        observe_configuration::{after_configuration_change, on_configuration_change},
        types::{
            button_config::{ButtonConfig, ButtonStyle},
            choice_config::{Choice, ChoiceConfig},
            composite_config::CompositeConfig,
            container_config::{ContainerConfig, ContainerStyle},
            float_config::FloatConfig,
            int_config::IntConfig,
            label_config::{LabelConfig, LabelKind},
            location_config::{Location, LocationConfig},
            panel_config::{OpenSide, PanelConfig},
            text_output_config::TextOutputConfig,
        },
    },
    traits::{Diagram, DiagramSection, DiagramSectionDrawer},
    types::{
        qdd::qdd_drawer::QDDDiagramDrawer,
        util::{
            drawing::{
                diagram_layout::{LayerStyle, NodeStyle},
                drawer::Drawer,
                layout_rules::LayoutRules,
                layouts::{
                    layer_group_sorting::ordering_group_alignment::OrderingGroupAlignment,
                    layer_orderings::{
                        combinators::sequence_ordering::SequenceOrdering,
                        edge_layer_ordering::EdgeLayerOrdering,
                        pseudo_random_layer_ordering::PseudoRandomLayerOrdering,
                        sugiyama_ordering::SugiyamaOrdering,
                    },
                    layer_positionings::brandes_kopf_positioning_corrected::BrandesKopfPositioningCorrected,
                    layered_layout::LayeredLayout,
                    layered_layout_traits::WidthLabel,
                    transition::transition_layout::TransitionLayout,
                },
                renderer::Renderer,
                renderers::{
                    latex_renderer::{
                        latex_headers, LatexLayerStyle, LatexNodeStyle, LatexRenderer,
                    },
                    util::Font::Font,
                    webgl::{
                        edge_renderer::EdgeRenderingType, node_renderer::NodeRenderingColorConfig,
                    },
                    webgl_renderer::{
                        LayerRenderingColorConfig, WebglLayerStyle, WebglNodeStyle, WebglRenderer,
                    },
                },
            },
            graph_structure::{
                graph_manipulators::{
                    group_presence_adjuster::GroupPresenceAdjuster,
                    label_adjusters::group_label_adjuster::GroupLabelAdjuster,
                    node_presence_adjuster::{
                        NodePresenceAdjuster, PresenceGroups, PresenceLabel, PresenceRemainder,
                    },
                    pointer_node_adjuster::{PointerLabel, PointerNodeAdjuster},
                    rc_graph::RCGraph,
                    terminal_level_adjuster::TerminalLevelAdjuster,
                },
                graph_structure::{DrawTag, EdgeType, GraphStructure},
                grouped_graph_structure::GroupedGraphStructure,
                oxidd_graph_structure::{NodeLabel, NodeType, OxiddGraphStructure},
            },
            group_manager::GroupManager,
            storage::state_storage::{Serializable, StateStorage},
        },
    },
    util::{
        color::{Color, TransparentColor},
        dummy_mtbdd::{
            DummyMTBDDEdge, DummyMTBDDFunction, DummyMTBDDManager, DummyMTBDDManagerRef,
            MTBDDTerminal,
        },
        logging::console,
        rc_refcell::MutRcRefCell,
        rectangle::Rectangle,
        transition::Interpolatable,
    },
    wasm_interface::{NodeGroupID, StepData, TargetID, TargetIDType},
};

pub struct MTBDDDiagram<MR: ManagerRef>
where
    for<'id> <<MR as oxidd::ManagerRef>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    manager_ref: MR,
}
impl MTBDDDiagram<DummyMTBDDManagerRef> {
    pub fn new() -> MTBDDDiagram<DummyMTBDDManagerRef> {
        let manager_ref = DummyMTBDDManagerRef::from(&DummyMTBDDManager::new());
        MTBDDDiagram { manager_ref }
    }
}

impl Diagram for MTBDDDiagram<DummyMTBDDManagerRef> {
    fn create_section_from_dddmp(
        &mut self,
        dddmp: String,
    ) -> Option<Box<dyn crate::traits::DiagramSection>> {
        let (roots, levels) = DummyMTBDDFunction::from_dddmp(&mut self.manager_ref, &dddmp);
        Some(Box::new(MTBDDDiagramSection::new(roots, levels)))
    }

    // Does not support other imports
    fn create_section_from_other(
        &mut self,
        data: String,
        vars: Option<String>,
    ) -> Option<Box<dyn crate::traits::DiagramSection>> {
        todo!()
    }

    fn create_section_from_ids(
        &self,
        sources: &[(oxidd::NodeID, &Box<dyn crate::traits::DiagramSection>)],
    ) -> Option<Box<dyn crate::traits::DiagramSection>> {
        let mut levels = Vec::new();
        let roots = sources
            .iter()
            .map(|&(id, section)| {
                let root_edge = DummyMTBDDEdge::new(Arc::new(id), self.manager_ref.clone());
                levels = section.get_level_labels();
                (DummyMTBDDFunction(root_edge), section.get_node_labels(id))
            })
            .collect_vec();
        Some(Box::new(MTBDDDiagramSection::new(roots, levels)))
    }
}

pub struct MTBDDDiagramSection<F: Function>
where
    for<'id> <<F as oxidd::Function>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    roots: Vec<(F, Vec<String>)>,
    labels: HashMap<NodeID, Vec<String>>,
    levels: Vec<String>,
}
impl<F: Function> MTBDDDiagramSection<F>
where
    for<'id> <<F as oxidd::Function>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    fn new(roots: Vec<(F, Vec<String>)>, levels: Vec<String>) -> Self {
        let s = MTBDDDiagramSection {
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
struct MTBDDColors {
    edge_true: Color,
    edge_false: Color,
    edge_label: Color,
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
impl MTBDDColors {
    const DARK: MTBDDColors = MTBDDColors {
        edge_true: Color(0.631, 0.749, 0.423),
        edge_false: Color(0.835, 0.341, 0.341),
        edge_label: Color(0.6, 0.6, 0.6),
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

    const LIGHT: MTBDDColors = MTBDDColors {
        edge_true: Color(0.2, 1.0, 0.2),
        edge_false: Color(1.0, 0.2, 0.2),
        edge_label: Color(0.6, 0.6, 0.6),
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

impl DiagramSection for MTBDDDiagramSection<DummyMTBDDFunction> {
    fn get_level_labels(&self) -> Vec<String> {
        self.levels.clone()
    }
    fn get_node_labels(&self, node: NodeID) -> Vec<String> {
        self.labels.get(&node).cloned().unwrap_or_else(|| vec![])
    }
    fn create_drawer(&self, canvas: HtmlCanvasElement) -> Box<dyn DiagramSectionDrawer> {
        let graph =
            OxiddGraphStructure::new(self.roots.iter().cloned().collect(), self.levels.clone());
        let diagram = MTBDDDiagramDrawer::new(graph, canvas);
        Box::new(diagram)
    }
    fn get_meta(&self) -> i128 {
        0
    }
}

#[derive(Clone)]
pub struct NodeData {
    color: Color,
    border_color: TransparentColor,
    width: f32,
    name: Option<String>,
    is_terminal: Option<MTBDDTerminal>,
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
        self.is_terminal
            .map(|v| ("terminal".to_string(), Some(format!("{}", v))))
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
type PresenceAdjuster =
    RCGraph<NodePresenceAdjuster<PointerNodeAdjuster<TerminalLevelAdjuster<BaseGraph>>>>;
type BaseGraph = OxiddGraphStructure<(), DummyMTBDDFunction, MTBDDTerminal>;

type Layout = TransitionLayout<
    LayeredLayout<
        GroupedGraph,
        SequenceOrdering<GroupedGraph, EdgeLayerOrdering, SugiyamaOrdering>,
        OrderingGroupAlignment,
        BrandesKopfPositioningCorrected,
    >,
>;

pub struct MTBDDDiagramDrawer {
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
                                ButtonConfig,
                                LabelConfig<ChoiceConfig<PresenceRemainder>>,
                                LabelConfig<ChoiceConfig<PresenceRemainder>>,
                                LabelConfig<CompositeConfig<(FloatConfig, FloatConfig)>>,
                            )>,
                        >,
                    >,
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

impl MTBDDDiagramDrawer {
    pub fn new(graph: BaseGraph, canvas: HtmlCanvasElement) -> Self {
        let colors = &MTBDDColors::LIGHT;

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
                    edge_rendering_type(colors.edge_label, 0.15, 1.0, 0.0),
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
        let layout = LayeredLayout::new(
            // SugiyamaOrdering::new(2, 2),
            SequenceOrdering::new(EdgeLayerOrdering, SugiyamaOrdering::new(2, 2)),
            // AverageGroupAlignment,
            OrderingGroupAlignment,
            // BrandesKopfPositioning,
            BrandesKopfPositioningCorrected,
            // DummyLayerPositioning,
            0.3,
        );
        let layout = TransitionLayout::new(layout);

        let original_roots = graph.get_roots().clone();
        let base_graph = TerminalLevelAdjuster::new(graph); // Make sure that terminal levels make sense before possibly adding pointers to these terminals
        let pointer_adjuster = PointerNodeAdjuster::new(
            base_graph,
            EdgeType { tag: (), index: 2 },
            true,
            "".to_string(),
        );
        let presence_adjuster = RCGraph::new(NodePresenceAdjuster::new(pointer_adjuster));
        let modified_graph = RCGraph::new(TerminalLevelAdjuster::new(presence_adjuster.clone()));
        let roots = modified_graph.get_roots();
        let group_manager = MutRcRefCell::new(GroupManager::new(modified_graph.clone()));

        let (terminal_min, terminal_max) = (FloatConfig::new(0.), FloatConfig::new(1.));
        let (terminal_min_ref, terminal_max_ref) = (terminal_min.clone(), terminal_max.clone());
        let mut grouped_graph = GroupPresenceAdjuster::new(GroupLabelAdjuster::new_shared(
            group_manager.clone(),
            move |nodes| {
                let (is_terminal, is_group, color) = match (nodes.get(0), nodes.get(1)) {
                    (
                        Some(&PresenceLabel {
                            original_label:
                                PointerLabel::Node(NodeLabel {
                                    pointers: _,
                                    kind: NodeType::Terminal(ref terminal),
                                }),
                            original_id: _,
                        }),
                        None,
                    ) => {
                        let min = terminal_min_ref.get();
                        let max = terminal_max_ref.get();
                        let per = ((terminal.0 - min) / (max - min)).max(0.0).min(1.0);
                        (
                            Some(*terminal),
                            false,
                            colors.node_false.mix(&colors.node_true, per),
                        )
                    }
                    (
                        Some(&PresenceLabel {
                            original_label: PointerLabel::Pointer(_),
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
                            original_label: PointerLabel::Pointer(ref text),
                            original_id: _,
                        }),
                        None,
                    ) => Some(text.clone()),
                    _ => None,
                }
                .or_else(|| is_terminal.map(|t| format!("{}", t)));

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
                ContainerStyle::new(),
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
                        ButtonConfig::new_labeled("Expand"),
                        LabelConfig::new("0 visibility", {
                            let mut c = ChoiceConfig::new([
                                Choice::new(PresenceRemainder::Show, "show"),
                                Choice::new(PresenceRemainder::Duplicate, "duplicate"),
                                Choice::new(PresenceRemainder::Hide, "hide"),
                            ]);
                            c.set_index(2).commit();
                            c
                        }),
                        LabelConfig::new(
                            "1 visibility",
                            ChoiceConfig::new([
                                Choice::new(PresenceRemainder::Show, "show"),
                                Choice::new(PresenceRemainder::Duplicate, "duplicate"),
                                Choice::new(PresenceRemainder::Hide, "hide"),
                            ]),
                        ),
                        LabelConfig::new(
                            "range",
                            CompositeConfig::new_horizontal(
                                (terminal_min, terminal_max),
                                |(f1, f2)| vec![Box::new(f1.clone()), Box::new(f2.clone())],
                            ),
                        ),
                    )),
                ),
            ),
            ContainerConfig::new(
                ContainerStyle::new().margin_top(TOP_MARGIN),
                LabelConfig::new_styled(
                    "LaTeX",
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

        let mut out = MTBDDDiagramDrawer {
            group_manager,
            graph: modified_graph,
            presence_adjuster,
            time: MutRcRefCell::new(0),
            drawer: MutRcRefCell::new(Drawer::new(
                renderer,
                layout,
                MutRcRefCell::new(grouped_graph),
            )),
            config,
        };

        let (expansion, terminals, latex_config) = &*composite_config;
        let (_max_expand_layers, _max_expand_nodes, expand_all) = &****expansion;
        let (generate_latex, latex_output, latex_headers_output) = &****latex_config;
        let (expand_terminals, zero_visibility, one_visibility, terminal_range) = &****terminals;
        let (_terminal_range_start, _terminal_range_end) = &***terminal_range;

        let drawer = out.drawer.clone();
        let mut latex_renderer = LatexRenderer::<Layout>::new();
        let mut output = latex_output.clone();
        generate_latex.clone().add_press_listener(move || {
            latex_renderer.update_layout(&drawer.get().get_current_layout());
            latex_renderer.render(u32::MAX);
            let out = latex_renderer.get_output();
            output.set(out.into()).commit();
        });
        latex_headers_output
            .clone()
            .set(latex_headers.to_string())
            .commit();

        let from = out.create_group(vec![TargetID(TargetIDType::NodeGroupID, 0)]);
        // let from = 0;
        for root in roots {
            out.create_group(vec![TargetID(TargetIDType::NodeID, root)]);
        }

        let max = 500;
        if out.group_manager.read().get_nodes_of_group(from).len() < max {
            reveal_all(&out.group_manager, from, max);
        }

        let group_manager = out.group_manager.clone();
        expand_all
            .clone()
            .add_press_listener(move || reveal_all(&group_manager, from, 10_000_000));

        let group_manager = out.group_manager.clone();
        let mut graph = out.graph.clone();
        expand_terminals.clone().add_press_listener(move || {
            for t in graph.get_terminals() {
                if graph.get_known_parents(t).len() > 0 {
                    group_manager
                        .get()
                        .create_group(vec![TargetID(TargetIDType::NodeID, t)]);
                }
            }
        });

        fn set_terminal_presence(
            presence_adjuster: &PresenceAdjuster,
            terminal: MTBDDTerminal,
            presence: PresenceRemainder,
        ) -> () {
            let mut adjuster = presence_adjuster.get();
            let terminals = adjuster.get_terminals();
            let mut terminals = terminals.iter().filter_map(|&node| {
                match adjuster.get_node_label(node).original_label {
                    PointerLabel::Node(NodeLabel {
                        pointers: _,
                        kind: NodeType::Terminal(t),
                    }) if t == terminal => Some(node),
                    _ => None,
                }
            });
            let Some(target_terminal) = terminals.next() else {
                return;
            };

            adjuster.set_node_presence(target_terminal, PresenceGroups::remainder(presence));
        }
        let false_config = zero_visibility.clone();
        let false_presence_adjuster = out.presence_adjuster.clone();
        let _ = on_configuration_change(zero_visibility, move || {
            set_terminal_presence(
                &false_presence_adjuster,
                MTBDDTerminal(0.),
                false_config.get(),
            );
        });
        let true_config = one_visibility.clone();
        let true_presence_adjuster = out.presence_adjuster.clone();
        let _ = on_configuration_change(one_visibility, move || {
            set_terminal_presence(
                &true_presence_adjuster,
                MTBDDTerminal(1.),
                true_config.get(),
            );
        });

        // Redraw on interaction
        let time = out.time.clone();
        let drawer = out.drawer.clone();
        let _ = after_configuration_change(&composite_config, move || {
            drawer.get().layout(*time.get());
        });

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

impl DiagramSectionDrawer for MTBDDDiagramDrawer {
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

    fn set_step(&mut self, step: i32) -> Option<StepData> {
        todo!()
    }

    fn set_group(&mut self, from: Vec<TargetID>, to: NodeGroupID) -> bool {
        self.group_manager.get().set_group(from, to)
    }

    fn create_group(&mut self, from: Vec<TargetID>) -> NodeGroupID {
        self.group_manager.get().create_group(from)
    }

    fn split_edges(&mut self, nodes: &[NodeID], fully: bool) {
        let (expansion, _terminals, _latex_config) = &****self.config;
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
        let time = *self.time.get();
        self.layout(time);
    }

    fn get_configuration(&self) -> AbstractConfigurationObject {
        self.config.get_abstract()
    }
}
