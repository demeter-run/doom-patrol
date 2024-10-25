use rocket::{get, serde::json::Json, State};

use crate::{context::Context, model::node::NodeSummary};

#[get("/heads")]
pub async fn heads(_context: &State<Context>) -> Json<Vec<NodeSummary>> {
    todo!()
}
