use std::rc::Rc;

use crate::{
    configuration::configuration_object::AbstractConfigurationObject,
    types::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceRemainder,
    util::rectangle::Rectangle,
};

use super::traits::{Diagram, DiagramSection, DiagramSectionDrawer};
use itertools::Itertools;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen]
pub struct DiagramBox(Box<dyn Diagram>);

impl DiagramBox {
    pub fn new(diagram: Box<dyn Diagram>) -> DiagramBox {
        DiagramBox(diagram)
    }
}
// Mirror Diagram trait in terms of interface, but using non-dynamic structs
#[wasm_bindgen()]
impl DiagramBox {
    pub fn create_section_from_dddmp(&mut self, dddmp: String) -> Option<DiagramSectionBox> {
        Some(DiagramSectionBox(self.0.create_section_from_dddmp(dddmp)?))
    }
    pub fn create_section_from_other(
        &mut self,
        data: String,
        vars: Option<String>,
    ) -> Option<DiagramSectionBox> {
        Some(DiagramSectionBox(
            self.0.create_section_from_other(data, vars)?,
        ))
    }
    pub fn create_section_from_ids(
        &self,
        ids: &[NodeID],
        section: &DiagramSectionBox,
    ) -> Option<DiagramSectionBox> {
        Some(DiagramSectionBox(self.0.create_section_from_ids(
            &ids.iter().map(|&id| (id, &section.0)).collect_vec(),
        )?))
    }
}

#[wasm_bindgen]
pub struct DiagramSectionBox(Box<dyn DiagramSection>);

impl DiagramSectionBox {
    pub fn new(diagram: Box<dyn DiagramSection>) -> DiagramSectionBox {
        DiagramSectionBox(diagram)
    }
}
// Mirror Diagram trait in terms of interface, but using non-dynamic structs
#[wasm_bindgen]
impl DiagramSectionBox {
    pub fn create_drawer(&self, canvas: HtmlCanvasElement) -> DiagramSectionDrawerBox {
        DiagramSectionDrawerBox(self.0.create_drawer(canvas))
    }
}
#[wasm_bindgen]
pub struct DiagramSectionDrawerBox(Box<dyn DiagramSectionDrawer>);
#[wasm_bindgen]
impl DiagramSectionDrawerBox {
    pub fn render(&mut self, time: u32) -> () {
        self.0.render(time);
    }
    pub fn layout(&mut self, time: u32) -> () {
        self.0.layout(time);
    }
    pub fn set_transform(&mut self, width: u32, height: u32, x: f32, y: f32, scale: f32) -> () {
        self.0.set_transform(width, height, x, y, scale);
    }
    pub fn get_bounding_box(&self) -> BoundingBox {
        self.0.get_bounding_box()
    }
    pub fn set_step(&mut self, step: i32) -> Option<StepData> {
        self.0.set_step(step)
    }

    /** Grouping */
    pub fn set_group(&mut self, from: Vec<TargetID>, to: NodeGroupID) -> bool {
        self.0.set_group(from, to)
    }
    pub fn create_group(&mut self, from: Vec<TargetID>) -> NodeGroupID {
        self.0.create_group(from)
    }

    /** Tools */
    pub fn split_edges(&mut self, nodes: &[NodeID], fully: bool) {
        self.0.split_edges(nodes, fully);
    }

    /** Node interaction */
    /// Coordinates in screen space (-0.5 to 0.5), not in world space. Additionally the max_group_expansion should be provided for determining the maximum number of nodes to select for every given group
    pub fn get_nodes(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        max_group_expansion: usize,
    ) -> Vec<NodeGroupID> {
        self.0
            .get_nodes(Rectangle::new(x, y, width, height), max_group_expansion)
    }
    pub fn set_selected_nodes(&mut self, selected_ids: &[NodeID], hovered_ids: &[NodeID]) {
        self.0.set_selected_nodes(selected_ids, hovered_ids);
    }
    /// Retrieves the sources (nodes of the source diagram) of the modified diagram
    pub fn local_nodes_to_sources(&self, nodes: &[NodeID]) -> Vec<NodeID> {
        self.0.local_nodes_to_sources(nodes)
    }
    /// Retrieves the local nodes representing the collection of sources
    pub fn source_nodes_to_local(&self, nodes: &[NodeID]) -> Vec<NodeID> {
        self.0.source_nodes_to_local(nodes)
    }

    /** Storage */
    pub fn serialize_state(&self) -> Vec<u8> {
        self.0.serialize_state()
    }
    pub fn deserialize_state(&mut self, state: Vec<u8>) {
        self.0.deserialize_state(state)
    }

    /** Settings */
    pub fn get_configuration(&self) -> AbstractConfigurationObject {
        self.0.get_configuration()
    }
}

#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct BoundingBox {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct TerminalData {
    pub id: String,
    pub name: String,
}

#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct StepData {
    pub description: String,
    pub group: StepGroup,
}

#[derive(Clone)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct StepGroup {
    pub start: i32, // Inclusive
    pub end: i32,   // Exclusive
    pub description: String,
    parent: Option<Rc<StepGroup>>,
}

#[wasm_bindgen]
impl StepGroup {
    pub fn get_parent(&self) -> Option<StepGroup> {
        match &self.parent {
            Some(x) => Some((**x).clone()),
            None => None,
        }
    }
}

// Argumentless structure to be compatible with JS enums
#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq)]
pub enum TargetIDType {
    NodeID,
    NodeGroupID,
}

#[derive(PartialEq)]
#[wasm_bindgen]
pub struct TargetID(pub TargetIDType, pub NodeID);
#[wasm_bindgen]
impl TargetID {
    pub fn new(id_type: TargetIDType, id: NodeID) -> TargetID {
        TargetID(id_type, id)
    }
}

pub type NodeGroupID = usize;
pub type NodeID = usize;
