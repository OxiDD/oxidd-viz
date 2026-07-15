use std::{any::Any, ops::Deref, rc::Rc};

use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsValue,
};

use crate::util::watchables::{
    DataState, Field, IntoWatchable, Listener, Watchable, WatchableSetter, WatchableState,
};

#[macro_export]
macro_rules! impl_watchable {
    ($StructName:ident, $ValueType:ty) => {
        impl crate::util::watchables::WatchableState for $StructName {
            fn state(&self) -> crate::util::watchables::DataState {
                self.watchable().state()
            }
            fn observe(
                &self,
                listener: Box<dyn crate::util::watchables::Listener>,
            ) -> crate::util::watchables::Observer {
                self.watchable().observe(listener)
            }
        }
        impl crate::util::watchables::Watchable for $StructName {
            type Output = $ValueType;
            fn get(&self) -> std::rc::Rc<Self::Output> {
                crate::util::watchables::Watchable::get(self.watchable())
            }
        }
        impl Into<crate::util::watchables::DynWatchable<$ValueType>> for $StructName {
            fn into(self) -> crate::util::watchables::DynWatchable<$ValueType> {
                crate::util::watchables::DynWatchable::new(self)
            }
        }
        impl crate::util::watchables::IntoWatchable<$ValueType> for $StructName {
            type Output = $StructName;
            fn into_watchable(self) -> Self::Output {
                self
            }
        }
        #[wasm_bindgen]
        impl $StructName {
            pub fn get(&self) -> $ValueType {
                (*crate::util::watchables::Watchable::get(self.watchable())).clone()
            }
            #[wasm_bindgen(js_name = onDirty)]
            pub fn on_dirty(
                &self,
                listener: js_sys::Function,
            ) -> crate::util::watchables::Observer {
                crate::util::watchables::WatchableState::observe(
                    self,
                    Box::new(crate::util::watchables::JsListener::new(listener, false)),
                )
            }
            #[wasm_bindgen(js_name = onChange)]
            pub fn on_change(
                &self,
                listener: js_sys::Function,
            ) -> crate::util::watchables::Observer {
                crate::util::watchables::WatchableState::observe(
                    self,
                    Box::new(crate::util::watchables::JsListener::new(listener, true)),
                )
            }
        }
    };
}

#[macro_export]
macro_rules! impl_setter {
    ($StructName:ident, $ValueType:ty) => {
        impl WatchableSetter for $StructName {
            fn set(&mut self, val: $ValueType) -> crate::util::watchables::DynSignaller {
                self.setter().set(val)
            }
        }
        impl Into<crate::util::watchables::DynWatchableSetter<$ValueType>> for $StructName {
            fn into(self) -> crate::util::watchables::DynWatchableSetter<$ValueType> {
                crate::util::watchables::DynWatchableSetter::new(self)
            }
        }
        impl crate::util::watchables::IntoWatchableSetter<$ValueType> for $StructName {
            type Output = $StructName;
            fn into_watchable_setter(self) -> Self::Output {
                self
            }
        }
        #[wasm_bindgen]
        impl $StructName {
            /// Creates a mutator that when committed changes the value, after committing the mutation, the state of this field is DataState::UpToDate again
            #[must_use = "Only once the mutator is committed, will the field be changed"]
            #[wasm_bindgen(js_name=set)]
            pub fn set_js(&mut self, val: $ValueType) -> Mutator {
                let mut field = self.setter().clone();
                Mutator::exec(move || {
                    crate::util::logging::console::log!("executed");
                    Box::new(field.set(val))
                })
            }
        }
    };
}

#[macro_export]
macro_rules! make_typed_dyn_watchable {
    ($StructName:ident, $ValueType:ty) => {
        #[allow(non_camel_case_types)]
        #[wasm_bindgen]
        #[derive(Clone)]
        pub struct $StructName(
            std::rc::Rc<dyn crate::util::watchables::Watchable<Output = $ValueType>>,
        );
        impl $StructName {
            pub fn new<W: crate::util::watchables::IntoWatchable<$ValueType> + 'static>(
                watchable: W,
            ) -> Self {
                Self(std::rc::Rc::new(watchable.into_watchable()))
            }
            fn watchable(&self) -> &dyn crate::util::watchables::Watchable<Output = $ValueType> {
                &*self.0
            }
        }
        crate::impl_watchable!($StructName, $ValueType);
    };
}

#[macro_export]
macro_rules! typed_field_derive {
    (true, { $($body:item)* }) => {
        #[derive(Clone, serde::Serialize, serde::Deserialize)]
        $($body)*
    };
    (false, { $($body:item)* }) => {
        #[derive(Clone)]
        $($body)*
    };
}

