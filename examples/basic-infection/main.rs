#![feature(assert_matches)]
// #![feature(trait_alias)]
/*!

The `basic-infection` example model in the Actor pattern.

*/
mod people;
mod message;
mod infection_manager;
mod transmission_manager;
mod incidence_reporter;

use std::{
    rc::Rc,
    cell::RefCell
};
use std::convert::Into;
use ordered_float::OrderedFloat;
use actor_model::{
    router::Router as GenericRouter,
    actor::{RcActor as GenericRcActor},
    rc_cell,
    rccell::RcCell
};
use actor_model::timeline::Time;
use crate::{
    message::{Message, Topic},
    people::Population
};
use crate::incidence_reporter::IncidenceReporter;
use crate::infection_manager::InfectionManager;
use crate::transmission_manager::TransmissionManager;

// Trait aliases haven't landed yet.
// pub(crate) trait Actor  = GenericActor<Message, Topic>;
pub(crate) type RcActor = GenericRcActor<Message, Topic>;
pub(crate) type Router  = GenericRouter<Message, Topic>;



static POPULATION        : u32 = 1000;
static SEED              : u32 = 123;
static MAX_TIME          : Time = OrderedFloat(303.0);
static FOI               : f64 = 0.1;
static INFECTION_DURATION: f64 = 5.0;


fn main() {
    let mut context = Router::new();
    context.add_actor(rc_cell!(InfectionManager::new()));
    context.add_actor(rc_cell!(TransmissionManager::new()));
    context.add_actor(rc_cell!(IncidenceReporter::new("./examples/basic-infection/incidence_report.csv")));
    context.add_actor(rc_cell!(Population::new(POPULATION)));

    context.run();
}
