pub mod factory;

pub use actix::prelude::{Addr, Handler};
pub use factory::{
    model::{BeanDefinition, FactoryData, FactoryEvent, Inject},
    BeanFactory, BeanFactoryCore,
};

pub use bean_factory_derive::*;
pub use inventory::iter;
pub use inventory::submit;

/// 注册所有声明的beans，并初始化工场，开始注入依赖Bean
pub fn setup_submitted_beans(factory: &BeanFactory) {
    for bean in iter::<BeanDefinition> {
        factory.register(bean.clone());
    }
    factory.init();
}

//获取所有注解声明bean列表
pub fn get_bean_definitions() -> Vec<BeanDefinition> {
    let mut beans = vec![];
    for bean in iter::<BeanDefinition> {
        beans.push(bean.clone());
    }
    beans
}

/// 只注册不初始化
/// 用于想要二次处理bean的场景
pub fn register_beans(factory: &BeanFactory) {
    for bean in iter::<BeanDefinition> {
        factory.register(bean.clone());
    }
}
