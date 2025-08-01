#[inline]
#[cold]
const fn cold() {}

/// Hints at the compiler that the condition is likely `true`.
#[inline]
#[allow(unused)]
pub const fn likely(b: bool) -> bool {
    if !b {
        cold();
    }

    b
}

/// Hints at the compiler that the condition is likely `false`.
#[inline]
#[allow(unused)]
pub const fn unlikely(b: bool) -> bool {
    if b {
        cold();
    }

    b
}
