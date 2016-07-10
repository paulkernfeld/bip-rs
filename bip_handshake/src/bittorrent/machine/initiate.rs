use std::sync::mpsc::{Sender, Receiver};
use std::net::SocketAddr;

use bip_util::send::TrySender;
use rotor::{Void, Scope, Response, Machine, EventSet, Notifier};
use rotor_stream::{Protocol, Accepted};

use bittorrent::handshake::HandshakeSeed;
use bittorrent::machine::status::PeerStatus;
use bittorrent::seed::{CompleteSeed, InitiateSeed};
use try_connect::TryConnect;

pub enum InitiateMessage {
    Initiate(InitiateSeed),
    Shutdown
}

// ----------------------------------------------------------------------------//

pub struct InitiateSender<S> {
    send: S,
    noti: Notifier,
}

impl<S> InitiateSender<S> {
    pub fn new(send: S, noti: Notifier) -> InitiateSender<S> {
        InitiateSender {
            send: send,
            noti: noti,
        }
    }
}

impl<S, T> TrySender<T> for InitiateSender<S>
    where S: TrySender<T>,
          T: Send
{
    fn try_send(&self, data: T) -> Option<T> {
        let ret = self.send.try_send(data);

        if ret.is_none() {
            self.noti
                .wakeup()
                .expect("bip_handshake: Failed To Wakeup State Machine To Initiate Connection")
        }
        ret
    }
}

impl<S> Clone for InitiateSender<S>
    where S: Clone
{
    fn clone(&self) -> InitiateSender<S> {
        InitiateSender {
            send: self.send.clone(),
            noti: self.noti.clone(),
        }
    }
}

// ----------------------------------------------------------------------------//

pub enum Initiate<H, C>
    where H: Protocol,
          C: Protocol
{
    Peer(PeerStatus<H, C>),
    Recv(Receiver<InitiateMessage>),
}

impl<H, C> Initiate<H, C>
    where H: Protocol,
          C: Protocol,
          C::Socket: TryConnect
{
    /// Try to receive an initiation seed from the given receiver.
    ///
    /// If a seed is received, a connection will be attempted and
    /// if successful, a new Peer state machine will be spawned.
    fn try_receive(recv: Receiver<InitiateMessage>, scope: &mut Scope<H::Context>) -> Response<Self, (C::Socket, InitiateSeed)> {
        let opt_message = recv.try_recv().ok();

        let self_variant = Initiate::Recv(recv);
        match opt_message {
            Some(InitiateMessage::Initiate(init)) => {
                match C::Socket::try_connect(init.addr()).ok() {
                    Some(stream) => Response::spawn(self_variant, (stream, init)),
                    None => Response::ok(self_variant)
                }
            },
            Some(InitiateMessage::Shutdown) => {
                scope.shutdown_loop();

                Response::done()
            },
            None => Response::ok(self_variant)
        }
    }
}

impl<H, C> Accepted for Initiate<H, C>
    where H: Protocol<Context = C::Context, Seed = (HandshakeSeed, Sender<C::Seed>), Socket = C::Socket>,
          C: Protocol,
          C::Socket: TryConnect
{
    type Seed = SocketAddr;
    type Socket = C::Socket;

    fn accepted(sock: Self::Socket, seed: SocketAddr, scope: &mut Scope<Self::Context>) -> Response<Self, Void> {
        PeerStatus::complete(CompleteSeed::new(seed), sock, scope).wrap(Initiate::Peer)
    }
}

impl<H, C> Machine for Initiate<H, C>
    where H: Protocol<Context = C::Context, Seed = (HandshakeSeed, Sender<C::Seed>), Socket = C::Socket>,
          C: Protocol,
          C::Socket: TryConnect
{
    type Context = H::Context;
    type Seed = (C::Socket, InitiateSeed);

    fn create((sock, seed): Self::Seed, scope: &mut Scope<Self::Context>) -> Response<Self, Void> {
        PeerStatus::initiate(seed, sock, scope).wrap(Initiate::Peer)
    }

    fn ready(self, events: EventSet, scope: &mut Scope<Self::Context>) -> Response<Self, Self::Seed> {
        match self {
            Initiate::Peer(p) => p.ready(events, scope).map(Initiate::Peer, |_| unreachable!()),
            Initiate::Recv(_) => unreachable!(),
        }
    }

    fn spawned(self, scope: &mut Scope<Self::Context>) -> Response<Self, Self::Seed> {
        match self {
            Initiate::Peer(_) => unreachable!(),
            Initiate::Recv(r) => Initiate::try_receive(r, scope),
        }
    }

    fn timeout(self, scope: &mut Scope<Self::Context>) -> Response<Self, Self::Seed> {
        match self {
            Initiate::Peer(p) => p.timeout(scope).map(Initiate::Peer, |_| unreachable!()),
            Initiate::Recv(_) => unreachable!(),
        }
    }

    fn wakeup(self, scope: &mut Scope<Self::Context>) -> Response<Self, Self::Seed> {
        match self {
            Initiate::Peer(p) => p.wakeup(scope).map(Initiate::Peer, |_| unreachable!()),
            Initiate::Recv(r) => Initiate::try_receive(r, scope),
        }
    }
}