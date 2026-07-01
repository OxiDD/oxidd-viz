use std::rc::Rc;

use app_macros::{builder_into_comp, wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    impl_default, impl_default_input_comp, impl_inheritable, impl_input_from, impl_into_comps,
    impl_saveable, impl_setter, impl_watchable,
    inputs::{
        bool_input::bool_input_comp_builder::SetWrapper,
        wrapper::{CompWrapper, ComponentInput},
        DefaultInputComp, GetDynWatchableSetter, InheritLabel, Inheritable, InheritedInput,
        WrapBuilder,
    },
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        BoolField, BoolWatchable, DynSignaller, DynWatchable, DynWatchableSetter, IntoWatchable,
        Mutator, Watchable, WatchableSetter,
    },
};

/// Boolean input data
#[derive(Clone)]
#[wasm_bindgen]
pub struct BoolInput(BoolField);
impl BoolInput {
    pub fn new(val: bool) -> Self {
        BoolInput(BoolField::new(val))
    }
    fn watchable(&self) -> &BoolField {
        &self.0
    }
    fn setter(&mut self) -> &mut BoolField {
        &mut self.0
    }
    pub fn toggle(&mut self) -> DynSignaller {
        self.0.set(!self.0.get())
    }
}
impl_watchable!(BoolInput, bool);
impl_setter!(BoolInput, bool);
impl_saveable!(BoolInput, bool);
impl_inheritable!(BoolInput);
impl_input_from!(BoolInput, bool);
impl_default!(BoolInput);
impl_default_input_comp!(bool, BoolInput, BoolInputComp);

/// BoolInput component
#[wasm_getters]
#[wasm_bindgen]
#[builder_into_comp]
#[watchable_setters]
#[derive(Builder, Clone)]
#[builder(start_fn(name=builder_raw, vis=""))]
pub struct BoolInputComp {
    /// The data of the component
    #[builder(start_fn, into)]
    data: DynWatchableSetter<bool>,
    /// Whether this input is disabled
    #[getter]
    #[setter(bool, false)]
    disabled: BoolWatchable,
    /// Wraps the output component
    #[builder(overwritable)]
    wrapper: Rc<dyn CompWrapper>,
}
impl<I> WrapBuilder<I> for BoolInputComp
where
    I: ComponentInput<Input = bool>,
{
    type Builder = BoolInputCompBuilder<SetWrapper>;
    fn builder(wrapper: I) -> Self::Builder {
        Self::builder_raw(wrapper.dyn_input()).wrapper(Rc::new(wrapper))
    }
}
impl BoolInputComp {
    fn watchable(&self) -> &DynWatchableSetter<bool> {
        &self.data
    }
    fn setter(&mut self) -> &mut DynWatchableSetter<bool> {
        &mut self.data
    }
}
impl_watchable!(BoolInputComp, bool);
impl_setter!(BoolInputComp, bool);
impl_into_comps!(BoolInput, BoolInputComp);

impl Into<Component> for BoolInputComp {
    fn into(self) -> Component {
        let wrapper = self.wrapper.clone();
        let comp = Component::new(ComponentOption::BoolInput(self));
        wrapper.wrap(comp)
    }
}
