#![feature(associated_type_defaults)]
#![feature(type_alias_impl_trait)]

pub use link_hub::grpc;

pub mod link {
    pub use link_hub::proto::*;
}

pub mod proto;
pub mod error;
pub mod sink_authenticator;
pub mod sink_session;
pub mod sink_app;
pub mod sink_hub;