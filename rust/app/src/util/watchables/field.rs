use std::{any::Any, cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::util::watchables::{
    signaller::{DynSignaller, Signaller},
    DynWatchableSetter, IntoWatchable, IntoWatchableSetter,
};

use super::{
    trackers::Trackers,
    watchable::{DataState, Listener, Observer, Watchable, WatchableState},
};

pub trait WatchableSetter: Watchable {
    /// Sets the field and returns a signaller which will signal about the change once dropped. In order to perform batching of setting fields, simply hold on to the signaller until the next field is set
    fn set(&mut self, val: Self::Output) -> DynSignaller;
}
pub struct Field<X> {
    inner: Rc<RefCell<FieldInner<X>>>,
}
impl<X> Clone for Field<X> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
struct FieldInner<X> {
    val: Rc<X>,
    dependents: Trackers,
}

const LOOPMESSAGE: &str = "Dependency loop detected!";
impl<X> Watchable for Field<X> {
    type Output = X;
    fn get(&self) -> Rc<Self::Output> {
        self.inner.try_borrow().expect(LOOPMESSAGE).val.clone()
    }
}
impl<X> IntoWatchable<X> for Field<X> {
    type Output = Field<X>;
    fn into_watchable(self) -> Self::Output {
        self
    }
}
impl<X: 'static> IntoWatchableSetter<X> for Field<X> {
    type Output = Field<X>;
    fn into_watchable_setter(self) -> Self::Output {
        self
    }
}
impl<X: 'static> Into<DynWatchableSetter<X>> for Field<X> {
    fn into(self) -> DynWatchableSetter<X> {
        DynWatchableSetter::new(self)
    }
}

impl<X> WatchableState for Field<X> {
    fn state(&self) -> DataState {
        self.inner
            .try_borrow()
            .expect(LOOPMESSAGE)
            .dependents
            .get_state()
    }

    fn observe(&self, tracker: Box<dyn Listener>) -> Observer {
        self.inner
            .try_borrow()
            .expect(LOOPMESSAGE)
            .dependents
            .observe(tracker)
    }
}

impl<X: 'static> Field<X> {
    pub fn new(init: X) -> Self {
        Field {
            inner: Rc::new(RefCell::new(FieldInner {
                val: Rc::new(init),
                dependents: Trackers::new(DataState::UpToDate),
            })),
        }
    }
    pub fn from<V: Into<X>>(init: V) -> Self {
        Self::new(init.into())
    }

    /// Creates a readonly reference to this field data
    pub fn read(&self) -> ReadonlyField<X> {
        ReadonlyField(self.clone())
    }
}

impl<X: 'static> WatchableSetter for Field<X> {
    fn set(&mut self, val: X) -> DynSignaller {
        let inner = self.inner.clone();
        inner
            .try_borrow()
            .expect(LOOPMESSAGE)
            .dependents
            .change_state(DataState::Outdated);
        inner.try_borrow_mut().expect(LOOPMESSAGE).val = Rc::new(val);
        Box::new(Signaller::new(move || {
            inner
                .try_borrow()
                .expect(LOOPMESSAGE)
                .dependents
                .change_state(DataState::UpToDate);
        }))
    }
}

// Serialization
impl<X: Serialize> Serialize for Field<X> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.get().serialize(serializer)
    }
}
impl<'de, X: Deserialize<'de> + 'static> Deserialize<'de> for Field<X> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let x = X::deserialize(deserializer)?;
        Ok(Field::new(x))
    }
}

/// Disallow writing to the field
pub struct ReadonlyField<X>(Field<X>);
impl<X> Clone for ReadonlyField<X> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<X> Watchable for ReadonlyField<X> {
    type Output = X;

    fn get(&self) -> Rc<Self::Output> {
        self.0.get()
    }
}
impl<X> WatchableState for ReadonlyField<X> {
    fn state(&self) -> DataState {
        self.0.state()
    }

    fn observe(&self, tracker: Box<dyn Listener>) -> Observer {
        self.0.observe(tracker)
    }
}
impl<X> IntoWatchable<X> for ReadonlyField<X> {
    type Output = ReadonlyField<X>;
    fn into_watchable(self) -> Self::Output {
        self
    }
}
impl<X: Serialize> Serialize for ReadonlyField<X> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

/// Disallows cloning of the field
pub struct ControlledField<X>(Field<X>);

impl<X> Watchable for ControlledField<X> {
    type Output = X;

    fn get(&self) -> Rc<Self::Output> {
        self.0.get()
    }
}
impl<X> WatchableState for ControlledField<X> {
    fn state(&self) -> DataState {
        self.0.state()
    }

    fn observe(&self, tracker: Box<dyn Listener>) -> Observer {
        self.0.observe(tracker)
    }
}
impl<X: 'static> ControlledField<X> {
    pub fn new(init: X) -> Self {
        Self(Field::new(init))
    }
    pub fn from<V: Into<X>>(init: V) -> Self {
        Self::new(init.into())
    }

    /// Creates a readonly reference to this field data
    pub fn read(&self) -> ReadonlyField<X> {
        self.0.read()
    }
}
impl<X: 'static> WatchableSetter for ControlledField<X> {
    fn set(&mut self, val: X) -> DynSignaller {
        self.0.set(val)
    }
}
impl<X: 'static> Into<DynWatchableSetter<X>> for ControlledField<X> {
    fn into(self) -> DynWatchableSetter<X> {
        DynWatchableSetter::new(self)
    }
}
impl<X: 'static> IntoWatchableSetter<X> for ControlledField<X> {
    type Output = ControlledField<X>;
    fn into_watchable_setter(self) -> Self::Output {
        self
    }
}
impl<X> IntoWatchable<X> for ControlledField<X> {
    type Output = ControlledField<X>;
    fn into_watchable(self) -> Self::Output {
        self
    }
}
impl<X: Serialize> Serialize for ControlledField<X> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}
impl<'de, X: Deserialize<'de> + 'static> Deserialize<'de> for ControlledField<X> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let field = Field::<X>::deserialize(deserializer)?;
        Ok(ControlledField(field))
    }
}
