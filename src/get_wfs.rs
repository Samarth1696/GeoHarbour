// Code for fetching WFS features from a server

use std::error::Error;
use reqwest;
use geojson::FeatureCollection;
use std::fs::File;
use std::io::prelude::*;

pub fn get_wfs_features(wfs_url: &str, layer_name: &str) -> Result<(), Box<dyn Error>> {
    let mut url = String::from(wfs_url);
    url.push_str(&format!("?service=WFS&version=2.0.0&request=GetFeature&typeName={}&outputFormat=application/json", layer_name));
    let response = reqwest::blocking::get(&url)?.text()?;
    let feature_collection: FeatureCollection = serde_json::from_str(&response)?;

    // Do something with the feature collection, like writing it to a file
    let mut file = File::create(format!("{}.geojson", layer_name))?;
    let serialized = serde_json::to_string(&feature_collection)?;
    file.write_all(serialized.as_bytes())?;

    Ok(())
}
