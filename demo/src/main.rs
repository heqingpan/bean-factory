use std::{collections::HashMap, sync::Arc};

use actix::prelude::*;
use bean_factory::bean;
use bean_factory::BeanDefinition;
use bean_factory::BeanFactory;
use bean_factory::Inject;

#[bean(actor)]
#[derive(Default)]
pub struct ConfigService {
    pub(crate) config_map: HashMap<Arc<String>, Arc<String>>,
}

impl Actor for ConfigService {
    type Context = Context<Self>;
}

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<ConfigResult>")]
pub enum ConfigCmd {
    Set(Arc<String>, Arc<String>),
    Query(Arc<String>),
}

pub enum ConfigResult {
    None,
    Value(Arc<String>),
}

impl Handler<ConfigCmd> for ConfigService {
    type Result = anyhow::Result<ConfigResult>;

    fn handle(&mut self, msg: ConfigCmd, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            ConfigCmd::Set(key, val) => {
                self.config_map.insert(key, val);
                Ok(ConfigResult::None)
            }
            ConfigCmd::Query(key) => {
                if let Some(v) = self.config_map.get(&key) {
                    Ok(ConfigResult::Value(v.clone()))
                } else {
                    Ok(ConfigResult::None)
                }
            }
        }
    }
}

#[bean(inject)]
#[derive(Default)]
pub struct ConfigApi {
    pub(crate) config_service: Option<Addr<ConfigService>>,
}

impl Inject for ConfigApi {
    type Context = Context<Self>;

    fn inject(
        &mut self,
        factory_data: bean_factory::FactoryData,
        _factory: bean_factory::BeanFactory,
        _ctx: &mut Self::Context,
    ) {
        self.config_service = factory_data.get_actor();
        if self.config_service.is_some() {
            println!("ConfigApi:inject success");
        } else {
            println!("ConfigApi:inject none");
        }
    }

    fn complete(&mut self, _ctx: &mut Self::Context) {
        println!("ConfigApi:inject complete");
    }
}

impl Actor for ConfigApi {
    type Context = Context<Self>;
}

impl Handler<ConfigCmd> for ConfigApi {
    type Result = ResponseActFuture<Self, anyhow::Result<ConfigResult>>;

    fn handle(&mut self, msg: ConfigCmd, _ctx: &mut Self::Context) -> Self::Result {
        let config_service = self.config_service.clone();
        let fut = async {
            if let Some(addr) = config_service {
                println!("inject success. use inject config_service handle msg");
                addr.send(msg).await?
            } else {
                Err(anyhow::anyhow!("inject failed. config_service is none"))
            }
        }
        .into_actor(self); //.map(|r,_act,_ctx|{r});
        Box::pin(fut)
    }
}

#[actix::main]
async fn main() -> anyhow::Result<()> {
    let factory = BeanFactory::new();
    factory.register(BeanDefinition::actor_with_inject_from_default::<ConfigApi>());
    factory.register(BeanDefinition::actor_from_default::<ConfigService>());
    factory.init();
    let api_addr: Addr<ConfigApi> = factory.get_actor().await.unwrap();
    let key = Arc::new("key".to_owned());
    api_addr.do_send(ConfigCmd::Set(
        key.clone(),
        Arc::new("test value".to_owned()),
    ));
    match api_addr.send(ConfigCmd::Query(key.clone())).await?? {
        ConfigResult::None => println!("not found value"),
        ConfigResult::Value(val) => println!("query value:{}", &val),
    };
    Ok(())
}
