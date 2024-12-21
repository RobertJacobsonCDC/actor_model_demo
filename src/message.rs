/*!

A `Message` is sent in an `Envelope`. The `Envelope` contains the target channel, the handle of the sender, and the `Message` (the payload).

*/

use std::{
  fmt::Debug,
  rc::Rc
};

use crate::{
  actor::ActorHandle,
  timeline::Time
};


pub type RcEnvelope = Rc<Envelope>;
pub type RcMessagePayload = Rc<dyn MessagePayload>;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Channel{
  // These are just examples. It's not clear if `Timeline` services should be
  // represented here.
  TimelineEvent, // Emitted by `Timeline`
  ScheduleEvent, // Request to schedule and event
  Topic(u32),
  Actor(ActorHandle),
  General,       // Catch all
}

// In principle, `Envelope` does not need to be `Copy` or even `Clone`, but we
// do it here for simplicity.
#[derive(Debug)]
pub struct Envelope {
  pub from   : ActorHandle,
  pub to     : Channel,
  pub message: Message
}

// ToDo: Is an enum really necessary? Why not just have a `MessagePayload`?
#[derive(Debug)]
pub enum Message{
  ScheduleEvent(RcEnvelope, Time),
  TimelineEvent(RcEnvelope, Time),
  Payload(RcMessagePayload),
}


/// `MessagePayload` marks a type as the content of a message.
pub trait MessagePayload: Debug {}

// A handful of message payloads
impl MessagePayload for () {}
impl MessagePayload for String {}
impl MessagePayload for ActorHandle {}
impl MessagePayload for u64 {}
impl MessagePayload for Time {}
