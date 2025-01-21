use crate::phone::Api;
use crate::report::send;
use clap::{Args, CommandFactory, Parser};
use log::{debug, info, trace, warn};
use std::env;
use std::fmt::Display;
use std::str::FromStr;
use clap::error::{ContextKind, ContextValue};

mod datatype;
mod phone;
mod report;
mod util;

const BASE_URL_ENV_KEY: &str = "PM_BASE_URL";
const SECRET_ENV_KEY: &str = "PM_SECRET";
const METRIC_NAMESPACE_ENV_KEY: &str = "PM_METRIC_NAMESPACE";
const DRY_RUN_ENV_KEY: &str = "PM_DRY_RUN";
const API_LOCATORS_ENV_KEY: &str = "PM_LOCATORS";

/// SmsF base URL and corresponding secret
#[derive(Debug, Clone)]
struct ApiLocator(String, String);

impl ApiLocator {
    fn from_env() -> Vec<Self> {
        env::var(API_LOCATORS_ENV_KEY)
            .iter()
            .flat_map(|s| s.split(" "))
            .filter_map(|s| s.parse::<ApiLocator>().ok())
            .collect::<Vec<_>>()
    }
}

impl Display for ApiLocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{}={}", self.0, self.1))
    }
}

impl FromStr for ApiLocator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((base_url, secret)) = s.split_once("=") {
            Ok(ApiLocator(String::from(base_url), String::from(secret)))
        } else {
            Err(format!(
                "invalid locator: '{}'. It must be of 'https://url:port=secret' convention",
                s
            ))
        }
    }
}

#[derive(Args, Debug, Clone)]
#[group(required = false, multiple = true)]
struct ApiConfig {
    /// Base URL of SmsF API server
    /// e.g. http://192.168.1.11:5000
    #[arg(long, env = BASE_URL_ENV_KEY, conflicts_with = "locator", requires = "secret")]
    base_url: Option<String>,
    /// Secret token of SmsF API server
    #[arg(long, env = SECRET_ENV_KEY, conflicts_with = "locator", requires = "base_url")]
    secret: Option<String>,
    /// Specify multiple API bases and corresponding secrets
    #[arg(long, short, env = API_LOCATORS_ENV_KEY, conflicts_with_all = ["base_url", "secret"], num_args = 1..)]
    locator: Vec<ApiLocator>,
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(flatten)]
    api_config: ApiConfig,
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

    let mut cli = Cli::parse();

    if cli.api_config.secret.is_none() || cli.api_config.base_url.is_none() {
        info!(
            "no base_url and secret provided, try to read api locators from env {}",
            API_LOCATORS_ENV_KEY
        );
        cli.api_config.locator = ApiLocator::from_env();
    } else {
        warn!(
            "base_url and secret provided, ignore api locators from env {}",
            API_LOCATORS_ENV_KEY
        );
        cli.api_config.locator = vec![ApiLocator(
            cli.api_config.base_url.as_ref().unwrap().clone(),
            cli.api_config.secret.as_ref().unwrap().clone(),
        )];
    }
    debug!("cli {:?}", &cli);

    if cli.api_config.locator.is_empty() {
        info!("no base_url and secret provided, exit");
        let mut err = clap::Error::new(clap::error::ErrorKind::MissingRequiredArgument).with_cmd(&Cli::command());
        err.insert(ContextKind::SuggestedArg, ContextValue::String(String::from("specify either --locator or --base-url + --secret")));
        let _ = err.print();
        return;
    }

    for locator in cli.api_config.locator {
        query_api(&locator.0, &locator.1, cli.dry_run, &cli.metric_namespace).await;
    }
}

async fn query_api(base_url: &str, secret: &str, dry_run: bool, metric_namespace: &str) {
    let base_url = base_url.trim_end_matches("/").to_string();
    info!("using base_url: {}", base_url);
    let api = Api::new(base_url, secret);
    let config = api.query_config().await.unwrap();
    let battery = api.query_battery().await.unwrap();
    trace!("config query response {:?}", config);
    trace!("battery query response {:?}", battery);

    if dry_run {
        info!("dry run mode, skip metric submitting");
        return;
    }

    let metric_namespace = metric_namespace;
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

    #[test]
    fn test_api_locator() {
        let locator = ApiLocator::from_str("xyz=abc=123").unwrap();
        assert_eq!(locator.0, "xyz");
        assert_eq!(locator.1, "abc=123");
    }

    #[test]
    fn test_api_locator_from_env() {
        env::set_var(API_LOCATORS_ENV_KEY, "xyz=abc=123 base=456");
        let locator = ApiLocator::from_env();
        assert_eq!(locator.len(), 2);
        assert_eq!(locator[0].0, "xyz");
        assert_eq!(locator[0].1, "abc=123");
        assert_eq!(locator[1].0, "base");
        assert_eq!(locator[1].1, "456");
    }
}
