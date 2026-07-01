use std::{cell::RefCell, ops::Index, rc::Rc};

use app_macros::{builder_into_comp, wasm_getters, watchable_setters};
use bon::Builder;
use itertools::Itertools;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::{
        composite_component::ComponentVecWatchable, Align, AlignMain, AlignMainWatchable,
        AlignWatchable,
    },
    impl_setter, impl_watchable,
    inputs::{
        variant_input::variant_input_comp_builder::{SetOptions, SetWrapper},
        wrapper::{CompWrapper, ComponentInput, IdentityWrapper},
        ComponentInputData, DefaultInputComp, GetDynWatchableSetter, InheritLabel, Inheritable,
        InheritedInput, Saveable, WrapBuilder,
    },
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        BoolWatchable, Constant, DataState, Derived, DynSignaller, DynWatchable,
        DynWatchableSetter, F32Watchable, Field, IntoWatchable, Listener, Mutator, Observer,
        OptionBoolWatchable, U32Watchable, Watchable, WatchableSetter, WatchableState, Watcher,
        Watching,
    },
};

/// A variant input
pub struct VariantInput<V: Sized + Clone + Eq + 'static> {
    value: DynWatchableSetter<V>,
    options: DynWatchable<Vec<V>>,
    filtered: Derived<V>,
}

impl<V> Clone for VariantInput<V>
where
    V: Sized + Clone + Eq,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            options: self.options.clone(),
            filtered: self.filtered.clone(),
        }
    }
}
impl<V: Sized + Clone + Eq> VariantInput<V> {
    pub fn new_options(value: V, options: impl IntoWatchable<Vec<V>> + 'static) -> VariantInput<V> {
        Self::from_options(Field::new(value), options)
    }
    fn from_options<F: Into<DynWatchableSetter<V>> + Clone>(
        value: F,
        options: impl IntoWatchable<Vec<V>> + 'static,
    ) -> VariantInput<V> {
        let value = value.into();
        let value_clone = value.clone();
        let options = DynWatchable::new(options.into_watchable());
        let options_clone = options.clone();
        VariantInput {
            value,
            options,
            filtered: Derived::new(move |t| {
                let selected = &*value_clone.watch(t);
                let options = options_clone.watch(t);
                let valid = options.contains(&selected);
                if valid {
                    selected.clone()
                } else {
                    if let Some(def) = options.first() {
                        def.clone()
                    } else {
                        selected.clone()
                    }
                }
            }),
        }
    }
    pub fn with_comp<I: Into<Component>, M: Fn(&V) -> I + 'static>(
        self,
        map: M,
    ) -> VariantInputComponents<V> {
        VariantInputComponents::new(self, map)
    }
    pub fn comp_builder<I: Into<Component>, M: Fn(&V) -> I + 'static>(
        self,
        map: M,
    ) -> VariantInputCompBuilder<SetWrapper<SetOptions>> {
        VariantInputComp::builder(self.with_comp(map))
    }
}

// Bound options
pub trait VariantOptions: Sized + Clone + Eq {
    /// The first value is considered the default, and used as a fallback
    fn get_variants() -> Vec<Self>;
}
impl<V> VariantInput<V>
where
    V: Sized + Clone + Eq + VariantOptions,
{
    pub fn new(value: V) -> VariantInput<V> {
        Self::from_options(Field::new(value), V::get_variants())
    }
}

// Watchable traits
impl<V> WatchableState for VariantInput<V>
where
    V: Sized + Clone + Eq + 'static,
{
    fn state(&self) -> DataState {
        self.filtered.state()
    }
    fn observe(&self, listener: Box<dyn Listener>) -> Observer {
        self.filtered.observe(listener)
    }
}
impl<V> Watchable for VariantInput<V>
where
    V: Sized + Clone + Eq + 'static,
{
    type Output = V;
    fn get(&self) -> Rc<Self::Output> {
        self.filtered.get()
    }
}
impl<V> WatchableSetter for VariantInput<V>
where
    V: Sized + Clone + Eq + 'static,
{
    fn set(&mut self, val: V) -> DynSignaller {
        self.value.set(val)
    }
}

