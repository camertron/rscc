pub mod parser;
pub mod compiler;
pub mod emitter;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub mod version_info {
    include!(concat!(env!("OUT_DIR"), "/version.rs"));
}
