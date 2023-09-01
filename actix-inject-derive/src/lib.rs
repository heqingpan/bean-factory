use proc_macro::TokenStream;
use quote::quote;
//use syn::{AttributeArgs, ItemFn, NestedMeta};

#[proc_macro_derive(Component)]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let gen = quote! {
        actix_inject::submit! {
            actix_inject::BeanDefinition::from_default::<#name>()
        }
    };

    gen.into()
}

#[proc_macro_derive(ActorComponent)]
pub fn actor_component_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let gen = quote! {
        actix_inject::submit! {
            actix_inject::BeanDefinition::actor_from_default::<#name>()
        }
    };

    gen.into()
}


#[proc_macro_derive(InjectComponent)]
pub fn inject_component_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let gen = quote! {
        impl actix_inject::Handler<actix_inject::FactoryEvent> for #name {
            type Result = ();
            fn handle(&mut self, msg: actix_inject::FactoryEvent, ctx: &mut Self::Context) -> Self::Result {
                match msg {
                    actix_inject::FactoryEvent::Inject {
                        factory,
                        factory_data,
                    } => {
                        Inject::inject(self, factory_data, factory, ctx);
                    }
                    actix_inject::FactoryEvent::Complete => {
                        Inject::complete(self, ctx);
                    }
                }
            }
        }
        actix_inject::submit! {
            actix_inject::BeanDefinition::actor_with_inject_from_default::<#name>()
        }
    };

    gen.into()
}
