# Actor Model Demo

This is a basic illustrative reproduction of enough of Ixa's functionality to implement the `basic-infection` example using the actor model. The code is in some ways more complicated than it needs to be: in practice one would use a library instead of implementing actors and message passing from scratch, but I wanted to show how it works. Likewise, because this implementation is not very sophisticated, the `basic-infection` implementation is more complicated than it would be in practice.

# Two and a Half Interacting Requirements

## Actor Model

Actors can hold local state and send messages to other actors. There is no global state. Sending/receiving messages 
can happen synchronously, asynchronously, across threads, etc.

A "real" implementation would use a library like [actix](https://actix.rs/docs/actix/actor). In this 
example we just have a simple synchronous single threaded version to illustrate. 

We use a star topology in which all communication is routed through a central manager called a `Router`, as it routes the messages. Other topologies are possible. For example, actors could communicate directly with each other, signalling the timeline actor when the next timeline event should fire. This is probably the better solution but is more complicated.

## Data Model

An "entity", like a person, has various "components", or pieces of data associated to them. So a person might have:
 
 - age
 - weight
 - immune competence
 - immune resilience
 - infection status
 - comorbidities
 - sex
 - immediate contacts / cohabitants

Different parts of the model may need to query this data and act on the results. We want to do this in a typesafe 
yet ergonomic way.

## Concurrency / Parallelism

Computation and I/O may be able to happen concurrently / in parallel, potentially with dramatic performance benefits.

## Non Issues

 - Typesafe I/O: This is a solved problem. 
 - Types need not be dynamic, as they are always known at compile time. 

## Candidate frameworks

 - Bevy ECS: Has built-in support for ergonomic data model (entity-component system) _and_ Actor model (events), 
   built-in concurrency / parallelism features; very mature and battle tested.
 - Actix: Powerful framework for Actor model, built-in concurrency / parallelism features. Orthogonal to data model; 
   very mature and battle tested; some support for distributed systems.

Existing solutions get us many developer years of very high quality work that we are unlikely to match. What's more, 
they are maintained by a third party. We just can't get the same capabilities with an in-house solution.

## Challenges with current code

Types that implement the `Any` trait must have static lifetimes. Thus, any implementing type is unable to hold a 
nonstatic reference. This is likely to be a problem. 

