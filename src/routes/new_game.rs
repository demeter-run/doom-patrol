use rocket::{get, http::Status, serde::json::Json, State};
use serde::Serialize;

use crate::{context::Context, k8s::PodInfo};

#[derive(Serialize)]
pub struct NewGameResponse {
    host: String,
    script_ref: String,
    admin_pkh: String,
    player_utxo: String,
    player_utxo_datum_hex: String,
}

impl From<PodInfo> for NewGameResponse {
    fn from(value: PodInfo) -> Self {
        Self {
            host: value.external_connection.clone(),
            script_ref: "script_ref".to_string(),
            admin_pkh: "admin_pkh".to_string(),
            player_utxo: "player_utxo".to_string(),
            player_utxo_datum_hex: "player_utxo_datum_hex".to_string(),
        }
    }
}

#[get("/new_game?<address>")]
#[allow(unused)]
pub async fn new_game(
    address: &str,
    context: &State<Context>,
) -> Result<Json<NewGameResponse>, Status> {
    let info = context
        .k8s
        .new_hydra_pod(address)
        .await
        .map_err(|e| Status::InternalServerError)?;
    Ok(Json(NewGameResponse::from(info)))
}
