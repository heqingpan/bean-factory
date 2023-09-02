pub mod factory;

pub use factory::{
    model::{BeanDefinition, FactoryEvent, Inject,FactoryData},
    BeanFactory, BeanFactoryCore,
};
pub use actix::prelude::{Handler,Addr};

pub use inventory::submit;
pub use bean_factory_derive::*;


/// 注册所有声明的beans，并初始化工场，开始注入依赖Bean
pub fn setup_submitted_beans(factory:&BeanFactory) {
    for bean in inventory::iter::<BeanDefinition> {
        factory.register(bean.clone());
    }
    factory.init();
}


/// 只注册不初始化
/// 用于想要二次处理bean的场景
pub fn register_beans(factory:&BeanFactory) {
    for bean in inventory::iter::<BeanDefinition> {
        factory.register(bean.clone());
    }
}
