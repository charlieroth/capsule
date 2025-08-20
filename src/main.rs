use capsule::config;

fn main() {
    let config = config::Config::from_env().expect("Failed to load configuration");
    println!("config: {:?}", config);
}
