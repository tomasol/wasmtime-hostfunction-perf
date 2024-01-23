cargo_component_bindings::generate!();

use crate::bindings::component::component::host_functions;
use crate::bindings::exports::runtime::runtime::host_functions::Guest;
struct Component;

impl Guest for Component {
    fn return_ok() {
        host_functions::return_ok()
    }
    fn return_err() {
        host_functions::return_err()
    }
    fn panic() {
        host_functions::panic()
    }
}
