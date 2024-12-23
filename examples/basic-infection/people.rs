/*!

An `Actor` that tracks the status of the population. It's only job is:
 - report the status of a person
 - change the status of a person
 - report the count of people of each status

*/

use serde::{Deserialize, Serialize};

use actor_model::actor::{Actor, ActorHandle};

use crate::{
    message::{
        Channel,
        Envelope,
        Message,
        RcEnvelope,
        Topic
    }
};

pub type PersonID = u32;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum InfectionStatus {
    Susceptible,
    Infected,
    Recovered,
}


pub struct Population {
    // A real implementation wouldn't keep track of each individual. It would only need the counts.
    people: Vec<InfectionStatus>,
    handle: ActorHandle,

    susceptible: u32,
    infected   : u32,
    recovered  : u32,
}

impl Population {
    #[inline(always)]
    pub fn new(person_count: u32) -> Population {
        let people: Vec<InfectionStatus> = vec![InfectionStatus::Susceptible; person_count as usize];
        Population {
            people,
            handle     : 0, // set upon registration
            susceptible: person_count,
            infected   : 0,
            recovered  : 0,
        }
    }

    #[inline(always)]
    fn get_person_status(&self, person_id: PersonID) -> RcEnvelope {
        let status = self.people[person_id as usize];

        RcEnvelope::new(
            Envelope {
                from   : self.handle,
                channel: Channel::Topic(Topic::PersonStatus),
                message: Some(
                    Message::PersonStatus(person_id, status)
                ),
                time  : None,
            }
        )


    }

    fn set_person_status(&mut self, person_id: PersonID, infection_status: InfectionStatus) {
        // The previous status is implicit in this model. In other models it may not be.
        // let previous_status: InfectionStatus = *self.people[person_id as usize];
        self.people[person_id as usize] = infection_status;

        match infection_status {
            InfectionStatus::Infected => {
                self.susceptible -= 1;
                self.infected    += 1;
            }
            InfectionStatus::Recovered => {
                self.infected  -= 1;
                self.recovered += 1;
            }
            InfectionStatus::Susceptible => {
                // Should not happen.
            }
        }
    }

    fn get_population_report(&self) -> RcEnvelope {
        Message::make_population_report(
            self.handle,
            self.susceptible,
            self.infected,
            self.recovered,
        )
    }

    fn person_count(&self) -> usize {
        self.people.len()
    }
}

impl Actor<Message, Topic> for Population {
    fn receive_message(&mut self, envelope: RcEnvelope) -> Vec<RcEnvelope> {
        // In general, we have a method that responds to every message type we know how to answer.

        // There are two ways for a person's status to change: directly, and through a
        // scheduled timeline event. The first two match branches are these cases respectively.

        let messages = match *envelope {

            Envelope {
                channel: Channel::Topic(Topic::ChangePersonStatus),
                message: Some(Message::PersonStatus(person_id, infection_status)),
                time,
                ..
            }
            | Envelope {
                channel: Channel::TimelineEvent,
                message: Some(Message::PersonStatus(person_id, infection_status)),
                time,
                ..
            }
             => {
                self.set_person_status(person_id, infection_status);
                // We emit the person's new status after the change, thereby notifying any potential listeners.
                let mut messages = vec![
                    Message::make_person_status(self.handle, person_id, infection_status, time)
                ];
                // Check if simulation is over.
                if self.recovered == self.person_count() as u32 {
                    #[cfg(feature = "print_messages")]
                    println!("All people recovered.");
                    messages.push(
                        Message::make_stop_message(self.handle)
                    );
                }
                messages
            }

            Envelope {
                channel: Channel::Topic(Topic::RequestPersonStatus),
                message: Some(Message::RequestPersonStatus(person_id)),
                ..
            } => {

                // This is a request for the status of a person. Note that the
                // time will not be set, indicating this is not a transition.
                vec![self.get_person_status(person_id)]
            }

            Envelope {
                channel: Channel::Topic(Topic::PopulationReport),
                message: None,
                ..
            } => {
                // This is a request for a population report
                vec![self.get_population_report()]
            }

            Envelope {
                channel: Channel::Topic(Topic::PopulationReport),
                message: Some(_),
                ..
            } => {
                // This actor sent this message, so ignore it.
                vec![]
            }

            _ => {
                // eprintln!("Population actor received malformed message received: {:?}", envelope);
                vec![]
            }

        };

        #[cfg(feature = "print_messages")]
        for message in &messages {
            println!("POPULATION: {:?}", message);
        }

        messages
    }


    fn register(&mut self, handle: ActorHandle) -> (Vec<Channel>, Vec<RcEnvelope>) {
        self.handle = handle;

        let initial_population_report = self.get_population_report();
        #[cfg(feature = "print_messages")]
        println!("ROUTER/TIMELINE: {:?}", initial_population_report);

        let subscriptions = vec![
            Channel::Topic(Topic::ChangePersonStatus),
            Channel::Topic(Topic::RequestPersonStatus),
            Channel::Topic(Topic::PopulationReport),
            Channel::TimelineEvent, // Wraps `ChangePersonStatus`

            // We emit but do not subscribe to the following:
            // Channel::Topic(Topic::PersonStatus),

        ];

        (subscriptions, vec![initial_population_report])
    }
}



#[cfg(test)]
mod test {
    use std::assert_matches::assert_matches;
    use crate::{RcActor, Router};
    use super::*;

    #[test]
    fn test_person_creation() {
        let mut context = Router::new();
        context.add_actor(RcActor::new(Population::new(10)));

        // Let's request the status of person_id 0.
        let response = context.silent_route(Message::make_person_status_request(0, 0));

        // There should be exactly 1 response.
        assert_eq!(1, response.len());

        assert_matches!(
            response[0],
            Envelope{
                message: Some(
                    Message::PersonStatus(0, InfectionStatus::Susceptible)
                    ),
                ..
            }
        );

        // Now set the status of person_id 0 to infected
        let response = context.silent_route(Message::make_person_status_change(0, 0, InfectionStatus::Infected, 1.0.into()));

        // There should be exactly 1 response, a population report.
        assert_eq!(1, response.len());

        assert_matches!(
            response[0],
            Envelope{
                message: Some(
                    Message::PopulationReport{susceptible: 9, infected: 1, recovered: 0}
                    ),
                ..
            }
        );

        // Request the status of person_id 0 again.
        let response = context.silent_route(Message::make_person_status_request(0, 0));

        // There should be exactly 1 response.
        assert_eq!(1, response.len());

        assert_matches!(
            response[0],
            Envelope{
                message: Some(
                    Message::PersonStatus(0, InfectionStatus::Infected)
                    ),
                ..
            }
        );

    }

    #[test]
    fn test_get_population() {
        let mut context = Router::new();
        context.add_actor(RcActor::new(Population::new(10)));

        let response = context.silent_route(Message::make_population_report_request(0));

        // There should be exactly 1 response, a population report.
        assert_eq!(1, response.len());

        assert_matches!(
            response[0],
            Envelope{
                message: Some(
                    Message::PopulationReport{susceptible: 10, infected: 0, recovered: 0}
                    ),
                ..
            }
        );
    }
}
