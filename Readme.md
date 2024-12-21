# Actor Model

Actors can hold local state and send messages to other actors. There is no global state. Sending/receiving messages 
can happen synchronously, asynchronously, across threads, etc.

An industrial strength implementation would use a library like [actix](https://actix.rs/docs/actix/actor). In this 
example we just have a simple synchronous single threaded version to illustrate. 
