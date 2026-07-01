use std::rc::Rc;

use app_macros::{builder_into_comp, wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{impl_default, impl_default_input_comp, impl_inheritable, impl_inherited_input_from, impl_input_from, impl_into_comps, impl_saveable, impl_setter, impl_watchable, inputs::{GetDynWatchableSetter, InheritLabel, Inheritable, InheritedInput, WrapBuilder, u32_input::u32_input_comp_builder::SetWrapper, wrapper::{CompWrapper, ComponentInput, IdentityWrapper}}, new_wasm_interface::{Component, ComponentOption}, util::watchables::{
     BoolWatchable, Derived, DynWatchable, DynWatchableSetter, IntoWatchable, Mutator, OptionU32Watchable, U32Field, U32Watchable, Watchable, WatchableSetter, Watching
}};


/// Number input data
#[derive(Clone)]
#[wasm_bindgen]
pub struct U32Input(U32Field);
impl U32Input {
    pub fn new(val: u32) -> Self {
        U32Input(U32Field::new(val).into())
    }
    fn watchable(&self) -> &U32Field {
        &self.0
    }
    fn setter(&mut self) -> &mut U32Field {
        &mut self.0
    }
}
impl_watchable!(U32Input, u32);
impl_setter!(U32Input, u32);
impl_saveable!(U32Input, u32);
impl_inheritable!(U32Input);
impl_input_from!(U32Input, u32);
impl_default!(U32Input);
impl_default_input_comp!(u32, U32Input, U32InputComp);

/// Clamped u32 input
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct U32InputClamped {
    /// The raw input
    #[getter]
    #[builder(start_fn, into)]
    input: U32Input,
    /// The minimum value
    #[getter]
    #[setter(Option<u32>)]
    min: OptionU32Watchable,
    /// The maximum value
    #[getter]
    #[setter(Option<u32>)]
    max: OptionU32Watchable,
    /// The precision to use for the number, e.g. `0.5` would round to the nearest half
    #[getter]
    #[setter(Option<u32>)]
    precision: OptionU32Watchable,
    /// The output clamped value based on the input and constraints
    #[builder(skip=clamp(
        U32Watchable::new(input.clone()), 
        min.clone(), 
        max.clone(), 
        precision.clone()
    ))]
    #[getter]
    clamped: U32Watchable,
}

fn clamp(
    val: U32Watchable,
    min: OptionU32Watchable,
    max: OptionU32Watchable,
    precision: OptionU32Watchable,
) -> U32Watchable {
    let clamped = Derived::new(move |t| {
        let mut val = *val.watch(t);
        if let Some(min) = *min.watch(t) {
            if val < min {
                val = min;
            }
        }
        if let Some(max) = *max.watch(t) {
            if val > max {
                val = max;
            }
        }
        if let Some(precision) = *precision.watch(t) {
            val = (((val as f32) / (precision as f32)).round() as u32) * precision;
        }
        val
    });
    U32Watchable::new(clamped)
}
impl U32InputClamped {
    pub fn clamp(&self, val: U32Watchable) -> U32Watchable {
        let min = self.min.clone();
        let max = self.max.clone();
        let precision = self.precision.clone();
        clamp(val, min, max, precision)
    }
    fn watchable(&self) -> &U32Watchable {
        &self.clamped
    }
    fn setter(&mut self) -> &mut U32Input {
        &mut self.input
    }
}
impl_watchable!(U32InputClamped, u32);
impl_setter!(U32InputClamped, u32);
impl_saveable!(U32InputClamped, u32);
impl_inherited_input_from!(U32InputClamped, u32);
impl_default_input_comp!(u32, U32InputClamped, U32InputComp);

impl Inheritable for InheritedInput<U32InputClamped> {
    fn inherit(&self, self_name: impl IntoWatchable<InheritLabel> + 'static) -> Self {
        let p = self.child_input();
        InheritedInput::new(
            U32InputClamped::builder(U32Input::new(p.get()))
                .min(p.min.clone())
                .max(p.max.clone())
                .precision(p.precision.clone())
                .build(),
            DynWatchable::new(self.clone()),
            self_name,
        )
    }
}
impl<X: Into<u32>> From<X> for U32InputClamped {
    fn from(value: X) -> Self {
        Self::builder(U32Input::from(value)).build()
    }
}

/// u32 input component
#[wasm_getters]
#[wasm_bindgen]
#[builder_into_comp]
#[watchable_setters]
#[derive(Builder, Clone)]
#[builder(start_fn(name=builder_raw, vis=""))]
pub struct U32InputComp {
    /// The data of the component
    #[builder(start_fn, into)]
    data: DynWatchableSetter<u32>,
    /// Whether to supply step buttons
    #[getter]
    #[setter(Option<u32>)]
    step_size: OptionU32Watchable,
    /// Whether to apply rounding when performing a step
    #[getter]
    #[setter(bool, false)]
    step_round: BoolWatchable,    
    /// Whether this input is disabled
    #[getter]
    #[setter(bool, false)]
    disabled: BoolWatchable,
    /// Wraps the output component
    #[builder(overwritable)]
    wrapper: Rc<dyn CompWrapper>,
}
impl<I> WrapBuilder<I> for U32InputComp
where
    I: ComponentInput<Input = u32>,
{
    type Builder = U32InputCompBuilder<SetWrapper>;
    fn builder(wrapper: I) -> Self::Builder {
        Self::builder_raw(wrapper.dyn_input()).wrapper(Rc::new(wrapper))
    }
}
impl U32InputComp {
    fn watchable(&self) -> &DynWatchableSetter<u32> {
        &self.data
    }
    fn setter(&mut self) -> &mut DynWatchableSetter<u32> {
        &mut self.data
    }
}
impl_watchable!(U32InputComp, u32);
impl_setter!(U32InputComp, u32);
impl_into_comps!(U32Input, U32InputComp);
impl_into_comps!(U32InputClamped, U32InputComp);
impl Into<Component> for U32InputComp {
    fn into(self) -> Component {
        let wrapper = self.wrapper.clone();
        let comp = Component::new(ComponentOption::U32Input(self));
        wrapper.wrap(comp)
    }
}