impl<V> Into<DynWatchable<V>> for VariantInput<V>
where
    V: Sized + Clone + Eq + 'static,
{
    fn into(self) -> DynWatchable<V> {
        DynWatchable::new(self)
    }
}
impl<V> Into<DynWatchableSetter<V>> for VariantInput<V>
where
    V: Sized + Clone + Eq + 'static,
{
    fn into(self) -> DynWatchableSetter<V> {
        DynWatchableSetter::new(self)
    }
}

// Component input traits
impl<V> CompWrapper for VariantInput<V>
where
    V: Sized + Clone + Eq + 'static,
{
    fn wrap(&self, comp: Component) -> Component {
        comp
    }
}
impl<V> ComponentInput for VariantInput<V>
where
    V: Sized + Clone + Eq + 'static,
{
    type Input = V;
    type Setter = Self;
    fn input(&self) -> &Self::Setter {
        self
    }
}
impl<V> Inheritable for InheritedInput<VariantInput<V>>
where
    V: Sized + Clone + Eq + 'static,
{
    fn inherit(&self, self_name: impl IntoWatchable<InheritLabel> + 'static) -> Self {
        let input = self.child_input();
        InheritedInput::new(
            VariantInput::from_options(Field::new((*input.get()).clone()), input.options.clone()),
            DynWatchable::new(self.clone()),
            self_name,
        )
    }
}
impl<V> Saveable for VariantInput<V>
where
    V: Sized + Clone + Eq + 'static + Serialize + DeserializeOwned,
{
    type Val = V;
    fn load_value(&mut self, val: V) -> DynSignaller {
        self.set(val)
    }

    fn save_value(&self) -> Self::Val {
        (&*self.get()).clone()
    }
}

// Conversion traits + default
impl<V> From<V> for VariantInput<V>
where
    V: Sized + Clone + Eq + VariantOptions + 'static,
{
    fn from(value: V) -> Self {
        VariantInput::from_options(Field::new(value.into()), V::get_variants())
    }
}
impl<V> From<V> for InheritedInput<VariantInput<V>>
where
    V: Sized + Clone + Eq + VariantOptions + 'static,
{
    fn from(value: V) -> Self {
        Self::from(VariantInput::from(value))
    }
}
impl<V> From<(V, Vec<V>)> for VariantInput<V>
where
    V: Sized + Clone + Eq + 'static,
{
    fn from((value, options): (V, Vec<V>)) -> Self {
        VariantInput::from_options(Field::new(value), options)
    }
}
impl<V> From<(V, Vec<V>)> for InheritedInput<VariantInput<V>>
where
    V: Sized + Clone + Eq + VariantOptions + 'static,
{
    fn from(value: (V, Vec<V>)) -> Self {
        Self::from(VariantInput::from(value))
    }
}
impl<V> Default for VariantInput<V>
where
    V: Sized + Clone + Eq + Default + VariantOptions + 'static,
{
    fn default() -> Self {
        Self::from(VariantInput::from(V::default()))
    }
}

// Component conversion
pub trait VariantComponentMapping {
    fn map(&self) -> Component;
}
impl<V> ComponentInputData for VariantInput<V>
where
    V: Sized + Clone + Eq + VariantComponentMapping + 'static,
{
    type InputData = (DynWatchable<Vec<V>>, ComponentVecWatchable);
    fn input_data(&self) -> Self::InputData {
        let variants = self.options.clone();
        (
            DynWatchable::new(variants.clone()),
            ComponentVecWatchable::new(Derived::new(move |t| {
                t.watch(&variants).iter().map(V::map).collect::<Vec<_>>()
            })),
        )
    }
}
impl<V> DefaultInputComp for VariantInput<V>
where
    V: Sized + Clone + Eq + VariantComponentMapping + 'static,
{
    type Comp = VariantInputComp;
}
impl<V> Into<Component> for VariantInput<V>
where
    V: Sized + Clone + Eq + VariantComponentMapping + 'static,
{
    fn into(self) -> Component {
        VariantInputComp::builder(self).build().into()
    }
}

