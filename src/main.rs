use crate::phone::Api;
use crate::report::send;

mod datatype;
mod phone;
mod report;
mod util;

const BASE_URL_ENV_KEY: &str = "PM_BASE_URL";
const SECRET_ENV_KEY: &str = "PM_SECRET";
const METRIC_NAMESPACE_ENV_KEY: &str = "PM_METRIC_NAMESPACE";

fn load_from_env(key: &str) -> String {
    std::env::var(key).expect(format!("Env var {} must be set", key).as_str())
}
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let base_url = load_from_env(BASE_URL_ENV_KEY)
        .trim_end_matches("/")
        .to_string();
    let secret = load_from_env(SECRET_ENV_KEY);

    let api = Api::new(base_url, secret);
    let config = api.query_config().await.unwrap();
    let battery = api.query_battery().await.unwrap();
    println!("{:?}", config);
    println!("{:?}", battery);
    let res = send(&load_from_env(METRIC_NAMESPACE_ENV_KEY), &config, &battery).await;
    println!("{:?}", res);
}
