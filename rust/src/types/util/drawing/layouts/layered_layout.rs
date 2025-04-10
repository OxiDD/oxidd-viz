use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
    marker::PhantomData,
};

use itertools::Itertools;
use num_rational::Ratio;
use oxidd::{LevelNo, NodeID};
use oxidd_core::Tag;
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    types::util::{
        drawing::{
            diagram_layout::{
                DiagramLayout, EdgeLayout, EdgePoint, LayerStyle, NodeGroupLayout, NodeStyle,
            },
            layout_rules::LayoutRules,
        },
        graph_structure::{
            graph_structure::DrawTag,
            grouped_graph_structure::{EdgeCountData, EdgeData, GroupedGraphStructure},
        },
    },
    util::{
        logging::console,
        point::Point,
        rectangle::Rectangle,
        transition::{Interpolatable, Transition},
    },
    wasm_interface::NodeGroupID,
};

use super::{
    layered_layout_traits::{LayerGroupSorting, LayerOrdering, NodePositioning, WidthLabel},
    util::{
        compute_layers_layout::compute_layers_layout,
        layered::layer_orderer::{get_sequence, EdgeLayoutData, EdgeMap, Order},
        remove_redundant_bendpoints::remove_redundant_bendpoints,
    },
};
pub struct LayeredLayout<
    G: GroupedGraphStructure,
    O: LayerOrdering<G>,
    GS: LayerGroupSorting<G>,
    P: NodePositioning<G>,
> {
    ordering: O,
    group_aligning: GS,
    positioning: P,
    max_curve_offset: f32,
    group_edge_data: EdgeLayoutData,
    graph: PhantomData<G>,
}

impl<
        G: GroupedGraphStructure,
        O: LayerOrdering<G>,
        GS: LayerGroupSorting<G>,
        P: NodePositioning<G>,
    > LayeredLayout<G, O, GS, P>
{
    pub fn new(ordering: O, group_aligning: GS, positioning: P, max_curve_offset: f32) -> Self {
        LayeredLayout {
            ordering,
            group_aligning,
            positioning,
            max_curve_offset,
            graph: PhantomData,
            group_edge_data: EdgeLayoutData {
                weight: 1000,
                order: -1,
            }, // TODO: make configurable
        }
    }

    pub fn get_ordering(&mut self) -> &mut O {
        &mut self.ordering
    }
    pub fn get_group_aligning(&mut self) -> &mut GS {
        &mut self.group_aligning
    }
    pub fn get_positioning_aligning(&mut self) -> &mut P {
        &mut self.positioning
    }
}

pub fn is_group_dummy(
    node: NodeGroupID,
    dummy_group_start_id: NodeGroupID,
    dummy_edge_start_id: NodeGroupID,
) -> bool {
    node >= dummy_group_start_id && node < dummy_edge_start_id
}
pub fn is_edge_dummy(node: NodeGroupID, dummy_edge_start_id: NodeGroupID) -> bool {
    node >= dummy_edge_start_id
}

impl<
        G: GroupedGraphStructure,
        O: LayerOrdering<G>,
        S: LayerGroupSorting<G>,
        P: NodePositioning<G>,
    > LayoutRules for LayeredLayout<G, O, S, P>
