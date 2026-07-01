use proc_macro::TokenStream;

mod builder;
mod component;
mod component_vec;
mod default;
mod inheritable;
mod saveable;
mod wasm;

use crate::{
    builder::{builder_into_comp_impl, watchable_setters_impl},
    component::derive_component_impl,
    component_vec::gen_tuple_into_component_vec_watchables_impl,
    default::derive_default_impl,
    inheritable::derive_inheritable_impl,
    saveable::derive_saveable_impl,
    wasm::wasm_getters_impl,
};

#[proc_macro_attribute]
pub fn wasm_getters(attr: TokenStream, item: TokenStream) -> TokenStream {
    wasm_getters_impl(attr, item)
}

#[proc_macro_attribute]
pub fn builder_into_comp(attr: TokenStream, item: TokenStream) -> TokenStream {
    builder_into_comp_impl(attr, item)
}

#[proc_macro_attribute]
pub fn watchable_setters(attr: TokenStream, item: TokenStream) -> TokenStream {
    watchable_setters_impl(attr, item)
}

#[proc_macro]
pub fn gen_tuple_into_component_vec_watchables(data: TokenStream) -> TokenStream {
    gen_tuple_into_component_vec_watchables_impl(data)
}

#[proc_macro_derive(Inheritable)]
pub fn derive_inheritable(input: TokenStream) -> TokenStream {
    derive_inheritable_impl(input)
}

#[proc_macro_derive(Saveable)]
pub fn derive_saveable(input: TokenStream) -> TokenStream {
    derive_saveable_impl(input)
}

#[proc_macro_derive(InitDefault, attributes(init))]
pub fn derive_default(input: TokenStream) -> TokenStream {
    derive_default_impl(input)
}

#[proc_macro_derive(Component, attributes(comp, label))]
pub fn derive_component(input: TokenStream) -> TokenStream {
    derive_component_impl(input)
}
