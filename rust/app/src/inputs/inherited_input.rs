use std::rc::Rc;

use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::{DynComp, LabelComp},
    inputs::{
        wrapper::{CompWrapper, ComponentInput},
        DefaultInputComp, Saveable, StringInput, StringInputComp, WrapBuilder,
    },
    make_typed_dyn_watchable,
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        BoolWatchable, Constant, DataState, Derived, DynSignaller, DynWatchable,
        DynWatchableSetter, Field, IntoWatchable, Listener, MutateSetter, Mutator, Observer,
        Watchable, WatchableSetter, WatchableState, Watcher, Watching,
    },
};

#[wasm_getters]
#[wasm_bindgen]
#[derive(Clone)]
pub struct InheritLabel {
    #[getter]
    inherited_label: String,
    #[getter]
    local_label: String,
}
make_typed_dyn_watchable!(InheritLabelWatchable, InheritLabel);
impl<W: IntoWatchable<String> + 'static> IntoWatchable<InheritLabel> for W {
    type Output = Derived<InheritLabel>;
    fn into_watchable(self) -> Self::Output {
        let watchable = self.into_watchable();
        Derived::new(move |t| {
            let text = t.watch(&watchable);
            InheritLabel {
                inherited_label: format!("Inheriting from {text}"),
                local_label: format!("Inherit from {text}"),
            }
        })
    }
}
pub trait Inheritable {
    /// Creates a new input that inherits from this input, and label this
    fn inherit(&self, self_name: impl IntoWatchable<InheritLabel> + Clone + 'static) -> Self;
}

/// Optionally inherited input data
#[watchable_setters]
#[derive(Builder)]
pub struct InheritedInput<F: ComponentInput> {
    /// The data of the component
    #[builder(start_fn, into)]
    local: F,
    /// The inherited value
    #[builder(into)]
    inherited: DynWatchable<F::Input>,
    /// Whether or not we are inheriting now
    #[builder(default=DynWatchableSetter::new(Field::new(true)))]
    inheriting: DynWatchableSetter<bool>,
    /// The name of the source of the inherited value
    #[setter(InheritLabel)]
    inherited_from: InheritLabelWatchable,
    /// The final output
    #[builder(skip=InheritedInput::<F>::inherited(local.clone(), inherited.clone(), inheriting.clone()))]
    output: Derived<F::Input>,
}
impl<F: ComponentInput> Clone for InheritedInput<F> {
    fn clone(&self) -> Self {
        Self {
            inherited: self.inherited.clone(),
            local: self.local.clone(),
            inheriting: self.inheriting.clone(),
            inherited_from: self.inherited_from.clone(),
            output: self.output.clone(),
        }
    }
}
impl<F: ComponentInput> InheritedInput<F>
where
    F::Input: Clone,
{
    pub fn default(local: F) -> Self {
        let default = (*local.input().get()).clone();
        Self::builder(local)
            .inherited(DynWatchable::new(Constant::new(default)))
            .inherited_from(InheritLabel {
                inherited_label: "Set to default".to_string(),
                local_label: "Reset to default".to_string(),
            })
            .build()
    }
}
impl<F: ComponentInput> InheritedInput<F> {
    pub fn new(
        local: F,
        inherited: impl Into<DynWatchable<F::Input>>,
        inherited_from: impl IntoWatchable<InheritLabel> + 'static,
    ) -> Self {
        Self::builder(local)
            .inherited(inherited)
            .inherited_from(inherited_from)
            .build()
    }
    fn inherited(
        local: F,
        inherited: DynWatchable<F::Input>,
        inheriting: DynWatchableSetter<bool>,
    ) -> Derived<F::Input> {
        Derived::new_rc(move |t| {
            if *inheriting.watch(t) {
                inherited.watch(t)
            } else {
                local.input().watch(t)
            }
        })
    }

    pub fn child_input(&self) -> &F {
        &self.local
    }
    pub fn set_inherit(&mut self, inherit: bool) -> DynSignaller {
        self.inheriting.set(inherit)
    }

