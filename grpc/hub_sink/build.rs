fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../proto/google/rpc/error_details.proto")?;
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        //.type_attribute("edger.hub.v1.link.AppRequest", "#[derive(serde::Serialize)]")
        .compile(&["../proto/v1/sink.proto"], &["../proto/v1"])
        .unwrap();
    Ok(())
}

