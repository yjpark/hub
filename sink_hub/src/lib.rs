pub use link_hub::grpc;

pub mod proto {
    pub use link_hub::proto::*;

    tonic::include_proto!("edger.hub.v1");
}