where
    G::GL: NodeStyle + WidthLabel,
    G::LL: LayerStyle,
{
    type T = G::T;
    type NS = G::GL;
    type LS = G::LL;
    type Tracker = G::Tracker;
    type G = G;

    fn layout(
        &mut self,
        graph: &G,
        old: &DiagramLayout<Self::T, Self::NS, Self::LS>,
        sources: &G::Tracker,
        time: u32,
    ) -> DiagramLayout<Self::T, Self::NS, Self::LS> {
        // Setup the layers and edges, and a way of adding o them
        let mut layers: Vec<Order> = Vec::new();
        let mut edges: EdgeMap = HashMap::new();

        let mut dummy_owners: HashMap<NodeGroupID, NodeGroupID> = HashMap::new();
        let mut next_free_id = 0; // uninitialized, will be initialized by add_groups

        let (dummy_group_start_id, group_layers) = add_groups_with_dummies(
            graph,
            &mut layers,
            &mut edges,
            self.group_edge_data,
            &mut dummy_owners,
            &mut next_free_id,
        );
        let dummy_edge_start_id = next_free_id;

        let (edge_bend_nodes, edge_connection_nodes) = add_edges_with_dummies(
            graph,
            &mut layers,
            &mut edges,
            &mut dummy_owners,
            &group_layers,
            &mut next_free_id,
        );

        let node_widths = &layers
            .iter()
            .flatten()
            .map(|(&node, _)| {
                (
                    node,
                    get_node_width(node, graph, &dummy_owners, dummy_group_start_id),
                )
            })
            .collect();

        // Perform node positioning
        let layers = self.ordering.order_nodes(
            graph,
            &layers,
            &edges,
            dummy_group_start_id,
            dummy_edge_start_id,
            &dummy_owners,
        );

        // Sort the groupings, such that they never cross each-other, and remove other edges that cross groups
        let layers = self.group_aligning.align_cross_layer_nodes(
            graph,
            &layers,
            &edges,
            dummy_group_start_id,
            dummy_edge_start_id,
            &dummy_owners,
        );
        remove_group_crossings(&layers, &mut edges, &dummy_owners);

        // Perform node-positioning
        let (node_positions, layer_positions) = self.positioning.position_nodes(
            graph,
            &layers,
            &edges,
            &node_widths,
            dummy_group_start_id,
            dummy_edge_start_id,
            &dummy_owners,
        );

        format_layout(
            graph,
            self.max_curve_offset,
            node_positions,
            &node_widths,
            layer_positions,
            edge_bend_nodes,
            edge_connection_nodes,
            dummy_group_start_id,
        )
    }
}

fn get_node_width<G: GroupedGraphStructure>(
    node: NodeGroupID,
    graph: &G,
    owners: &HashMap<NodeGroupID, NodeGroupID>,
    dummy_group_start_id: NodeGroupID,
) -> f32
where
    G::GL: WidthLabel,
{
    let owner = owners.get(&node).cloned().unwrap_or(node);
    if owner < dummy_group_start_id {
        graph.get_group_label(owner).get_width()
    } else {
        0.
    }
}

fn add_to_layer(layers: &mut Vec<Order>, layer: usize, id: NodeGroupID) {
    while layer >= layers.len() {
        layers.push(HashMap::new());
    }
    let layer = layers.get_mut(layer).unwrap();
    layer.insert(id, layer.len());
}

fn add_to_edges(edges: &mut EdgeMap, from: NodeGroupID, to: NodeGroupID, data: EdgeLayoutData) {
    edges
        .entry(from)
        .or_insert_with(|| HashMap::new())
        .insert(to, data);
}

fn add_groups_with_dummies<G: GroupedGraphStructure>(
    graph: &G,
    layers: &mut Vec<Order>,
    edges: &mut EdgeMap,
    group_edge_data: EdgeLayoutData,
    dummy_owners: &mut HashMap<NodeGroupID, NodeGroupID>,
    next_free_id: &mut NodeGroupID,
) -> (NodeGroupID, HashMap<NodeGroupID, HashMap<u32, usize>>)
where
    G::GL: NodeStyle,
    G::LL: LayerStyle,
{
    let mut group_layers: HashMap<NodeGroupID, HashMap<u32, usize>> = HashMap::new();
    for group in graph.get_all_groups() {
        let (start, _end) = graph.get_level_range(group);
        add_to_layer(layers, start as usize, group);
        group_layers.insert(group, HashMap::from([(start, group)]));
        if group >= *next_free_id {
            *next_free_id = group + 1;
        }
    }
    let dummy_group_start_id = *next_free_id;

    for group in graph.get_all_groups() {
        let (start, end) = graph.get_level_range(group);
        dummy_owners.insert(group, group);
        let mut prev = group;
        for layer in start + 1..=end {
            let layer_group_id = *next_free_id;
            *next_free_id += 1;
            add_to_edges(edges, prev, layer_group_id, group_edge_data);
            dummy_owners.insert(layer_group_id, group);
            add_to_layer(layers, layer as usize, layer_group_id);
            group_layers
                .entry(group)
                .or_default()
                .insert(layer, layer_group_id);
            prev = layer_group_id;
        }
    }

    (dummy_group_start_id, group_layers)
}

