#[derive(Clone, Copy, Debug)]
pub enum PointerKind {
    Tracked, // &iso
    Raw,     // *iso
}
