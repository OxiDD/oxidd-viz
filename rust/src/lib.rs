mod configuration;
mod traits;
mod types;
mod util;
mod wasm_interface;

use std::collections::BTreeMap;

use itertools::Itertools;
// use js_sys::Uint32Array;
use oxidd::{bdd::BDDManagerRef, ManagerRef};
use util::{logging::console, panic_hook::set_panic_hook};
// use utils::*;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, Window};

use configuration::configuration_object::ConfigurationObject;
use oxidd::{bdd::BDDFunction, util::AllocResult, BooleanFunction};
use types::{mtbdd::mtbdd_drawer::MTBDDDiagram, qdd::qdd_drawer::QDDDiagram};

use swash::{
    proxy::{CharmapProxy, MetricsProxy},
    scale::{ScaleContext, ScalerBuilder, StrikeWith},
    shape::{ShapeContext, ShaperBuilder},
    Action, CacheKey, Charmap, FontRef,
};

use crate::{
    types::bdd_min::bdd_min_drawer::BDDMinDiagram,
    util::dummy_bdd::{DummyBDDFunction, DummyBDDManager, DummyBDDManagerRef},
    wasm_interface::DiagramBox,
};

#[wasm_bindgen]
pub fn create_qdd_diagram() -> Option<DiagramBox> // And some DD type param
{
    set_panic_hook();
    Some(DiagramBox::new(Box::new(QDDDiagram::new())))
}

#[wasm_bindgen]
pub fn create_bdd_min_diagram() -> Option<DiagramBox> // And some DD type param
{
    set_panic_hook();
    Some(DiagramBox::new(Box::new(BDDMinDiagram::new())))
}

#[wasm_bindgen]
pub fn create_mtbdd_diagram() -> Option<DiagramBox> // And some DD type param
{
    set_panic_hook();
    Some(DiagramBox::new(Box::new(MTBDDDiagram::new())))
}
