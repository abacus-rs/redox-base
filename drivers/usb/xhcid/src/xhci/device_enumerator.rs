use crate::xhci::port::PortFlags;
use crate::xhci::power_manager::Store;
use crate::xhci::{PortId, Xhci, PortStore};
use common::io::Io;
use crossbeam_channel;
use log::{debug, info, warn};
use std::sync::Arc;
use std::time::Duration;
use syscall::EAGAIN;



use tock_registers::interfaces::Readable;


pub struct DeviceEnumerationRequest {
    pub port_id: PortId,
}

pub struct DeviceEnumerator {
    hci: Arc<Xhci>,
    request_queue: crossbeam_channel::Receiver<DeviceEnumerationRequest>,
}

impl DeviceEnumerator {
    pub fn new(hci: Arc<Xhci>) -> Self {
        let request_queue = hci.device_enumerator_receiver.clone();
        DeviceEnumerator { hci, request_queue }
    }

    pub fn run(&mut self) {
        loop {
            debug!("Start Device Enumerator Loop");
            let request = match self.request_queue.recv() {
                Ok(req) => req,
                Err(err) => {
                    panic!("Failed to received an enumeration request! error: {}", err)
                }
            };

            let port_id = request.port_id;
            let port_array_index = port_id.root_hub_port_index();

            info!("Device Enumerator request for port {}", port_id);

            let ports = self.hci.ports.lock().unwrap();

            let len = ports.len();
        
            if port_array_index >= len {
                    warn!(
                        "Received out of bounds Device Enumeration request for port {}",
                        port_id
                    );
                    continue;
                } 


            ports[port_array_index].map(|port_store|{ 
                let port = if let PortStore::Disabled(port) = port_store {
                    // let start = crate::xhci::start();
                    let reset_result = self.hci.reset_port(port, port_id);
                    // let stop = crate::xhci::stop();
                    // crate::xhci::log_cycle_difference_with_name("reset port", start, stop);
                    std::thread::sleep(Duration::from_millis(16)); //Some controllers need some extra time to make the transition.
                    if let PortStore::Enabled(port) = reset_result {
                        info!("xhcid Port {} was reset", port_id); 
                        // continue
                        port
                    } else {
                        info!("Port {} is not in a valid state after reset", port_id);
                       return (reset_result, ())
                    }
                } else if let PortStore::Enabled(port) = port_store {
                    // continue
                    port
                } else {
                    info!("Received Device Enumeration request for port {} which is not in a valid state", port_id);
                    return (port_store, ())
                };

            let block = {
                let start = crate::xhci::start();
                let res = self.hci.attach_device(port_id, port);
                let stop = crate::xhci::stop();
                crate::xhci::log_cycle_difference_with_name("attach_device", start, stop);
                res
            };

            //let start = crate::xhci::start();
            let result = futures::executor::block_on(block);
            //let stop = crate::xhci::stop();
            //crate::xhci::log_cycle_difference_with_name("attach device", start, stop);
            match result {
                Ok(port) => {
                        info!("Device on port {} was attached", port_id);
                        return (port, ())
                }
                Err(err) => {
                    unimplemented!("Error attaching device on port {}: {}", port_id, err);
                    // if err.errno == EAGAIN {
                    //         info!("Received a device connect notification for an already connected device. Ignoring...")
                    // } else {
                    // //        warn!("processing of device attach request failed! Error: {}", err);
                    // }
                }
            }


            }).unwrap();


        }
    }
}
