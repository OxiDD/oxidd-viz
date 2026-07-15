use std::collections::HashSet;

use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::HtmlCanvasElement;

use crate::util::watchables::{Field, WatchableSetter, WatchableState};
use crate::{
    make_typed_dyn_watchable, make_typed_field,
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::Mutator,
};

// Define watchable canvas collections
make_typed_dyn_watchable!(CanvasSetWatchable, Vec<HtmlCanvasElement>);
make_typed_field!(
    CanvasSetField,
    CanvasSetWatchable,
    Vec<HtmlCanvasElement>,
    false
);

/// Canvas component.
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct CanvasComp {
    #[builder(default=CanvasSetField::new(Vec::new()))]
    data: CanvasSetField,
    /// The current html canvas instances of this component ont he webpage
    #[getter]
    #[builder(skip=data.read())]
    instances: CanvasSetWatchable,
}
impl CanvasComp {
    pub fn new() -> Self {
        Self::builder().build()
    }
}

#[wasm_bindgen]
impl CanvasComp {
    #[must_use = "Only once the mutator is committed, will the canvas be added"]
    #[wasm_bindgen(js_name = addInstance)]
    pub fn add_instance(&mut self, canvas: HtmlCanvasElement) -> Mutator {
        self.data.set_js(
            self.data
                .get()
                .into_iter()
                .filter(|e| e != &canvas)
                .chain(std::iter::once(canvas.clone()))
                .collect(),
        )
    }
    #[must_use = "Only once the mutator is committed, will the canvas be removed"]
    #[wasm_bindgen(js_name = removeInstance)]
    pub fn remove_instance(&mut self, canvas: HtmlCanvasElement) -> Mutator {
        self.data.set_js(
            self.data
                .get()
                .into_iter()
                .filter(|e| e != &canvas)
                .collect(),
        )
    }
}

impl Into<Component> for CanvasComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::Canvas(self))
    }
}
