use k8s_openapi::api::core::v1::{ResourceRequirements, Toleration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub hydra_pod: HydraPodConfig,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct HydraPodConfig {
    pub namespace: String,
    pub image: String,
    pub configmap_name: String,
    pub external_domain: String,
    pub resources: Option<ResourceRequirements>,
    pub tolerations: Option<Vec<Toleration>>,
}
