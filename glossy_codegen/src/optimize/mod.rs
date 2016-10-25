#[cfg(feature = "optimizer")]
mod optimizer;

#[cfg(feature = "optimizer")]
pub use self::optimizer::*;

#[cfg(not(feature = "optimizer"))]
pub type Optimizer = ();

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SourceKind {
    Vertex,
    Fragment,
    Unknown,
}
