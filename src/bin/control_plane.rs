use kube::Client;
use rocket::{http::Method, routes};
use rocket_cors::{AllowedOrigins, CorsOptions};

use doom_patrol::{
    context::Context,
    routes::{global::global, head::head, heads::heads, new_game::new_game},
};

#[rocket::main]
async fn main() {
    let client = Client::try_default()
        .await
        .expect("Failed to create K8s client");

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

    let _rocket = rocket::build()
        .manage(Context { client })
        .mount("/", routes![new_game, heads, head, global])
        .attach(cors.to_cors().unwrap())
        .launch()
        .await
        .expect("Failed to run rocket server");
}
