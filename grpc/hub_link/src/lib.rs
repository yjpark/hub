#![feature(associated_type_defaults)]
#![feature(type_alias_impl_trait)]

pub mod grpc {
    tonic::include_proto!("google.rpc");
}
pub mod link {
    tonic::include_proto!("edger.hub.v1.link");
}
pub mod proto;
pub mod hub_session;