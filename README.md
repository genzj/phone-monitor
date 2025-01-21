# Phone Monitor

A monitoring application that collects and tracks phone battery level across
multiple [SMS Forwarder](https://github.com/pppscn/SmsForwarder/) servers and push the metrics to AWS CloudWatch.

## Description

Phone Monitor is a Rust-based application that connects to
multiple [SMS Forwarder remote control API endpoints](https://github.com/pppscn/SmsForwarder/wiki/%E9%99%84%E5%BD%952%EF%BC%9A%E4%B8%BB%E5%8A%A8%E8%AF%B7%E6%B1%82(%E8%BF%9C%E7%A8%8B%E6%8E%A7%E5%88%B6))
and publishes metrics to AWS CloudWatch. It supports multiple locator endpoints with their corresponding authentication
secrets.

## Prerequisites

- Rust (latest stable version)
- AWS IAM credential with `cloudwatch:PutMetricData` permission
- Install the SMS Forwarder on the phone to be monitored, enable API server and set a secret

## Configuration

The application uses either CLI arguments or environment variables for configuration. Read `phone-monitor --help` and
[`env.sample`](env.sample) for details. The application supports [dotenv](https://docs.rs/dotenv/0.15.0/dotenv/) so environment
variables can be set in the `.env` file.

## Docker

Periodic metrics collection can be achieved by setting up a docker stack with this application and a cron-like
scheduler. See the [`docker-compose.yaml`](docker-compose.yaml) for an example.

