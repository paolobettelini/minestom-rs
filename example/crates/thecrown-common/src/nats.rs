use async_nats::{ConnectError, Subscriber, ToServerAddrs, client::Client, subject::ToSubject};
use futures::StreamExt;
use serde::Deserialize;
use serde::ser::Serialize;
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use thecrown_protocol::ProtocolPacket;

#[derive(Debug, Clone)]
pub struct NatsClient {
    nats: Client,
}

/// The type for the async handler function used when subscripting to a queue
pub type CallbackType<StateType, PacketType> = dyn for<'a> Fn(
        &'a StateType,
        PacketType,
    ) -> Pin<Box<dyn Future<Output = Option<PacketType>> + Send + 'a>>
    + Send
    + Sync;

impl NatsClient {
    pub async fn new<A: ToServerAddrs>(addrs: A) -> Result<Self, ConnectError> {
        let nats = async_nats::connect(addrs).await?;
        let client = NatsClient { nats };

        Ok(client)
    }

    pub async fn publish<T: Serialize + ProtocolPacket>(&self, value: &T) {
        let subject = <T as ProtocolPacket>::get_nats_subject();
        log::info!("Publishing packet to {}", &subject);
        self.publish_with_subject(subject, value).await;
    }

    pub async fn publish_with_subject<T: Serialize, S: ToSubject>(&self, subject: S, value: &T) {
        let msg = match serde_cbor::to_vec(&value) {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("Failed to serialize value: {}", e);
                return;
            }
        };

        if let Err(e) = self.nats.publish(subject, msg.into()).await {
            log::error!("Failed to publish message: {}", e);
            return;
        }

        // Flush buffer
        if let Err(e) = self.nats.flush().await {
            log::error!("Failed to flush client: {}", e);
            return;
        }
    }

    pub async fn request<T: Serialize + for<'a> Deserialize<'a> + ProtocolPacket>(
        &self,
        value: &T,
    ) -> Option<T> {
        let subject = <T as ProtocolPacket>::get_nats_subject();
        self.request_with_subject(subject, value).await
    }

    pub async fn request_with_subject<T: Serialize + for<'a> Deserialize<'a>, S: ToSubject>(
        &self,
        subject: S,
        value: &T,
    ) -> Option<T> {
        let msg = match serde_cbor::to_vec(&value) {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("Failed to serialize value: {}", e);
                return None;
            }
        };

        match self.nats.request(subject, msg.into()).await {
            Err(e) => {
                log::error!("Failed to publish message: {}", e);
            }
            Ok(resp) => match serde_cbor::from_slice::<T>(&resp.payload) {
                Err(e) => {
                    log::error!("Failed to deserialize message: {}", e);
                }
                Ok(res) => return Some(res),
            },
        }

        None
    }

    pub async fn handle_subscription<StateType, PacketType>(
        &self,
        state: StateType,
        callback: &CallbackType<StateType, PacketType>,
    ) where
        PacketType: Serialize + ProtocolPacket + for<'a> Deserialize<'a>,
    {
        // Subscribe to the subject
        let subject = <PacketType as ProtocolPacket>::get_nats_subject();
        self.handle_subscription_with_subject(subject, state, callback)
            .await;
    }

    pub async fn handle_subscription_with_subject<StateType, PacketType, S: ToSubject + Display>(
        &self,
        subject: S,
        state: StateType,
        callback: &CallbackType<StateType, PacketType>,
    ) where
        PacketType: Serialize + ProtocolPacket + for<'a> Deserialize<'a>,
    {
        log::info!("Subscribing to {subject}");
        let mut subscription = self.subscribe(subject).await;

        while let Some(request) = subscription.next().await {
            // Deserialize packet
            let msg = match serde_cbor::from_slice(&request.payload) {
                Ok(msg) => msg,
                Err(e) => {
                    log::error!("Failed to deserialize packet: {}", e);
                    continue;
                }
            };

            // Packet logic
            let reply_pkct = callback(&state, msg).await;

            // Reply if necessary
            if let Some(reply_pkct) = reply_pkct {
                if let Some(reply_subject) = request.reply {
                    self.publish_with_subject(reply_subject, &reply_pkct).await;
                }
            }
        }
    }

    async fn subscribe<S: ToSubject>(&self, subject: S) -> Subscriber {
        match self.nats.subscribe(subject).await {
            Ok(subscriber) => subscriber,
            Err(e) => {
                log::error!("Failed to subscribe to subject: {}", e);
                std::process::exit(1);
            }
        }
    }
}
