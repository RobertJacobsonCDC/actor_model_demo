pub mod timeline;
pub mod message;
pub mod actor;
pub mod router;
pub mod rccell;



#[cfg(test)]
mod tests {
  use crate::router::Router;
  use super::*;

  #[test]
  fn it_works() {
    let mut router = Router::<(), ()>::new();
    router.run();

  }
}
