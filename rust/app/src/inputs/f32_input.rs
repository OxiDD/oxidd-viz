use std::rc::Rc;

use app_macros::{builder_into_comp, wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{impl_default, impl_default_input_comp, impl_inheritable, impl_inherited_input_from, impl_input_from, impl_into_comps, impl_saveable, impl_setter, impl_watchable, inputs::{ GetDynWatchableSetter, InheritLabel, Inheritable, InheritedInput, WrapBuilder, f32_input::f32_input_comp_builder::SetWrapper, wrapper::{CompWrapper, ComponentInput, IdentityWrapper}}, new_wasm_interface::{Component, ComponentOption}, util::watchables::{
    BoolWatchable, Derived, DynWatchable, DynWatchableSetter, F32Field, F32Watchable, IntoWatchable, Mutator, OptionF32Watchable, Watchable, WatchableSetter, Watching
}};

/// Number input data
#[derive(Clone)]
#[wasm_bindgen]
pub struct F32Input(F32Field);
impl F32Input {
    pub fn new(val: f32) -> Self {
        F32Input(F32Field::new(val).into())
    }
    fn watchable(&self) -> &F32Field {
        &self.0
    }
    fn setter(&mut self) -> &mut F32Field {
        &mut self.0
    }
}
impl_watchable!(F32Input, f32);
impl_setter!(F32Input, f32);
impl_saveable!(F32Input, f32);
impl_inheritable!(F32Input);
impl_input_from!(F32Input, f32);
impl_default!(F32Input);
impl_default_input_comp!(f32, F32Input, F32InputComp);

/// Clamped f32 input
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct F32InputClamped {
    /// The raw input
    #[getter]
    #[builder(start_fn, into)]
    input: F32Input,
    /// The minimum value
    #[getter]
    #[setter(Option<f32>)]
    min: OptionF32Watchable,
    /// The maximum value
    #[getter]
    #[setter(Option<f32>)]
    max: OptionF32Watchable,
    /// The precision to use for the number, e.g. `0.5` would round to the nearest half
    #[getter]
    #[setter(Option<f32>)]
    precision: OptionF32Watchable,
    /// The output clamped value based on the input and constraints
    #[builder(skip=clamp(
        F32Watchable::new(input.clone()), 
        min.clone(), 
        max.clone(), 
        precision.clone()
    ))]
    #[getter]
    clamped: F32Watchable,
}

fn clamp(
    val: F32Watchable,
    min: OptionF32Watchable,
    max: OptionF32Watchable,
    precision: OptionF32Watchable,
) -> F32Watchable {
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
            val = (val / precision).round() * precision;
        }
        val
    });
    F32Watchable::new(clamped)
}
impl F32InputClamped {
    pub fn clamp(&self, val: F32Watchable) -> F32Watchable {
        let min = self.min.clone();
        let max = self.max.clone();
        let precision = self.precision.clone();
        clamp(val, min, max, precision)
    }
    fn watchable(&self) -> &F32Watchable {
        &self.clamped
    }
    fn setter(&mut self) -> &mut F32Input {
        &mut self.input
    }
}
impl_watchable!(F32InputClamped, f32);
impl_setter!(F32InputClamped, f32);
impl_saveable!(F32InputClamped, f32);
impl_inherited_input_from!(F32InputClamped, f32);
impl_default_input_comp!(f32, F32InputClamped, F32InputComp);

impl Inheritable for InheritedInput<F32InputClamped> {
    fn inherit(&self, self_name: impl IntoWatchable<InheritLabel> + 'static) -> Self {
        let p = self.child_input();
        InheritedInput::new(
            F32InputClamped::builder(F32Input::new(p.get()))
                .min(p.min.clone())
                .max(p.max.clone())
                .precision(p.precision.clone())
                .build(),
            DynWatchable::new(self.clone()),
            self_name,
        )
    }
}
impl<X: Into<f32>> From<X> for F32InputClamped {
    fn from(value: X) -> Self {
        Self::builder(F32Input::from(value)).build()
    }
}

/// f32 input component
#[wasm_getters]
#[wasm_bindgen]
#[builder_into_comp]
#[watchable_setters]
#[derive(Builder, Clone)]
#[builder(start_fn(name=builder_raw, vis=""))]
pub struct F32InputComp {
    /// The data of the component
    #[builder(start_fn, into)]
    data: DynWatchableSetter<f32>,
    /// Whether to supply step buttons
    #[getter]
    #[setter(Option<f32>)]
    step_size: OptionF32Watchable,
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
impl<I> WrapBuilder<I> for F32InputComp
where
    I: ComponentInput<Input = f32>,
{
    type Builder = F32InputCompBuilder<SetWrapper>;
    fn builder(wrapper: I) -> Self::Builder {
        Self::builder_raw(wrapper.dyn_input()).wrapper(Rc::new(wrapper))
    }
}
impl F32InputComp {
    fn watchable(&self) -> &DynWatchableSetter<f32> {
        &self.data
    }
    fn setter(&mut self) -> &mut DynWatchableSetter<f32> {
        &mut self.data
    }
}
impl_watchable!(F32InputComp, f32);
impl_setter!(F32InputComp, f32);
impl_into_comps!(F32Input, F32InputComp);
impl_into_comps!(F32InputClamped, F32InputComp);

impl Into<Component> for F32InputComp {
    fn into(self) -> Component {
        let wrapper = self.wrapper.clone();
        let comp = Component::new(ComponentOption::F32Input(self));
        wrapper.wrap(comp)
    }
}
