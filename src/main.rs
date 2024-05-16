use GeoHarbour::get_caps::{Wms, WebMappingService};

#[tokio::main]
async fn main() {
    let url =
  "https://bbmaps.itu.int/geoserver/wms?request=GetCapabilities&service=WMS&version=1.3.0".to_string();
  let capa = WebMappingService::from_url(url.clone()).unwrap()
        .get_capabilities().await.expect("Failure during GetCapabilities call");
  println!("{:?}", capa);
  assert_eq!(capa.service.name, "WMS");
  assert_eq!(capa.service.title, "GeoServer Web Map Service");
}

