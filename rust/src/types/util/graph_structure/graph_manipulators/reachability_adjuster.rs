use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use itertools::Itertools;
use oxidd::{LevelNo, NodeID};

use crate::{
    types::util::{
        graph_structure::{
            graph_manipulators::edge_to_adjuster::EdgeToAdjuster,
            graph_structure::{
                Change, EdgeType, GraphEventsReader, GraphEventsWriter, GraphStructure,
            },
        },
        storage::state_storage::{Serializable, StateStorage},
    },
    util::logging::console,
};

pub struct ReachabilityAdjuster<G: GraphStructure> {
    graph: G,
    remove_edges: HashMap<NodeID, HashSet<EdgeType<G::T>>>,
    remove_level_edges: HashMap<oxidd::LevelNo, HashSet<EdgeType<G::T>>>,
    reachable: HashMap<NodeID, bool>,

    level_nodes: HashMap<oxidd::LevelNo, HashSet<NodeID>>,

    event_writer: GraphEventsWriter,
    graph_events: GraphEventsReader,
}

impl<G: GraphStructure> ReachabilityAdjuster<G> {
    pub fn new(mut graph: G) -> Self {
        let roots = graph.get_roots();
        let mut out = Self {
            graph_events: graph.create_event_reader(),
            reachable: HashMap::new(),
            graph,
            event_writer: GraphEventsWriter::new(),
            remove_edges: HashMap::new(),
            remove_level_edges: HashMap::new(),
            level_nodes: HashMap::new(),
        };
        for root in roots {
            out.update_reachable(root);
        }
        out
    }

    pub fn set_remove_edges(
        &mut self,
        edges: impl Iterator<Item = (NodeID, EdgeType<G::T>)>,
    ) -> () {
        self.process_graph_changes();
        let old_remove_edges = self.remove_edges.clone();
        self.remove_edges = iter_to_map(edges);
        for (changed_edge_node, _edge) in old_remove_edges
            .iter()
            .chain(self.remove_edges.clone().iter())
        {
            self.update_reachable_children(*changed_edge_node);
        }
    }

    pub fn set_remove_level_edges(
        &mut self,
        edges: impl Iterator<Item = (oxidd::LevelNo, EdgeType<G::T>)>,
    ) -> () {
        self.process_graph_changes();
        let old_remove_level_edges = self.remove_level_edges.clone();
        self.remove_level_edges = iter_to_map(edges);
        for (changed_level, _edge) in old_remove_level_edges
            .iter()
            .chain(self.remove_level_edges.clone().iter())
        {
            let Some(nodes) = self.level_nodes.get(changed_level) else {
                continue;
            };
            for node in nodes.clone() {
                self.update_reachable_children(node);
            }
        }
    }

    pub fn get_known_levels(&self) -> HashSet<LevelNo> {
        self.level_nodes.keys().cloned().collect()
    }

    fn process_graph_changes(&mut self) {
        let events = self.graph.consume_events(&self.graph_events);
        for event in events {
            match event {
                Change::ParentDiscover { child } => {
                    self.update_reachable(child);
                    self.event_writer.write(event)
                }
                Change::NodeInsertion { node, source: _ } => {
                    self.update_reachable(node);
                    self.update_node_level(node);
                    self.event_writer.write(event)
                }
                Change::NodeConnectionsChange { node } => {
                    self.update_reachable(node);
                    self.event_writer.write(event)
                }
                Change::NodeRemoval { node } => {
                    self.reachable.remove(&node);
                    self.remove_node_level(node);
                    self.event_writer.write(event)
                }
                Change::LevelChange { node } => {
                    self.update_reachable(node);
                    self.update_reachable_children(node);
                    self.event_writer.write(event)
                }
                _ => self.event_writer.write(event),
            }
        }
    }

    fn update_node_level(&mut self, node: NodeID) {
        let level = self.graph.get_level(node);
        self.level_nodes
            .entry(level)
            .or_insert_with(HashSet::new)
            .insert(node);
    }

    fn remove_node_level(&mut self, node: NodeID) {
        let level = self.graph.get_level(node);
        let set = self.level_nodes.entry(level).or_insert_with(HashSet::new);
        set.remove(&node);
        if set.len() == 0 {
            self.level_nodes.remove(&level);
        }
    }

