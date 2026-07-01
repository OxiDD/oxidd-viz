use std::rc::Rc;

use app_macros::{builder_into_comp, wasm_getters, watchable_setters};
use bon::Builder;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    impl_default, impl_default_input_comp, impl_inheritable, impl_input_from, impl_into_comps,
    impl_saveable, impl_setter, impl_watchable,
    inputs::{
        binary_input::binary_input_comp_builder::SetWrapper,
        wrapper::{CompWrapper, ComponentInput, IdentityWrapper},
        DefaultInputComp, GetDynWatchableSetter, InheritLabel, Inheritable, InheritedInput,
        WrapBuilder,
    },
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        BoolWatchable, DynWatchable, DynWatchableSetter, Field, IntoWatchable, Mutator,
        OptionStringWatchable, Watchable, WatchableSetter,
    },
};

#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize)]
pub struct FileData {
    data: Vec<u8>,
    name: String,
}
#[wasm_bindgen]
impl FileData {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, data: Vec<u8>) -> FileData {
        FileData { name, data }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }
}

/// Binary input data
#[derive(Clone)]
#[wasm_bindgen]
pub struct BinaryInput(Field<Option<FileData>>);
impl BinaryInput {
    pub fn new(value: Option<FileData>) -> Self {
        BinaryInput(Field::new(value))
    }
    fn watchable(&self) -> &Field<Option<FileData>> {
        &self.0
    }
    fn setter(&mut self) -> &mut Field<Option<FileData>> {
        &mut self.0
    }
}
impl_watchable!(BinaryInput, Option<FileData>);
impl_setter!(BinaryInput, Option<FileData>);
impl_saveable!(BinaryInput, Option<FileData>);
impl_inheritable!(BinaryInput);
impl_input_from!(BinaryInput, Option<FileData>);
impl_default!(BinaryInput);
impl_default_input_comp!(Option<FileData>, BinaryInput, BinaryInputComp);

/// BinaryInput component
#[wasm_getters]
#[wasm_bindgen]
#[builder_into_comp]
#[watchable_setters]
#[derive(Builder, Clone)]
#[builder(start_fn(name=builder_raw, vis=""))]
pub struct BinaryInputComp {
    /// The data of the component
    #[builder(start_fn, into)]
    data: DynWatchableSetter<Option<FileData>>,
    /// A comma separated list of extensions
    #[getter]
    #[setter(Option<String>)]
    formats: OptionStringWatchable,
    /// Whether the user should be able to remove the file
    #[getter]
    #[setter(bool, false)]
    allow_unset: BoolWatchable,
    /// Whether this input is disabled
    #[getter]
    #[setter(bool, false)]
    disabled: BoolWatchable,
    /// Wraps the output component
    #[builder(overwritable)]
    wrapper: Rc<dyn CompWrapper>,
}
impl<I> WrapBuilder<I> for BinaryInputComp
where
    I: ComponentInput<Input = Option<FileData>>,
{
    type Builder = BinaryInputCompBuilder<SetWrapper>;
    fn builder(wrapper: I) -> Self::Builder {
        Self::builder_raw(wrapper.dyn_input()).wrapper(Rc::new(wrapper))
    }
}
impl BinaryInputComp {
    fn watchable(&self) -> &DynWatchableSetter<Option<FileData>> {
        &self.data
    }
    fn setter(&mut self) -> &mut DynWatchableSetter<Option<FileData>> {
        &mut self.data
    }
}
impl_watchable!(BinaryInputComp, Option<FileData>);
impl_setter!(BinaryInputComp, Option<FileData>);
impl_into_comps!(BinaryInput, BinaryInputComp);

impl Into<Component> for BinaryInputComp {
    fn into(self) -> Component {
        let wrapper = self.wrapper.clone();
        let comp = Component::new(ComponentOption::BinaryInput(self));
        wrapper.wrap(comp)
    }
}
