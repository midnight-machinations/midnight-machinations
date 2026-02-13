use crate::{log, websocket_connections::{connection::Connection, ForceLock}, websocket_listener::WebsocketListener};
use tokio_tungstenite::tungstenite::Message;
use std::{future::Future, net::SocketAddr, pin::pin, sync::Arc};
use tokio::sync::Mutex;

use futures_util::{future::{self, Either}, StreamExt, SinkExt};

use tokio::sync::{mpsc, broadcast};
use tokio::net::{TcpListener, TcpStream};

pub async fn create_ws_server(server_address: &str) {
    #[expect(clippy::panic, reason = "Server cannot start without TCP listener")]
    let tcp_listener = TcpListener::bind(&server_address).await.unwrap_or_else(|err| {
        panic!("Failed to bind websocket server to address {server_address}: {err}")
    });
    
    let mut crash_signal = broadcast::channel(1);

    {
        // Remove the hook from the previous server instance, if any.
        let _ = std::panic::take_hook();
        // Set the new hook
        let panic_crash_signal_sender = crash_signal.0.clone();
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = panic_crash_signal_sender.send(());
            original_hook(info)
        }))
    }

    let event_listener: Arc<Mutex<_>> = Arc::new(Mutex::new(WebsocketListener::new().await));
    WebsocketListener::start_tick(event_listener.clone());

    log!(important "Server"; "Started listening on {server_address}");

    loop {
        let (stream, client_address) = match future::select(
            pin!(tcp_listener.accept()), 
            pin!(crash_signal.1.recv())
        ).await {
            Either::Left((Ok((stream, client_address)), _)) => (stream, client_address),
            Either::Left((Err(_), _)) => continue, // TCP connection failed
            Either::Right(_) => break // Received crash signal
        };
        
        let event_listener = event_listener.clone();
        let crash_signal = (crash_signal.0.clone(), crash_signal.1.resubscribe());

        tokio::spawn(handle_connection(stream, client_address, event_listener.clone(), crash_signal));
    }

    log!(fatal "Server"; "The server panicked!");
    log!(important "Server"; "Shutting down...");
}

struct ConnectionError;

enum NextEvent<A, B, C>
where 
    A: Future + Unpin,
    B: Future + Unpin,
    C: Future + Unpin
{
    TcpRecieved(A::Output),
    MpscReceieved(B::Output),
    CrashSignal(C::Output),
}

impl<A, B, C> NextEvent<A, B, C> 
where 
    A : Future + Unpin,
    B : Future + Unpin,
    C : Future + Unpin
{
    async fn from_futures(tcp_message: A, mpsc_message: B, crash_signal: C) -> Self {
        match future::select(tcp_message, future::select(mpsc_message, crash_signal)).await {
            Either::Left((tcp_message, _)) => Self::TcpRecieved(tcp_message),
            Either::Right((Either::Left((mpsc_message, _)), _)) => Self::MpscReceieved(mpsc_message),
            Either::Right((Either::Right((crash_signal, _)), _)) => Self::CrashSignal(crash_signal),
        }
    }
}

// Code within this function __SHOULD NOT PANIC__ except for listener methods.
// There is a panic hook that restarts the server. The server doesn't need to restart if a connection fails, so don't panic -- just disconnect.
/// This runs until the connection is closed.
async fn handle_connection(
    raw_stream: TcpStream, 
    client_address: SocketAddr, 
    listener: Arc<Mutex<WebsocketListener>>,
    mut crash_signal: (broadcast::Sender<()>, broadcast::Receiver<()>)
) -> Result<(), ConnectionError> {
    let ws_stream = match tokio_tungstenite::accept_async(raw_stream).await {
        Ok(ws_stream) => ws_stream,
        Err(error) => {
            log!(info "Connection"; "Failed to accept websocket handshake with {}: {}", client_address, error);
            return Err(ConnectionError);
        }
    };

    // Messages in this channel get received and rerouted to the client over TCP
    let (mpsc_sender, mut mpsc_receiver) = mpsc::unbounded_channel();

    let (mut tcp_sender, mut tcp_receiver) = ws_stream.split();
    
    let connection = {
        let mut listener = listener.lock().await;
        let connection = Connection::new(mpsc_sender, client_address);
        log!(important "Connection"; "Connected: {}", client_address);
        listener.on_connect(&connection);
        connection
    };

    loop {
        match NextEvent::from_futures(
            pin!(tcp_receiver.next()),
            pin!(mpsc_receiver.recv()),
            pin!(crash_signal.1.recv())
        ).await {
            NextEvent::TcpRecieved(None) => break, // Channel has been closed
            NextEvent::TcpRecieved(Some(message)) => {
                let mut listener = listener.lock().await;

                match message {
                    Ok(message) => {
                        listener.on_message(&connection, &message);
                    }
                    Err(tokio_tungstenite::tungstenite::Error::ConnectionClosed) => break,
                    Err(err) => {
                        log!(error "Connection"; "Failed to receive packet. {}", err);
                        break
                    },
                }
            }
            NextEvent::MpscReceieved(None) => break, // Channel has been closed
            NextEvent::MpscReceieved(Some(message)) => {
                let Ok(json_message) = serde_json::to_string(&message) else {
                    log!(error "Connection"; "Failed to parse packet. {:?}", &message);
                    break
                };
    
                match tcp_sender.send(Message::text(json_message)).await {
                    Ok(_) => {},
                    Err(tokio_tungstenite::tungstenite::Error::ConnectionClosed) => break,
                    Err(err) => {
                        log!(error "Connection"; "Failed to send packet. {}", err);
                        break
                    },
                }
            }
            NextEvent::CrashSignal(..) => break, // Server has been closed
        };
    }

    let _ = tcp_sender.close().await;

    listener.lock().await.on_disconnect(connection);
    log!(important "Connection"; "Disconnected {}", client_address);

    Ok(())
}
