use log::{debug, info, trace};
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
    let v = std::env::var(key).expect(format!("Env var {} must be set", key).as_str());
    debug!("loaded {} from env: {}", key, v);
    v
}
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    pretty_env_logger::init();

    let base_url = load_from_env(BASE_URL_ENV_KEY)
        .trim_end_matches("/")
        .to_string();
    info!("using base_url: {}", base_url);
    let secret = load_from_env(SECRET_ENV_KEY);

    let api = Api::new(base_url, secret);
    let config = api.query_config().await.unwrap();
    let battery = api.query_battery().await.unwrap();
    trace!("config query response {:?}", config);
    trace!("battery query response {:?}", battery);
    let metric_namespace = load_from_env(METRIC_NAMESPACE_ENV_KEY);
    info!("submitting metric to namespace {}", metric_namespace);
    let res = send(&metric_namespace, &config, &battery).await;
    trace!("metric submitting response {:?}", res);
    info!("metric submitted")
}
