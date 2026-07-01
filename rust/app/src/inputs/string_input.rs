use std::rc::Rc;

use app_macros::{builder_into_comp, wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    impl_default, impl_default_input_comp, impl_inheritable, impl_input_from, impl_into_comps,
    impl_saveable, impl_setter, impl_watchable,
    inputs::{
        string_input::string_input_comp_builder::SetWrapper,
        wrapper::{CompWrapper, ComponentInput, IdentityWrapper},
        DefaultInputComp, GetDynWatchableSetter, InheritLabel, Inheritable, InheritedInput,
        Saveable, WrapBuilder,
    },
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        BoolWatchable, DynWatchable, DynWatchableSetter, IntoWatchable, Mutator,
        OptionU32Watchable, StringField, Watchable, WatchableSetter,
    },
};

/// String input data
#[derive(Clone)]
#[wasm_bindgen]
pub struct StringInput(StringField);
impl StringInput {
    pub fn new(val: String) -> Self {
        StringInput(StringField::new(val).into())
    }
    fn watchable(&self) -> &StringField {
        &self.0
    }
    fn setter(&mut self) -> &mut StringField {
        &mut self.0
    }
}
impl_watchable!(StringInput, String);
impl_setter!(StringInput, String);
impl_saveable!(StringInput, String);
impl_inheritable!(StringInput);
impl_input_from!(StringInput, String);
impl_default!(StringInput);
impl_default_input_comp!(String, StringInput, StringInputComp);

/// StringInput component
#[wasm_getters]
#[wasm_bindgen]
#[builder_into_comp]
#[watchable_setters]
#[derive(Builder, Clone)]
#[builder(start_fn(name=builder_raw, vis=""))]
pub struct StringInputComp {
    /// The data of the component
    #[builder(start_fn, into)]
    data: DynWatchableSetter<String>,
    /// Whether to allow multiline inputs
    #[getter]
    #[setter(bool, false)]
    multiline: BoolWatchable,
    /// The minimum number of lines to show
    #[getter]
    #[setter(Option<u32>)]
    multiline_min: OptionU32Watchable,
    /// The maximum number of lines to show
    #[getter]
    #[setter(Option<u32>)]
    multiline_max: OptionU32Watchable,
    /// Whether to dynamically adjust the shown number of lines based on content
    #[getter]
    #[setter(bool, false)]
    multiline_dynamic: BoolWatchable,
    /// Whether the user may resize the text field
    #[getter]
    #[setter(bool, false)]
    multiline_resizable: BoolWatchable,
    /// Whether the data should only be changed when the field is blurred, or enter/ctrl-enter is pressed
    #[getter]
    #[setter(bool, false)]
    late_submit: BoolWatchable,
    /// Whether this input is readonly
    #[getter]
    #[setter(bool, false)]
    readonly: BoolWatchable,
    /// Whether this input is disabled
    #[getter]
    #[setter(bool, false)]
    disabled: BoolWatchable,
    /// Wraps the output component
    #[builder(overwritable)]
    wrapper: Rc<dyn CompWrapper>,
}
impl<I> WrapBuilder<I> for StringInputComp
where
    I: ComponentInput<Input = String>,
{
    type Builder = StringInputCompBuilder<SetWrapper>;
    fn builder(wrapper: I) -> Self::Builder {
        Self::builder_raw(wrapper.dyn_input()).wrapper(Rc::new(wrapper))
    }
}
impl StringInputComp {
    fn watchable(&self) -> &DynWatchableSetter<String> {
        &self.data
    }
    fn setter(&mut self) -> &mut DynWatchableSetter<String> {
        &mut self.data
    }
}
impl_watchable!(StringInputComp, String);
impl_setter!(StringInputComp, String);
impl_into_comps!(StringInput, StringInputComp);
impl Into<Component> for StringInputComp {
    fn into(self) -> Component {
        self.wrapper
            .clone()
            .wrap(Component::new(ComponentOption::StringInput(self)))
    }
}
