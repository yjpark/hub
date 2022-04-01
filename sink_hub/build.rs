fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        //.type_attribute("edger.hub.v1.link.AppRequest", "#[derive(serde::Serialize)]")
        .compile(&["../proto/v1/sink.proto"], &["../proto/v1"])
        .unwrap();
    Ok(())
}
