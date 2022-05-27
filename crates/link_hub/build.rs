fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../../proto/google/rpc/error_details.proto")?;
    tonic_build::configure()
        //.type_attribute("edger.hub.v1.AppRequest", "#[derive(serde::Serialize)]")
        .compile(&["../../proto/v1/link.proto"], &["../../proto/v1"])
        .unwrap();
    Ok(())
}
