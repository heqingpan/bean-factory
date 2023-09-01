
use quote::quote;
use proc_macro::TokenStream;
use syn::{ItemFn, AttributeArgs, NestedMeta};

#[proc_macro_derive(Component)]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let gen = quote! {
        impl autowired::Component for #name {
            fn new_instance() -> Option<Self> {
               Some(Default::default())
            }
        }
        autowired::submit! {
            autowired::Bean::new_unchecked::<#name>()
        }
    };

    gen.into()
}

