use crate::phone::Api;
use crate::report::send;
use clap::Parser;
use log::{debug, info, trace};

mod datatype;
mod phone;
mod report;
mod util;

const BASE_URL_ENV_KEY: &str = "PM_BASE_URL";
const SECRET_ENV_KEY: &str = "PM_SECRET";
const METRIC_NAMESPACE_ENV_KEY: &str = "PM_METRIC_NAMESPACE";
const DRY_RUN_ENV_KEY: &str = "PM_DRY_RUN";

fn load_from_env(key: &str) -> String {
    let v = std::env::var(key).expect(format!("Env var {} must be set", key).as_str());
    debug!("loaded {} from env: {}", key, v);
    v
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Base URL of SmsF API server
    /// e.g. http://192.168.1.11:5000
    #[arg(long, env = BASE_URL_ENV_KEY)]
    base_url: String,
    /// Secret token of SmsF API server
    #[arg(long, env = SECRET_ENV_KEY)]
    secret: String,
    /// Cloudwatch Metric namespace
    #[arg(long, env = METRIC_NAMESPACE_ENV_KEY, default_value = "phone")]
    metric_namespace: String,
    /// Query SmsF API server and print the result without publishing any metric
    #[arg(long, short, env = DRY_RUN_ENV_KEY, default_value = "false")]
    dry_run: bool,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    pretty_env_logger::init();

    let cli = Cli::parse();

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

    if cli.dry_run {
        info!("dry run mode, skip metric submitting");
        return;
    }

    let metric_namespace = load_from_env(METRIC_NAMESPACE_ENV_KEY);
    info!("submitting metric to namespace {}", metric_namespace);
    let res = send(&metric_namespace, &config, &battery).await;
    trace!("metric submitting response {:?}", res);
    info!("metric submitted")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}