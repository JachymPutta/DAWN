use atomic_enum::atomic_enum;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::mpsc,
};

use debug_types::{
    events::{Event, EventBody},
    responses::Response,
};
use futures::{SinkExt, StreamExt};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc,
    },
    thread::JoinHandle,
};
use tokio_util::codec::{FramedRead, FramedWrite};
use tvix_debugger::commands::{Command, CommandReply};

use either::Either;
use tracing::error;

use crate::{
    codec::DebugAdapterCodec,
    dap_requests::{ExtendedMessageKind, ExtendedProtocolMessage, ExtendedRequestCommand},
};

/// A list of possible states the adapter can be in.
/// NOTE: the atomic version of this (which we intend to use!)
/// is backed by a usize
/// and is denoted `AtomicState`
#[atomic_enum]
#[derive(PartialEq, PartialOrd)]
pub enum State {
    /// Server has not received an `initialize` request.
    Uninitialized = 0,
    /// Server received an `initialize` request, but has not yet responded.
    Initializing = 1,
    /// Server received and responded success to an `initialize` request.
    Initialized = 2,
    /// Server received a `shutdown` request.
    ShutDown = 3,
    /// Server received an `exit` notification.
    Exited = 4,
}

/// Struct used to communicated with the debugger server process
pub struct Server {
    /// channel to send commands to the debugger
    pub sender: mpsc::Sender<Command>,
    /// channel for replies from the debugger
    pub receiver: mpsc::Receiver<CommandReply>,
    /// debugger handle
    pub debugger: JoinHandle<()>,
    /// shutdown token
    pub shutdown: Arc<AtomicBool>,
}

/// Struct used to abstract away communication with client
/// TODO rename to something a bit more intuitive
pub struct Client<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    /// which state we're on
    state: AtomicState,
    /// the sequence number we're on
    send_seq: i64,
    /// reader for stdin
    pub reader: FramedRead<R, DebugAdapterCodec<ExtendedProtocolMessage>>,
    /// reader for stdout
    pub writer: FramedWrite<W, DebugAdapterCodec<ExtendedProtocolMessage>>,
}

impl<R, W> Client<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    /// create new client
    #[must_use]
    pub fn new(
        reader: FramedRead<R, DebugAdapterCodec<ExtendedProtocolMessage>>,
        writer: FramedWrite<W, DebugAdapterCodec<ExtendedProtocolMessage>>,
    ) -> Self {
        Self {
            state: State::Uninitialized.into(),
            send_seq: 0,
            reader,
            writer,
        }
    }

    /// modify the underlying state
    /// Note: we expect to ONLY EVER INCREMENT BY ONE
    /// and this code is designed to do that.
    /// # Panics
    /// - if the swap is not what's expected
    /// - if `compare_exchange` somehow doesn't do what's expeected in the success case
    pub fn set_state(&mut self, new_state: State) {
        // prevent overflow
        if new_state == State::Uninitialized {
            error!("Not setting state. Only can progress forward");
        }
        let current_state: State = AtomicState::from_usize(new_state as usize - 1);

        error!("SETTING TO {new_state:?}");
        let result = self
            .state
            .compare_exchange(current_state, new_state, Relaxed, Relaxed);
        error!("SET TO {new_state:?}");

        match result {
            Ok(viewed_state) => {
                assert!(viewed_state == current_state);
                error!("Successfully set state to {new_state:?}");
            }
            Err(viewed_state) => {
                error!("Failed to set state! Old state was {viewed_state:?}, but we expected {current_state:?}");
            }
        }
    }

    /// get the underlying state
    #[must_use]
    pub fn get_state(&self) -> State {
        self.state.load(Relaxed)
    }

    // TODO get send_response and send_event

    /// send event to client (only possible way)
    pub async fn send(&mut self, body: Either<EventBody, Response>) {
        let message = match body {
            Either::Left(event_body) => ExtendedMessageKind::Event(Event {
                body: Some(event_body),
            }),
            Either::Right(response_body) => ExtendedMessageKind::Response(response_body),
        };

        let message = ExtendedProtocolMessage {
            seq: self.send_seq,
            message,
        };

        if let Err(e) = self.writer.send(message).await {
            // TODO should this be panic?
            error!("Error sending response{e}");
        }

        self.send_seq += 1;
    }

    /// request next message of substance from client
    pub async fn next_msg(&mut self) -> ExtendedProtocolMessage {
        loop {
            if let Some(msg) = self.reader.next().await {
                match msg {
                    Ok(r) => {
                        return r;
                    }
                    Err(e) => {
                        error!("Error parsing message: {e:?}");
                    }
                }
            }
        }
    }
}

/// a debug adapter mt
pub trait DebugAdapter {
    /// how to handle various requests
    fn handle_request(
        &mut self,
        seq: i64,
        command: ExtendedRequestCommand,
    ) -> impl std::future::Future<Output = ()>;
}
