use nerve_search_adapter::client;
use tracing::info;

fn main()->std::io::Result<()>{
    tracing_subscriber::fmt::init();

    let socket_path = "/tmp/nerve.sock";
    info!("starting NERVE-SEARCH-ADAPTER");

    client::run(socket_path)
}