use std::fs::File;
use std::io::Write;
use anyhow::Context;
use async_trait::async_trait;
use serde_xml_rs::from_reader;
use std::collections::HashSet;
use url::Url;
use serde::{Deserialize, Serialize};

// #[tokio::main]
// async fn main() {
//   let capabilities_url = format!("{}/geoserver/gwc/service/wmts?request=getcapabilities", geo_server_url);
//   let url = "https://ows.terrestris.de/osm/service?";
//   let bytes = WebMappingService::from_url(url.to_string()).unwrap().get_map(
//     GetMapParameters {
//       layers: vec!["OSM-WMS".to_string()],
//       srs: "EPSG:4326".to_string(),
//       bbox: BoundingBox {
//           srs: "EPSG:4326".to_string(),
//           minx: -180.0,
//           miny: -90.0,
//           maxx: 180.0,
//           maxy: 90.0,
//       },
//       ..GetMapParameters::default()
//     }).await.unwrap();
//   assert_ne!(bytes.len(), 0);
//   let mut file = File::create("/tmp/terrestris-get-map.png").unwrap();
//   assert!(file.write_all(&bytes).is_ok());
// }

/// Behaviour for a Web Mapping Service endpoint as per the specification.
#[async_trait(?Send)]
pub trait Wms {
  /// The GetCapabilities request
  async fn get_capabilities(&mut self) -> anyhow::Result<GetCapabilities>;
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct GetCapabilities {
  #[serde(rename = "Service", default)]
  pub service: Service,
  #[serde(rename = "Capability", default)]
  pub capability: Capability,
}

/// General service metadata
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Service {
  #[serde(rename = "Abstract", default)]
  pub abstr: String,
  #[serde(rename = "Name", default)]
  pub name: String,
  #[serde(rename = "Title", default)]
  pub title: String,
  #[serde(rename = "MaxWidth", default)]
  pub max_width: Option<u32>,
  #[serde(rename = "MaxHeight", default)]
  pub max_height: Option<u32>,
}

/// The root element
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Capability {
  #[serde(rename = "Layer", default)]
  pub layer: Option<Layer>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Layer {
  #[serde(rename = "Abstract", default)]
  pub abstr: String,
  /// The LatLonBoundingBox element
  #[serde(rename = "LatLonBoundingBox", default)]
  pub ll_bbox: Option<LatLonBoundingBox>,
  #[serde(rename = "BoundingBox", default)]
  pub bbox: Vec<BoundingBox>,
  #[serde(rename = "Name", default)]
  pub name: String,
  #[serde(rename = "CRS", default)]
  crs: HashSet<String>,
  #[serde(rename = "SRS", default)]
  srs: HashSet<String>, // 1.1.0 compat

  #[serde(rename = "KeywordList", default)]
  pub keyword_list: KeywordList, 

  #[serde(rename = "Title", default)]
  pub title: String,
  #[serde(rename = "Layer", default)]
  pub layers: Vec<Layer>,
}

impl Layer {
  /// The combined CRS values for this Layer
  pub fn crs(&self) -> HashSet<String> {
    let mut combined_crs = HashSet::new();
    combined_crs.extend(self.crs.clone());
    combined_crs.extend(self.srs.clone());
    combined_crs
  }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct KeywordList {
  #[serde(rename = "Keyword", default)]
  pub keyword: Vec<String>
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LatLonBoundingBox {
  pub minx: f32,
  pub miny: f32,
  pub maxx: f32,
  pub maxy: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct BoundingBox {
  pub minx: f32,
  pub miny: f32,
  pub maxx: f32,
  pub maxy: f32,

  #[serde(rename = "SRS", default)]
  pub srs: String,
}

impl Default for BoundingBox {
  fn default() -> Self {
    BoundingBox {
      srs: "EPSG:4326".to_string(),
      minx: -180.0,
      miny: -90.0,
      maxx: 180.0,
      maxy: 90.0,
    }
  }
}

impl BoundingBox {
  /// Yield minx,miny,maxx,maxy as-per the usual formatting of a bounding box
  fn to_str(&self) -> String {
    format!("{},{},{},{}", self.minx, self.miny, self.maxx, self.maxy)
  }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WebMappingService {
  pub version: String,
  url: Option<Url>,
  raw_xml: Option<String>,
}

impl WebMappingService {
  /// Use the raw XML string as this "endpoint" for service calls
  fn from_string(xml: String) -> Self {
    WebMappingService {
      version: "1.3.0".to_string(),
      url: None,
      raw_xml: Some(xml),
    }
  }

  /// Use the given URL as the endpoint for service calls
  /// The URL should be the base URL for a WMS Service. Request parameters essential for
  /// WMS requests will be replaced accordingly.
  pub fn from_url(url: String) -> anyhow::Result<Self> {
    let mut url = Url::parse(&url)?;
    url
      .query_pairs_mut()
      .append_pair("REQUEST", "GetCapabilities")
      .append_pair("SERVICE", "WMS");
    Ok(WebMappingService {
      version: "1.3.0".to_string(),
      url: Some(url),
      raw_xml: None,
    })
  }
}

#[async_trait(?Send)]
impl Wms for WebMappingService {
  /// The WMS GetCapabilities request
  async fn get_capabilities(&mut self) -> anyhow::Result<GetCapabilities> {
    match &self.raw_xml {
      None => match reqwest::get(self.url.clone().unwrap()).await?.text().await {
        Ok(xml) => {
          self.raw_xml = Some(xml);
          self.get_capabilities().await
        }
        Err(e) => Err(anyhow::Error::msg(e)),
      },
      Some(xml) => match from_reader(xml.as_bytes()) {
        Ok(w) => Ok(w),
        Err(e) => Err(anyhow::Error::msg(e)),
      },
    }
  }
}
