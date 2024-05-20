use GeoHarbour::get_caps::{WebMappingTileService, Wmts};
use GeoHarbour::get_wfs::{WebFeatureService, Wfs};
use std::fs::File;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = "https://bbmaps.itu.int/".to_string();
    // let capa = WebMappingTileService::from_url(url.clone()).unwrap()
    //       .get_capabilities().await.expect("Failure during GetCapabilities call");
    // assert_eq!(capa.service.name, "WMS");
    // assert_eq!(capa.service.title, "GeoServer Web Map Service");
    let features = WebFeatureService::from_url(url.clone())
        .unwrap()
        .get_wfs_features()
        .await
        .expect("Failure during get Features call");
    println!("{:?}", features);

    // Serialize WFSCapabilities to JSON
    let json_data = serde_json::to_string(&features)?;

    // Write JSON data to a file asynchronously
    let filename = "C:/Users/SAMARTH/Desktop/rustprojects/GeoHarbour/wfs_capabilities.json";
    WebFeatureService::write_to_file(&json_data, filename);

    println!("File written to {}", filename);

    Ok(())
}

