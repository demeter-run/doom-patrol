use std::collections::BTreeMap;

use k8s_openapi::api::{
    apps::v1::{Deployment, DeploymentSpec},
    core::v1::{
        ConfigMapVolumeSource, Container, ContainerPort, EmptyDirVolumeSource, PodSpec,
        PodTemplateSpec, Service, ServicePort, ServiceSpec, Volume, VolumeMount,
    },
};
use kube::{
    api::{DeleteParams, ObjectMeta, PostParams},
    Api, Client,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    config::{Config, HydraPodConfig},
    error::Error,
};

pub struct K8sConstants {
    pub config_dir: String,
    pub data_dir: String,
    pub persistence_dir: String,
    pub node_port: i32,
    pub port: i32,
}
impl Default for K8sConstants {
    fn default() -> Self {
        Self {
            config_dir: "/etc/config".to_string(),
            data_dir: "/var/data".to_string(),
            persistence_dir: "/var/persistence".to_string(),
            node_port: 5001,
            port: 4001,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct PodInfo {
    pub id: String,
    pub local_connection: String,
    pub external_connection: String,
    pub port: i32,
}

pub struct K8sHelper {
    pub config: HydraPodConfig,
    pub client: Client,
    pub constants: K8sConstants,
}

impl K8sHelper {
    pub fn new(client: Client, config: Config) -> Self {
        Self {
            client,
            constants: Default::default(),
            config: config.hydra_pod,
        }
    }

    /// Create new Hydra Pod.
    ///
    /// This consists of a Deployment that hosts a K8s pod containing the Hydra node,
    /// a service to access the pod internally and a HttpRoute to access the Pod externally.
    pub async fn new_hydra_pod(&self, _player_address: &str) -> Result<PodInfo, Error> {
        let id = uuid::Uuid::new_v4().to_string();

        match tokio::join!(self.create_deployment(&id), self.create_service(&id)) {
            (Ok(_), Ok(_)) => (),
            (Ok(_), Err(_)) => {
                self.remove_deployment(&id).await?;
                return Err(Error::K8sError("Failed to create resources".to_string()));
            }
            (Err(_), Ok(_)) => {
                self.remove_service(&id).await?;
                return Err(Error::K8sError("Failed to create resources".to_string()));
            }
            _ => return Err(Error::K8sError("Failed to create resources".to_string())),
        };

        Ok(self.pod_info_from_id(&id))
    }

    pub async fn remove_hydra_pod(&self, id: &str) -> Result<(), Error> {
        match tokio::join!(self.remove_deployment(id), self.remove_service(id)) {
            (Ok(_), Ok(_)) => (),
            (Ok(_), Err(_)) => {
                return Err(Error::K8sError("Failed to remove service".to_string()));
            }
            (Err(_), Ok(_)) => {
                return Err(Error::K8sError("Failed to remove deployment".to_string()));
            }
            _ => return Err(Error::K8sError("Failed to remove resources".to_string())),
        };

        Ok(())
    }

    fn name_from_id(id: &str) -> String {
        format!("hydra-pod-{}", id)
    }

    fn labels_from_id(id: &str) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("component".to_string(), "hydra-pod".to_string()),
            ("hydra-pod-id".to_string(), id.to_string()),
        ])
    }

    fn pod_info_from_id(&self, id: &str) -> PodInfo {
        PodInfo {
            local_connection: format!("ws://{}.svc.cluster.local", Self::name_from_id(id)),
            external_connection: format!("ws://{}.{}", id, self.config.external_domain),
            id: id.to_string(),
            port: self.constants.port,
        }
    }

