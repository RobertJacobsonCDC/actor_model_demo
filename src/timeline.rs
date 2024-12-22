/*!

A `Timeline` emits messages at scheduled "times." `Actor`s can schedule messages on a `Timeline`.

*/

use std::{
  cmp::{Ordering, Reverse},
  collections::{BinaryHeap},
  fmt::Debug
};

use ordered_float::OrderedFloat;

use crate::{
  message::RcEnvelope,
  message::BoundedTopic
};

pub type Time = OrderedFloat<f64>;


// region Event

pub struct Event<Message, Topic>
    where Message: Clone + Debug,
          Topic  : BoundedTopic
{
  pub time    : Time,
  /// We take the simple approach of just taking an `Envelope` and letting the scheduler
  /// decide its contents. This way there is a `to` and `from` built-in.
  pub envelope: RcEnvelope<Message, Topic>,
  // We could also record the actor who scheduled the event, etc.
}

// Implements ordering of events in the timeline's priority queue. This is necessary because `BinaryHeap` is a max heap, not a min heap, and we want a min heap.
//
// Be warned that `Event`s are equal if they are scheduled at the same time regardless of envelope.
impl<Message, Topic> PartialEq for Event<Message, Topic>
    where Message: Clone + Debug,
          Topic  : BoundedTopic
{
  fn eq(&self, other: &Self) -> bool {
    self.time == other.time
  }
}
impl<Message, Topic> Eq for Event<Message, Topic>
    where Message: Clone + Debug,
          Topic  : BoundedTopic
{}
impl<Message, Topic> PartialOrd for Event<Message, Topic>
    where Message: Clone + Debug,
          Topic  : BoundedTopic
{
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(&other))
  }
}
impl<Message, Topic> Ord for Event<Message, Topic>
    where Message: Clone + Debug,
          Topic  : BoundedTopic
{
  fn cmp(&self, other: &Self) -> Ordering {
    Reverse(self.time).cmp(&Reverse(other.time))
  }
}

// endregion Event

pub struct Timeline<Message, Topic>
    where Message: Clone + Debug,
          Topic  : BoundedTopic
{
  now         : Time,
  event_queue : BinaryHeap<Event<Message, Topic>>,
  // actor_handle: ActorHandle,
}

impl<Message, Topic> Default for Timeline<Message, Topic>
where Message: Clone + Debug,
      Topic  : BoundedTopic
{
  fn default() -> Self {
    Self {
      now         : Time::default(),
      event_queue : BinaryHeap::new(),
      // actor_handle: ActorHandle::default(),
    }
  }
}


impl<Message, Topic> Timeline<Message, Topic>
    where Message: Clone + Debug,
          Topic  : BoundedTopic
{
  #[inline(always)]
  pub fn now(&self) -> Time {
    self.now
  }

  #[inline(always)]
  pub fn push(&mut self, event: Event<Message, Topic>) {
    self.event_queue.push(event)
  }

  #[inline(always)]
  pub fn pop(&mut self) -> Option<Event<Message, Topic>> {
    let popped = self.event_queue.pop();
    if let Some(Event{ time, .. }) = &popped {
      self.now = time.clone();
    }

    popped
  }
}

/*
impl<Message, Topic> Actor<Message, Topic> for Timeline<Message, Topic>
    where Message: Clone + Debug,
          Topic  : BoundedTopic
{
  /// Destructures the message and schedules the event it contains
  fn receive_message(&mut self, envelope: RcEnvelope<Message, Topic>) -> Vec<RcEnvelope<Message, Topic>> {
    let Envelope{ from: _from_actor, to: from_channel, message, time } = envelope.as_ref();

    if let (Channel::ScheduleEvent, Some(time)) = (from_channel, time) {
      self.push(
        Event{
          time    : time.clone(),
          envelope: envelope.clone()
        }
      );
    } else {
      // This implementation only receives `ScheduleEvent` messages.
      unreachable!("Malformed Envelope sent to Channel::ScheduleEvent");
    }

    // Nothing to emit
    vec![]
  }

  fn register(&mut self, handle: ActorHandle) -> (Vec<Channel<Topic>>, Vec<RcEnvelope<Message, Topic>>) {
    self.actor_handle = handle;

    // We subscribe to requests to schedule events and emit no messages.
    (vec![Channel::ScheduleEvent], vec![])
  }
}
*/
