fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .build_client(false)
        //.type_attribute("edger.hub.v1.link.AppRequest", "#[derive(serde::Serialize)]")
        .compile(&["../proto/v1/echo.proto"], &["../proto/v1"])
        .unwrap();
    Ok(())
}