    async fn create_deployment(&self, id: &str) -> Result<Deployment, Error> {
        let deployments: Api<Deployment> =
            Api::namespaced(self.client.clone(), &self.config.namespace);

        let name = Self::name_from_id(id);
        let labels = Self::labels_from_id(id);

        // Define the deployment
        let deployment = Deployment {
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
                            image: Some(self.config.image.clone()),
                            args: Some(vec![
                                "gen-hydra-key".to_string(),
                                "--output-file".to_string(),
                                format!("{}/hydra", self.constants.data_dir),
                            ]),
                            volume_mounts: Some(vec![
                                VolumeMount {
                                    name: "config".to_string(),
                                    mount_path: self.constants.config_dir.clone(),
                                    ..Default::default()
                                },
                                VolumeMount {
                                    name: "data".to_string(),
                                    mount_path: self.constants.data_dir.clone(),
                                    ..Default::default()
                                },
                            ]),
                            ..Default::default()
                        }]),
                        containers: vec![Container {
                            name: "main".to_string(),
                            image: Some(self.config.image.clone()),
                            args: Some(vec![
                                "offline".to_string(),
                                "--host".to_string(),
                                "0.0.0.0".to_string(),
                                "--api-host".to_string(),
                                "0.0.0.0".to_string(),
                                "--port".to_string(),
                                "5001".to_string(),
                                "--api-port".to_string(),
                                self.constants.port.to_string(),
                                "--hydra-signing-key".to_string(),
                                format!("${}/hydra.sk", self.constants.data_dir),
                                "--ledger-protocol-parameters".to_string(),
                                format!("${}/protocol-parameters.json", self.constants.config_dir),
                                "--initial-utxo".to_string(),
                                format!("${}/utxo.json", self.constants.config_dir),
                                "--persistence-dir".to_string(),
                                format!("${}/hydra-state", self.constants.persistence_dir),
                            ]),
                            ports: Some(vec![ContainerPort {
                                name: Some("api".to_string()),
                                container_port: self.constants.port,
                                protocol: Some("TCP".to_string()),
                                ..Default::default()
                            }]),
                            volume_mounts: Some(vec![
                                VolumeMount {
                                    name: "config".to_string(),
                                    mount_path: self.constants.config_dir.clone(),
                                    ..Default::default()
                                },
                                VolumeMount {
                                    name: "data".to_string(),
                                    mount_path: self.constants.data_dir.clone(),
                                    ..Default::default()
                                },
                                // VolumeMount {
                                //     name: "persistence".to_string(),
                                //     mount_path: self.constants.persistence_dir.clone(),
                                //     ..Default::default()
                                // },
                            ]),
                            resources: self.config.resources.clone(),
                            ..Default::default()
                        }],
                        volumes: Some(vec![
                            Volume {
                                name: "data".to_string(),
                                empty_dir: Some(EmptyDirVolumeSource::default()),
                                ..Default::default()
                            },
                            Volume {
                                name: "config".to_string(),
                                config_map: Some(ConfigMapVolumeSource {
                                    name: self.config.configmap_name.clone(),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                        ]),
                        tolerations: self.config.tolerations.clone(),
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        };

        deployments
            .create(&PostParams::default(), &deployment)
            .await
            .map_err(|e| {
                error!(
                    error = e.to_string(),
                    "Failed to create hydra pod deployment."
                );
                Error::K8sError("Failed to create Hydra Pod Deployment.".to_string())
            })
    }

    async fn remove_deployment(&self, id: &str) -> Result<(), Error> {
        let deployments: Api<Deployment> =
            Api::namespaced(self.client.clone(), &self.config.namespace);
        let dp = DeleteParams::default();

        match deployments.delete(&Self::name_from_id(id), &dp).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!(error = e.to_string(), "Failed to delete deployment {}", id);
                Err(Error::K8sError("Failed to delete deployment".to_string()))
            }
        }
    }

    async fn create_service(&self, id: &str) -> Result<Service, Error> {
        let name = Self::name_from_id(id);
        let labels = Self::labels_from_id(id);
        let service = Service {
            metadata: ObjectMeta {
                name: Some(name),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                selector: Some(labels),
                ports: Some(vec![ServicePort {
                    port: self.constants.port,
                    target_port: Some(
                        k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(
                            self.constants.port,
                        ),
                    ),
                    protocol: Some("TCP".to_string()),
                    ..Default::default()
                }]),
                type_: Some("ClusterIP".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        // Apply the service to the cluster
        let services: Api<Service> = Api::namespaced(self.client.clone(), &self.config.namespace);
        services
            .create(&PostParams::default(), &service)
            .await
            .map_err(|e| {
                error!(
                    error = e.to_string(),
                    "Failed to create service for pod {}", id
                );
                Error::K8sError("Failed to create service".to_string())
            })
    }

    async fn remove_service(&self, id: &str) -> Result<(), Error> {
        let services: Api<Service> = Api::namespaced(self.client.clone(), &self.config.namespace);
        let dp = DeleteParams::default();
        match services.delete(&Self::name_from_id(id), &dp).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!(error = e.to_string(), "Failed to delete service {}", id);
                Err(Error::K8sError("Failed to delete service".to_string()))
            }
        }
    }
}
