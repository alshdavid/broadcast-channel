use std::sync::Arc;
use std::thread::JoinHandle;
use std::thread::{self};
use std::sync::mpsc::SendError;

use kanal::Receiver;
use kanal::Sender;
use kanal::unbounded;

#[derive(Clone)]
pub struct Subject {
  sender: Sender<SubjectMessage>,
  _handle: Arc<JoinHandle<()>>,
}

impl Subject {
  pub fn new() -> Self {
    let (sender, rx) = unbounded::<SubjectMessage>();

    let handle = thread::spawn(move || {
      let mut senders = Vec::<Sender<usize>>::new();
      'main_loop: while let Ok(msg) = rx.recv() {
        match msg {
          SubjectMessage::Send(data) => senders.retain(|sender| sender.send(data.clone()).is_ok()),
          SubjectMessage::Subscribe(sender) => senders.push(sender),
          SubjectMessage::Disconnect => break 'main_loop,
        }
      }
    });

    return Subject {
      sender,
      _handle: Arc::new(handle),
    };
  }

  pub fn send(
    &self,
    data: usize,
  ) -> Result<(), SendError<usize>> {
    if self.sender.send(SubjectMessage::Send(data.clone())).is_err() {
      return Err(SendError(data));
    };
    return Ok(());
  }

  pub fn subscribe(&self) -> Result<Receiver<usize>, ()> {
    let (tx, rx) = unbounded::<usize>();
    if self.sender.send(SubjectMessage::Subscribe(tx)).is_err() {
      return Err(());
    };
    return Ok(rx);
  }

  pub fn close(&self) -> Result<(), ()> {
    if self.sender.send(SubjectMessage::Disconnect).is_err() {
      return Err(());
    }
    return Ok(());
  }
}

enum SubjectMessage {
  Send(usize),
  Subscribe(Sender<usize>),
  Disconnect,
}