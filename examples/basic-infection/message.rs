/*!

Defines an enum of topics the actors in this model will listen to.

In addition to these topics, each actor may define their own messages
in the form of an implementor of `MessagePayload`.

*/

use actor_model::{
  actor::ActorHandle,
  message::{
    Channel    as GenericChannel,
    Envelope   as GenericEnvelope,
    RcEnvelope as GenericRcEnvelope
  },
  timeline::Time
};

use crate::people::{InfectionStatus, PersonID};

// We "concretize" the generic types for this model.
pub(crate) type Channel    = GenericChannel<Topic>;
pub(crate) type Envelope   = GenericEnvelope<Message  , Topic>;
pub(crate) type RcEnvelope = GenericRcEnvelope<Message, Topic>;


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Topic {
  // Messages related to `Population`
  PersonStatus,       // Send the status of a person
  ChangePersonStatus, // Change the status of a person
  RequestPersonStatus,
  PopulationReport    // Send/Query the population report
}


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Message {
  /// Used to hold status change for timeline events and direct requests for status change
  /// and to respond to queries for a person's current status.
  PersonStatus(PersonID, InfectionStatus),
  RequestPersonStatus(PersonID),
  PopulationReport{
    susceptible: u32,
    infected   : u32,
    recovered  : u32,
  },
  AttemptInfection,
}

impl Message {
  // Convenience methods

  #[inline(always)]
  pub fn make_person_status_change(actor_handle: ActorHandle, person_id: PersonID, infection_status: InfectionStatus, time: Time) -> RcEnvelope {
    RcEnvelope::new(
      Envelope {
        from   : actor_handle,
        channel: Channel::Topic(Topic::ChangePersonStatus),
        message: Some(
          Message::PersonStatus(person_id, infection_status)
        ),
        time   : Some(time),
      }
    )
  }

  #[inline(always)]
  pub fn make_person_status_request(actor_handle: ActorHandle, person_id: PersonID) -> RcEnvelope {
    RcEnvelope::new(
      Envelope {
        from   : actor_handle,
        channel: Channel::Topic(Topic::RequestPersonStatus),
        message: Some(
          Message::RequestPersonStatus(person_id)
        ),
        time   : None,
      }
    )
  }

  #[inline(always)]
  pub fn make_person_status(
    actor_handle: ActorHandle,
    person_id: PersonID,
    infection_status: InfectionStatus,
    time: Option<Time>
  ) -> RcEnvelope {
    RcEnvelope::new(
      Envelope {
        from   : actor_handle,
        channel: Channel::Topic(Topic::PersonStatus),
        message: Some(
          Message::PersonStatus(person_id, infection_status)
        ),
        time,
      }
    )
  }

  #[inline(always)]
  pub fn make_population_report_request(actor_handle: ActorHandle) -> RcEnvelope {
    RcEnvelope::new(
      Envelope {
        from   : actor_handle,
        channel: Channel::Topic(Topic::PopulationReport),
        message: None,
        time   : None,
      }
    )
  }

  #[inline(always)]
  pub fn make_population_report(
    actor_handle: ActorHandle,
    susceptible: u32,
    infected: u32,
    recovered: u32
  ) -> RcEnvelope {
    RcEnvelope::new(
      Envelope {
        from   : actor_handle,
        channel: Channel::Topic(Topic::PopulationReport),
        message: Some(
          Message::PopulationReport {
            susceptible,
            infected,
            recovered,
          }
        ),
        time  : None,
      }
    )
  }

  #[inline(always)]
  pub fn make_schedule_attempt_infection(actor_handle: ActorHandle, time: Time) -> RcEnvelope {
    RcEnvelope::new(
      Envelope {
        from   : actor_handle,
        channel: Channel::ScheduleEvent,
        message: Some(Message::AttemptInfection),
        time   : Some(time),
      }
    )
  }

  #[inline(always)]
  pub fn make_stop_message(actor_handle: ActorHandle) -> RcEnvelope {
    RcEnvelope::new(
      Envelope {
        from   : actor_handle,
        channel: Channel::Stop,
        message: None,
        time   : None,
      }
    )
  }

}
