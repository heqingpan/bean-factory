//use std::{any::type_name, sync::Arc};

use actix::prelude::*;

use bean_factory::{
    setup_submitted_beans, ActorComponent, BeanDefinition, BeanFactory, BeanFactoryCore,
    FactoryData, Inject, InjectComponent,
};

struct Ping(usize);

impl Message for Ping {
    type Result = usize;
}

#[derive(Default, ActorComponent)]
struct FooActor {}

impl Actor for FooActor {
    type Context = Context<Self>;
}

impl Handler<Ping> for FooActor {
    type Result = usize;

    fn handle(&mut self, msg: Ping, _: &mut Context<Self>) -> Self::Result {
        println!("handle ping by FooActor,{}", msg.0);
        0
    }
}

/// Actor
#[derive(Default, InjectComponent)]
struct MyActor {
    count: usize,
    foo_addr: Option<Addr<FooActor>>,
}

impl Inject for MyActor {
    type Context = Context<Self>;
    fn inject(
        &mut self,
        factory_data: FactoryData,
        _factory: BeanFactory,
        _ctx: &mut Self::Context,
    ) {
        self.foo_addr = factory_data.get_actor();
        println!("MyActor inject");
    }

    fn complete(&mut self, _ctx: &mut Self::Context) {
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
        self.foo_addr.as_ref().map(|x| x.do_send(Ping(self.count)));
        self.count
    }
}

/*
impl Handler<FactoryEvent> for MyActor {
    type Result = ();

    fn handle(&mut self, msg: FactoryEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            FactoryEvent::Inject {
                factory,
                factory_data,
            } => {
                Inject::inject(self, factory_data, factory, ctx);
            }
            FactoryEvent::Complete => {
                Inject::complete(self, ctx);
            }
        }
    }
}
 */

#[actix::test]
async fn register_001() {
    let factory_core = BeanFactoryCore::start_default();
    let factory = BeanFactory::new_by_core(factory_core);
    /*
    let name = type_name::<MyActor>();
    let bean = BeanDefinition {
        type_name: name.to_string(),
        provider: Arc::new(||{Some(Arc::new(MyActor::default().start()))}),
        notify: Some(Arc::new(|a,event| {
            a.downcast::<Addr<MyActor>>().ok().map(|e|e.do_send(event));
        })),
        //inject: true,
    };
     */
    let bean = BeanDefinition::actor_with_inject_from_default::<MyActor>();
    factory.register(bean);
    /*
    let name = type_name::<FooActor>();
    let bean=BeanDefinition {
        type_name: name.to_string(),
        provider: Arc::new(||{Some(Arc::new(FooActor::default().start()))}),
        notify: None,
        //inject: true,
    };
     */
    let bean = BeanDefinition::actor_from_default::<FooActor>();
    factory.register(bean);
    println!("------001");
    factory.do_init();
    println!("------002");
    take(&factory).await;
    take(&factory).await;
    take(&factory).await;
    take(&factory).await;
}

#[actix::test]
async fn register_002() {
    let factory = BeanFactory::new();
    setup_submitted_beans(&factory);
    let bean_names = factory.query_bean_names().await;
    println!("all beans size:{}", bean_names.len());
    for item in &bean_names {
        println!("\t{}", item)
    }
    take(&factory).await;
    take(&factory).await;
    take(&factory).await;
    take(&factory).await;
}

async fn take(factory: &BeanFactory) {
    let component: Addr<MyActor> = factory.get_actor().await.unwrap();
    let c = component.send(Ping(2)).await.unwrap();
    println!("take result: {}", c)
}
