extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate toml;

use std::collections::HashMap;
use std::error::Error;

// ToDo not supported
#[derive(Debug, Deserialize, Serialize)]
pub struct Range(pub i32, pub i32);
#[derive(Debug, Deserialize, Serialize)]
pub enum Axis {
    Y1,
    Y2,
}
// ToDo not supported
#[derive(Debug, Deserialize, Serialize)]
pub struct AxisRange(pub Range, pub Range);

#[derive(Debug, Deserialize, Serialize)]
pub enum AggOp {
    Sum,
    Mean,
}

// ToDo not supported
#[derive(Debug, Deserialize, Serialize)]
pub struct Aggregate(pub AggOp, pub Option<String>);

#[derive(Debug, Deserialize, Serialize)]
pub enum Transform {
    Rate,
    Mul(i32),
    Div(i32),
    Add(i32),
    Sub(i32),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Display {
    Stack,
    Invert,
    Percent,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Metric {
    pub name:      String,
    pub selector:  String,
    pub aggregate: Vec<Aggregate>,
    pub transform: Vec<Transform>,
    pub display:   Vec<Display>,
    pub axis:      Axis,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Group {
    pub name:       String,
    pub range:      Option<Range>,
    pub axis_range: Option<AxisRange>,
    pub metrics:    Vec<Metric>,
}

// ToDo not supported
#[derive(Debug, Deserialize, Serialize)]
pub enum Collector {
    Prometheus(String),
    Statsd(String),
    Telegraf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Metrics {
    pub collect: Collector,
    pub groups:  HashMap<String, Group>,
}

pub fn parse_toml(contents: &str) -> Result<Metrics, Box<Error>> {
    let metrics: Metrics = toml::from_str(contents)?;
    Ok(metrics)
}

pub fn parse_yaml(contents: &str) -> Result<Metrics, Box<Error>> {
    let metrics: Metrics = serde_yaml::from_str(&contents)?;
    Ok(metrics)
}
