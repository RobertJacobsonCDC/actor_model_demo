/*!

A `Router` owns the actors and orchestrates message passing.

*/

use std::{
    rc::Rc,
    collections::HashMap,
    cell::RefCell
};

use crate::{
    rccell::RcCell,
    message::{
        Channel,
        RcEnvelope,
        Envelope,
    },
    actor::{ActorHandle, RcActor},
    timeline::{
        Timeline,
        Event
    },
    rc_cell,
};
use crate::message::Message;

pub struct Router {
    /// List of `Actor`s participating in this `Router`. In this implementation, the
    /// `Router` owns the `Actor`s. The `Router` automatically inserts a `Timeline`
    /// as the first actor.
    actors       : Vec<RcActor>,
    timeline     : RcActor,
    /// Map from channels to the actors subscribed to those channels.
    /// If the number of actors is known to be small, say, < 128, then you can
    /// use a bit mask instead of a `Vec<ActorHandle>`. You might also make this
    /// a HashSet or something to prevent double subscriptions.
    subscriptions: RefCell<HashMap<Channel, Vec<ActorHandle>>>,
    /// Queue of messages ready for immediate processing
    message_queue: Vec<RcEnvelope>
}

impl Default for Router {
    fn default() -> Self {
        let rc_timeline: RcActor = rc_cell!(Timeline::default());
        let mut new_router = Router{
            actors       : vec![],
            timeline     : rc_timeline.clone(),
            subscriptions: RefCell::new(HashMap::default()),
            message_queue: vec![],
        };
        new_router.add_actor(new_router.timeline.clone());
        new_router
    }
}

impl Router {
    pub fn new() -> Self {
      Router::default()
    }

    fn pop_timeline_event(&mut self) -> Option<Event> {
        let mut actor = self.actors[0].borrow_mut();
        let timeline = actor.as_any_mut()
                            .downcast_mut::<Timeline>()
                            .unwrap(); // First element is `Timeline` by construction.
        timeline.pop()
    }

    /// Adds the actor to the router. The `Router` owns the actor, so we take a `BxActor`.
    /// (We could allow actors in multiple routers, but we don't.)
    pub fn add_actor(&mut self, actor: RcActor) {
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

            if let Some(event) = self.pop_timeline_event() {
                let Event{ envelope: event_envelope, time} = event;

                let envelope = Envelope{
                    from   : 0, // From the timeline
                    to     : Channel::TimelineEvent,
                    message: Message::TimelineEvent(event_envelope, time),
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
    fn route(&mut self, envelope: RcEnvelope) {
        let mut subscriptions = self.subscriptions.borrow_mut();
        let subscribers = subscriptions.entry(envelope.to).or_insert_with(Vec::new);

        for handle in subscribers {
            let subscriber: &RcActor = self.actors.get(*handle as usize).unwrap();
            let mut receiver = subscriber.borrow_mut();
            receiver.receive_message(envelope.clone());
        }
    }

}
