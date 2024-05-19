use GeoHarbour::get_caps::{Wmts, WebMappingTileService};

#[tokio::main]
async fn main() {
    let url =
  "https://bbmaps.itu.int/".to_string();
  let capa = WebMappingTileService::from_url(url.clone()).unwrap()
        .get_capabilities().await.expect("Failure during GetCapabilities call");
  // assert_eq!(capa.service.name, "WMS");
  // assert_eq!(capa.service.title, "GeoServer Web Map Service");
  println!("{:?}", capa);
}

