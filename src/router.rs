/*!

A `Router` owns the actors and orchestrates message passing.

*/

use std::{
    collections::HashMap,
    cell::RefCell,
    fmt::Debug
};

use crate::{
    actor::{
        ActorHandle,
        RcActor
    },
    message::{
        Channel,
        RcEnvelope,
        Envelope,
        BoundedTopic
    },
    timeline::{
        Timeline,
        Event
    },
};

pub const TIMELINE_HANDLE: ActorHandle = 0;

/// It would be nice to just treat the timeline like any other actor. We could do that if we had a notion of
pub struct Router<Message, Topic>
    where Message: Clone + Debug,
          Topic  : BoundedTopic
{
    /// List of `Actor`s participating in this `Router`. In this implementation, the
    /// `Router` owns the `Actor`s.
    actors       : Vec<RcActor<Message, Topic>>,
    timeline     : Timeline<Message, Topic>,
    /// Map from channels to the actors subscribed to those channels.
    /// If the number of actors is known to be small, say, < 128, then you can
    /// use a bit mask instead of a `Vec<ActorHandle>`. You might also make this
    /// a HashSet or something to prevent double subscriptions.
    subscriptions: RefCell<HashMap<Channel<Topic>, Vec<ActorHandle>>>,
    /// Queue of messages ready for immediate processing
    message_queue: Vec<RcEnvelope<Message, Topic>>
}

impl<Message, Topic> Default for Router<Message, Topic>
    where Message: Clone + Debug,
          Topic  : BoundedTopic
{
    fn default() -> Self {
        Router{
            actors       : vec![],
            timeline     : Timeline::default(),
            subscriptions: RefCell::new(HashMap::default()),
            message_queue: vec![],
        }
    }
}

impl<Message, Topic> Router<Message, Topic>
    where Message: Clone + Debug,
          Topic  : BoundedTopic
{
    pub fn new() -> Self {
      Router::default()
    }

    /// Adds the actor to the router. The `Router` owns the actor, so we take a `BxActor`.
    /// (We could allow actors in multiple routers, but we don't.)
    pub fn add_actor(&mut self, actor: RcActor<Message, Topic>) {
        let actor_handle = self.actors.len() as ActorHandle;
        self.actors.push(actor.clone());

        // Inform the actor of its registration with the router.
        let mut actor_mut = actor.borrow_mut();
        let (new_subscriptions, new_messages) = actor_mut.register(actor_handle);

        // Act on the actor's subscriptions and messages
        let mut subscriptions = self.subscriptions.borrow_mut();
        for channel in new_subscriptions {
            let subscribers = subscriptions.entry(channel).or_insert_with(Vec::new);
            subscribers.push(actor_handle);
        }
        // Enqueue the actor's initial outgoing messages
        self.message_queue.extend(new_messages);
    }

    /// Begins the event loop
    pub fn run(&mut self) {
        loop {
            // Message queue processed before timeline.
            if let Some(envelope) = self.message_queue.pop() {
                self.route(envelope);
                // We continue, because an actor might have placed something in the
                // message queue that needs processing before we process the timeline.
                continue;
            }

            if let Some(event) = self.timeline.pop() {
                let Event{ envelope: event_envelope, time} = event;
                let Envelope{from, ..} = event_envelope.as_ref();

                let envelope = Envelope{
                    from   : *from,
                    to     : Channel::TimelineEvent,
                    message: event_envelope.message.clone(),
                    time   : Some(time)
                };
                self.route(RcEnvelope::new(envelope));
            } else {
                // All messages & events are exhausted.
                break;
            }
        }
    }


    /// Handles a single message in the message queue.
    /// (This method could be public.)
    fn route(&mut self, envelope: RcEnvelope<Message, Topic>) {
        // Check for timeline-specific messages.
        if let Envelope{to: Channel::ScheduleEvent, time: Some(time), ..} = envelope.as_ref(){
            // A real implementation would have more elaborate error handling.
            assert!(*time >= self.timeline.now());

            self.timeline.push(
                Event{
                    time: *time,
                    envelope: envelope.clone(),
                }
            )
            // We do not return, because other actors might wish to act on timeline messages
        }

        let mut subscriptions = self.subscriptions.borrow_mut();
        let subscribers = subscriptions.entry(envelope.to).or_insert_with(Vec::new);

        for handle in subscribers {
            let subscriber: &RcActor<Message, Topic> = self.actors.get(*handle as usize).unwrap();
            let mut receiver = subscriber.borrow_mut();
            receiver.receive_message(envelope.clone());
        }
    }

}
