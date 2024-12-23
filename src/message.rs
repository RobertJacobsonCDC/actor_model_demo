/*!

A `Message` is sent in an `Envelope`. The `Envelope` contains the target
channel, the handle of the sender, and the `Message` (the payload).

The structure of messages can be as simple or as complex as you want. I've used
composition of `Channel`s, `Envelope`s, `Message`s, and `MessagePayload`s, but that's
not the only way to encode and route messages that makes sense. In fact, having the
content of the message (`MessagePayload`) be three layers deep might be overkill.

*/

use std::{
  fmt::Debug,
  rc::Rc,
  hash::Hash
};

use crate::{
  actor::ActorHandle,
  timeline::Time
};

// Envelopes and messages should generally be immutable, as multiple actors
// will potentially access them.
pub type RcEnvelope<M, T> = Rc<Envelope<M, T>>;

/// The `Channel` struct below is parameterized by `Topic` which as a lot of trait
/// bounds. Instead of listing all the bounds everywhere, we just require `Topic`
/// to implement a trait that has all the bounds as supertraits.
///
/// An alternative to this is to make topics a numeric `TopicHandle` which is assigned
/// by something that client code registers the topic with (presumably `Router`).
pub trait BoundedTopic: Copy + Clone + Debug + PartialEq + Eq + Hash {}
/// Conveniently, we implement `BoundedTopic` for *any* type with the right trait bounds.
impl<T> BoundedTopic for T where T: Copy + Clone + Debug + PartialEq + Eq + Hash {}


/// `Channel`s are the recipient's of messages (`Envelope`s). You could conceivably
/// just have a `Topic` generic, but having a parameterized `Channel` guarantees
/// variants for timeline-related messages.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Channel<Topic>
    where Topic: BoundedTopic
{
  // System Messages
  Stop,  // Request early exit
  Debug, // Request command line

  // Timeline-related Messages
  TimelineEvent, // Emitted by `Timeline`. Could choose to allow topic in here, too.
  ScheduleEvent, // Request to schedule an event
  Time,          // Time request and answer

  // Channels used by client code. This is the mechanism by which we extend `Channel`.
  Topic(Topic),

  // Just some examples
  Actor(ActorHandle),
  General,       // Catch all
}

impl<Topic> Channel<Topic>
    where Topic: BoundedTopic
{
  pub fn with_topic<T: Into<Topic>>(self, topic: T) -> Self {
    Channel::Topic(topic.into())
  }
}

#[derive(Debug)]
pub struct Envelope<Message, Topic>
    where Topic: BoundedTopic,
          Message: Clone + Debug
{
  pub from   : ActorHandle,
  pub channel: Channel<Topic>,
  pub message: Option<Message>,
  pub time   : Option<Time>
}
