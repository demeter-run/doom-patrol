use rocket::{get, http::Status, serde::json::Json, State};
use serde::Serialize;

use crate::context::Context;

#[derive(Serialize)]
pub struct NewGameResponse {
    ip: String,
    script_ref: String,
    admin_pkh: String,
    player_utxo: String,
    player_utxo_datum_hex: String,
}

#[get("/new_game?<address>")]
#[allow(unused)]
pub async fn new_game(
    address: &str,
    _context: &State<Context>,
) -> Result<Json<NewGameResponse>, Status> {
    todo!()
}
