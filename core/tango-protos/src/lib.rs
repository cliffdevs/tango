pub mod signaling {
    include!(concat!(env!("OUT_DIR"), "/tango.signaling.rs"));
}

pub mod ipc {
    include!(concat!(env!("OUT_DIR"), "/tango.ipc.rs"));
}

pub mod iceconfig {
    include!(concat!(env!("OUT_DIR"), "/tango.iceconfig.rs"));
}
