use template_rust::core::config::Config;
use template_rust::domain::health::get_health;

fn main() {
    let cfg = Config::from_env();
    let health = get_health(&cfg.app_name, &cfg.app_env);

    println!(
        "Application: {} [Env: {}, Port: {}]",
        health.app_name, health.environment, cfg.port
    );
    println!("Status: {} at {}", health.status, health.timestamp);
}
