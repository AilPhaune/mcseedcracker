pub mod durability;
pub mod enchants;

#[inline(always)]
#[cold]
pub fn cold() {}

#[inline(always)]
pub fn likely(b: bool) -> bool {
    if !b {
        cold()
    }
    b
}

#[inline(always)]
pub fn unlikely(b: bool) -> bool {
    if b {
        cold()
    }
    b
}
