use k8s_openapi::api::{
    apps::v1::{Deployment, DeploymentSpec},
    core::v1::{
        ConfigMapVolumeSource, Container, ContainerPort, EmptyDirVolumeSource, PodSpec,
        PodTemplateSpec, Service, ServicePort, ServiceSpec, Volume, VolumeMount,
    },
};
use kube::{api::ObjectMeta, CustomResource, ResourceExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::config::Config;

use super::controller::K8sConstants;

pub static HYDRA_DOOM_POD_FINALIZER: &str = "hydradoompod/finalizer";

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    kind = "HydraDoomPod",
    group = "hydra.doom",
    version = "v1alpha1",
    shortname = "hydradoompod",
    category = "hydradoom",
    plural = "hydradoompods",
    namespaced
)]
#[kube(status = "HydraDoomPodStatus")]
#[kube(printcolumn = r#"
        {"name": "Local URI", "jsonPath":".status.localUrl", "type": "string"}, 
        {"name": "External URI", "jsonPath": ".status.externalUrl", "type": "string"}
    "#)]
#[serde(rename_all = "camelCase")]
pub struct HydraDoomPodSpec {
    pub image: Option<String>,
    pub open_head_image: Option<String>,
    pub configmap: Option<String>,
    pub network_id: u8,
    pub seed_input: String,
    pub participant: String,
    pub party: String,
    pub commit_inputs: Vec<String>,
    pub blockfrost_key: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HydraDoomPodStatus {
    pub local_url: String,
    pub external_url: String,
}

impl HydraDoomPod {
    pub fn pod_name(&self) -> String {
        format!("hydra-pod-{}", self.name_any())
    }

    pub fn pod_labels(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("component".to_string(), "hydra-pod".to_string()),
            ("hydra-pod-id".to_string(), self.name_any()),
        ])
    }

    pub fn deployment(&self, config: &Config, constants: &K8sConstants) -> Deployment {
        let name = self.pod_name();
        let labels = self.pod_labels();
        let mut open_head_args = vec![
            "--network-id".to_string(),
            self.spec.network_id.to_string(),
            "--seed-input".to_string(),
            self.spec.seed_input.clone(),
            "--participant".to_string(),
            self.spec.participant.clone(),
            "--party".to_string(),
            self.spec.party.clone(),
            "--cardano-key-file".to_string(),
            format!("{}/admin.sk", constants.config_dir),
            "--blockfrost-key".to_string(),
            self.spec
                .blockfrost_key
                .clone()
                .unwrap_or(config.blockfrost_key.clone()),
            "--commit-inputs".to_string(),
        ];
        open_head_args.extend(self.spec.commit_inputs.clone());

        Deployment {
            metadata: ObjectMeta {
                name: Some(name),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                replicas: Some(1),
                selector: k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector {
                    match_labels: Some(labels.clone()),
                    ..Default::default()
                },
                template: PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(labels.clone()),
                        ..Default::default()
                    }),
                    spec: Some(PodSpec {
                        init_containers: Some(vec![Container {
                            name: "init".to_string(),
                            image: Some(self.spec.image.clone().unwrap_or(config.image.clone())),
                            args: Some(vec![
                                "gen-hydra-key".to_string(),
                                "--output-file".to_string(),
                                format!("{}/hydra", constants.data_dir),
                            ]),
                            volume_mounts: Some(vec![
                                VolumeMount {
                                    name: "config".to_string(),
                                    mount_path: constants.config_dir.clone(),
                                    ..Default::default()
                                },
                                VolumeMount {
                                    name: "data".to_string(),
                                    mount_path: constants.data_dir.clone(),
                                    ..Default::default()
                                },
                            ]),
                            ..Default::default()
                        }]),
                        containers: vec![
                            Container {
                                name: "main".to_string(),
                                image: Some(
                                    self.spec.image.clone().unwrap_or(config.image.clone()),
                                ),
                                args: Some(vec![
                                    "offline".to_string(),
                                    "--host".to_string(),
                                    "0.0.0.0".to_string(),
                                    "--api-host".to_string(),
                                    "0.0.0.0".to_string(),
                                    "--port".to_string(),
                                    "5001".to_string(),
                                    "--api-port".to_string(),
                                    constants.port.to_string(),
                                    "--hydra-signing-key".to_string(),
                                    format!("${}/hydra.sk", constants.data_dir),
                                    "--ledger-protocol-parameters".to_string(),
                                    format!("${}/protocol-parameters.json", constants.config_dir),
                                    "--initial-utxo".to_string(),
                                    format!("${}/utxo.json", constants.config_dir),
                                    "--persistence-dir".to_string(),
                                    format!("${}/hydra-state", constants.persistence_dir),
                                ]),
                                ports: Some(vec![ContainerPort {
                                    name: Some("api".to_string()),
                                    container_port: constants.port,
                                    protocol: Some("TCP".to_string()),
                                    ..Default::default()
                                }]),
                                volume_mounts: Some(vec![
                                    VolumeMount {
                                        name: "config".to_string(),
                                        mount_path: constants.config_dir.clone(),
                                        ..Default::default()
                                    },
                                    VolumeMount {
                                        name: "data".to_string(),
                                        mount_path: constants.data_dir.clone(),
                                        ..Default::default()
                                    },
                                ]),
                                resources: None, // TODO: This should be parameterizable
                                ..Default::default()
                            },
                            // Container {
                            //     name: "open-head".to_string(),
                            //     image: Some(self.spec.open_head_image.clone()),
                            //     args: Some(open_head_args),
                            //     volume_mounts: Some(vec![VolumeMount {
                            //         name: "config".to_string(),
                            //         mount_path: constants.config_dir.clone(),
                            //         ..Default::default()
                            //     }]),
                            //     resources: None, // TODO: Parametrize this
                            //     ..Default::default()
                            // },
                        ],
                        volumes: Some(vec![
                            Volume {
                                name: "data".to_string(),
                                empty_dir: Some(EmptyDirVolumeSource::default()),
                                ..Default::default()
                            },
                            Volume {
                                name: "config".to_string(),
                                config_map: Some(ConfigMapVolumeSource {
                                    name: self
                                        .spec
                                        .configmap
                                        .clone()
                                        .unwrap_or(config.image.clone()),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                        ]),
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn service(&self, _config: &Config, constants: &K8sConstants) -> Service {
        let name = self.pod_name();
        let labels = self.pod_labels();
        Service {
            metadata: ObjectMeta {
                name: Some(name),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                selector: Some(labels),
                ports: Some(vec![ServicePort {
                    port: constants.port,
                    target_port: Some(
                        k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(
                            constants.port,
                        ),
                    ),
                    protocol: Some("TCP".to_string()),
                    ..Default::default()
                }]),
                type_: Some("ClusterIP".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