fn add_edges_with_dummies<G: GroupedGraphStructure>(
    graph: &G,
    layers: &mut Vec<Order>,
    edges: &mut EdgeMap,
    dummy_owners: &mut HashMap<NodeGroupID, NodeGroupID>,
    group_layers: &HashMap<NodeGroupID, HashMap<u32, usize>>,
    next_free_id: &mut NodeGroupID,
) -> (
    HashMap<(NodeGroupID, EdgeData<G::T>), Vec<NodeGroupID>>,
    HashMap<(NodeGroupID, EdgeData<G::T>), (NodeGroupID, NodeGroupID)>,
)
where
    G::GL: NodeStyle,
    G::LL: LayerStyle,
{
    let mut edge_bend_nodes: HashMap<(NodeGroupID, EdgeData<G::T>), Vec<NodeGroupID>> =
        HashMap::new();
    let mut edge_connection_nodes: HashMap<
        (NodeGroupID, EdgeData<G::T>),
        (NodeGroupID, NodeGroupID),
    > = HashMap::new();

    for group in graph.get_all_groups() {
        // let (parent_start_level, parent_end_level) = graph.get_level_range(group);

        for EdgeCountData {
            to: to_group,
            from_level: edge_start_level,
            to_level: edge_end_level,
            edge_type,
            count: _,
        } in graph.get_children(group)
        {
            let edge_data = EdgeData::new(to_group, edge_start_level, edge_end_level, edge_type);

            let Some(group_connections) = group_layers.get(&group) else {
                continue;
            };
            let Some(group_connection) = group_connections.get(&edge_start_level) else {
                continue;
            };

            let mut prev = *group_connection;
            let mut bends = Vec::new();
            let first_bend_id = *next_free_id;

            for layer in (edge_start_level + 1)..edge_end_level {
                let id = *next_free_id;
                *next_free_id += 1;
                dummy_owners.insert(id, first_bend_id);
                bends.push(id);
                add_to_layer(layers, layer as usize, id);
                add_to_edges(
                    edges,
                    prev,
                    id,
                    EdgeLayoutData {
                        weight: 1,
                        order: edge_type.index,
                    },
                );
                prev = id;
            }
            edge_bend_nodes.insert((group, edge_data.clone()), bends);

            let Some(to_group_connections) = group_layers.get(&to_group) else {
                console::log!(
                    "Non existent target group: {};{} -> {};{}",
                    group,
                    edge_start_level,
                    to_group,
                    edge_end_level
                );
                continue;
            };
            let Some(&to_group_connection) = to_group_connections.get(&edge_end_level) else {
                console::log!(
                    "Non existent target level: {};{} -> {};{}",
                    group,
                    edge_start_level,
                    to_group,
                    edge_end_level
                );
                continue;
            };
            edge_connection_nodes
                .insert((group, edge_data), (*group_connection, to_group_connection));
            add_to_edges(
                edges,
                prev,
                to_group_connection,
                EdgeLayoutData {
                    weight: 1,
                    order: edge_type.index,
                },
            );
        }
    }

    (edge_bend_nodes, edge_connection_nodes)
}

fn remove_group_crossings(
    layers: &Vec<Order>,
    edges: &mut EdgeMap,
    dummy_owners: &HashMap<NodeGroupID, NodeGroupID>,
) {
    if layers.len() == 0 {
        return;
    }

    let layer_order = layers.iter().map(get_sequence).collect_vec();
    let all_layer_group_indices = layers
        .iter()
        .map(|l| {
            l.iter()
                .filter_map(|(node, index)| dummy_owners.get(node).map(|owner| (*owner, *index)))
                .collect::<HashMap<_, _>>()
        })
        .collect_vec();

    for i in 0..(layers.len() - 1) {
        let layer = &layer_order[i];
        let next_layer = &layers[i + 1];

        let next_layer_groups = &all_layer_group_indices[i + 1];
        let mut shared_layer_groups = all_layer_group_indices[i]
            .iter()
            .filter_map(|(group, from_index)| {
                next_layer_groups
                    .get(group)
                    .map(|to_index| (*group, *from_index, *to_index))
            })
            .sorted_by_key(|(_, from_index, _)| *from_index)
            .collect_vec();

        // Remove left to right downwards crossings
        let mut node_index = 0;
        for &(_group, from_index, to_index) in &shared_layer_groups {
            // For each node to the left of from_index, remove any edges to the right of to_index (keep everything that's to the left of to_index)
            while node_index < from_index {
                let node = layer[node_index];
                if let Some(node_edges) = edges.get_mut(&node) {
                    node_edges.retain(|to_node, _| {
                        next_layer
                            .get(to_node)
                            .map(|&index| index <= to_index)
                            .unwrap_or(false)
                    });
                }
                node_index += 1;
            }
        }

        // Remove right to left downwards crossings
        shared_layer_groups.reverse();
        if layer.len() == 0 {
            continue;
        }
        node_index = layer.len() - 1;
        for &(_group, from_index, to_index) in &shared_layer_groups {
            while node_index > from_index {
                let node = layer[node_index];
                if let Some(node_edges) = edges.get_mut(&node) {
                    node_edges.retain(|to_node, _| {
                        next_layer
                            .get(to_node)
                            .map(|&index| index >= to_index)
                            .unwrap_or(false)
                    });
                }

                node_index -= 1;
            }
        }
    }
}

