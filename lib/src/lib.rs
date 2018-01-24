extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate toml;

use regex::Regex;
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeSeq;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub struct Range(pub i64, pub i64);

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
                let lo: Option<String> = sa.next_element()?;
                let hi: Option<String> = sa.next_element()?;
                if let (Some(l), Some(h)) = (lo, hi) {
                    let lo_p = parse_with_unit(&l).map_err(|e| serde::de::Error::custom(e))?;
                    let hi_p = parse_with_unit(&h).map_err(|e| serde::de::Error::custom(e))?;
                    Ok(Range(lo_p, hi_p))
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
        seq.serialize_element(&format!("{}", self.0))?;
        seq.serialize_element(&format!("{}", self.1))?;
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

#[derive(Debug, PartialEq)]
pub enum AggOp {
    Sum,
    Mean,
}

#[derive(Debug, PartialEq)]
pub struct Aggregate(pub AggOp, pub Option<String>);

fn parse_aggregate_str(s: &str) -> Result<Aggregate, Box<Error>> {
    let mut elems = s.split(":");
    let res_op: Result<AggOp, Box<Error>> = match elems.next() {
        Some("sum") => Ok(AggOp::Sum),
        Some("mean") => Ok(AggOp::Mean),
        Some(o) => Err(format!("operation {} is not supported", &o).into()),
        _ => Err("Expected aggregate operation, got empty string".into()),
    };
    let op = res_op?;

    Ok(Aggregate(op, elems.next().map(|s| s.to_owned())))
}

#[test]
fn parse_aggregate_str_test() {
    assert_eq!(
        parse_aggregate_str("sum").unwrap(),
        Aggregate(AggOp::Sum, None)
    );
    assert_eq!(
        parse_aggregate_str("mean").unwrap(),
        Aggregate(AggOp::Mean, None)
    );
    assert_eq!(
        parse_aggregate_str("sum:broker").unwrap(),
        Aggregate(AggOp::Sum, Some("broker".into()))
    );
    assert_eq!(
        parse_aggregate_str("mean:broker").unwrap(),
        Aggregate(AggOp::Mean, Some("broker".into()))
    );
}

impl<'de> Deserialize<'de> for Aggregate {
    fn deserialize<D>(deserializer: D) -> Result<Aggregate, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct AggregateVisitor;
        impl<'de> serde::de::Visitor<'de> for AggregateVisitor {
            type Value = Aggregate;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("expected \"<operation>\" or \"<operation>:<label>\"")
            }

            fn visit_str<E>(self, value: &str) -> Result<Aggregate, E>
            where
                E: serde::de::Error,
            {
                parse_aggregate_str(value).map_err(|e| serde::de::Error::custom(e))
            }
        }
        deserializer.deserialize_str(AggregateVisitor)
    }
}

impl Serialize for Aggregate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let op_str = match &self.0 {
            &AggOp::Sum => format!("sum"),
            &AggOp::Mean => format!("mean"),
        };
        serializer.serialize_str(&op_str)
    }
}

#[derive(Debug, PartialEq)]
pub enum Transform {
    Rate,
    Mul(i64),
    Div(i64),
    Add(i64),
    Sub(i64),
}

fn parse_with_unit(s: &str) -> Result<i64, Box<Error>> {
    let re = Regex::new(r"^(-?\d+)([KMGT]?)$")?;
    let result: Result<regex::Captures, Box<Error>> = re.captures(s).ok_or("Missing number".into());
    let captures = result?;
    let number_str = &captures.get(1).unwrap();
    let unit_str = &captures.get(2);

    let number: i64 = FromStr::from_str(number_str.as_str())?;
    let unit_mul: i64 = match unit_str.map(|e| e.as_str()) {
        Some("K") => 1024,
        Some("M") => 1024_i64 * 1024,
        Some("G") => 1024_i64 * 1024 * 1024,
        Some("T") => 1024_i64 * 1024 * 1024 * 1024,
        Some("") | None => 1,
        Some(_) => unimplemented!(),
    };

    Ok(number * unit_mul)
}

#[test]
fn test_parse_with_unit() {
    assert_eq!(parse_with_unit("10").unwrap(), 10);
    assert_eq!(parse_with_unit("10K").unwrap(), 10_i64 * 1024);
    assert_eq!(parse_with_unit("10M").unwrap(), 10_i64 * 1024 * 1024);
    assert_eq!(parse_with_unit("10G").unwrap(), 10_i64 * 1024 * 1024 * 1024);
    assert_eq!(
        parse_with_unit("10T").unwrap(),
        10_i64 * 1024 * 1024 * 1024 * 1024
    );

    assert_eq!(parse_with_unit("-10").unwrap(), -10);
    assert_eq!(parse_with_unit("-10K").unwrap(), -10_i64 * 1024);
    assert_eq!(parse_with_unit("-10M").unwrap(), -10_i64 * 1024 * 1024);
    assert_eq!(
        parse_with_unit("-10G").unwrap(),
        -10_i64 * 1024 * 1024 * 1024
    );
    assert_eq!(
        parse_with_unit("-10T").unwrap(),
        -10_i64 * 1024 * 1024 * 1024 * 1024
    );
}

fn parse_next_int(os: Option<&str>) -> Result<i64, Box<Error>> {
    match os {
        Some(s) => {
            let n: i64 = parse_with_unit(s)?;
            Ok(n)
        }
        None => Err("Missing parameter".into()),
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
    assert_eq!(
        parse_transform_str("div:2G").unwrap(),
        Transform::Div(2_i64 * 1024 * 1024 * 1024)
    );
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
    Stacked,
    Inverted,
    Percentage,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Metric {
    pub name:      String,
    pub unit:      Option<String>,
    pub selector:  String,
    pub aggregate: Option<Vec<Aggregate>>,
    pub transform: Option<Vec<Transform>>,
    pub display:   Option<Vec<Display>>,
    pub axis:      Option<Axis>,
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

pub fn generate_toml(metrics: &Metrics) -> Result<String, Box<Error>> {
    let o = toml::to_string(metrics)?;
    Ok(o)
}

pub fn generate_yaml(metrics: &Metrics) -> Result<String, Box<Error>> {
    let o = serde_yaml::to_string(metrics)?;
    Ok(o)
}
