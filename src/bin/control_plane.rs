use kube::Client;
use rocket::{http::Method, routes};
use rocket_cors::{AllowedOrigins, CorsOptions};

use doom_patrol::{config::Config, context::Context, k8s::K8sHelper, routes::new_game::new_game};

#[rocket::main]
async fn main() {
    let rocket = rocket::build();
    let figment = rocket.figment();
    let config = figment.extract::<Config>().expect("invalid config");

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

    let _rocket = rocket
        .manage(Context::new(K8sHelper::new(client, config)))
        .mount("/", routes![new_game])
        .attach(cors.to_cors().unwrap())
        .launch()
        .await
        .expect("Failed to run rocket server");
}