fn format_layout<G: GroupedGraphStructure>(
    graph: &G,
    max_curve_offset: f32,
    node_positions: HashMap<usize, Point>,
    node_widths: &HashMap<NodeGroupID, f32>,
    layer_positions: HashMap<LevelNo, f32>,
    edge_bend_nodes: HashMap<(NodeGroupID, EdgeData<G::T>), Vec<NodeGroupID>>,
    edge_connection_nodes: HashMap<(NodeGroupID, EdgeData<G::T>), (NodeGroupID, NodeGroupID)>,
    dummy_group_start_id: usize,
) -> DiagramLayout<G::T, G::GL, G::LL>
where
    G::GL: NodeStyle + WidthLabel,
    G::LL: LayerStyle,
{
    let node_size = 1.; // TODO: make configurable
    let node_size_shift = -0.5
        * Point {
            x: node_size,
            y: node_size,
        };
    let node_positions: HashMap<usize, Point> = node_positions
        .iter()
        .map(|(&group_id, &pos)| (group_id, pos + node_size_shift))
        .collect();
    let bottom_node_positions: HashMap<usize, Point> = node_positions
        .iter()
        .map(|(&group_id, pos)| {
            (
                group_id,
                (if group_id >= dummy_group_start_id {
                    *pos
                } else {
                    let (s, e) = graph.get_level_range(group_id);
                    Point {
                        x: pos.x,
                        y: pos.y
                            - (layer_positions.get(&s).unwrap_or(&0.)
                                - layer_positions.get(&e).unwrap_or(&0.)),
                    }
                }),
            )
        })
        .collect();

    // Map to a diagram layout
    DiagramLayout {
        layers: compute_layers_layout(
            graph,
            node_positions
                .iter()
                .filter(|(&group_id, _)| group_id < dummy_group_start_id)
                .map(|(&group_id, pos)| {
                    let (s, e) = graph.get_level_range(group_id);

                    let start_layer_y = layer_positions.get(&s).unwrap_or(&0.);
                    let prev_layer_y = (if s > 0 {
                        layer_positions.get(&(s - 1)).cloned()
                    } else {
                        None
                    })
                    .unwrap_or(start_layer_y + 2. * node_size);
                    let start_y = (start_layer_y + prev_layer_y) / 2.0;

                    let end_layer_y = *layer_positions.get(&e).unwrap_or(&0.);
                    let next_layer_y = layer_positions
                        .get(&(e + 1))
                        .cloned()
                        .unwrap_or(end_layer_y - 2. * node_size);
                    let end_y = (end_layer_y + next_layer_y) / 2.0;
                    (group_id, Rectangle::new(0., end_y, 0., start_y - end_y))
                }),
        ),
        groups: graph
            .get_all_groups()
            .iter()
            .map(|&group_id| {
                let (s, e) = graph.get_level_range(group_id);
                let node_width = graph.get_group_label(group_id).get_width();
                (
                    group_id,
                    NodeGroupLayout {
                        style: Transition::plain(graph.get_group_label(group_id)),
                        position: Transition::plain(*bottom_node_positions.get(&group_id).unwrap()),
                        size: Transition::plain(Point {
                            x: node_width,
                            y: node_size
                                + (layer_positions.get(&s).unwrap_or(&0.)
                                    - layer_positions.get(&e).unwrap_or(&0.))
                                    * node_size,
                        }),
                        level_range: (s, e),
                        exists: Transition::plain(1.),
                        edges: graph
                            .get_children(group_id)
                            .into_iter()
                            .enumerate()
                            .map(|(index, ed)| {
                                (
                                    (
                                        ed.to,
                                        ed.from_level,
                                        ed.to_level,
                                        // An extra value such that grouping only occurs if the level delta is 1
                                        if (ed.to_level as i32) - (ed.from_level as i32) == 1 {
                                            // (In a correct graph, this cannot be negative, but handy to not throw-errors when visually debugging wrong graphs)
                                            0
                                        } else {
                                            index
                                        },
                                    ),
                                    ed,
                                )
                            })
                            .sorted_by_key(|(g, _ed)| *g)
                            .group_by(|(g, _ed)| *g)
                            .into_iter()
                            .flat_map(|(_g, edge_datas)| {
                                let edge_datas =
                                    edge_datas.map(|(_g, ed)| ed).sorted().collect_vec();
                                let len = edge_datas.len();
                                edge_datas
                                    .iter()
                                    .enumerate()
                                    .map(|(index, edge_data)| {
                                        (
                                            edge_data.drop_count(),
                                            format_edge(
                                                &edge_data,
                                                if len > 1 {
                                                    ((index as f32 / (len - 1) as f32) - 0.5)
                                                        * 2.0
                                                        * max_curve_offset
                                                } else {
                                                    0.
                                                },
                                                group_id,
                                                &node_positions,
                                                &bottom_node_positions,
                                                &edge_bend_nodes,
                                                &edge_connection_nodes,
                                                node_size,
                                            ),
                                        )
                                    })
                                    .collect_vec()
                            })
                            .collect(),
                    },
                )
            })
            .collect(),
    }
}

