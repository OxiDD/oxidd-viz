use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident};

pub fn derive_saveable_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let struct_name = syn::Ident::new(&format!("{}Data", name), name.span());
    let struct_def_sig = quote! { #struct_name #ty_generics #where_clause };
    let (struct_def, load_body, save_body) = match input.data {
        Data::Struct(data) => saveable_struct(&data.fields, &struct_def_sig),
        // Data::Enum(data) => saveable_enum(&name, &data.variants, &struct_def_sig),
        _ => panic!("Can only derive Saveable for structs"),
    };

    let expanded = quote! {
        #[derive(serde::Serialize, serde::Deserialize)]
        #struct_def
        impl #impl_generics crate::inputs::saveable::Saveable for #name #ty_generics #where_clause {
            type Val = #struct_name<#ty_generics>;
            fn load_value(&mut self, val: Self::Val) -> crate::util::watchables::DynSignaller {
                #load_body
            }

            fn save_value(&self) -> Self::Val {
                #save_body
            }
        }
    };

    expanded.into()
}
fn saveable_struct(
    fields: &syn::Fields,
    struct_sig: &proc_macro2::TokenStream,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    match fields {
        Fields::Named(fields) => {
            let saveable_fields = fields.named.iter().map(|f| {
                let name = &f.ident;
                let f_type = &f.ty;
                quote! {
                    #name: <#f_type as Saveable>::Val
                }
            });
            let saveable_loads = fields.named.iter().map(|f| {
                let name = &f.ident;
                quote! {
                    self.#name.load_value(val.#name)
                }
            });
            let saveable_saves = fields.named.iter().map(|f| {
                let name = &f.ident;
                quote! {
                    #name: self.#name.save_value()
                }
            });

            (
                quote! { pub struct #struct_sig {
                    #(#saveable_fields),*
                } },
                quote! { Box::new((
                    #(#saveable_loads),*
                )) },
                quote! { Self::Val {
                    #(#saveable_saves),*
                } },
            )
        }
        Fields::Unnamed(fields) => {
            let saveable_fields = fields.unnamed.iter().map(|f| {
                let f_type = &f.ty;
                quote! {
                    <#f_type as Saveable>::Val
                }
            });
            let saveable_loads = fields.unnamed.iter().enumerate().map(|(i, _)| {
                let idx = syn::Index::from(i);
                quote! {
                    self.#idx.load_value(val.#idx)
                }
            });
            let saveable_saves = fields.unnamed.iter().enumerate().map(|(i, _)| {
                let idx = syn::Index::from(i);
                quote! {
                    self.#idx.save_value()
                }
            });

            (
                quote! { pub struct #struct_sig (
                       #(#saveable_fields),*
                ) },
                quote! { Box::new((
                    #(#saveable_loads),*
                )) },
                quote! { Self::Val (
                       #(#saveable_saves),*
                ) },
            )
        }
        Fields::Unit => (
            quote! { pub struct #struct_sig; },
            quote! { Box::new(()) },
            quote! { Self::Val },
        ),
    }
}

// Todo: fix the below, and consider what to do if data type does not match with enum type when loading
// fn saveable_enum(
//     name: &syn::Ident,
//     variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
//     struct_sig: &proc_macro2::TokenStream,
// ) -> (
//     proc_macro2::TokenStream,
//     proc_macro2::TokenStream,
//     proc_macro2::TokenStream,
// ) {

//     let constructors = variants.iter().map(|variant| {
//         let vname = &variant.ident;

//         match &variant.fields {
//             Fields::Named(fields) => {
//                 let saveable_fields = fields.named.iter().map(|f| {
//                     let name = &f.ident;
//                     let f_type = &f.ty;
//                     quote! {
//                         #name: <#f_type as Saveable>::Val
//                     }
//                 });
//                 quote! { #vname {
//                     #(#saveable_fields),*
//                 } }
//             }
//             Fields::Unnamed(fields) => {
//                 let saveable_fields = fields.unnamed.iter().map(|f| {
//                     let f_type = &f.ty;
//                     quote! {
//                         <#f_type as Saveable>::Val
//                     }
//                 });

//                 quote! { #vname (
//                        #(#saveable_fields),*
//                 ) }
//             }
//             Fields::Unit => quote! { #vname },
//         }
//     });
//     let load_values = variants.iter().map(|variant| {
//         let vname = &variant.ident;

//         match &variant.fields {
//             Fields::Named(fields) => {
//                 let data_names = fields.named.iter().map(|f| {
//                     f.ident
//                         .as_ref()
//                         .map(|name| syn::Ident::new(&format!("{}_data", name), name.span()))
//                 });
//                 let names = fields.named.iter().map(|f| f.ident.as_ref().unwrap());
//                 let saveable_fields = names.clone().zip(data_names.clone()).map(|(n, data_n)| {
//                     quote! {
//                         #n.load_value(#data_n)
//                     }
//                 });
//                 quote! {
//                     (self { #(ref #names),*}, Self::Val::#vname { #( ref #data_names ),* }) => {
//                         Box::new((
//                             #(#saveable_fields),*
//                         ))
//                     }
//                 }
//             }
//             Fields::Unnamed(fields) => {
//                 let bindings_data= (0..fields.unnamed.len())
//                     .map(|i| syn::Ident::new(&format!("f{i}_data"), proc_macro2::Span::call_site()));
//                 let bindings= (0..fields.unnamed.len())
//                     .map(|i| syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site()));

//                 quote! { #vname (
//                        #(#saveable_fields),*
//                 ) }
//             }
//             Fields::Unit => quote! { #vname },
//         }
//     });

//     // quote! {
//     //     match self {
//     //         #(#arms),*
//     //     }
//     // }
//     todo!()
// }
