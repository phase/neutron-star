#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceCapability {
    Iso, // &iso ?/ &own
    Trn, // &trn
    Mut, // &mut / &ref
    Val, // &val ?/ &view/&imm/&frozen/&ice/&const
    Box, // &box / &
    Tag, // &tag ?/ &id/&opaque
}

impl ReferenceCapability {
    /// Can this refcap be aliased as this other refcap?
    fn _can_alias(&self, other: Self) -> bool {
        use ReferenceCapability::*;
        match (*self, other) {
            (Iso, Tag) => true,
            (Iso, _) => false,
            (Trn, Box) => true,
            (Trn, _) => false,
            (x, y)  if x == y => true,
            _ => false,
        }
    }

    /// Can this refcap be sent to another actor?
    fn _sendable(&self) -> bool {
        use ReferenceCapability::*;
        match self {
            Iso | Tag | Val => true,
            _  => false,
        }
    }

    /// Can this refcap be mutated?
    fn _is_mutable(&self) -> bool {
        use ReferenceCapability::*;
        match self {
            Iso | Trn | Mut => true,
            _ => false,
        }
    }

    /// Combine the origin structure's refcap and a field's refcap.
    /// The origin has a "viewpoint" and its fields can be seen from it.
    fn _adapt_viewpoint(&self, field: Self) -> Option<Self> {
        use ReferenceCapability::*;
        match (self, field) {
            // iso origin
            (Iso, Iso) => Some(Iso),
            (Iso, Trn | Mut | Box) => Some(Tag),
            (Iso, x) => Some(x),
            // trn origin
            (Trn, Trn | Mut) => Some(Box),
            (Trn, x) => Some(x),
            // mut origin
            (Mut, x) => Some(x),
            // val origin
            (Val, Tag) => Some(Tag),
            (Val, _) => Some(Val),
            // box origin
            (Box, Iso | Trn | Mut) => Some(Tag),
            (Box, x) => Some(x),
            // tag origin
            (Tag, _) => None,
        }
    }
}

impl ToString for ReferenceCapability {
    fn to_string(&self) -> String {
        match self {
            ReferenceCapability::Iso => "iso".to_string(),
            ReferenceCapability::Trn => "trn".to_string(),
            ReferenceCapability::Mut => "mut".to_string(),
            ReferenceCapability::Val => "val".to_string(),
            ReferenceCapability::Box => "box".to_string(),
            ReferenceCapability::Tag => "tag".to_string(),
        }
    }
}