fn format_edge<T: DrawTag>(
    edge: &EdgeCountData<T>,
    curve_offset: f32,
    group_id: NodeGroupID,
    node_positions: &HashMap<usize, Point>,
    bottom_node_positions: &HashMap<usize, Point>,
    edge_bend_nodes: &HashMap<(NodeGroupID, EdgeData<T>), Vec<NodeGroupID>>,
    edge_connection_nodes: &HashMap<(NodeGroupID, EdgeData<T>), (NodeGroupID, NodeGroupID)>,
    node_size: f32,
) -> EdgeLayout {
    let EdgeCountData {
        to,
        from_level,
        to_level,
        edge_type: _,
        count: _,
    } = edge;
    let edge_data = edge.drop_count();

    let (start_pos, end_pos) = edge_connection_nodes
        .get(&(group_id, edge_data.clone()))
        .map_or_else(
            || (None, None),
            |(start_id, end_id)| {
                (
                    node_positions.get(&start_id).cloned(),
                    node_positions.get(&end_id).cloned(),
                )
            },
        );

    let start_offset = start_pos
        .and_then(|start_point| {
            bottom_node_positions
                .get(&group_id)
                .map(|base_point| start_point - *base_point)
        })
        .unwrap_or_default();

    let end_offset = end_pos
        .and_then(|end_point| {
            bottom_node_positions
                .get(&to)
                .map(|base_point| end_point - *base_point)
        })
        .unwrap_or_default();

    let edge_center_offset = Point {
        x: 0.,
        y: node_size,
    } * 0.5;

    EdgeLayout {
        start_offset: Transition::plain(start_offset + edge_center_offset),
        end_offset: Transition::plain(end_offset + edge_center_offset),
        points: edge_bend_nodes.get(&(group_id, edge_data)).map_or_else(
            || Vec::new(),
            |nodes| {
                let bend_points = nodes
                    .iter()
                    .map(|dummy_id| *node_positions.get(&dummy_id).unwrap() + edge_center_offset);

                // // We can consider the start/end points when reducing, but this can cause nasty animations when bend points are introduced in the first layers
                // let all_bend_points = (Some(start_pos.unwrap_or_default() + edge_center_offset))
                //     .into_iter()
                //     .chain(bend_points)
                //     .chain(Some(end_pos.unwrap_or_default() + edge_center_offset));
                // let reduced_points = remove_redundant_bendpoints(&all_bend_points.collect());
                // let reduced_bend_points = reduced_points[1..reduced_points.len() - 1];

                let reduced_bend_points = remove_redundant_bendpoints(&bend_points.collect());
                reduced_bend_points
                    .iter()
                    .map(|&point| EdgePoint {
                        point: Transition::plain(point),
                        exists: Transition::plain(1.),
                    })
                    .collect()
            },
        ),
        exists: Transition::plain(1.),
        curve_offset: Transition::plain(curve_offset),
    }
}
