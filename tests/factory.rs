use std::{any::type_name, sync::Arc};

use actix::prelude::*;
use actix_inject::factory::{model::{IInject, FactoryEvent, RegisterBean, ActorBean}, BeanFactory, BeanFactoryCore};


struct Ping(usize);

impl Message for Ping {
    type Result = usize;
}

#[derive(Default)]
struct FooActor {

}

impl Actor for FooActor {
    type Context = Context<Self>;
}

impl Handler<Ping> for FooActor {
    type Result = usize;

    fn handle(&mut self, msg: Ping,_:&mut Context<Self>) -> Self::Result {
        println!("do ping by FooActor,{}",msg.0);
        0
    }
}

/// Actor
#[derive(Default)]
struct MyActor {
    count: usize,
    foo_addr: Option<Addr<FooActor>>,
}

impl IInject for MyActor {
    fn inject(&mut self,factory_data:actix_inject::factory::model::FactoryData,_factory:actix_inject::factory::BeanFactory) {
        self.foo_addr = factory_data.get_actor();
        println!("MyActor inject");
    }

    fn complete(&mut self,_factory:actix_inject::factory::BeanFactory) {
        println!("MyActor inject complete");
    }
}

/// Declare actor and its context
impl Actor for MyActor {
    type Context = Context<Self>;
}

/// Handler for `Ping` message
impl Handler<Ping> for MyActor {
    type Result = usize;

    fn handle(&mut self, msg: Ping, _: &mut Context<Self>) -> Self::Result {
        self.count += msg.0;
        self.foo_addr.as_ref().map(|x|x.do_send(Ping(self.count)));
        self.count
    }
}

impl Handler<FactoryEvent> for MyActor {
    type Result = ();

    fn handle(&mut self, msg: FactoryEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            FactoryEvent::Inject { factory, data } => {
                self.inject(data, factory)
            },
            FactoryEvent::Complete { factory } => {
                self.complete(factory)
            },
        }
    }
}

#[actix::test]
async fn register_foo() {
    let factory_core = BeanFactoryCore::start_default();
    let factory = BeanFactory::new_by_core(factory_core);
    let name = type_name::<MyActor>();
    let bean = RegisterBean{ 
        type_name: name.to_string(), 
        bean: ActorBean { 
            type_name: name.to_string(), 
            provider: Arc::new(||{Some(Arc::new(MyActor::default().start()))}), 
            notify: Some(Arc::new(|a,event| {
                a.downcast::<Addr<MyActor>>().ok().map(|e|e.do_send(event));
            })), 
            inject: true,
        }
    };
    factory.register(bean);
    let name = type_name::<FooActor>();
    let bean = RegisterBean{ 
        type_name: name.to_string(), 
        bean: ActorBean { 
            type_name: name.to_string(), 
            provider: Arc::new(||{Some(Arc::new(FooActor::default().start()))}), 
            notify: None, 
            inject: true,
        }
    };
    factory.register(bean);
    println!("------001");
    factory.init();
    println!("------002");
    take(&factory).await;
    take(&factory).await;
    take(&factory).await;
    take(&factory).await;
}

async fn take(factory:&BeanFactory) {
    let component:Addr<MyActor> = factory.get_actor().await.unwrap();
    let c = component.send(Ping(2)).await.unwrap();
    println!("take result: {}",c)
}