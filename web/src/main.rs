extern crate metrics_lib;
#[macro_use]
extern crate stdweb;

fn toml_to_yaml( string: String ) -> String {
    let parsed = metrics_lib::parse_toml(&string).unwrap();
    let yaml = metrics_lib::generate_yaml(&parsed).unwrap();
    yaml.to_string()
}

fn main() {
    stdweb::initialize();

    js! {
        Module.exports.toml_to_yaml = @{toml_to_yaml};
    }
}
