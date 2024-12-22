#![feature(assert_matches)]
// #![feature(trait_alias)]
/*!

The `basic-infection` example model in the Actor pattern.

*/
mod people;
mod message;

use std::{
    rc::Rc,
    cell::RefCell
};

use ixa2::{
    router::Router as GenericRouter,
    actor::{RcActor as GenericRcActor},
    rc_cell,
    rccell::RcCell
};

use crate::{
    message::{Message, Topic},
    people::Population
};

// Trait aliases haven't landed yet.
// pub(crate) trait Actor  = GenericActor<Message, Topic>;
pub(crate) type RcActor = GenericRcActor<Message, Topic>;
pub(crate) type Router  = GenericRouter<Message, Topic>;



static POPULATION        : u64 = 1000;
static SEED              : u64 = 123;
static MAX_TIME          : f64 = 303.0;
static FOI               : f64 = 0.1;
static INFECTION_DURATION: f64 = 5.0;

/*
fn initialize() -> Result<Router, ()> {
    let mut context = Router::new();

    context.init_random(SEED);

    for _ in 0..POPULATION {
        context.create_person();
    }

    // transmission_manager::init(&mut context);
    // infection_manager::init(&mut context);
    // incidence_report::init(&mut context)?;

    context.add_plan(MAX_TIME, |context| {
        context.shutdown();
    });
    Ok(context)
}
*/

fn main() {
    let mut context = Router::new();
    context.add_actor(rc_cell!(Population::new(10)));

    context.run();
}
