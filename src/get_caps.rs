use async_trait::async_trait;
use quick_xml::de::from_str;
use serde::Deserialize;
use url::Url;

/// Behaviour for a Web Mapping Tile Service endpoint as per the specification.
#[async_trait(?Send)]
pub trait Wmts {
    /// The GetCapabilities request
    async fn get_capabilities(&mut self) -> anyhow::Result<Capabilities>;
}

#[derive(Clone, Debug, Deserialize)]
pub struct Capabilities {
    #[serde(rename = "Contents")]
    pub capabilities: Contents,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Contents {
    #[serde(rename = "Layer")]
    pub layers: Vec<Layer>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Layer {
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Abstract")]
    pub abstr: Option<String>,
    #[serde(rename = "Identifier")]
    pub identifier: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WebMappingTileService {
    pub version: String,
    url: Option<Url>,
    raw_xml: Option<String>,
}

impl WebMappingTileService {
    /// Use the raw XML string as this "endpoint" for service calls
    pub fn from_string(xml: String) -> Self {
        WebMappingTileService {
            version: "1.0.0".to_string(),
            url: None,
            raw_xml: Some(xml),
        }
    }

    /// Use the given URL as the endpoint for service calls
    /// The URL should be the base URL for a WMTS Service. Request parameters essential for
    /// WMTS requests will be replaced accordingly.
    pub fn from_url(url: String) -> anyhow::Result<Self> {
        let url = Url::parse(&url)?;
        let mut new_url = url.join("/geoserver/gwc/service/wmts").unwrap();
        new_url
            .query_pairs_mut()
            .append_pair("REQUEST", "GetCapabilities")
            .append_pair("SERVICE", "WMTS");
        println!("{}", new_url);
        Ok(WebMappingTileService {
            version: "1.0.0".to_string(),
            url: Some(new_url),
            raw_xml: None,
        })
    }
}

#[async_trait(?Send)]
impl Wmts for WebMappingTileService {
    /// The WMTS GetCapabilities request
    async fn get_capabilities(&mut self) -> anyhow::Result<Capabilities> {
        match &self.raw_xml {
            None => match reqwest::get(self.url.clone().unwrap()).await?.text().await {
                Ok(xml) => {
                    self.raw_xml = Some(xml);
                    self.get_capabilities().await
                }
                Err(e) => Err(anyhow::Error::msg(e)),
            },
            Some(xml) => match from_str::<Capabilities>(xml) {
                Ok(w) => Ok(w),
                Err(e) => Err(anyhow::Error::msg(e)),
            },
        }
    }
}
