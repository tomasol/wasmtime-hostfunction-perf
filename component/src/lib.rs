cargo_component_bindings::generate!();

use crate::bindings::exports::runtime::runtime::host_functions::Guest;
use crate::bindings::runtime::runtime::host_functions;
struct Component;

impl Guest for Component {
    fn return_ok() {
        host_functions::return_ok()
    }
    fn return_err() {
        host_functions::return_err()
    }
    fn panic(host: bool) {
        if host {
            host_functions::panic(host)
        } else {
            panic!("guest")
        }
    }
}
