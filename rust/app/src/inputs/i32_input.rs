use std::rc::Rc;

use app_macros::{builder_into_comp, wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;


use crate::{
    impl_default, impl_default_input_comp, impl_inheritable, impl_inherited_input_from, impl_input_from, impl_into_comps, impl_saveable, impl_setter, impl_watchable, inputs::{ GetDynWatchableSetter, InheritLabel, Inheritable, InheritedInput, WrapBuilder, i32_input::i32_input_comp_builder::SetWrapper, wrapper::{CompWrapper, ComponentInput, IdentityWrapper}}, new_wasm_interface::{Component, ComponentOption}, util::watchables::{
        BoolWatchable, Derived, DynWatchable, DynWatchableSetter, I32Field, I32Watchable, IntoWatchable, Mutator, OptionI32Watchable, Watchable, WatchableSetter, Watching
    }
};


/// Number input data
#[derive(Clone)]
#[wasm_bindgen]
pub struct I32Input(I32Field);
impl I32Input {
    pub fn new(val: i32) -> Self {
        I32Input(I32Field::new(val).into())
    }
    fn watchable(&self) -> &I32Field {
        &self.0
    }
    fn setter(&mut self) -> &mut I32Field {
        &mut self.0
    }
}
impl_watchable!(I32Input, i32);
impl_setter!(I32Input, i32);
impl_saveable!(I32Input, i32);
impl_inheritable!(I32Input);
impl_input_from!(I32Input, i32);
impl_default!(I32Input);
impl_default_input_comp!(i32, I32Input, I32InputComp);

/// Clamped i32 input
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct I32InputClamped {
    /// The raw input
    #[builder(start_fn, into)]
    input: I32Input,
    /// The minimum value
    #[getter]
    #[setter(Option<i32>)]
    min: OptionI32Watchable,
    /// The maximum value
    #[getter]
    #[setter(Option<i32>)]
    max: OptionI32Watchable,
    /// The precision to use for the number, e.g. `0.5` would round to the nearest half
    #[getter]
    #[setter(Option<i32>)]
    precision: OptionI32Watchable,
    /// The output clamped value based on the input and constraints
    #[builder(skip=clamp(
        I32Watchable::new(input.clone()), 
        min.clone(), 
        max.clone(), 
        precision.clone()
    ))]
    #[getter]
    clamped: I32Watchable,
}

fn clamp(
    val: I32Watchable,
    min: OptionI32Watchable,
    max: OptionI32Watchable,
    precision: OptionI32Watchable,
) -> I32Watchable {
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
            val = (((val as f32) / (precision as f32)).round() as i32) * precision;
        }
        val
    });
    I32Watchable::new(clamped)
}
impl I32InputClamped {
    pub fn clamp(&self, val: I32Watchable) -> I32Watchable {
        let min = self.min.clone();
        let max = self.max.clone();
        let precision = self.precision.clone();
        clamp(val, min, max, precision)
    }
    fn watchable(&self) -> &I32Watchable {
        &self.clamped
    }
    fn setter(&mut self) -> &mut I32Input {
        &mut self.input
    }
}
impl_watchable!(I32InputClamped, i32);
impl_setter!(I32InputClamped, i32);
impl_saveable!(I32InputClamped, i32);
impl_inherited_input_from!(I32InputClamped, i32);
impl_default_input_comp!(i32, I32InputClamped, I32InputComp);

impl Inheritable for InheritedInput<I32InputClamped> {
    fn inherit(&self, self_name: impl IntoWatchable<InheritLabel> + 'static) -> Self {
        let p = self.child_input();
        InheritedInput::new(
            I32InputClamped::builder(I32Input::new(p.get()))
                .min(p.min.clone())
                .max(p.max.clone())
                .precision(p.precision.clone())
                .build(),
            DynWatchable::new(self.clone()),
            self_name,
        )
    }
}
impl<X: Into<i32>> From<X> for I32InputClamped {
    fn from(value: X) -> Self {
        Self::builder(I32Input::from(value)).build()
    }
}

/// i32 input component
#[wasm_getters]
#[wasm_bindgen]
#[builder_into_comp]
#[watchable_setters]
#[derive(Builder, Clone)]
#[builder(start_fn(name=builder_raw, vis=""))]
pub struct I32InputComp {
    /// The data of the component
    #[builder(start_fn, into)]
    data: DynWatchableSetter<i32>,
    /// Whether to supply step buttons
    #[getter]
    #[setter(Option<i32>)]
    step_size: OptionI32Watchable,
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
impl<I> WrapBuilder<I> for I32InputComp
where
    I: ComponentInput<Input = i32>,
{
    type Builder = I32InputCompBuilder<SetWrapper>;
    fn builder(wrapper: I) -> Self::Builder {
        Self::builder_raw(wrapper.dyn_input()).wrapper(Rc::new(wrapper))
    }
}
impl I32InputComp {
    fn watchable(&self) -> &DynWatchableSetter<i32> {
        &self.data
    }
    fn setter(&mut self) -> &mut DynWatchableSetter<i32> {
        &mut self.data
    }
}
impl_watchable!(I32InputComp, i32);
impl_setter!(I32InputComp, i32);
impl_into_comps!(I32Input, I32InputComp);
impl_into_comps!(I32InputClamped, I32InputComp);
impl Into<Component> for I32InputComp {
    fn into(self) -> Component {
        let wrapper = self.wrapper.clone();
        let comp = Component::new(ComponentOption::I32Input(self));
        wrapper.wrap(comp)
    }
}