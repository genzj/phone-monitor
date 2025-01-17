use crate::datatype::{BatteryResponse, ConfigResponse};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use hmac_sha256::HMAC;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::time::SystemTime;

trait Clock {
    fn timestamp(&self) -> u64;
}

struct SystemClock;

impl Clock for SystemClock {
    fn timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

pub struct Api {
    base_url: String,
    secret: String,
    clock: Box<dyn Clock>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DataPayload {
    data: Value,
    timestamp: u64,
    sign: String,
}

type ApiResult<T> = Result<T, Box<dyn Error>>;

const API_PATH_QUERY_CONFIG: &str = "/config/query";
const API_PATH_QUERY_BATTERY: &str = "/battery/query";

impl Api {
    pub fn new(base_url: impl Into<String>, secret: impl Into<String>) -> Api {
        Api {
            base_url: base_url.into(),
            secret: secret.into(),
            clock: Box::new(SystemClock),
        }
    }

    fn sign(&self, timestamp: u64) -> String {
        let input = format!("{}\n{}", timestamp, self.secret);
        let sign = BASE64_STANDARD.encode(HMAC::mac(input.as_bytes(), &self.secret.as_bytes()));
        let sign = sign
            .replace('+', "%2B")
            .replace('/', "%2F")
            .replace('=', "%3D");
        println!("sign timestamp {} == {}", timestamp, sign);
        sign
    }

    fn make_payload(&self, data: Value) -> DataPayload {
        let timestamp = self.clock.timestamp();
        let sign = self.sign(timestamp);
        let payload = DataPayload {
            data: data.clone(),
            timestamp,
            sign,
        };
        println!("payload: {:?}", payload);
        payload
    }

    async fn send_post(&self, path: String, data: Option<Value>) -> ApiResult<Response> {
        let client = reqwest::Client::new();
        let url = format!("{}{}", self.base_url, path);
        let data = data.unwrap_or_else(|| Value::Object(Default::default()));
        match client.post(url).json(&self.make_payload(data)).send().await {
            Ok(response) => Ok(response),
            Err(err) => Err(Box::new(err)),
        }
    }

    async fn query<T>(&self, path: String, data: Option<Value>) -> ApiResult<T> 
    where T: for <'a> Deserialize<'a>
    {
        self.send_post(path, data)
            .await?
            .json::<T>()
            .await
            .map_err(|e| Box::<dyn Error>::from(e))
    }

    pub async fn query_config(&self) -> ApiResult<ConfigResponse> {
        self.query(API_PATH_QUERY_CONFIG.to_string(), None).await
    }

    pub async fn query_battery(&self) -> ApiResult<BatteryResponse> {
        self.query(API_PATH_QUERY_BATTERY.to_string(), None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::init_test_logger;
    use mockito::{Matcher, Server, ServerGuard};

    struct FixedClock(u64);

    impl Clock for FixedClock {
        fn timestamp(&self) -> u64 {
            self.0
        }
    }

    fn create_api(server: &Server, timestamp: u64) -> Api {
        Api {
            base_url: server.url(),
            secret: "VyWatNuqAp6GYDG".to_string(),
            clock: Box::new(FixedClock(timestamp)),
        }
    }

    async fn create_mock_server() -> ServerGuard {
        Server::new_async().await
    }

    async fn mock_config_query(server: &mut Server) -> &mut Server {
        server
                .mock("POST", "/config/query")
                .match_body(Matcher::PartialJsonString(
                    r#"{"data": {}, "timestamp": 1737055057812, "sign": "zlRf047zhWs%2B1XH5DUqUV8Fv07doAFpJUwmj6U7rh8s%3D"}"#.to_string(),
                ))
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body_from_file("test_data/config_query.json")
                .create_async()
                .await;
        server
    }

    async fn mock_battery_query(server: &mut Server) -> &mut Server {
        server
                .mock("POST", "/battery/query")
                .match_body(Matcher::PartialJsonString(
                    r#"{"data": {}, "timestamp": 1737055058101, "sign": "CKT7Zg8Apu84wMEsfvifZgDKLLPrwBL%2Fwn%2Fgmm7SqcU%3D"}"#.to_string(),
                ))
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body_from_file("test_data/battery_query.json")
                .create_async()
                .await;
        server
    }

    #[tokio::test]
    async fn test_sign() {
        let api = Api::new("", "VyWatNuqAp6GYDG");
        assert_eq!(
            api.sign(1737055057812),
            "zlRf047zhWs%2B1XH5DUqUV8Fv07doAFpJUwmj6U7rh8s%3D"
        );
    }

    #[tokio::test]
    async fn test_query_config() {
        init_test_logger();
        let mut server = create_mock_server().await;
        let server = mock_config_query(&mut server).await;
        let api = create_api(&server, 1737055057812);
        let response = api.query_config().await.unwrap();
        assert_eq!(response.code, 200);
        assert_eq!(response.msg, Some("success".into()));
        assert_eq!(response.timestamp, 1737054664286);
        assert!(response.data.is_some());
        let data = response.data.unwrap();
        assert!(data.enable_api_battery_query);
        assert!(data.sim_info_list.is_some());
        let sim_info_list = data.sim_info_list.unwrap();
        assert_eq!(sim_info_list.len(), 2);
        assert!(sim_info_list.contains_key("0"));
        assert_eq!(sim_info_list.get("0").unwrap().carrier_name, "CMCC");
    }
    
    #[tokio::test]
    async fn test_query_battery() {
        init_test_logger();
        let mut server = create_mock_server().await;
        let server = mock_battery_query(&mut server).await;
        let api = create_api(&server, 1737055058101);
        let response = api.query_battery().await.unwrap();
        assert_eq!(response.code, 200);
        assert_eq!(response.msg, Some("success".into()));
        assert_eq!(response.timestamp, 1737054664309);
        assert!(response.data.is_some());
        let data = response.data.unwrap();
        assert_eq!(data.level, "36%");
        assert_eq!(data.health, "良好");
    }
}
