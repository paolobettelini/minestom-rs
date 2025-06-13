use std::sync::Arc;
use thecrown_common::nats::NatsClient;
mod state;
mod handler;
use handler::*;
use thecrown_common::nats::CallbackType;
use tokio::task;
use state::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    /* === Read configuration === */

    //let args = args::App::parse();
    //let config = parse_toml_config::<_, RelayConfig>(args.config)?;

    /* === Init logger === */

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
        .init();

    /* === Construct global state === */

    //let db_url = env::var(config.database.url_env)?;
    let nats_url = String::from("127.0.0.1:4222");

    //let db_client = Arc::new(Database::new(db_url));
    let nats_client = Arc::new(NatsClient::new(nats_url).await?);

    let state = State::new(nats_client.clone() /*, db_client */);
    //state.start_cleanup_task().await;

    /* === Nats consumer loop === */
    let async_handler: Arc<CallbackType<_, _>> =
        Arc::new(|state, msg| Box::pin(handle_msg(state, msg)));
    let task_handle = task::spawn(async move {
        nats_client
            .clone()
            .handle_subscription(state.clone(), async_handler.as_ref())
            .await;
    });
    let out = task_handle.await?;

    Ok(())
}
