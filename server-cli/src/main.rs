use log::info;

fn main() {
    env_logger::init();
    info!("Starting Server CLI");
    server::server().unwrap();
}
