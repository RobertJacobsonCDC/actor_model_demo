# Actor Model

Actors can hold local state and send messages to other actors. There is no global state. Sending/receiving messages 
can happen synchronously, asynchronously, across threads, etc.

A "real" implementation would use a library like [actix](https://actix.rs/docs/actix/actor). In this 
example we just have a simple synchronous single threaded version to illustrate. 

We use a star topology in which all communication is routed through a central manager called a `Router`, as it routes the messages. Other topologies are possible. For example, actors could communicate directly with each other, signalling the timeline actor when the next timeline event should fire. This is probably the better solution but is more complicated.

