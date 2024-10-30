use anyhow::Result;
use futures::StreamExt;
use kube::{runtime::controller::Controller, Api, Client};
use std::sync::Arc;
use tracing::{error, info, instrument};

use doom_patrol::{
    config::Config,
    controller::{error_policy, reconcile, K8sContext},
    custom_resource::HydraDoomNode,
};

#[tokio::main]
#[instrument("controller run", skip_all)]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Initiating operator.");
    let client = Client::try_default().await?;
    let config = Config::from_env();
    let context = Arc::new(K8sContext::new(client.clone(), config));

    // Create controller for MyApp custom resource
    let api: Api<HydraDoomNode> = Api::all(client);
    info!("Running controller.");
    Controller::new(api, Default::default())
        .run(reconcile, error_policy, context)
        .for_each(|res| async move {
            match res {
                Ok(o) => info!("Reconciled {:?}", o),
                Err(e) => error!("Reconcile failed: {:?}", e),
            }
        })
        .await;

    Ok(())
}
