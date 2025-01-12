use std::io;
use reqwest::Error;
use serde_json::{Value};
use io::stdin;

#[derive(Debug, Clone)]
struct Version {
    id: String,
    _type: String,
    url: String,
    time: String,
}

impl Version {
    fn new(id: String, _type: String, url: String, time: String) -> Self {
        Version {
            id,
            _type,
            url,
            time,
        }
    }
    fn display(&self) -> String {
        match self._type.as_str() {
            "release" => format!("{}", self.id),
            "snapshot" => format!("Snapshot {}", self.id),
            "old_alpha" => format!("Alpha {}", self.id),
            "old_beta" => format!("Beta {}", self.id),
            _ => panic!("Unknown type {}", self._type),
        }
    }
}

#[derive(Debug, Clone)]
struct VersionManifest {
    versions: Vec<Version>,
    snapshots: Vec<Version>,
    old_betas: Vec<Version>,
    old_alphas: Vec<Version>,
    latest: Version,
}

impl VersionManifest {
    fn new(json: Value) -> Self {
        let mut versions: Vec<Version> = Vec::new();
        let mut snapshots: Vec<Version> = Vec::new();
        let mut old_betas: Vec<Version> = Vec::new();
        let mut old_alphas: Vec<Version> = Vec::new();
        for v in json["versions"].as_array().unwrap() {
            match v["type"].as_str().unwrap() {
                "snapshot" => {
                    snapshots.push(Version::new(v["id"].as_str().unwrap().to_string(),
                                                v["type"].as_str().unwrap().to_string(),
                                                v["url"].as_str().unwrap().to_string(),
                                                v["releaseTime"].as_str().unwrap().to_string()));
                },
                "release" => {
                    versions.push(Version::new(v["id"].as_str().unwrap().to_string(),
                                                v["type"].as_str().unwrap().to_string(),
                                                v["url"].as_str().unwrap().to_string(),
                                                v["releaseTime"].as_str().unwrap().to_string()));
                },
                "old_alpha" => {
                    old_alphas.push(Version::new(v["id"].as_str().unwrap().to_string(),
                                                v["type"].as_str().unwrap().to_string(),
                                                v["url"].as_str().unwrap().to_string(),
                                                v["releaseTime"].as_str().unwrap().to_string()));
                },
                "old_beta" => {
                    old_betas.push(Version::new(v["id"].as_str().unwrap().to_string(),
                                                v["type"].as_str().unwrap().to_string(),
                                                v["url"].as_str().unwrap().to_string(),
                                                v["releaseTime"].as_str().unwrap().to_string()));
                },
                _ => panic!("Unknown version type: {}", v["type"].as_str().unwrap()),
            }
        }
        VersionManifest {
            versions,
            snapshots,
            old_betas,
            old_alphas,
            latest: Version::new(json["versions"][0]["id"].as_str().unwrap().to_string(),
                                 json["versions"][0]["type"].as_str().unwrap().to_string(),
                                 json["versions"][0]["url"].as_str().unwrap().to_string(),
                                 json["versions"][0]["releaseTime"].as_str().unwrap().to_string()),
        }
    }

    fn get_latest(&self) -> &Version {
        &self.latest
    }

    fn get_version(&self, id: &str) -> Option<&Version> {
        match self.versions.iter().find(|v| v.id == id) {
            Some(v) => Some(v),
            None => {
                match self.snapshots.iter().find(|v| v.id == id) {
                    Some(v) => Some(v),
                    None => {
                        match self.old_betas.iter().find(|v| v.id == id) {
                            Some(v) => Some(v),
                            None => {
                                match self.old_alphas.iter().find(|v| v.id == id) {
                                    Some(v) => Some(v),
                                    None => Some(&self.latest)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn generate_version_choice(manifest: &VersionManifest) -> Version {
    println!("Choose release type: \n[0] Release\n[1] Snapshot\n[2] Beta\n[3] Alpha");
    let mut choice = String::new();
    stdin().read_line(&mut choice).expect("Failed to read line");
    let mut version_list = Vec::new();
    match choice.trim() {
        "0" => {
            version_list = manifest.versions.clone();
        },
        "1" => {
            version_list = manifest.snapshots.clone();
        },
        "2" => {
            version_list = manifest.old_betas.clone();
        },
        "3" => {
            version_list = manifest.old_alphas.clone();
        },
        _ => panic!("Unknown choice: {}", choice),
    }
    println!("Choose version: \n");
    for (i, val) in version_list.iter().enumerate() {
        println!("[{}] {}", i, val.id);
    }
    let mut ver_choice: String = String::new();
    stdin().read_line(&mut ver_choice).expect("Failed to read line");
    let ver_int: usize = ver_choice.trim().parse().expect("Not a valid integer");
    let version = version_list[ver_int].clone();
    version.clone()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let body = reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .await?
        .text()
        .await?;
    let json: Value = serde_json::from_str(body.as_str()).unwrap();
    let manifest: VersionManifest = VersionManifest::new(json);
    println!("{:#?}",generate_version_choice(&manifest));
    Ok(())
}
