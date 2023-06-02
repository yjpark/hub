use once_cell::sync::Lazy;

use hub_http_dispatcher::{Config, Dispatcher, config::{Proxy, Action}};

pub static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let config = create_dispatcher_config();
    Dispatcher::new(config)
});

pub fn create_dispatcher_config() -> Config {
    let proxy = Proxy::http("10.0.1.6", 29002);
    //let proxy = Proxy::socks5("10.0.1.6", 29001);
    Config {
        proxies: vec![proxy.clone()],
        rules: Vec::new(),
        fallback: Action::Proxy(proxy)
    }
}