    /// Creates the component for this inherited input
    pub fn comp<I: Into<Component>, M: FnOnce(Self, &F) -> I>(self, map: M) -> InheritedInputComp {
        InheritedInputComp::new(self.clone(), |val| map(self, val))
    }
}

// Watchable setter behavior
impl<F: ComponentInput> WatchableState for InheritedInput<F> {
    fn state(&self) -> DataState {
        self.output.state()
    }
    fn observe(&self, listener: Box<dyn Listener>) -> Observer {
        self.output.observe(listener)
    }
}
impl<F: ComponentInput> Watchable for InheritedInput<F> {
    type Output = F::Input;
    fn get(&self) -> Rc<Self::Output> {
        self.output.get()
    }
}
impl<F: ComponentInput> WatchableSetter for InheritedInput<F> {
    fn set(&mut self, val: F::Input) -> DynSignaller {
        Box::new((
            self.local.input().clone().set(val),
            self.inheriting.set(false),
        ))
    }
}

// Watchable setter conversion traits
impl<F: ComponentInput> Into<DynWatchableSetter<F::Input>> for InheritedInput<F> {
    fn into(self) -> DynWatchableSetter<F::Input> {
        DynWatchableSetter::new(self)
    }
}

// Component input traits
impl<F: ComponentInput> CompWrapper for InheritedInput<F> {
    fn wrap(&self, comp: Component) -> Component {
        InheritedInputComp::new(self.clone(), |_| comp).into()
    }
}
impl<F: ComponentInput> ComponentInput for InheritedInput<F> {
    type Input = F::Input;
    type Setter = InheritedInput<F>;
    fn input(&self) -> &Self {
        self
    }
}
impl<F> DefaultInputComp for InheritedInput<F>
where
    F: ComponentInput + DefaultInputComp,
    F::Comp: WrapBuilder<Self>,
{
    type Comp = F::Comp;
}
#[derive(Serialize, Deserialize)]
pub struct InheritedData<V> {
    value: V,
    inheriting: bool,
}
impl<F> Saveable for InheritedInput<F>
where
    F: ComponentInput + Saveable,
    F::Val: Serialize + DeserializeOwned,
{
    type Val = InheritedData<F::Val>;
    fn load_value(&mut self, val: InheritedData<F::Val>) -> DynSignaller {
        Box::new((
            self.inheriting.set(val.inheriting),
            self.local.load_value(val.value),
        ))
    }

    fn save_value(&self) -> Self::Val {
        InheritedData {
            value: self.local.save_value(),
            inheriting: *self.inheriting.get(),
        }
    }
}

// Wrapper traits
impl<F: ComponentInput> From<F> for InheritedInput<F>
where
    F::Input: Clone,
{
    fn from(value: F) -> Self {
        InheritedInput::default(value)
    }
}
impl<F: ComponentInput + Default> Default for InheritedInput<F>
where
    F::Input: Clone,
{
    fn default() -> Self {
        InheritedInput::default(F::default())
    }
}

/// Inherited input component.
#[wasm_getters]
#[wasm_bindgen]
#[derive(Clone)]
pub struct InheritedInputComp {
    /// The input component.
    #[getter]
    input: DynComp,
    /// Whether the value is currently being inherited.
    #[getter]
    inheriting: BoolWatchable,
    /// The text indicating where the inherited value comes from.
    #[getter]
    inherited_from: InheritLabelWatchable,
    /// The field that stores whether inheriting
    inheriting_field: DynWatchableSetter<bool>,
}
impl InheritedInputComp {
    pub fn new<F: ComponentInput, I: Into<Component>, M: FnOnce(&F) -> I>(
        data: InheritedInput<F>,
        map: M,
    ) -> Self {
        Self {
            input: DynComp::new(map(&data.local).into()),
            inheriting_field: data.inheriting.clone(),
            inheriting: BoolWatchable::new(data.inheriting.clone()),
            inherited_from: data.inherited_from.clone(),
        }
    }
}

#[wasm_bindgen]
impl InheritedInputComp {
    /// Starts inheriting the value
    pub fn set_inherit(&mut self, inheriting: bool) -> Mutator {
        self.inheriting_field.mutate_set(inheriting)
    }
}

impl<F> Into<Component> for InheritedInput<F>
where
    F: ComponentInput + DefaultInputComp,
    F::Comp: WrapBuilder<Self>,
    <F::Comp as WrapBuilder<Self>>::Builder: Into<Component>,
{
    fn into(self) -> Component {
        F::Comp::builder(self).into()
    }
}

// impl<T: WatchableSetter, I: WrapBuilder<T>, F: WatchableSetter<Output = I> + Clone + 'static>
//     WrapBuilder<T> for InheritedInput<F>
// {
//     type Builder = I::Builder;
//     fn builder(wrapper: impl ComponentInput<T>) -> Self::Builder {
//         I::builder(wrapper)
//     }
// }

// impl<I, F: WatchableSetter<Output = I> + Clone + DefaultInputComp + 'static> Into<Component>
//     for InheritedInput<F>
// where
//     F::Comp: WrapBuilder<InheritedInput<F>>,
//     <F::Comp as WrapBuilder<InheritedInput<F>>>::Builder: Into<Component>,
// {
//     fn into(self) -> Component {
//         F::Comp::wrap_builder(self).into()
//     }
// }

impl Into<Component> for InheritedInputComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::InheritedInput(self))
    }
}
