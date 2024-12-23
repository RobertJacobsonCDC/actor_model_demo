/*!

A `TransmissionManager` is responsible for deciding when which people become
infected. Once they are infected, they are the responsibility of the
`InfectionManager`.

With a food-borne illness (i.e., constant force of infection), each _person_
experiences an exponentially distributed time until infected. Here, we
use a per-person force of infection derived from the population-level to
represent a constant risk of infection for individuals in the population.

An attempt at infection has two phases:

 1. At regular intervals we select a person at random and query their status.
 2. When we receive the person's status, if they are susceptible we change
    their status to infected.

An alternative implementation calculates each person's time to infection at
the beginning of the simulation and schedules their infection at that time.

*/

use rand::{prelude::SmallRng, SeedableRng};
use rand_distr::{Distribution, Exp, Uniform};

use actor_model::{
    actor::{Actor, ActorHandle},
    timeline::Time,
};

use crate::{
    message::{
        Channel,
        Envelope,
        Message,
        RcEnvelope,
        Topic
    },
    people::{InfectionStatus, PersonID},
    FOI,
    MAX_TIME,
};

pub struct TransmissionManager {
    handle              : ActorHandle,
    rng                 : SmallRng,
    exp_dist            : Exp<f64>,
    people_count        : u32,
    current_attempt_time: Time,
    selected_person     : Option<PersonID>,
}

impl TransmissionManager {
    pub fn new() -> Self {
        TransmissionManager {
            handle              : 0,
            rng                 : SmallRng::seed_from_u64(42),
            exp_dist            : Exp::new(FOI).unwrap(),
            people_count        : 0,
            current_attempt_time: 0.0.into(),
            selected_person     : None,
        }
    }

    /// Infects a person if they are susceptible and schedules the next infection attempt.
    fn attempt_infection(&mut self, infection_status: InfectionStatus) -> Vec<RcEnvelope> {
        let mut messages = vec![];

        if let Some(person_id) = self.selected_person {
            // Schedule the next attempt if there is time left
            let next_attempt_time =
                self.current_attempt_time + self.exp_dist.sample(&mut self.rng) / (self.people_count as f64);
            if next_attempt_time <= MAX_TIME {
                messages.push(Message::make_schedule_attempt_infection(self.handle, next_attempt_time))
            }

            // If the person is susceptible, change their status to infected.
            if infection_status == InfectionStatus::Susceptible {
                let status_change_message = Message::make_person_status_change(
                    self.handle,
                    person_id,
                    InfectionStatus::Infected,
                    self.current_attempt_time,
                );
                messages.push(status_change_message);
            }

            // Update the time of the attempt for the next attempt
            self.current_attempt_time = next_attempt_time;
            self.selected_person = None;
        }

        messages
    }

    /// Uniformly selects a person from the population to attempt to infect and requests
    /// their infection status.
    fn query_random_person_id(&mut self) -> RcEnvelope {
        let person_id: PersonID = Uniform::new_inclusive(0, self.people_count - 1)
            .unwrap()
            .sample(&mut self.rng);

        self.selected_person = Some(person_id);
        Message::make_person_status_request(self.handle, person_id)
    }
}

impl Actor<Message, Topic> for TransmissionManager {
    fn receive_message(&mut self, envelope: RcEnvelope) -> Vec<RcEnvelope> {
        // In general, we have a method that responds to every message type we know how to answer.

        let messages = match *envelope {
            // Record the size of the population
            Envelope {
                channel: Channel::Topic(Topic::PopulationReport),
                message:
                Some(Message::PopulationReport {
                         susceptible,
                         infected,
                         recovered,
                     }),
                ..
            } => {
                let mut messages = vec![];
                // Here we can use `self.people_count` as a signal that we should schedule
                // the first infection attempt.
                if self.people_count == 0 {
                    self.people_count = susceptible + infected + recovered;
                    // Initiate first infection attempt.
                    messages.push(self.query_random_person_id());
                } else {
                    // In this model the size of the population is constant, but in
                    // other models it may change.
                    self.people_count = susceptible + infected + recovered;
                }
                messages
            }

            Envelope {
                channel: Channel::Topic(Topic::PersonStatus),
                message: Some(Message::PersonStatus(person_id, infection_status)),
                ..
            } => {
                if Some(person_id) == self.selected_person {
                    self.attempt_infection(infection_status)
                } else {
                    vec![]
                }
            }

            Envelope {
                channel: Channel::TimelineEvent,
                message: Some(Message::AttemptInfection),
                ..
            } => {
                // It's time for Typhoid Mary to make the donuts.
                vec![self.query_random_person_id()]
            }

            _ => {
                /* pass */
                vec![]
            }
        };

        #[cfg(feature = "print_messages")]
        for message in &messages {
            println!("TRANSMISSION MANAGER: {:?}", message);
        }

        messages
    }

    fn register(&mut self, handle: ActorHandle) -> (Vec<Channel>, Vec<RcEnvelope>) {
        self.handle = handle;

        let subscriptions = vec![
            Channel::Topic(Topic::PopulationReport),
            Channel::Topic(Topic::PersonStatus),
            Channel::TimelineEvent,
        ];

        // We have no messages to send until we know the population size.
        (subscriptions, vec![])
    }
}


/*
#[cfg(test)]
mod test {
    use ixa::context::Context;

    use super::*;
    use crate::{
        people::{ContextPeopleExt, InfectionStatus},
        SEED,
    };

    #[test]
  fn test_attempt_infection() {
    let mut context = Context::new();
    context.init_random(SEED);
    context.create_person();
    attempt_infection(&mut context);
    let person_status = context.get_person_status(0);
    assert_eq!(person_status, InfectionStatus::I);
    context.execute();
  }
}
*/
