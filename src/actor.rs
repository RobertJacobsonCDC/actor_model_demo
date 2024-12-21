/*

An `Actor` is an entity that can emit messages and subscribe to channels.

*/

use std::any::Any;

use crate::{
  message::{Channel, RcEnvelope},
  rccell::RcCell
};

pub type ActorHandle = u32;
pub type RcActor = RcCell<dyn Actor>;

pub trait Actor: Any {
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
  /*
  Implement these as
  fn as_any(&self) -> &dyn Any { self }
  fn as_any_mut(&mut self) -> &mut dyn Any { self }
  */

  fn emit_message(&mut self) -> RcEnvelope;
  fn receive_message(&mut self, envelope: RcEnvelope);

  fn set_handle(&mut self, handle: ActorHandle);
  fn get_handle(&self) -> ActorHandle;

  // Called when the `Router` is adding this actor with the provided `ActorHandle`.
  // The `Actor` has an opportunity to subscribe to channels and send messages.
  fn register(&mut self, handle: ActorHandle) -> (Vec<Channel>, Vec<RcEnvelope>){
    self.set_handle(handle);
    (Vec::new(), Vec::new())
  }
}