    /// Computes whether the given node is reachable, and caches the result
    fn get_reachable(&mut self, node: NodeID) -> bool {
        if let Some(reachable) = self.reachable.get(&node).cloned() {
            return reachable;
        }

        let mut reachable = false;
        for (parent_edge, parent) in self.get_known_parents(node) {
            let level = self.graph.get_level(parent);
            if let Some(ref removed_level_edges) = self.remove_level_edges.get(&level).cloned() {
                if removed_level_edges.contains(&parent_edge) {
                    continue; // Edge was removed, doesn't apply reachability
                }
            }

            if let Some(ref removed_edges) = self.remove_edges.get(&parent).cloned() {
                if removed_edges.contains(&parent_edge) {
                    continue; // Edge was removed, doesn't apply reachability
                }
            }

            if self.get_reachable(parent) {
                reachable = true;
                break;
            }
        }
        if !reachable {
            reachable = self.get_roots().contains(&node);
        }

        self.reachable.insert(node, reachable);
        reachable
    }

    fn update_reachable_children(&mut self, node: NodeID) {
        for (_, child) in self.get_children(node) {
            self.update_reachable(child);
        }
    }

    fn update_reachable(&mut self, node: NodeID) {
        let was_reachable = self.reachable.get(&node).cloned();

        // Force recompute reachability
        self.reachable.remove(&node);
        let reachable = self.get_reachable(node);

        // Exit if status has not changed
        if Some(reachable) == was_reachable {
            return;
        }

        // Send event and update children
        self.event_writer.write(Change::NodeLabelChange { node });
        for (_child_edge, child) in self.get_children(node) {
            if self.reachable.contains_key(&child) {
                self.update_reachable(child);
            }
        }
    }
}

#[derive(Clone)]
pub struct ReachabilityLabel<T> {
    pub original_label: T,
    pub reachable: bool,
}

impl<G: GraphStructure> GraphStructure for ReachabilityAdjuster<G> {
    type T = G::T;
    type NL = ReachabilityLabel<G::NL>;
    type LL = G::LL;
    fn get_roots(&self) -> Vec<NodeID> {
        self.graph.get_roots()
    }

    fn get_terminals(&self) -> Vec<NodeID> {
        self.graph.get_terminals()
    }

    fn get_known_parents(&mut self, node: NodeID) -> Vec<(EdgeType<G::T>, NodeID)> {
        self.process_graph_changes();
        let parents = self.graph.get_known_parents(node);
        for (_, parent) in &parents {
            self.get_reachable(*parent);
        }
        parents
    }

    fn get_children(&mut self, node: NodeID) -> Vec<(EdgeType<G::T>, NodeID)> {
        self.process_graph_changes();
        let children = self.graph.get_children(node);
        for (_, child) in &children {
            self.get_reachable(*child);
        }
        children
    }

    fn get_level(&mut self, node: NodeID) -> oxidd::LevelNo {
        self.update_node_level(node);
        self.graph.get_level(node)
    }

    fn get_node_label(&self, node: NodeID) -> ReachabilityLabel<G::NL> {
        ReachabilityLabel {
            original_label: self.graph.get_node_label(node),
            reachable: self.reachable.get(&node).cloned().unwrap_or_default(),
        }
    }

    fn get_level_label(&self, level: oxidd::LevelNo) -> G::LL {
        self.graph.get_level_label(level)
    }

    fn create_event_reader(&mut self) -> GraphEventsReader {
        self.event_writer.create_reader()
    }

    fn consume_events(
        &mut self,
        reader: &GraphEventsReader,
    ) -> Vec<crate::types::util::graph_structure::graph_structure::Change> {
        self.process_graph_changes();
        self.event_writer.read(reader)
    }

    fn local_nodes_to_sources(&self, nodes: Vec<NodeID>) -> Vec<NodeID> {
        self.graph.local_nodes_to_sources(nodes)
    }

    fn source_nodes_to_local(&self, nodes: Vec<NodeID>) -> Vec<NodeID> {
        self.graph.source_nodes_to_local(nodes)
    }
}

