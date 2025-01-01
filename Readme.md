# Actor Model Demo

This is a basic illustrative reproduction of enough of Ixa's functionality to implement the `basic-infection` example using the actor model. The code is in some ways more complicated than it needs to be: in practice one would use a library instead of implementing actors and message passing from scratch, but I wanted to show how it works. Likewise, because this implementation is not very sophisticated, the `basic-infection` implementation is more complicated than it would be in practice.

Turn off the `print_messages` feature to make it run (nearly) silently. Printing messages contributes most of the running time.

```toml
[features]
default = ["print_messages"]

print_messages = []
```

# Two and a Half Interacting Requirements

The point is to illustrate the first of the two (and a bit) major requirements of a discrete event agent modeling framework. Here are a few disjointed thoughts on these requirements, a kind of brain dump written mostly for my own benefit.

## Actor Model

Actors can hold local state and send messages to other actors. There is no global state. Sending/receiving messages 
can happen synchronously, asynchronously, across threads, etc.

A "real" implementation would use a library like [actix](https://actix.rs/docs/actix/actor). In this 
example we just have a simple synchronous single threaded version to illustrate. 

We use a star topology in which all communication is routed through a central manager called a `Router`, as it routes the messages. Other topologies are possible–indeed, probably preferred. For example, actors could communicate directly with each other, signalling the timeline actor when the next timeline event should fire. This is probably the better solution but is more complicated.

The main advantage of this model is two important forms of decoupling:

1. Decoupling of computation (in response to events or messages) from complex orchestration of behaviors and events. Computation occurs based on local conditions alone involving local data alone.
2. Decoupling of "messages" from the senders and receivers of those messages. The senders and receivers do not need to know anything about each other, even whether each other exist at all.

Actors don't need to know anything about the sender or receiver of a message, nor do they require complex orchestration with other actors

## Entity Component System (ECS) Data Model

ECS is in some sense the opposite of how data works in the actor model: data is globally available. Three "things" exist in this model (stolen from Wikipedia):

 - **Entity:** An entity represents a general-purpose object. In a game engine context, for example, every coarse game object is represented as an entity. Usually, it only consists of a unique id. Implementations typically use a plain integer for this.

 - **Component:** A component characterizes an entity as possessing a particular aspect, and holds the data needed to model that aspect. For example, every game object that can take damage might have a Health component associated with its entity. Implementations typically use structs, classes, or associative arrays.

 - **System:** A system is a process which acts on all entities with the desired components. For example, a physics system may query for entities having mass, velocity and position components, and iterate over the results doing physics calculations on the set of components for each entity.

Different parts of the model may need to query this data and act on the results. We want to do this in a typesafe 
yet ergonomic way.

## Concurrency / Parallelism

Computation and I/O may be able to happen concurrently / in parallel, potentially with dramatic performance benefits.

## Candidate frameworks

 - Bevy ECS: Has built-in support for ergonomic data model (entity-component system) _and_ Actor model (events), 
   built-in concurrency / parallelism features; very mature and battle tested.
 - Actix: Powerful framework for Actor model, built-in concurrency / parallelism features. Orthogonal to data model; 
   very mature and battle tested; some support for distributed systems.

It is not clear to me if, for example, Bevy ECS can't do everything we need. But smart people have tried and couldn't get it to work, so I need to understand our needs better and, if Bevy *can* get the job done, justify that it can in a sufficiently convincing way.

## Challenges with current code

Types that implement the `Any` trait must have static lifetimes. Thus, any implementing type is unable to hold a 
nonstatic reference. This is likely to eventually be a problem. 

Subverting the type system is usually not what one wants to do. Rather, we should leverage the type system to make sure users (and ourselves as library devs) do the right things.

It isn't idiomatic or aesthetic, though this is not really an argument. 

At least some of the reasons for the current design choices actually show up in this demo:

 - The `Channel` enum (in `src/message.rs`) needs to be extensible, which I accomplish using a generic `Topic` implementing the trait bounds encoded in `BoundedTopic`.
 - Likewise with the `Message` generic type in, for example, `pub struct Envelope<Message, Topic>  where Topic: BoundedTopic, Message: Clone + Debug`. I make a `Message` enum with all possible messages. In practice, this must be known at compile time, but may only be known to the compiler rather than to the programmer herself.

<hr>

Begin boilerplate…

**General disclaimer** This repository was created for use by CDC programs to collaborate on public health related projects in support of the [CDC mission](https://www.cdc.gov/about/organization/mission.htm).  GitHub is not hosted by the CDC, but is a third party website used by CDC and its partners to share information and collaborate on software. CDC use of GitHub does not imply an endorsement of any one particular service, product, or enterprise.

## Public Domain Standard Notice
This repository constitutes a work of the United States Government and is not
subject to domestic copyright protection under 17 USC § 105. This repository is in
the public domain within the United States, and copyright and related rights in
the work worldwide are waived through the [CC0 1.0 Universal public domain dedication](https://creativecommons.org/publicdomain/zero/1.0/).
All contributions to this repository will be released under the CC0 dedication. By
submitting a pull request you are agreeing to comply with this waiver of
copyright interest.

## License Standard Notice
The repository utilizes code licensed under the terms of the Apache Software
License and therefore is licensed under ASL v2 or later.

This source code in this repository is free: you can redistribute it and/or modify it under
the terms of the Apache Software License version 2, or (at your option) any
later version.

This source code in this repository is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the Apache Software License for more details.

You should have received a copy of the Apache Software License along with this
program. If not, see http://www.apache.org/licenses/LICENSE-2.0.html

The source code forked from other open source projects will inherit its license.

## Privacy Standard Notice
This repository contains only non-sensitive, publicly available data and
information. All material and community participation is covered by the
[Disclaimer](DISCLAIMER.md)
and [Code of Conduct](code-of-conduct.md).
For more information about CDC's privacy policy, please visit [http://www.cdc.gov/other/privacy.html](https://www.cdc.gov/other/privacy.html).

## Contributing Standard Notice
Anyone is encouraged to contribute to the repository by [forking](https://help.github.com/articles/fork-a-repo)
and submitting a pull request. (If you are new to GitHub, you might start with a
[basic tutorial](https://help.github.com/articles/set-up-git).) By contributing
to this project, you grant a world-wide, royalty-free, perpetual, irrevocable,
non-exclusive, transferable license to all users under the terms of the
[Apache Software License v2](http://www.apache.org/licenses/LICENSE-2.0.html) or
later.

All comments, messages, pull requests, and other submissions received through
CDC including this GitHub page may be subject to applicable federal law, including but not limited to the Federal Records Act, and may be archived. Learn more at [http://www.cdc.gov/other/privacy.html](http://www.cdc.gov/other/privacy.html).

## Records Management Standard Notice
This repository is not a source of government records, but is a copy to increase
collaboration and collaborative potential. All government records will be
published through the [CDC web site](http://www.cdc.gov).

## Additional Standard Notices
Please refer to [CDC's Template Repository](https://github.com/CDCgov/template) for more information about [contributing to this repository](https://github.com/CDCgov/template/blob/main/CONTRIBUTING.md), [code of conduct](https://github.com/CDCgov/template/blob/main/code-of-conduct.md), and other related documentation.