#[macro_export]
macro_rules! make_typed_field {
    ($StructName:ident, $WatchableStructName:ident, $ValueType:ty, $Serialize:tt) => {
        crate::typed_field_derive!($Serialize, {
            #[allow(non_camel_case_types)]
            #[wasm_bindgen]
            pub struct $StructName(Field<$ValueType>);
            impl $StructName {
                pub fn new(init: $ValueType) -> Self {
                    Self(Field::new(init))
                }
                pub fn from<V: Into<$ValueType>>(init: V) -> Self {
                    Self::new(init.into())
                }
                fn watchable(&self) -> &Field<$ValueType> {
                    &self.0
                }
                fn setter(&mut self) -> &mut Field<$ValueType> {
                    &mut self.0
                }
            }
            crate::impl_watchable!($StructName, $ValueType);
            crate::impl_setter!($StructName, $ValueType);
            #[wasm_bindgen]
            impl $StructName {
                /// Creates a readonly reference to this field data
                pub fn read(&self) -> $WatchableStructName {
                    $WatchableStructName::new(self.0.read())
                }
            }
        });
    };
}

pub struct JsListener {
    listener: js_sys::Function,
    on_up_to_date: bool,
}
impl JsListener {
    pub fn new(listener: js_sys::Function, on_up_to_date: bool) -> Self {
        JsListener {
            listener,
            on_up_to_date,
        }
    }
}
impl Listener for JsListener {
    fn state_changed(&self, state: DataState) {
        let is_up_to_date = state == DataState::UpToDate;
        if self.on_up_to_date == is_up_to_date {
            let this = JsValue::null();
            let _ = self.listener.call0(&this);
        }
    }
}

#[wasm_bindgen]
pub struct Mutator {
    perform: Option<Box<dyn FnOnce() -> Box<dyn FnOnce() -> ()>>>,
    signal: Option<Box<dyn FnOnce() -> ()>>,
}

impl Mutator {
    pub fn exec<F: FnOnce() -> Box<dyn Any> + 'static>(func: F) -> Self {
        Mutator {
            perform: Some(Box::new(move || {
                let signaller = func();
                Box::new(move || drop(signaller))
            })),
            signal: None,
        }
    }
}
#[wasm_bindgen]
impl Mutator {
    pub fn new(perform: js_sys::Function, signal: js_sys::Function) -> Self {
        let this1 = JsValue::null();
        let this2 = JsValue::null();
        Mutator {
            perform: Some(Box::new(move || {
                let _ = perform.call0(&this1);
                Box::new(move || {
                    let _ = signal.call0(&this2);
                })
            })),
            signal: None,
        }
    }
    pub fn commit(mut self) {
        self.perform();
        self.signal();
    }
    pub fn perform(&mut self) {
        let Some(perform) = self.perform.take() else {
            return;
        };
        self.signal = Some(perform());
    }
    pub fn signal(&mut self) {
        let Some(signal) = self.signal.take() else {
            return;
        };
        signal();
    }
    pub fn chain(mut self, mut next: Mutator) -> Mutator {
        Mutator {
            perform: Some(Box::new(move || {
                self.perform();
                next.perform();
                Box::new(move || {
                    self.signal();
                    next.signal();
                })
            })),
            signal: None,
        }
    }
}
pub trait MutateSetter<T> {
    fn mutate_set(&mut self, val: T) -> Mutator;
}
impl<S: WatchableSetter + Clone + 'static> MutateSetter<S::Output> for S {
    fn mutate_set(&mut self, val: S::Output) -> Mutator {
        let mut field = self.clone();
        Mutator::exec(move || Box::new(field.set(val)))
    }
}

// pub use impl_setter;
// pub use impl_watchable;
// pub use make_typed_dyn_watchable;
// pub use make_typed_field;

make_typed_dyn_watchable!(StringWatchable, String);
make_typed_dyn_watchable!(OptionStringWatchable, Option<String>);
make_typed_dyn_watchable!(BoolWatchable, bool);
make_typed_dyn_watchable!(OptionBoolWatchable, Option<bool>);
make_typed_dyn_watchable!(U32Watchable, u32);
make_typed_dyn_watchable!(OptionU32Watchable, Option<u32>);
make_typed_dyn_watchable!(I32Watchable, i32);
make_typed_dyn_watchable!(OptionI32Watchable, Option<i32>);
make_typed_dyn_watchable!(F32Watchable, f32);
make_typed_dyn_watchable!(OptionF32Watchable, Option<f32>);

make_typed_field!(StringField, StringWatchable, String, true);
make_typed_field!(
    OptionStringField,
    OptionStringWatchable,
    Option<String>,
    true
);
make_typed_field!(BoolField, BoolWatchable, bool, true);
make_typed_field!(U32Field, U32Watchable, u32, true);
make_typed_field!(I32Field, I32Watchable, i32, true);
make_typed_field!(F32Field, F32Watchable, f32, true);
