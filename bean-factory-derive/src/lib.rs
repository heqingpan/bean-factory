use proc_macro::TokenStream;
use quote::quote;
//use syn::{AttributeArgs, ItemFn, NestedMeta};

#[proc_macro_derive(Component)]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let gen = quote! {
        bean_factory::submit! {
            bean_factory::BeanDefinition::from_default::<#name>()
        }
    };

    gen.into()
}

#[proc_macro_derive(ActorComponent)]
pub fn actor_component_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let gen = quote! {
        bean_factory::submit! {
            bean_factory::BeanDefinition::actor_from_default::<#name>()
        }
    };

    gen.into()
}


#[proc_macro_derive(InjectComponent)]
pub fn inject_component_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let gen = quote! {
        impl bean_factory::Handler<bean_factory::FactoryEvent> for #name {
            type Result = ();
            fn handle(&mut self, msg: bean_factory::FactoryEvent, ctx: &mut Self::Context) -> Self::Result {
                match msg {
                    bean_factory::FactoryEvent::Inject {
                        factory,
                        factory_data,
                    } => {
                        Inject::inject(self, factory_data, factory, ctx);
                    }
                    bean_factory::FactoryEvent::Complete => {
                        Inject::complete(self, ctx);
                    }
                }
            }
        }
        bean_factory::submit! {
            bean_factory::BeanDefinition::actor_with_inject_from_default::<#name>()
        }
    };

    gen.into()
}