/// Variant input with bound options
pub struct VariantInputComponents<V: Sized + Clone + Eq + 'static> {
    input: VariantInput<V>,
    map: Rc<dyn Fn(&V) -> Component>,
    components: ComponentVecWatchable,
}
impl<V> Clone for VariantInputComponents<V>
where
    V: Sized + Clone + Eq + 'static,
{
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            map: self.map.clone(),
            components: self.components.clone(),
        }
    }
}

// Component input traits
impl<V> VariantInputComponents<V>
where
    V: Sized + Clone + Eq + 'static,
{
    pub fn new<I: Into<Component>, M: Fn(&V) -> I + 'static>(
        input: VariantInput<V>,
        map: M,
    ) -> Self {
        Self::new_raw(input, Rc::new(move |v| (map)(v).into()))
    }
    fn new_raw(input: VariantInput<V>, map: Rc<dyn Fn(&V) -> Component>) -> Self {
        let input_options = input.options.clone();
        let map_clone = map.clone();
        let components = ComponentVecWatchable::new(Derived::new(move |t| {
            input_options
                .watch(t)
                .iter()
                .map(|v| map_clone(v))
                .collect::<Vec<_>>()
        }));
        VariantInputComponents {
            input,
            map,
            components,
        }
    }
}
impl<V> CompWrapper for VariantInputComponents<V>
where
    V: Sized + Clone + Eq + 'static,
{
    fn wrap(&self, comp: Component) -> Component {
        comp
    }
}
impl<V> ComponentInput for VariantInputComponents<V>
where
    V: Sized + Clone + Eq + 'static,
{
    type Input = V;
    type Setter = VariantInput<V>;
    fn input(&self) -> &Self::Setter {
        &self.input
    }
}

// Component conversion
impl<V> ComponentInputData for VariantInputComponents<V>
where
    V: Sized + Clone + Eq + 'static,
{
    type InputData = (DynWatchable<Vec<V>>, ComponentVecWatchable);
    fn input_data(&self) -> Self::InputData {
        (self.input.options.clone(), self.components.clone())
    }
}
impl<V> Into<Component> for VariantInputComponents<V>
where
    V: Sized + Clone + Eq + 'static,
{
    fn into(self) -> Component {
        VariantInputComp::builder(self).build().into()
    }
}

impl<V> Inheritable for InheritedInput<VariantInputComponents<V>>
where
    V: Sized + Clone + Eq + 'static,
{
    fn inherit(&self, self_name: impl IntoWatchable<InheritLabel> + 'static) -> Self {
        let input = self.child_input();
        let copy = VariantInputComponents::new_raw(
            VariantInput::from_options(
                Field::new((*input.input.get()).clone()),
                input.input.options.clone(),
            ),
            input.map.clone(),
        );
        InheritedInput::new(copy, DynWatchable::new(self.clone()), self_name)
    }
}
impl<V> Saveable for VariantInputComponents<V>
where
    V: Sized + Clone + Eq + 'static + Serialize + DeserializeOwned,
{
    type Val = V;
    fn load_value(&mut self, val: V) -> DynSignaller {
        self.input.set(val)
    }

    fn save_value(&self) -> Self::Val {
        (&*self.input.get()).clone()
    }
}

/// Watchable options to index conversion
struct IndexedVariantInput<V> {
    input: DynWatchableSetter<V>,
    options: DynWatchable<Vec<V>>,
    output: Derived<u32>,
}
impl<V> Clone for IndexedVariantInput<V> {
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            options: self.options.clone(),
            output: self.output.clone(),
        }
    }
}
impl<V: Eq + 'static> IndexedVariantInput<V> {
    pub fn new<W: WatchableSetter<Output = V> + Clone + 'static>(
        data: W,
        options: DynWatchable<Vec<V>>,
    ) -> Self {
        Self {
            input: DynWatchableSetter::new(data.clone()),
            options: options.clone(),
            output: Derived::new(move |t| {
                let options = options.watch(t);
                let val = data.watch(t);
                let index = (*options)
                    .iter()
                    .position(|v| v == &*val)
                    .unwrap_or_default();
                index as u32
            }),
        }
    }
}
impl<V> WatchableState for IndexedVariantInput<V> {
    fn state(&self) -> DataState {
        self.output.state()
    }