fn iter_to_map<K: Clone + Eq + Hash, T: Clone + Eq + Hash>(
    edges: impl Iterator<Item = (K, T)>,
) -> HashMap<K, HashSet<T>> {
    edges.fold(HashMap::new(), |mut acc, (node, edge)| {
        acc.entry(node.clone())
            .or_insert_with(HashSet::new)
            .insert(edge);
        acc
    })
}
fn map_to_iter<K: Clone, V>(edges: HashMap<K, HashSet<V>>) -> impl Iterator<Item = (K, V)> {
    edges
        .into_iter()
        .flat_map(|(node, edges)| edges.into_iter().map(move |edge| (node.clone(), edge)))
}

impl<G: GraphStructure + StateStorage> StateStorage for ReachabilityAdjuster<G>
where
    G::T: Serializable,
{
    fn write(&self, stream: &mut std::io::Cursor<&mut Vec<u8>>) -> std::io::Result<()> {
        self.graph.write(stream)?;

        let remove_edges_vec = map_to_iter(self.remove_edges.clone()).collect::<Vec<_>>();
        let count = remove_edges_vec.len();
        stream.write_u32::<LittleEndian>(count as u32)?;
        for (node_id, edge) in remove_edges_vec {
            stream.write_u32::<LittleEndian>(node_id as u32)?;
            stream.write_i32::<LittleEndian>(edge.index)?;
            edge.tag.serialize(stream)?;
        }

        let remove_level_edges_vec =
            map_to_iter(self.remove_level_edges.clone()).collect::<Vec<_>>();
        let count = remove_level_edges_vec.len();
        stream.write_u32::<LittleEndian>(count as u32)?;
        for (level_id, edge) in remove_level_edges_vec {
            stream.write_u32::<LittleEndian>(level_id as u32)?;
            stream.write_i32::<LittleEndian>(edge.index)?;
            edge.tag.serialize(stream)?;
        }

        let count = self.reachable.len();
        stream.write_u32::<LittleEndian>(count as u32)?;
        for (node_id, reachable) in &self.reachable {
            stream.write_u32::<LittleEndian>(*node_id as u32)?;
            stream.write_u8(*reachable as u8)?;
        }

        let node_levels_vec = map_to_iter(self.level_nodes.clone()).collect::<Vec<_>>();
        let count = node_levels_vec.len();
        stream.write_u32::<LittleEndian>(count as u32)?;
        for (level_id, node) in node_levels_vec {
            stream.write_u32::<LittleEndian>(level_id as u32)?;
            stream.write_u32::<LittleEndian>(node as u32)?;
        }
        Ok(())
    }
    fn read(&mut self, stream: &mut std::io::Cursor<&Vec<u8>>) -> std::io::Result<()> {
        self.graph.read(stream)?;

        let count = stream.read_u32::<LittleEndian>()?;
        let mut remove_edges = HashSet::new();
        for _ in 0..count {
            let node = stream.read_u32::<LittleEndian>()? as usize;
            let index = stream.read_i32::<LittleEndian>()?;
            let tag = G::T::deserialize(stream)?;
            remove_edges.insert((node, EdgeType { tag, index }));
        }
        self.remove_edges = iter_to_map(remove_edges.into_iter());

        let count = stream.read_u32::<LittleEndian>()?;
        let mut remove_level_edges = HashSet::new();
        for _ in 0..count {
            let level = stream.read_u32::<LittleEndian>()?;
            let index = stream.read_i32::<LittleEndian>()?;
            let tag = G::T::deserialize(stream)?;
            remove_level_edges.insert((level, EdgeType { tag, index }));
        }
        self.remove_level_edges = iter_to_map(remove_level_edges.into_iter());

        let mut reachability = HashMap::new();
        let count = stream.read_u32::<LittleEndian>()?;
        for _ in 0..count {
            let node_id = stream.read_u32::<LittleEndian>()? as usize;
            let reachable = stream.read_u8()? != 0;
            reachability.insert(node_id, reachable);
        }
        self.reachable = reachability;

        let count = stream.read_u32::<LittleEndian>()?;
        let mut node_levels = HashSet::new();
        for _ in 0..count {
            let level = stream.read_u32::<LittleEndian>()?;
            let node = stream.read_u32::<LittleEndian>()? as usize;
            node_levels.insert((level, node));
        }
        self.level_nodes = iter_to_map(node_levels.into_iter());

        Ok(())
    }
}
