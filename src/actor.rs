/*

An `Actor` is an entity that can emit messages and subscribe to channels.

*/

use std::fmt::Debug;

use crate::{
  message::{
    Channel,
    RcEnvelope,
    BoundedTopic
  },
  rccell::RcCell,
};

pub type ActorHandle = u32;
pub type RcActor<Message, Topic> = RcCell<dyn Actor<Message, Topic>>;

pub trait Actor<Message, Topic>
    where Message: Clone + Debug,
          Topic  : BoundedTopic
{
  /// A message is delivered to this `Actor`. The `Actor` has the opportunity to respond
  fn receive_message(&mut self, envelope: RcEnvelope<Message, Topic>) -> Vec<RcEnvelope<Message, Topic>>;

  /// Called when the `Router` is adding this actor with the provided `ActorHandle`.
  /// Implementations should store their own `ActorHandle` for later use. The
  /// `Actor` has an opportunity to subscribe to channels and send initial messages.
  fn register(&mut self, handle: ActorHandle) -> (Vec<Channel<Topic>>, Vec<RcEnvelope<Message, Topic>>);
}