    fn observe(&self, listener: Box<dyn Listener>) -> Observer {
        self.output.observe(listener)
    }
}
impl<V> Watchable for IndexedVariantInput<V> {
    type Output = u32;
    fn get(&self) -> Rc<Self::Output> {
        self.output.get()
    }
}
impl<V: Clone> WatchableSetter for IndexedVariantInput<V> {
    fn set(&mut self, val: Self::Output) -> DynSignaller {
        let options = self.options.get();
        let option = (*options).get(val as usize);
        Box::new(option.map(|option| self.input.set(option.clone())))
    }
}

/// Variant component.
#[wasm_getters]
#[wasm_bindgen]
#[builder_into_comp]
#[watchable_setters]
#[derive(Builder, Clone)]
#[builder(start_fn(name=builder_raw, vis=""))]
pub struct VariantInputComp {
    /// The data of the component.
    #[builder(start_fn, into)]
    data: DynWatchableSetter<u32>,
    /// The options of the dropdown
    #[getter]
    #[setter(Vec<Component>)]
    options: ComponentVecWatchable,
    /// Whether the variants should display horizontally (if advanced).
    #[getter]
    #[setter(Option<bool>)]
    horizontal: OptionBoolWatchable,
    /// The main axis alignment (if advanced).
    #[getter]
    #[setter(AlignMain, AlignMain::Start)]
    main_align: AlignMainWatchable,
    /// The off axis alignment (if advanced).
    #[getter]
    #[setter(Align, Align::Start)]
    perpendicular_align: AlignWatchable,
    /// The gap between child elements (if advanced).
    #[getter]
    #[setter(f32, 1.0)]
    gap: F32Watchable,
    /// Whether this input is disabled
    #[getter]
    #[setter(bool, false)]
    disabled: BoolWatchable,
    /// Wraps the output component
    #[builder(overwritable)]
    wrapper: Rc<dyn CompWrapper>,
}

impl<I, V> WrapBuilder<I> for VariantInputComp
where
    V: Sized + Clone + Eq + 'static,
    I: ComponentInputData<Input = V, InputData = (DynWatchable<Vec<V>>, ComponentVecWatchable)>,
{
    type Builder = VariantInputCompBuilder<SetWrapper<SetOptions>>;
    fn builder(wrapper: I) -> Self::Builder {
        let (options, components) = wrapper.input_data();
        let input = IndexedVariantInput::new(wrapper.dyn_input(), options);

        Self::builder_raw(DynWatchableSetter::new(input))
            .options(components)
            .wrapper(Rc::new(wrapper))
    }
}
impl VariantInputComp {
    fn watchable(&self) -> &DynWatchableSetter<u32> {
        &self.data
    }
    fn setter(&mut self) -> &mut DynWatchableSetter<u32> {
        &mut self.data
    }
}
impl_watchable!(VariantInputComp, u32);
impl_setter!(VariantInputComp, u32);

impl Into<Component> for VariantInputComp {
    fn into(self) -> Component {
        let wrapper = self.wrapper.clone();
        let comp = Component::new(ComponentOption::VariantInput(self));
        wrapper.wrap(comp)
    }
}

// Add inherited input default comp support
impl<V, VI> ComponentInputData for InheritedInput<VI>
where
    V: Sized + Clone + Eq + 'static,
    VI: ComponentInputData<Input = V, InputData = (DynWatchable<Vec<V>>, ComponentVecWatchable)>,
{
    type InputData = (DynWatchable<Vec<V>>, ComponentVecWatchable);

    fn input_data(&self) -> Self::InputData {
        self.child_input().input_data()
    }
}
