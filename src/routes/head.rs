use rocket::{get, serde::json::Json, State};

use crate::{context::Context, model::node::Node};

#[get("/heads/<head_id>")]
#[allow(unused)]
pub async fn head(_context: &State<Context>, head_id: &str) -> Json<Vec<Node>> {
    todo!()
}
