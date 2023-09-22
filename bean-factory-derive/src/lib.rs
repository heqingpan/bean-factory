#![allow(unused_assignments)]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
//use syn::{AttributeArgs, NestedMeta};
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

/// Full feature example: `#[bean(actor, inject)]`
#[proc_macro_attribute]
pub fn bean(args: TokenStream, input: TokenStream) -> TokenStream {
    let arg_str = args.to_string();
    let config = read_bean_config(&arg_str);
    let input_clone: proc_macro2::TokenStream = input.clone().into();
    let ast = syn::parse(input).unwrap();
    let stream = impl_bean_derive(&ast, config);
    let s: proc_macro2::TokenStream = stream.into();
    let qt = quote! {
       #input_clone
       #s
    };
    qt.into()
}

#[derive(Debug, Default)]
pub(crate) struct BeanConfig {
    pub is_actor: bool,
    pub is_inject: bool,
    pub is_register: bool,
}

///
/// read bean config
/// actor,inject,register
fn read_bean_config(arg: &str) -> BeanConfig {
    let mut config = BeanConfig::default();
    let keys: Vec<&str> = arg.split(",").collect();
    for key in keys {
        let item = key.trim();
        match item {
            "actor" => config.is_actor = true,
            "inject" => {
                config.is_actor = true;
                config.is_inject = true;
            }
            "register" => config.is_register = true,
            _ => {}
        }
    }
    config
}

fn impl_bean_derive(ast: &syn::DeriveInput, config: BeanConfig) -> TokenStream {
    let name = &ast.ident;
    let inject_handler = if config.is_inject {
        quote! {
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
        }
    } else {
        quote! {}
    };
    let register = if config.is_register {
        match (config.is_actor, config.is_inject) {
            (true, true) => quote! {
                bean_factory::submit! {
                    bean_factory::BeanDefinition::actor_with_inject_from_default::<#name>()
                }
            },
            (true, false) => quote! {
                bean_factory::submit! {
                    bean_factory::BeanDefinition::actor_from_default::<#name>()
                }
            },
            (false, true) => quote! {
                panic!("not actors cannot be injected!");
            },
            (false, false) => quote! {
                bean_factory::submit! {
                    bean_factory::BeanDefinition::from_default::<#name>()
                }
            },
        }
    } else {
        quote! {}
    };
    let gen = quote! {
        #inject_handler
        #register
    };
    gen.into()
}
