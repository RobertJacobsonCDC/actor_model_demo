/*!

The `InfectionManager` is responsible for

*/

use std::rc::Rc;

use rand::prelude::SmallRng;
use rand::SeedableRng;
use rand_distr::{Distribution, Exp};

use actor_model::{
    actor::{Actor, ActorHandle},
    timeline::Time
};

use crate::{
    INFECTION_DURATION,
    message::{RcEnvelope, Envelope, Channel, Message, Topic},
    people::{InfectionStatus, PersonID}
};

pub struct InfectionManager {
    handle  : ActorHandle,
    rng     : SmallRng,
    exp_dist: Exp<f64>,
}

impl InfectionManager {
    pub fn new() -> InfectionManager {
        InfectionManager{
            handle  : 0,
            rng     : SmallRng::seed_from_u64(42),
            exp_dist: Exp::new(1.0 / INFECTION_DURATION).unwrap()
        }
    }

    fn schedule_recovery(&mut self, person_id: PersonID, time: Time) -> RcEnvelope {
        let recovery_time = time + self.exp_dist.sample(&mut self.rng);

        let to_be_scheduled = Message::PersonStatus(person_id, InfectionStatus::Recovered);

        let shedule_request = Envelope {
            from: self.handle,
            channel: Channel::ScheduleEvent,
            message: Some(to_be_scheduled),
            time: Some(recovery_time),
        };
        Rc::new(shedule_request)
    }

}


impl Actor<Message, Topic> for InfectionManager {
    fn receive_message(&mut self, envelope: RcEnvelope) -> Vec<RcEnvelope> {
        // In general, we have a method that responds to every message type we know how to answer.

        // The only messages we care about are when a person changes status to infected.

        let messages =
        match *envelope {
            Envelope {
                channel: Channel::Topic(Topic::PersonStatus),
                message: Some(Message::PersonStatus(person_id, InfectionStatus::Infected)),
                time   : Some(time), // Time is set only when it's a transition.
                ..
            }
            => {
                vec![self.schedule_recovery(person_id, time)]
            }

            _ => {
                // A status change we don't care about
                vec![]
            }
        };

        #[cfg(feature = "print_messages")]
        for message in &messages {
            println!("INFECTION MANAGER: {:?}", message);
        }

        messages
    }

    fn register(&mut self, handle: ActorHandle) -> (Vec<Channel>, Vec<RcEnvelope>) {
        self.handle = handle;

        // We respond to infection status changes. We have no initial messages.
        let subscriptions = vec![
            Channel::Topic(Topic::PersonStatus),
        ];
        (subscriptions, vec![])
    }
}

/*
#[cfg(test)]
mod test {
    use crate::people::ContextPeopleExt;
    use crate::people::InfectionStatus;
    use crate::people::InfectionStatusEvent;
    use ixa::context::Context;
    use ixa::define_data_plugin;
    use ixa::random::ContextRandomExt;

    define_data_plugin!(RecoveryPlugin, usize, 0);

    fn handle_recovery_event(context: &mut Context, event: InfectionStatusEvent) {
        if matches!(event.updated_status, InfectionStatus::R) {
            *context.get_data_container_mut(RecoveryPlugin) += 1;
        }
    }

    #[test]
    fn test_handle_infection_change() {
        use super::init;
        let mut context = Context::new();
        context.init_random(42);
        init(&mut context);

        context.subscribe_to_event::<InfectionStatusEvent>(move |context, event| {
            handle_recovery_event(context, event);
        });

        let population_size = 10;
        for id in 0..population_size {
            context.create_person();
            context.set_person_status(id, InfectionStatus::I);
        }

        context.execute();
        let recovered_size: usize = *context.get_data_container(RecoveryPlugin).unwrap();

        assert_eq!(recovered_size, population_size);
    }
}
*/
