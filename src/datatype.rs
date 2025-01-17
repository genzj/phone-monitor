use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SimInfo {
    pub carrier_name: String,
    pub country_iso: String,
    pub icc_id: String,
    pub number: String,
    pub sim_slot_index: i32,
    pub subscription_id: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub enable_api_battery_query: bool,
    pub enable_api_call_query: bool,
    pub enable_api_clone: bool,
    pub enable_api_contact_query: bool,
    pub enable_api_sms_query: bool,
    pub enable_api_sms_send: bool,
    pub enable_api_wol: bool,
    pub extra_device_mark: Option<String>,
    pub extra_sim1: Option<String>,
    pub extra_sim2: Option<String>,
    pub sim_info_list: Option<HashMap<String, SimInfo>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Battery {
    pub level: String,
    pub scale: Option<String>,
    pub voltage: Option<String>,
    pub temperature: Option<String>,
    pub status: String,
    pub health: String,
    pub plugged: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResponseData {
    Config(Config),
    Battery(Battery),
    None,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResponseWrapper<T>
where for <'a> Option<T>: core::fmt::Debug + Serialize + Deserialize<'a>,
{
    pub code: i32,
    pub msg: Option<String>,
    pub data: Option<T>,
    pub timestamp: i64,
    pub sign: Option<String>,
}

pub type ConfigResponse = ResponseWrapper<Config>;
pub type BatteryResponse = ResponseWrapper<Battery>;