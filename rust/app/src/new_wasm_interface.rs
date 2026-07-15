use std::rc::Rc;

use crate::{
    components::{
        button_component::ButtonComp,
        composite_component::{CompositeComp, CompositeItemComp},
        container_component::ContainerComp,
        dyn_component::DynComp,
        fill_component::FillComp,
        label_component::LabelComp,
        modal_component::ModalComp,
        overlay_component::OverlayComp,
        panel_component::PanelComp,
        text_component::TextComp,
        CanvasComp, PanelHandleComp, TooltipComp,
    },
    configuration::configuration_object::AbstractConfigurationObject,
    inputs::{
        binary_input::BinaryInputComp, bool_input::BoolInputComp, f32_input::F32InputComp,
        i32_input::I32InputComp, string_input::StringInputComp, u32_input::U32InputComp,
        variant_input::VariantInputComp, InheritedInputComp,
    },
    types::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceRemainder,
    util::rectangle::Rectangle,
};

use super::traits::{Diagram, DiagramSection, DiagramSectionDrawer};
use itertools::Itertools;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

macro_rules! impl_as_variant {
    ($fn_name:ident, $variant:ident, $ty:ty) => {
        #[wasm_bindgen]
        impl Component {
            pub fn $fn_name(&self) -> Option<$ty> {
                if let ComponentOption::$variant(x) = &self.component {
                    Some(x.clone())
                } else {
                    None
                }
            }
        }
    };
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Component {
    component: ComponentOption,
}
impl Component {
    pub fn new(option: ComponentOption) -> Self {
        Self { component: option }
    }
}

#[derive(Clone)]
pub enum ComponentOption {
    BoolInput(BoolInputComp),
    BinaryInput(BinaryInputComp),
    F32Input(F32InputComp),
    I32Input(I32InputComp),
    U32Input(U32InputComp),
    StringInput(StringInputComp),
    VariantInput(VariantInputComp),
    InheritedInput(InheritedInputComp),
    Button(ButtonComp),
    Container(ContainerComp),
    Fill(FillComp),
    Label(LabelComp),
    Text(TextComp),
    Panel(PanelComp),
    Modal(ModalComp),
    Composite(CompositeComp),
    CompositeItem(CompositeItemComp),
    Dyn(DynComp),
    Overlay(OverlayComp),
    Tooltip(TooltipComp),
    PanelHandle(PanelHandleComp),
    Canvas(CanvasComp),
}

impl_as_variant!(as_bool_input, BoolInput, BoolInputComp);
impl_as_variant!(as_binary_input, BinaryInput, BinaryInputComp);
impl_as_variant!(as_f32_input, F32Input, F32InputComp);
impl_as_variant!(as_i32_input, I32Input, I32InputComp);
impl_as_variant!(as_u32_input, U32Input, U32InputComp);
impl_as_variant!(as_string_input, StringInput, StringInputComp);
impl_as_variant!(as_variant_input, VariantInput, VariantInputComp);
impl_as_variant!(as_inherited_input, InheritedInput, InheritedInputComp);
impl_as_variant!(as_button, Button, ButtonComp);
impl_as_variant!(as_container, Container, ContainerComp);
impl_as_variant!(as_fill, Fill, FillComp);
impl_as_variant!(as_label, Label, LabelComp);
impl_as_variant!(as_text, Text, TextComp);
impl_as_variant!(as_panel, Panel, PanelComp);
impl_as_variant!(as_modal, Modal, ModalComp);
impl_as_variant!(as_composite, Composite, CompositeComp);
impl_as_variant!(as_composite_item, CompositeItem, CompositeItemComp);
impl_as_variant!(as_dyn, Dyn, DynComp);
impl_as_variant!(as_overlay, Overlay, OverlayComp);
impl_as_variant!(as_tooltip, Tooltip, TooltipComp);
impl_as_variant!(as_panel_handle, PanelHandle, PanelHandleComp);
impl_as_variant!(as_canvas, Canvas, CanvasComp);
