extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate toml;

use serde::{Deserialize, Serialize, Serializer};
use serde::ser::{SerializeSeq};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub struct Range(pub i32, pub i32);

impl<'de> Deserialize<'de> for Range {
    fn deserialize<D>(deserializer: D) -> Result<Range, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct RangeVisitor;
        impl<'de> serde::de::Visitor<'de> for RangeVisitor {
            type Value = Range;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("expected a range like [0, 100]")
            }

            fn visit_seq<A>(self, mut sa: A) -> Result<Range, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let lo = sa.next_element()?;
                let hi = sa.next_element()?;
                if let (Some(l), Some(h)) = (lo, hi) {
                    Ok(Range(l, h))
                } else {
                    Err(serde::de::Error::custom("Expected a range like [0, 100]"))
                }
            }
        }
        deserializer.deserialize_tuple(2, RangeVisitor)
    }
}

impl Serialize for Range {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.0)?;
        seq.serialize_element(&self.1)?;
        seq.end()
    }
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    Y1,
    Y2,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AxisRange {
    pub y1: Range,
    pub y2: Range,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AggOp {
    Sum,
    Mean,
}

// ToDo not supported
#[derive(Debug, Deserialize, Serialize)]
pub struct Aggregate(pub AggOp, pub Option<String>);

#[derive(Debug,PartialEq)]
pub enum Transform {
    Rate,
    Mul(i32),
    Div(i32),
    Add(i32),
    Sub(i32),
}

fn parse_next_int(os: Option<&str>) -> Result<i32, Box<Error>> {
    match os {
        Some(s) => {
            let n: i32 = FromStr::from_str(s)?;
            Ok(n)
        },
        None => Err("Missing parameter".into())
    }
}

fn parse_transform_str(s: &str) -> Result<Transform, Box<Error>> {
    let mut elems = s.split(":");
    match elems.next() {
        Some("rate") => Ok(Transform::Rate),
        Some("mul") => parse_next_int(elems.next()).map(Transform::Mul),
        Some("div") => parse_next_int(elems.next()).map(Transform::Div),
        Some("add") => parse_next_int(elems.next()).map(Transform::Add),
        Some("sub") => parse_next_int(elems.next()).map(Transform::Sub),
        Some(o) => Err(format!("operation {} is not supported", &o).into()),
        _ => Err("Expected tranformation, got empty string".into()),
    }
}

#[test]
fn parse_transform_str_test() {
    assert_eq!(parse_transform_str("rate").unwrap(), Transform::Rate);
    assert_eq!(parse_transform_str("mul:8").unwrap(), Transform::Mul(8));
    assert_eq!(parse_transform_str("div:2").unwrap(), Transform::Div(2));
    assert_eq!(parse_transform_str("add:-1").unwrap(), Transform::Add(-1));
    assert_eq!(parse_transform_str("sub:24").unwrap(), Transform::Sub(24));
}

impl<'de> Deserialize<'de> for Transform {
    fn deserialize<D>(deserializer: D) -> Result<Transform, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct TransformVisitor;
        impl<'de> serde::de::Visitor<'de> for TransformVisitor {
            type Value = Transform;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("expected \"<operation>\" or \"<operation>:<value>\"")
            }

            fn visit_str<E>(self, value: &str) -> Result<Transform, E>
            where
                E: serde::de::Error,
            {
                parse_transform_str(value).map_err(|e| serde::de::Error::custom(e))
            }
        }
        deserializer.deserialize_str(TransformVisitor)
    }
}

impl Serialize for Transform {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let result = match self {
            &Transform::Rate => format!("rate"),
            &Transform::Mul(nb) => format!("mul:{}", nb),
            &Transform::Div(nb) => format!("div:{}", nb),
            &Transform::Add(nb) => format!("add:{}", nb),
            &Transform::Sub(nb) => format!("sub:{}", nb),
        };
        serializer.serialize_str(&result)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum Collector {
    Prometheus { endpoint: String },
    Statsd { endpoint: String },
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
