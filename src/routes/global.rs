use rocket::{get, http::Status, serde::json::Json, State};

use crate::{context::Context, model::node::NodeStats};

#[get("/global")]
pub async fn global(_context: &State<Context>) -> Result<Json<NodeStats>, Status> {
    todo!()
}
