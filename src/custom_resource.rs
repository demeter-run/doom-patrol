use k8s_openapi::api::{
    apps::v1::{Deployment, DeploymentSpec},
    core::v1::{
        ConfigMapVolumeSource, Container, ContainerPort, EmptyDirVolumeSource, PodSpec,
        PodTemplateSpec, Service, ServicePort, ServiceSpec, Volume, VolumeMount,
    },
    networking::v1::{
        HTTPIngressPath, HTTPIngressRuleValue, Ingress, IngressBackend, IngressRule,
        IngressServiceBackend, IngressSpec, ServiceBackendPort,
    },
};
use kube::{api::ObjectMeta, CustomResource, ResourceExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::config::Config;

use super::controller::K8sConstants;

pub static HYDRA_DOOM_NODE_FINALIZER: &str = "hydradoomnode/finalizer";

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    kind = "HydraDoomNode",
    group = "hydra.doom",
    version = "v1alpha1",
    shortname = "hydradoomnode",
    category = "hydradoom",
    plural = "hydradoomnodes",
    namespaced
)]
#[kube(status = "HydraDoomNodeStatus")]
#[kube(printcolumn = r#"
        {"name": "State", "jsonPath":".status.state", "type": "string"}, 
        {"name": "Transactions", "jsonPath":".status.transactions", "type": "string"}, 
        {"name": "Local URI", "jsonPath":".status.localUrl", "type": "string"}, 
        {"name": "External URI", "jsonPath": ".status.externalUrl", "type": "string"}
    "#)]
#[serde(rename_all = "camelCase")]
pub struct HydraDoomNodeSpec {
    pub image: Option<String>,
    pub open_head_image: Option<String>,
    pub sidecar_image: Option<String>,
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
pub struct HydraDoomNodeStatus {
    pub local_url: String,
    pub external_url: String,
    pub state: String,
    pub transactions: i64,
}
impl HydraDoomNodeStatus {
    pub fn offline(crd: &HydraDoomNode, config: &Config, constants: &K8sConstants) -> Self {
        Self {
            state: "Offline".to_string(),
            transactions: 0,
            local_url: format!("ws://{}:{}", crd.internal_host(), constants.port),
            external_url: format!(
                "ws://{}:{}",
                crd.external_host(config, constants),
                config.external_port
            ),
        }
    }
}

impl HydraDoomNode {
    pub fn internal_name(&self) -> String {
        format!("hydra-doom-node-{}", self.name_any())
    }

    pub fn internal_labels(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("component".to_string(), "hydra-doom-node".to_string()),
            ("hydra-doom-node-id".to_string(), self.name_any()),
            ("run-on".to_string(), "fargate".to_string()),
        ])
    }

    pub fn internal_host(&self) -> String {
        format!(
            "{}.{}.svc.cluster.local",
            self.name_any(),
            self.namespace().unwrap(),
        )
    }

    pub fn external_host(&self, config: &Config, _constants: &K8sConstants) -> String {
        format!("{}.{}", self.name_any(), config.external_domain,)
    }

    pub fn deployment(&self, config: &Config, constants: &K8sConstants) -> Deployment {
        let name = self.internal_name();
        let labels = self.internal_labels();
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
                            volume_mounts: Some(vec![VolumeMount {
                                name: "data".to_string(),
                                mount_path: constants.data_dir.clone(),
                                ..Default::default()
                            }]),
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
                                    format!("{}/hydra.sk", constants.data_dir),
                                    "--ledger-protocol-parameters".to_string(),
                                    format!("{}/protocol-parameters.json", constants.config_dir),
                                    "--initial-utxo".to_string(),
                                    format!("{}/utxo.json", constants.config_dir),
                                    "--persistence-dir".to_string(),
                                    format!("{}/hydra-state", constants.persistence_dir),
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
                            Container {
                                name: "sidecar".to_string(),
                                image: Some(
                                    self.spec
                                        .sidecar_image
                                        .clone()
                                        .unwrap_or(config.sidecar_image.clone()),
                                ),
                                args: Some(vec![
                                    "--host".to_string(),
                                    "localhost".to_string(),
                                    "--port".to_string(),
                                    "0.0.0.0".to_string(),
                                    constants.port.to_string(),
                                ]),
                                ports: Some(vec![ContainerPort {
                                    name: Some("metrics".to_string()),
                                    container_port: constants.metrics_port,
                                    protocol: Some("TCP".to_string()),
                                    ..Default::default()
                                }]),
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
                                        .unwrap_or(config.configmap.clone()),
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
        let name = self.internal_name();
        let labels = self.internal_labels();
        Service {
            metadata: ObjectMeta {
                name: Some(name),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                selector: Some(labels),
                ports: Some(vec![
                    ServicePort {
                        port: constants.port,
                        target_port: Some(
                            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(
                                constants.port,
                            ),
                        ),
                        protocol: Some("TCP".to_string()),
                        ..Default::default()
                    },
                    ServicePort {
                        port: constants.metrics_port,
                        target_port: Some(
                            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(
                                constants.metrics_port,
                            ),
                        ),
                        protocol: Some("TCP".to_string()),
                        ..Default::default()
                    },
                ]),
                type_: Some("ClusterIP".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn ingress(&self, config: &Config, constants: &K8sConstants) -> Ingress {
        let name = self.internal_name();
        Ingress {
            metadata: ObjectMeta {
                name: Some(name.clone()),
                annotations: Some(constants.ingress_annotations.clone()),
                ..Default::default()
            },
            spec: Some(IngressSpec {
                ingress_class_name: Some(constants.ingress_class_name.clone()),
                rules: Some(vec![IngressRule {
                    host: Some(self.external_host(config, constants)),
                    http: Some(HTTPIngressRuleValue {
                        paths: vec![HTTPIngressPath {
                            path: Some("/".to_string()),
                            path_type: "Prefix".to_string(),
                            backend: IngressBackend {
                                service: Some(IngressServiceBackend {
                                    name: name.clone(),
                                    port: Some(ServiceBackendPort {
                                        number: Some(constants.port),
                                        ..Default::default()
                                    }),
                                }),
                                ..Default::default()
                            },
                        }],
                    }),
                }]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
