use std::{any::type_name, collections::HashMap, sync::Arc, vec};

use actix::prelude::*;
//use actix::dev::ToEnvelope;

use self::model::{
    BeanDefinition, BeanFactoryCmd, BeanFactoryResult, DynAny, FactoryData, FactoryEvent,
    InitFactory, QueryBean,
};

pub mod model;

fn spawn_start(inner: BeanFactoryCore) -> Addr<BeanFactoryCore> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let rt = System::new();
        let addrs = rt.block_on(async { inner.start() });
        tx.send(addrs).unwrap();
        rt.run().unwrap();
    });
    rx.recv().unwrap()
}

#[derive(Default)]
pub struct BeanFactoryCore {
    bean_map: HashMap<String, Arc<DynAny>>,
    bean_definition_map: HashMap<String, BeanDefinition>,
}

impl BeanFactoryCore {
    pub fn spawn_start(self) -> Addr<Self> {
        spawn_start(self)
    }

    fn init(&mut self) {
        for (_, bean) in &self.bean_definition_map {
            match &bean.provider {
                model::Provieder::Fn(f) => {
                    if let Some(v) = f() {
                        self.bean_map.insert(bean.type_name.to_owned(), v);
                    }
                }
                model::Provieder::Value(v) => {
                    self.bean_map.insert(bean.type_name.to_owned(), v.clone());
                }
            }
        }
    }

    /*
    fn do_notify<T>(c:Arc<DynAny>,event: FactoryEvent)
    where T: Actor<Context = Context<T>> + Handler<FactoryEvent>,
        <T as Actor>::Context: AsyncContext<T> + ToEnvelope<T,FactoryEvent>
    {
        c.downcast::<Addr<T>>().ok().map(|e|e.do_send(event));
    }
    */

    fn do_notify_event(&mut self, event: FactoryEvent) {
        for (name, bean) in &self.bean_definition_map {
            /*
            if !bean.inject {
                continue;
            }
            */
            //self.bean_map.get(name).map(|e| Self::do_notify2(e.clone(), event));
            match (self.bean_map.get(name), bean.notify.as_ref()) {
                (Some(c), Some(notify)) => notify(c.clone(), event.clone()),
                (_, _) => {}
            }
        }
    }

    fn inject(&mut self, ctx: &mut Context<Self>) {
        let inject_event = FactoryEvent::Inject {
            factory: BeanFactory::new_by_core(ctx.address()),
            factory_data: FactoryData(Arc::new(self.bean_map.clone())),
        };
        let complete_event = FactoryEvent::Complete;
        self.do_notify_event(inject_event);
        self.do_notify_event(complete_event);
    }
}

impl Actor for BeanFactoryCore {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        log::info!("BeanFactoryCore started")
    }
}

impl Handler<BeanDefinition> for BeanFactoryCore {
    type Result = ();
    fn handle(&mut self, msg: BeanDefinition, _ctx: &mut Self::Context) -> Self::Result {
        self.bean_definition_map
            .insert(msg.type_name.to_owned(), msg);
    }
}

impl Handler<InitFactory> for BeanFactoryCore {
    type Result = ();

    fn handle(&mut self, _msg: InitFactory, ctx: &mut Self::Context) -> Self::Result {
        self.init();
        self.inject(ctx);
    }
}

impl Handler<QueryBean> for BeanFactoryCore {
    type Result = Option<Arc<DynAny>>;

    fn handle(&mut self, msg: QueryBean, _ctx: &mut Self::Context) -> Self::Result {
        self.bean_map.get(&msg.0).map(|e| e.clone())
    }
}

impl Handler<BeanFactoryCmd> for BeanFactoryCore {
    type Result = Option<BeanFactoryResult>;

    fn handle(&mut self, msg: BeanFactoryCmd, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            BeanFactoryCmd::Init => {
                self.init();
                self.inject(ctx);
                Some(BeanFactoryResult::None)
            }
            BeanFactoryCmd::QueryBean(name) => {
                let v = self.bean_map.get(&name).map(|e| e.clone());
                Some(BeanFactoryResult::Bean(v))
            }
            BeanFactoryCmd::QueryBeanNames => {
                let v = self
                    .bean_definition_map
                    .keys()
                    .into_iter()
                    .cloned()
                    .collect();
                Some(BeanFactoryResult::BeanNames(v))
            }
        }
    }
}

#[derive(Clone)]
pub struct BeanFactory {
    pub core_addr: Addr<BeanFactoryCore>,
}

impl BeanFactory {
    /// 在actic环境下创建BeanFactory
    pub fn new() -> Self {
        BeanFactory {
            core_addr: BeanFactoryCore::start_default(),
        }
    }

    /// 在普通环境下，在一个新建的actix线程创建BeanFactoryCore,再创建BeanFactory
    pub fn spawn_new() -> Self {
        BeanFactory {
            core_addr: BeanFactoryCore::default().spawn_start(),
        }
    }

    pub fn new_by_core(core_addr: Addr<BeanFactoryCore>) -> Self {
        Self { core_addr }
    }

    /// 注册bean
    /// 只注册没有执行创建实例
    pub fn register(&self, bean: BeanDefinition) {
        self.core_addr.do_send(bean);
    }

    /// 初始化工厂
    /// 创建bean实例
    /// 并触发依赖注入
    pub fn init(&self) {
        self.core_addr.do_send(InitFactory);
    }

    pub async fn query_bean_names(&self) -> Vec<String> {
        match self.core_addr.send(BeanFactoryCmd::QueryBeanNames).await {
            Ok(resp) => resp.map_or(vec![], |r| match r {
                BeanFactoryResult::BeanNames(v) => v,
                _ => vec![],
            }),
            Err(_) => vec![],
        }
    }

    pub async fn get_actor_by_name<T: Actor>(&self, name: &str) -> Option<Addr<T>> {
        match self.core_addr.send(QueryBean(name.to_owned())).await {
            Ok(v) => v
                .map(|x| x.downcast::<Addr<T>>().ok())
                .flatten()
                .map(|x| x.as_ref().clone()),
            Err(_) => None,
        }
    }

    pub async fn get_actor<T: Actor>(&self) -> Option<Addr<T>> {
        self.get_actor_by_name(type_name::<T>()).await
    }

    pub async fn get_bean_by_name<T: 'static + Send + Sync>(&self, name: &str) -> Option<Arc<T>> {
        match self.core_addr.send(QueryBean(name.to_owned())).await {
            Ok(v) => v.map(|x| x.downcast::<T>().ok()).flatten(),
            Err(_) => None,
        }
    }

    pub async fn get_bean<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        self.get_bean_by_name(type_name::<T>()).await
    }
}
