use std::f64;

// This exists so I can impl Hash for a struct with an f64 field
#[allow(clippy::unreadable_literal)]
pub const fn defloat(val: f64) -> (u64, i16, i8) {
    let bits: u64 = f64::to_bits(val);
    let sign: i8 = if bits >> 63 == 0 { 1 } else { -1 };
    let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;

    let mantissa = if exponent == 0 {
        (bits & 0xfffffffffffff) << 1
    } else {
        (bits & 0xfffffffffffff) | 0x10000000000000
    };

    exponent -= 1023 + 52;
    (mantissa, exponent, sign)
}
