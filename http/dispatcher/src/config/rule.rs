use globset::{Glob, GlobSet, GlobSetBuilder};
use super::Action;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Pattern {
    pub hosts: Vec<String>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "::std::option::Option::is_none"))]
    pub ports: Option<Vec<u16>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "::std::option::Option::is_none"))]
    pub paths: Option<Vec<String>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    pub hosts_matcher: GlobSet,
    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    pub paths_matcher: Option<GlobSet>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Rule {
    pub pattern: Pattern,
    pub action: Action,
}

impl Pattern {
    fn build_matcher(patterns: &Vec<String>) -> GlobSet {
        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            builder.add(Glob::new(pattern).expect("failed to create glob"));
        }
        builder.build().expect("failed to create matcher")
    }

    pub fn new(hosts: Vec<String>, ports: Option<Vec<u16>>, paths: Option<Vec<String>>) -> Self {
        let hosts_matcher = Self::build_matcher(&hosts);
        let paths_matcher = paths.as_ref().map(|x| Self::build_matcher(&x));
        Self {
            hosts, 
            ports,
            paths,
            hosts_matcher,
            paths_matcher,
        }
    }

    pub fn is_match_uri(&self, uri: &http::Uri) -> bool {
        let Some(host) = uri.host() else {
            return false;
        };
        if !self.hosts_matcher.is_match(host) {
            return false;
        }
        if let Some(ports) = &self.ports {
            let Some(port) = uri.port_u16() else {
                return false;
            };
            if !ports.contains(&port) {
                return false;
            }
        }
        if let Some(paths_matcher) = &self.paths_matcher {
            let path = uri.path();
            if !paths_matcher.is_match(path) {
                return false;
            }
        }
        true
    }

    pub fn is_match_url(&self, url: &reqwest::Url) -> bool {
        let Some(host) = url.host() else {
            return false;
        };
        if !self.hosts_matcher.is_match(host.to_string().as_str()) {
            return false;
        }
        if let Some(ports) = &self.ports {
            let Some(port) = url.port() else {
                return false;
            };
            if !ports.contains(&port) {
                return false;
            }
        }
        if let Some(paths_matcher) = &self.paths_matcher {
            let path = url.path();
            if !paths_matcher.is_match(path) {
                return false;
            }
        }
        true
    }
}

impl Rule {
    pub fn action_uri(&self, uri: &http::Uri) -> Option<&Action> {
        if self.pattern.is_match_uri(uri) {
            Some(&self.action)
        } else {
            None
        }
    }

    pub fn action_url(&self, url: &reqwest::Url) -> Option<&Action> {
        if self.pattern.is_match_url(url) {
            Some(&self.action)
        } else {
            None
        }
    }
}