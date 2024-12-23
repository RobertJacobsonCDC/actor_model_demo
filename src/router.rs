/*!

A `Router` owns the actors and orchestrates message passing.

*/

use std::{
    collections::HashMap,
    cell::RefCell,
    fmt::Debug
};
use std::collections::VecDeque;
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
    /// A FIFO queue of messages ready for immediate processing
    message_queue: VecDeque<RcEnvelope<Message, Topic>>,
    /// An early exit has been triggered
    stop_requested: bool,
    /// Debug session has been triggered.
    debug_requested: bool,
}

impl<Message, Topic> Default for Router<Message, Topic>
    where Message: Clone + Debug,
          Topic  : BoundedTopic
{
    fn default() -> Self {
        Router{
            actors         : vec![],
            timeline       : Timeline::default(),
            subscriptions  : RefCell::new(HashMap::default()),
            message_queue  : VecDeque::new(),
            stop_requested : false,
            debug_requested: false,
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
            if self.stop_requested {
                eprintln!("Stopping early.");
                return;
            }

            // Message queue processed before timeline.
            if let Some(envelope) = self.message_queue.pop_front() {
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
                    channel: Channel::TimelineEvent,
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
    pub fn route(&mut self, envelope: RcEnvelope<Message, Topic>) {
        // Process system messages
        if self.act_on_system_message(envelope.clone()) {
            // The `act_on_system_message()` function returns true if we should stop routing.
            return;
        }

        let mut subscriptions = self.subscriptions.borrow_mut();
        let subscribers = subscriptions.entry(envelope.channel).or_insert_with(Vec::new);

        for handle in subscribers {
            // let subscriber: &RcActor<Message, Topic> = self.actors.get(*handle as usize).unwrap();
            let subscriber: &RcActor<Message, Topic> = &self.actors[*handle as usize];

            let mut receiver = subscriber.borrow_mut();
            let response     = receiver.receive_message(envelope.clone());

            self.message_queue.extend(response);
        }
    }

    /// Factored out from `route()`, returns true if `route()` should return without routing.
    fn act_on_system_message(&mut self, envelope: RcEnvelope<Message, Topic>) -> bool {
        // Check for system- or timeline-specific messages
        match envelope.as_ref() {
            Envelope { channel: Channel::ScheduleEvent, time: Some(time), .. } => {
                // A real implementation would have more elaborate error handling.
                assert!(*time >= self.timeline.now());

                self.timeline.push(
                    Event {
                        time: *time,
                        envelope: envelope.clone(),
                    }
                );
                // We do not return, because other actors might wish to act on timeline messages
                false
            }

            Envelope { channel: Channel::Time, time: None, .. } => {
                // If the time is empty, it's a request for the current time.
                let new_envelope = Envelope {
                    from   : ActorHandle::default(),
                    channel: Channel::Time,
                    message: None,
                    time   : Some(self.timeline.now())
                };
                #[cfg(feature = "print_messages")]
                println!("ROUTER/TIMELINE: {:?}", new_envelope);
                self.message_queue.push_back(RcEnvelope::new(new_envelope));
                false
            }

            Envelope { channel: Channel::Stop, .. } => {
                self.stop_requested = true;
                // ToDo: Should we return without routing anything else?
                true
            }

            Envelope { channel: Channel::Debug, .. } => {
                self.debug_requested = true;
                // ToDo: Should we return without routing anything else?
                true
            }

            _ => {
                // Not a system message
                false
            }
        } // end match
    }

    /// Processes system messages without broadcasting to non system actors. For non system messages,
    /// Routes the envelope, but collects the responses in a vector and returns them instead of
    /// putting them in a queue.
    ///
    /// This is useful for testing / debugging.
    pub fn silent_route(&mut self, envelope: RcEnvelope<Message, Topic>) -> Vec<RcEnvelope<Message, Topic>> {
        // Process system messages
        if self.act_on_system_message(envelope.clone()) {
            // The `act_on_system_message()` function returns true if we should stop routing.
            return vec![];
        }

        let mut subscriptions = self.subscriptions.borrow_mut();
        let subscribers       = subscriptions.entry(envelope.channel).or_insert_with(Vec::new);
        let mut responses     = vec![];

        for handle in subscribers {
            let subscriber: &RcActor<Message, Topic> = self.actors.get(*handle as usize).unwrap();

            let mut receiver = subscriber.borrow_mut();
            let response     = receiver.receive_message(envelope.clone());
            // Instead of adding the responses the message queue, we accumulate and return them.
            responses.extend(response);
        }

        responses
    }

}
