
use std::{collections::HashMap, sync::Arc, any::type_name};

use actix::prelude::*;

use self::model::{ActorBean, DynAny, FactoryEvent, FactoryData, RegisterBean, InitFactory, QueryBean};

pub mod model;

#[derive(Default)]
pub struct BeanFactoryCore {
    component_map: HashMap<String,Arc<DynAny>>,
    bean_map: HashMap<String,ActorBean>,
}

impl BeanFactoryCore {
    fn init(&mut self)  {
        for (_,bean) in &self.bean_map {
            if let Some(v) = (bean.provider)() {
                self.component_map.insert(bean.type_name.to_owned(), v);
            }
        }
    }

    fn do_notify_event(&mut self,event: FactoryEvent) {
        for (name,bean) in &self.bean_map {
            if !bean.inject {
                continue;
            }
            match (self.component_map.get(name),bean.notify.as_ref()) {
                (Some(c),Some(notify)) => {
                    notify(c.clone(),event.clone())
                }
                (_,_) => {}
            }
        }
    }

    fn inject(&mut self,ctx: &mut Context<Self>) {
        let bean_factory = BeanFactory::new_by_core(ctx.address());
        let inject_event = FactoryEvent::Inject { 
            factory: bean_factory.clone(), 
            data:  FactoryData(Arc::new(self.component_map.clone())),
        };
        let complete_event = FactoryEvent::Complete { factory: bean_factory };
        self.do_notify_event(inject_event);
        self.do_notify_event(complete_event);
    }
}

impl Actor for BeanFactoryCore {
    type Context = Context<Self>;

    fn started(&mut self,_ctx: &mut Self::Context) {
        log::info!("BeanFactoryCore started")
    }
}

impl Handler<RegisterBean> for BeanFactoryCore {
    type Result = ();
    fn handle(&mut self, msg: RegisterBean, ctx: &mut Self::Context) -> Self::Result {
        self.bean_map.insert(msg.type_name, msg.bean);
    }
}

impl Handler<InitFactory> for BeanFactoryCore {
    type Result=();

    fn handle(&mut self, _msg: InitFactory, ctx: &mut Self::Context) -> Self::Result {
        self.init();
        self.inject(ctx);
    }
}

impl Handler<QueryBean> for BeanFactoryCore {
    type Result=Option<Arc<DynAny>>;

    fn handle(&mut self, msg: QueryBean, ctx: &mut Self::Context) -> Self::Result {
        self.component_map.get(&msg.0).map(|e|e.clone())
    }
}




#[derive(Clone)]
pub struct BeanFactory {
    core_addr: Addr<BeanFactoryCore>,
}

impl BeanFactory {
    pub fn new_by_core(core_addr: Addr<BeanFactoryCore>) -> Self {
        Self {
            core_addr
        }
    }

    pub fn register(&self,bean:RegisterBean) {
        self.core_addr.do_send(bean);
    }

    pub fn init(&self) {
        self.core_addr.do_send(InitFactory);
    }

    pub async fn get_actor_by_name<T:Actor>(&self,name:&str) -> Option<Addr<T>> {
        match self.core_addr.send(QueryBean(name.to_owned())).await {
            Ok(v) => {
                v.map(|x|x.downcast::<Addr<T>>().ok())
                .flatten().map(|x|x.as_ref().clone())
            },
            Err(_) => None,
        }
    }

    pub async fn get_actor<T:Actor>(&self) -> Option<Addr<T>> {
        self.get_actor_by_name(type_name::<T>()).await
    }

    pub async fn get_bean_by_name<T:'static + Send + Sync>(&self,name:&str) -> Option<Arc<T>> {
        match self.core_addr.send(QueryBean(name.to_owned())).await {
            Ok(v) => {
                v.map(|x|x.downcast::<T>().ok())
                .flatten()
            },
            Err(_) => None,
        }
    }

    pub async fn get_bean<T:'static + Send + Sync>(&self) -> Option<Arc<T>> {
        self.get_bean_by_name(type_name::<T>()).await
    }
}
