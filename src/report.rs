use crate::datatype::{BatteryResponse, ConfigResponse};
use aws_config::Region;
use aws_sdk_cloudwatch::config::http::HttpResponse;
use aws_sdk_cloudwatch::error::SdkError;
use aws_sdk_cloudwatch::operation::put_metric_data::{PutMetricDataError, PutMetricDataOutput};
use aws_sdk_cloudwatch::primitives::DateTime;
use aws_sdk_cloudwatch::types::{Dimension, MetricDatum, StandardUnit};
use aws_sdk_cloudwatch::Client;

fn create_metric_data(config: &ConfigResponse, battery: &BatteryResponse) -> MetricDatum {
    let timestamp_ms = battery.timestamp;
    let battery_level = battery
        .data
        .as_ref()
        .into_iter()
        .flat_map(|data| data.level.strip_suffix("%"))
        .flat_map(|x| x.parse::<f64>())
        .nth(0)
        .unwrap_or(0f64);
    let phone_id = config
        .data
        .as_ref()
        .into_iter()
        .flat_map(|config| &config.extra_device_mark)
        .map(|x| x.as_str())
        .nth(0)
        .unwrap_or("unknown");

    MetricDatum::builder()
        .metric_name("battery")
        .dimensions(
            Dimension::builder()
                .name("phone_id")
                .value(phone_id)
                .build(),
        )
        .timestamp(DateTime::from_millis(timestamp_ms as i64))
        .value(battery_level)
        .unit(StandardUnit::Percent)
        .build()
}

async fn send(
    config: &ConfigResponse,
    battery: &BatteryResponse,
) -> Result<PutMetricDataOutput, SdkError<PutMetricDataError, HttpResponse>> {
    let shared_config = aws_config::from_env()
        .region(Region::new("us-west-2"))
        .load()
        .await;
    let client = Client::new(&shared_config);
    client
        .put_metric_data()
        .namespace("phone")
        .metric_data(create_metric_data(config, battery))
        .send()
        .await
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_metric_data() {
        let config =
            serde_json::from_reader(std::fs::File::open("test_data/config_query.json").unwrap())
                .unwrap();
        let battery =
            serde_json::from_reader(std::fs::File::open("test_data/battery_query.json").unwrap())
                .unwrap();
        let metric = create_metric_data(&config, &battery);
        assert_eq!(metric.metric_name().unwrap(), "battery");
        assert_eq!(metric.dimensions().len(), 1);
        let dimension = metric.dimensions().first().unwrap();
        assert_eq!(dimension.name().unwrap(), "phone_id");
        assert_eq!(dimension.value().unwrap(), "Dev");
        assert_eq!(metric.value().unwrap(), 36.0);
        assert_eq!(*metric.unit().unwrap(), StandardUnit::Percent);
    }
}
