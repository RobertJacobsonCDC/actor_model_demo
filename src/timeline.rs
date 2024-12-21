/*!

A `Timeline` emits messages at scheduled "times." `Actor`s can schedule messages on a `Timeline`.

*/

use std::{
  any::Any,
  cmp::{Ordering, Reverse},
  collections::{BinaryHeap},
  rc::Rc,
};

use ordered_float::OrderedFloat;

use crate::{
  actor::{Actor, ActorHandle},
  message::{
    Channel,
    Envelope,
    Message,
    RcEnvelope
  },
};

pub type Time = OrderedFloat<f64>;


// region Event

pub struct Event {
  pub time    : Time,
  pub envelope: RcEnvelope,
  // We could also record the actor who scheduled the event, etc.
}

// Implements ordering of events in the timeline's priority queue. This is necessary because `BinaryHeap` is a max heap, not a min heap, and we want a min heap.
//
// Be warned that `Event`s are equal if they are scheduled at the same time regardless of envelope.
impl PartialEq for Event {
  fn eq(&self, other: &Self) -> bool {
    self.time == other.time
  }
}
impl Eq for Event {}
impl PartialOrd for Event {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(&other))
  }
}
impl Ord for Event {
  fn cmp(&self, other: &Self) -> Ordering {
    Reverse(self.time).cmp(&Reverse(other.time))
  }
}

// endregion Event

#[derive(Default)]
pub struct Timeline {
  now         : Time,
  event_queue : BinaryHeap<Event>,
  actor_handle: ActorHandle,
}


impl Timeline {
  #[inline(always)]
  pub fn push(&mut self, event: Event) {
    self.event_queue.push(event)
  }

  #[inline(always)]
  pub fn pop(&mut self) -> Option<Event> {
    let popped = self.event_queue.pop();
    if let Some(Event{ time, .. }) = &popped {
      self.now = time.clone();
    }

    popped
  }
}

impl Actor for Timeline {
  fn as_any(&self) -> &dyn Any { self }
  fn as_any_mut(&mut self) -> &mut dyn Any { self }

  fn emit_message(&mut self) -> RcEnvelope {
    match self.event_queue.pop() {

      Some(Event{time, envelope}) => {
        Rc::new(
          Envelope{
            from   : self.actor_handle,
            to     : Channel::TimelineEvent,
            message: Message::TimelineEvent(envelope, time)
          }
        )
      }

      None => {
        // The event queue is checked first before `emit_message(..)` is called,
        // so this is unreachable.
        unreachable!()
      }
    }
  }

  fn receive_message(&mut self, envelope: RcEnvelope) {
    // Destructures the message and schedules the event it contains
    let Envelope{ from: _from_actor, to: from_channel, message } = envelope.as_ref();

    if let Channel::ScheduleEvent = from_channel {
      if let Message::ScheduleEvent(event_envelope, time) = message {
        self.push(
          Event{
            time: time.clone(),
            envelope: event_envelope.clone()
          }
        );
      }
      else {
        // This PoC only receives `ScheduleEvent` messages.
        unreachable!()
      }
    }
    else {
      // This PoC only receives `ScheduleEvent` messages.
      unreachable!()
    }
  }

  fn set_handle(&mut self, handle: ActorHandle) {
    self.actor_handle = handle;
  }

  fn get_handle(&self) -> ActorHandle {
    self.actor_handle
  }
}
