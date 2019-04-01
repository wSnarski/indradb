use crate::client_datastore::ClientDatastore;
use indradb::util::generate_temporary_path;
use indradb::{Datastore, Transaction};
use crate::server;
use std::panic::catch_unwind;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread::spawn;

const START_PORT: u16 = 27616;

lazy_static! {
    static ref CURRENT_PORT: AtomicUsize = AtomicUsize::new(START_PORT as usize);
}

full_test_impl!({
    let port = (*CURRENT_PORT).fetch_add(1, Ordering::SeqCst);
    spawn(move || server::start(&format!("127.0.0.1:{}", port), "memory://", 1));
    ClientDatastore::new(port as u16)
});

#[test]
fn should_panic_on_bad_connection_string() {
    let result = catch_unwind(|| server::start("127.0.0.1:9999", "foo://", 1));
    assert!(result.is_err());
}
