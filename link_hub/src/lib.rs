#![feature(associated_type_defaults)]
#![feature(type_alias_impl_trait)]

pub mod grpc {
    tonic::include_proto!("google.rpc");
}
pub mod proto {
    tonic::include_proto!("edger.hub.v1");
}
pub mod error;
pub mod link_session;
pub mod link_authenticator;
pub mod hub_app;
pub mod link_hub;