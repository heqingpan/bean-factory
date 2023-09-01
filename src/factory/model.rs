
use std::default;
use std::{sync::Arc, collections::HashMap};
use std::any::{Any, type_name};

use actix::dev::ToEnvelope;
use actix::prelude::*;

use super::BeanFactory;

pub type DynAny = dyn Any + 'static + Send + Sync;

#[derive(Clone,Message)]
#[rtype(result="()")]
pub struct BeanDefinition {
    pub type_name: String,
    pub provider: Arc<dyn Fn() -> Option<Arc<DynAny>> + Send + Sync>,
    pub notify: Option<Arc<dyn Fn(Arc<DynAny>,FactoryEvent) -> () + Send + Sync >>,
    //pub inject: bool,
}

impl BeanDefinition {
    pub fn from_default<C: Default + Any + 'static + Send + Sync>() -> Self {
        Self {
            type_name: std::any::type_name::<C>().to_string(),
            provider: Arc::new(move || Some(C::default()).map(|x| Arc::new(x) as Arc::<DynAny>)),
            notify: None,
            //inject: false
        }
    }

    pub fn actor_from_default<T>() -> Self
        where T: Default + Actor<Context = Context<T>>
    {
        Self {
            type_name: std::any::type_name::<T>().to_string(),
            provider: Arc::new(move || Some(T::default().start()).map(|x| Arc::new(x) as Arc::<DynAny>)),
            notify: None,
            //inject: false
        }
    }

    pub fn actor_with_inject_from_default<T>() -> Self
        where T: Default + Actor<Context = Context<T>> + Handler<FactoryEvent>,
        <T as Actor>::Context: ToEnvelope<T,FactoryEvent>
    {
        Self {
            type_name: std::any::type_name::<T>().to_string(),
            provider: Arc::new(move || Some(T::default().start()).map(|x| Arc::new(x) as Arc::<DynAny>)),
            notify: Some(Arc::new(|a,event| {
                a.downcast::<Addr<T>>().ok().map(|e|e.do_send(event));
            })),
            //inject: false
        }
    }

    pub fn from_fn<T: 'static + Send + Sync>(f: impl Fn() -> T + 'static + Send + Sync) -> Self {
        Self {
            type_name: std::any::type_name::<T>().to_string(),
            provider: Arc::new(move || Some(f()).map(|x| Arc::new(x) as Arc::<DynAny>)),
            notify: None,
            //inject: false,
        }
    }

    pub fn actor_from_fn<T: Actor>(f: impl Fn() -> Addr<T> + 'static + Send + Sync) -> Self {
        Self {
            type_name: std::any::type_name::<T>().to_string(),
            provider: Arc::new(move || Some(f()).map(|x| Arc::new(x) as Arc::<DynAny>)),
            notify: None,
            //inject: false,
        }
    }

    pub fn actor_with_inject_from_fn<T>(f: impl Fn() -> Addr<T> + 'static + Send + Sync,
        notify: Arc<dyn Fn(Arc<DynAny>,FactoryEvent) -> () + Send + Sync >
    ) -> Self 
        where T: Default + Actor<Context = Context<T>> + Handler<FactoryEvent>,
        <T as Actor>::Context: ToEnvelope<T,FactoryEvent>
    {
        Self {
            type_name: std::any::type_name::<T>().to_string(),
            provider: Arc::new(move || Some(f()).map(|x| Arc::new(x) as Arc::<DynAny>)),
            notify: Some(notify),
            //inject: false,
        }
    }

}

#[derive(Debug,Clone)]
pub struct FactoryData(pub Arc<HashMap<String,Arc<DynAny>>>);

impl FactoryData {
    pub fn get_actor_by_name<T:Actor>(&self,name:&str) -> Option<Addr<T>> {
        self.0.get(name).map(|x|x.clone().downcast::<Addr<T>>().ok())
            .flatten().map(|x|x.as_ref().clone())
    }

    pub fn get_actor<T:Actor>(&self) -> Option<Addr<T>> {
        self.get_actor_by_name(type_name::<T>())
    }

    pub fn get_bean_by_name<T:'static + Send + Sync>(&self,name:&str) -> Option<Arc<T>> {
        self.0.get(name).map(|x|x.clone().downcast::<T>().ok())
            .flatten()
    }

    pub fn get_bean<T:'static + Send + Sync>(&self) -> Option<Arc<T>> {
        self.get_bean_by_name(type_name::<T>())
    }
}

pub trait IInject {
    fn inject(&mut self,factory_data:FactoryData,factory:BeanFactory);
    fn complete(&mut self,factory:BeanFactory);
}

#[derive(Message)]
#[rtype(result="()")]
pub struct InitFactory;

#[derive(Message)]
#[rtype(result="Option<Arc<DynAny>>")]
pub struct QueryBean(pub String);


#[derive(Message,Clone)]
#[rtype(result="()")]
pub enum FactoryEvent {
    Inject {
        factory: BeanFactory,
        data: FactoryData,
    },
    Complete {
        factory: BeanFactory,
    }
}

