#[macro_export]
macro_rules! impl_into_comps {
    ($StructInput:ident, $StructComp:ident) => {
        impl Into<$StructComp> for $StructInput {
            fn into(self) -> $StructComp {
                $StructComp::builder(self).build()
            }
        }
        impl Into<$StructComp> for InheritedInput<$StructInput> {
            fn into(self) -> $StructComp {
                $StructComp::builder(self).into()
            }
        }
        impl Into<Component> for $StructInput {
            fn into(self) -> Component {
                Into::<$StructComp>::into(self).into()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_default_input_comp {
    ($ValueType:ty, $StructInput:ident, $StructComp:ident) => {
        impl crate::inputs::wrapper::CompWrapper for $StructInput {
            fn wrap(&self, comp: Component) -> Component {
                comp
            }
        }
        impl crate::inputs::wrapper::ComponentInput for $StructInput {
            type Input = $ValueType;
            type Setter = $StructInput;
            fn input(&self) -> &Self::Setter {
                self
            }
        }
        impl crate::inputs::wrapper::DefaultInputComp for $StructInput {
            type Comp = $StructComp;
        }
    };
}

#[macro_export]
macro_rules! impl_inheritable {
    ($StructInput:ident) => {
        impl Inheritable for InheritedInput<$StructInput> {
            fn inherit(&self, self_name: impl IntoWatchable<InheritLabel> + 'static) -> Self {
                InheritedInput::new(
                    $StructInput::from(
                        (*crate::inputs::inherited_input::InheritedInput::get(&self)).clone(),
                    ),
                    DynWatchable::new(self.clone()),
                    self_name,
                )
            }
        }
    };
}

#[macro_export]
macro_rules! impl_input_from {
    ($StructInput:ident, $ValueType:ty) => {
        impl<X: Into<$ValueType>> From<X> for $StructInput {
            fn from(value: X) -> Self {
                $StructInput::new(value.into())
            }
        }
        crate::impl_inherited_input_from!($StructInput, $ValueType);
    };
}
#[macro_export]
macro_rules! impl_inherited_input_from {
    ($StructInput:ident, $ValueType:ty) => {
        impl<X: Into<$ValueType>> From<X> for InheritedInput<$StructInput> {
            fn from(value: X) -> Self {
                InheritedInput::from($StructInput::from(value))
            }
        }
    };
}

#[macro_export]
macro_rules! impl_default {
    ($StructInput:ident) => {
        impl Default for $StructInput {
            fn default() -> Self {
                Self::new(Default::default())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_saveable {
    ($StructInput:ident,  $ValueType:ty) => {
        impl crate::inputs::Saveable for $StructInput {
            type Val = $ValueType;
            fn load_value(&mut self, val: $ValueType) -> crate::util::watchables::DynSignaller {
                self.setter().set(val)
            }

            fn save_value(&self) -> Self::Val {
                let val: &Self::Val = &self.watchable().get();
                val.clone()
            }
        }
    };
}
