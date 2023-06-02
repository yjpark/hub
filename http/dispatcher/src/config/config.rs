use super::Action;
use super::Proxy;
use super::Rule;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Config {
    pub proxies: Vec<Proxy>,
    pub rules: Vec<Rule>,
    pub fallback: Action,
}

impl Config {
    pub fn action_uri(&self, uri: &http::Uri) -> &Action {
        for rule in &self.rules {
            if let Some(action) = rule.action_uri(uri) {
                return action;
            }
        }
        &self.fallback
    }

    pub fn action_url(&self, url: &reqwest::Url) -> &Action {
        for rule in &self.rules {
            if let Some(action) = rule.action_url(url) {
                return action;
            }
        }
        &self.fallback
    }
}