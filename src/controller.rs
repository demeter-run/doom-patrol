use anyhow::{bail, Context};
use k8s_openapi::api::{apps::v1::Deployment, core::v1::Service};
use kube::{
    api::{DeleteParams, Patch, PatchParams},
    runtime::controller::Action,
    Api, Client, ResourceExt,
};
use serde_json::json;
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tracing::{error, info};

use crate::{config::Config, custom_resource::HydraDoomNodeStatus};

use super::custom_resource::{HydraDoomNode, HYDRA_DOOM_NODE_FINALIZER};

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

pub struct K8sContext {
    pub client: Client,
    pub config: Config,
    pub constants: K8sConstants,
}

impl K8sContext {
    pub fn new(client: Client, config: Config) -> Self {
        Self {
            client,
            config,
            constants: Default::default(),
        }
    }

    pub async fn patch(&self, crd: &HydraDoomNode) -> anyhow::Result<()> {
        info!("Running patch");
        match tokio::join!(
            self.patch_deployment(crd),
            self.patch_service(crd),
            self.patch_crd(crd)
        ) {
            (Ok(_), Ok(_), Ok(_)) => (),
            (Ok(_), Err(_), Ok(_)) => {
                self.remove_deployment(crd)
                    .await
                    .context("Failed to create resources")?;
            }
            (Err(_), Ok(_), Ok(_)) => {
                self.remove_service(crd)
                    .await
                    .context("Failed to create resources")?;
            }
            (Ok(_), Ok(_), Err(_)) => {
                self.remove_deployment(crd)
                    .await
                    .context("Failed to create resources")?;
                self.remove_service(crd)
                    .await
                    .context("Failed to create resources")?;
            }
            _ => bail!("Failed to create resources"),
        };

        Ok(())
    }

    pub async fn delete(&self, crd: &HydraDoomNode) -> anyhow::Result<()> {
        match tokio::join!(self.remove_deployment(crd), self.remove_service(crd)) {
            (Ok(_), Ok(_)) => Ok(()),
            (Ok(_), Err(err)) => Err(err.context("Failed to remove service.")),
            (Err(err), Ok(_)) => Err(err.context("Failed to remove deployment.")),
            (Err(err), Err(_)) => Err(err.context("Failed to remove resources.")),
        }
    }

    async fn patch_crd(&self, crd: &HydraDoomNode) -> anyhow::Result<HydraDoomNode> {
        let api: Api<HydraDoomNode> =
            Api::namespaced(self.client.clone(), &crd.namespace().unwrap());

        // Create or patch the deployment
        let status = serde_json::to_value(HydraDoomNodeStatus {
            local_url: format!(
                "ws://{}.{}.svc.cluster.local:{}",
                crd.name_any(),
                crd.namespace().unwrap(),
                self.constants.port
            ),
            external_url: format!(
                "wss://{}.{}:{}",
                crd.name_any(),
                self.config.external_domain,
                self.config.external_port
            ),
        })
        .unwrap();
        api.patch(
            &crd.name_any(),
            &PatchParams::default(),
            &Patch::Merge(json!({
                "status": status,
                "metadata": {
                    "finalizers": [HYDRA_DOOM_NODE_FINALIZER]
                }
            })),
        )
        .await
        .map_err(|err| {
            error!(err = err.to_string(), "Failed to patch CRD.");
            anyhow::Error::from(err)
        })?;

        api.patch_status(
            &crd.name_any(),
            &PatchParams::default(),
            &Patch::Merge(json!({ "status": status })),
        )
        .await
        .map_err(|err| {
            error!(err = err.to_string(), "Failed to patch CRD.");
            anyhow::Error::from(err)
        })
    }

    async fn patch_deployment(&self, crd: &HydraDoomNode) -> anyhow::Result<Deployment> {
        let deployments: Api<Deployment> =
            Api::namespaced(self.client.clone(), &crd.namespace().unwrap());

        // Create or patch the deployment
        deployments
            .patch(
                &crd.internal_name(),
                &PatchParams::apply("hydra-doom-pod-controller"),
                &Patch::Apply(&crd.deployment(&self.config, &self.constants)),
            )
            .await
            .map_err(|err| {
                error!(err = err.to_string(), "Failed to create deployment.");
                err.into()
            })
    }

    async fn remove_deployment(&self, crd: &HydraDoomNode) -> anyhow::Result<()> {
        let deployments: Api<Deployment> =
            Api::namespaced(self.client.clone(), &crd.namespace().unwrap());
        let dp = DeleteParams::default();

        match deployments.delete(&crd.internal_name(), &dp).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn patch_service(&self, crd: &HydraDoomNode) -> anyhow::Result<Service> {
        // Apply the service to the cluster
        let services: Api<Service> =
            Api::namespaced(self.client.clone(), &crd.namespace().unwrap());
        services
            .patch(
                &crd.internal_name(),
                &PatchParams::apply("hydra-doom-pod-controller"),
                &Patch::Apply(&crd.service(&self.config, &self.constants)),
            )
            .await
            .map_err(|err| {
                error!(err = err.to_string(), "Failed to create service.");
                err.into()
            })
    }

    async fn remove_service(&self, crd: &HydraDoomNode) -> anyhow::Result<()> {
        let services: Api<Service> =
            Api::namespaced(self.client.clone(), &crd.namespace().unwrap());
        let dp = DeleteParams::default();
        match services.delete(&crd.internal_name(), &dp).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

// Auxiliary error value because K8s controller api doesnt go along with anyhow.
#[derive(Debug, Error)]
pub enum Error {
    #[error("ReconcileError")]
    ReconcileError,
}
impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        error!("Reconcile error: {}", value.to_string());
        Self::ReconcileError
    }
}
type Result<T, E = Error> = std::result::Result<T, E>;

pub async fn reconcile(crd: Arc<HydraDoomNode>, ctx: Arc<K8sContext>) -> Result<Action, Error> {
    tracing::info!("Reconciling {}", crd.name_any());
    // Check if deletion timestamp is set
    if crd.metadata.deletion_timestamp.is_some() {
        let hydra_doom_pod_api: Api<HydraDoomNode> =
            Api::namespaced(ctx.client.clone(), &crd.namespace().unwrap());
        // Finalizer logic for cleanup
        if crd
            .finalizers()
            .contains(&HYDRA_DOOM_NODE_FINALIZER.to_string())
        {
            // Delete associated resources
            ctx.delete(&crd).await?;
            // Remove finalizer
            let patch = json!({
                "metadata": {
                    "finalizers": crd.finalizers().iter().filter(|f| *f != HYDRA_DOOM_NODE_FINALIZER).collect::<Vec<_>>()
                }
            });
            let _ = hydra_doom_pod_api
                .patch(
                    &crd.name_any(),
                    &PatchParams::default(),
                    &Patch::Merge(&patch),
                )
                .await
                .map_err(anyhow::Error::from)?;
        }
        return Ok(Action::await_change());
    }

    // Ensure finalizer is set
    ctx.patch(&crd).await?;
    Ok(Action::await_change())
}

pub fn error_policy(crd: Arc<HydraDoomNode>, err: &Error, _ctx: Arc<K8sContext>) -> Action {
    error!(
        error = err.to_string(),
        crd = serde_json::to_string(&crd).unwrap(),
        "reconcile failed"
    );
    Action::requeue(Duration::from_secs(5))
}
