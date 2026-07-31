use crate::{
    configuration::configuration_object::AbstractConfigurationObject,
    types::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceRemainder,
    util::rectangle::Rectangle,
    wasm_interface::{BoundingBox, NodeID},
};

use super::wasm_interface::{NodeGroupID, StepData, TargetID};
use web_sys::HtmlCanvasElement;

pub trait Diagram {
    fn create_section_from_dddmp(&mut self, dddmp: String) -> Option<Box<dyn DiagramSection>>; // TODO: error type
    fn create_section_from_other(
        &mut self,
        data: String,
        vars: Option<String>,
    ) -> Option<Box<dyn DiagramSection>>; // TODO: error type
    fn create_section_from_ids(
        &self,
        id: &[(oxidd::NodeID, &Box<dyn DiagramSection>)],
    ) -> Option<Box<dyn DiagramSection>>;
}

pub trait DiagramSection {
    fn create_drawer(&self, canvas: HtmlCanvasElement) -> Box<dyn DiagramSectionDrawer>;
    fn get_level_labels(&self) -> Vec<String>;
    fn get_node_labels(&self, node: NodeID) -> Vec<String>;
    fn get_meta(&self) -> i128;
}

pub trait DiagramSectionDrawer {
    fn render(&mut self, time: u32) -> ();
    fn layout(&mut self, time: u32) -> ();
    fn set_transform(&mut self, width: u32, height: u32, x: f32, y: f32, scale: f32) -> ();
    fn get_bounding_box(&self) -> BoundingBox;
    fn set_step(&mut self, step: i32) -> Option<StepData>;

    /* Grouping */
    fn set_group(&mut self, from: Vec<TargetID>, to: NodeGroupID) -> bool;
    fn create_group(&mut self, from: Vec<TargetID>) -> NodeGroupID;

    /** Tools */
    /// Splits the edges of a given group such that each edge type goes to a unique group, if fully is specified it also ensures that each group that an edge goes to only contains a single node
    fn split_edges(&mut self, nodes: &[NodeID], fully: bool) -> ();

    /** Node interaction */
    /// Retrieves the nodes in the given rectangle, expanding each node group up to at most max_group_expansion nodes of the nodes it contains
    fn get_nodes(&self, area: Rectangle, max_group_expansion: usize) -> Vec<NodeID>;
    /// The selected and hover _ids are node ids, not node group ids
    fn set_selected_nodes(&mut self, selected_ids: &[NodeID], hovered_ids: &[NodeID]);
    /// Retrieves the sources (nodes of the source diagram) of the modified diagram
    fn local_nodes_to_sources(&self, nodes: &[NodeID]) -> Vec<NodeID>;
    /// Retrieves the local nodes representing the collection of sources
    fn source_nodes_to_local(&self, nodes: &[NodeID]) -> Vec<NodeID>;

    /** Storage */
    fn serialize_state(&self) -> Vec<u8>;
    fn deserialize_state(&mut self, state: Vec<u8>) -> ();

    /** Settings */
    fn get_configuration(&self) -> AbstractConfigurationObject;
}
