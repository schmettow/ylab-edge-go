#![no_std]
#![feature(type_alias_impl_trait)]

pub mod ysns; // Ylab sensors
pub mod yuio; // YLab UI Output
pub mod yuii; // YLab UI Input
pub mod ytfk; // YLab transfer formats & kodices

// use embassy_time::Instant;

/* struct Reading {
    time: Instant,
    reading: [f32]
}*/

/*impl fmt::Display for Reading {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{}", self.0)
    }
}*/
