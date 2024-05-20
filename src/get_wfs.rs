// Code for fetching WFS features from a server
use async_trait::async_trait;
use quick_xml::de::from_str;
use serde::{Deserialize, Serialize};
use url::Url;
use tokio::fs::File;
use std::path::Path;
use tokio::io::AsyncWriteExt;

#[async_trait(?Send)]
pub trait Wfs {
    async fn get_wfs_features(&mut self) -> anyhow::Result<WFSCapabilities>;

    async fn write_to_file(data: &str, filename: &str) -> std::io::Result<()>;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WFSCapabilities {
    #[serde(rename = "ServiceIdentification")]
    pub service_identification: ServiceIdentification,
    #[serde(rename = "ServiceProvider")]
    pub service_provider: ServiceProvider,
    #[serde(rename = "OperationsMetadata")]
    pub operations_metadata: OperationsMetadata,
    #[serde(rename = "FeatureTypeList")]
    pub list: FeatureTypeList,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServiceIdentification {
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Abstract")]
    pub abstract_desc: Option<String>,
    #[serde(rename = "Keywords")]
    pub keywords: Keywords,
    #[serde(rename = "ServiceType")]
    pub service_type: String,
    #[serde(rename = "ServiceTypeVersion")]
    pub service_type_version: String,
    #[serde(rename = "Fees")]
    pub fees: String,
    #[serde(rename = "AccessConstraints")]
    pub access_constraints: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Keywords {
    #[serde(rename = "Keyword")]
    pub keyword: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServiceProvider {
    #[serde(rename = "ProviderName")]
    pub provider_name: String,
    #[serde(rename = "ServiceContact")]
    pub service_contact: ServiceContact,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServiceContact {
    #[serde(rename = "IndividualName")]
    pub individual_name: String,
    #[serde(rename = "PositionName")]
    pub position_name: Option<String>,
    #[serde(rename = "ContactInfo")]
    pub contact_info: ContactInfo,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContactInfo {
    #[serde(rename = "Phone")]
    pub phone: Phone,
    #[serde(rename = "Address")]
    pub address: Address,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Phone {
    #[serde(rename = "Voice")]
    pub voice: Option<String>,
    #[serde(rename = "Facsimile")]
    pub facsimile: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Address {
    #[serde(rename = "DeliveryPoint")]
    pub delivery_point: Option<String>,
    #[serde(rename = "City")]
    pub city: String,
    #[serde(rename = "AdministrativeArea")]
    pub administrative_area: Option<String>,
    #[serde(rename = "PostalCode")]
    pub postal_code: Option<String>,
    #[serde(rename = "Country")]
    pub country: String,
    #[serde(rename = "ElectronicMailAddress")]
    pub electronic_mail_address: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OperationsMetadata {
    #[serde(rename = "Operation")]
    pub operations: Vec<Operation>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Operation {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "DCP")]
    pub dcp: Vec<DCP>,
    #[serde(rename = "Parameter")]
    pub parameters: Option<Vec<Parameter>>,
    #[serde(rename = "Constraint")]
    pub constraints: Option<Vec<Constraint>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DCP {
    #[serde(rename = "HTTP")]
    pub http: HTTP,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HTTP {
    #[serde(rename = "Get")]
    pub get: Option<HTTPMethod>,
    #[serde(rename = "Post")]
    pub post: Option<HTTPMethod>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HTTPMethod {
    #[serde(rename = "@href")]
    pub href: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Parameter {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "AllowedValues")]
    pub allowed_values: AllowedValues,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AllowedValues {
    #[serde(rename = "Value")]
    pub values: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Constraint {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "NoValues")]
    pub no_values: Option<String>,
    #[serde(rename = "DefaultValue")]
    pub default_value: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FeatureTypeList {
    #[serde(rename = "FeatureType")]
    pub list: Vec<FeatureType>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FeatureType {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Keywords")]
    pub keywords: Keywords,
    #[serde(rename = "WGS84BoundingBox")]
    pub bbox: BoundingBox,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BoundingBox {
    #[serde(rename = "LowerCorner")]
    lower_corner: String,
    #[serde(rename = "UpperCorner")]
    upper_corner: String,
}

pub struct WebFeatureService {
    pub version: String,
    url: Option<Url>,
    raw_xml: Option<String>,
}

impl WebFeatureService {
    pub fn from_string(xml: String) -> Self {
        WebFeatureService {
            version: "1.0.0".to_string(),
            url: None,
            raw_xml: Some(xml),
        }
    }

    pub fn from_url(url: String) -> anyhow::Result<Self> {
        let url = Url::parse(&url)?;
        let mut new_url = url.join("/geoserver/wfs").unwrap();
        new_url
            .query_pairs_mut()
            .append_pair("REQUEST", "GetCapabilities")
            .append_pair("SERVICE", "WFS");
        println!("{}", new_url);
        Ok(WebFeatureService {
            version: "1.0.0".to_string(),
            url: Some(new_url),
            raw_xml: None,
        })
    }
}

#[async_trait(?Send)]
impl Wfs for WebFeatureService {
    async fn get_wfs_features(&mut self) -> anyhow::Result<WFSCapabilities> {
        match &self.raw_xml {
            None => match reqwest::get(self.url.clone().unwrap()).await?.text().await {
                Ok(xml) => {
                    self.raw_xml = Some(xml);
                    self.get_wfs_features().await
                }
                Err(e) => Err(anyhow::Error::msg(e)),
            },
            Some(xml) => match from_str::<WFSCapabilities>(xml) {
                Ok(w) => Ok(w),
                Err(e) => Err(anyhow::Error::msg(e)),
            },
        }
    }

    async fn write_to_file(json_data: &str, filename: &str) -> std::io::Result<()> {
        let filepath = Path::new(filename);
        let mut file = File::create(filepath).await?;
        file.write_all(json_data.as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }
}
