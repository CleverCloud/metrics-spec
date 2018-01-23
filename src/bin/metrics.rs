#[macro_use]
extern crate maplit;
extern crate metrics_lib;
extern crate serde_yaml;
extern crate toml;

use std::io::{self, Read};

use metrics_lib::*;



fn main() {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    //let c = parse_toml(&buffer);
    let c = parse_yaml(&buffer);
    println!("{:?}", &c);

    let metric = Metric {
        name:     "test".into(),
        selector: "test".into(),
        axis:     Some(Axis::Y1),
        aggregate: Some(vec![Aggregate(AggOp::Mean, None)]),
        transform: Some(vec![Transform::Sub(20)]),
        display: Some(vec![Display::Stacked]),
    };
    let group = Group {
        name:       "group_name".into(),
        metrics:    vec![metric],
        range:      Some(Range(0, 100)),
        axis_range: None,
    };

    let metrics = Metrics {
        collect: Collector::Prometheus {
            endpoint: "http://localhost:9200".into()
        },
        groups:  hashmap!(
            "group1".into() => group
        ),
    };
    let output = toml::to_string(&metrics).unwrap();
    println!("{}", &output);

    let outputy = serde_yaml::to_string(&metrics).unwrap();
    println!("{}", &outputy);
}
