//! Port of `src/utils/SkFloatToDecimal.cpp`.
//!
//! The output grammar and boundaries are identical to Skia's implementation;
//! this is used by PDF and debugging paths that must not emit exponent notation.

pub const K_MAXIMUM_SK_FLOAT_TO_DECIMAL_LENGTH: usize = 49;

fn pow_by_squaring(mut value: f64, mut base: f64, mut e: i32) -> f64 {
    debug_assert!(e > 0);
    loop {
        if e & 1 != 0 {
            value *= base;
        }
        e >>= 1;
        if e == 0 {
            return value;
        }
        base *= base;
    }
}

fn pow10(e: i32) -> f64 {
    match e {
        0 => 1.0,
        1 => 10.0,
        2 => 100.0,
        3 => 1e+03,
        4 => 1e+04,
        5 => 1e+05,
        6 => 1e+06,
        7 => 1e+07,
        8 => 1e+08,
        9 => 1e+09,
        10 => 1e+10,
        11 => 1e+11,
        12 => 1e+12,
        13 => 1e+13,
        14 => 1e+14,
        15 => 1e+15,
        _ if e > 15 => pow_by_squaring(1e+15, 10.0, e - 15),
        _ => {
            debug_assert!(e < 0);
            pow_by_squaring(1.0, 0.1, -e)
        }
    }
}

fn frexp_exponent(value: f32) -> i32 {
    debug_assert!(value.is_finite() && value != 0.0);
    let bits = value.abs().to_bits();
    let raw_exp = ((bits >> 23) & 0xff) as i32;
    if raw_exp != 0 {
        raw_exp - 127 + 1
    } else {
        let frac = bits & 0x7f_ffff;
        (31 - frac.leading_zeros() as i32 - 149) + 1
    }
}

/// Convert `value` into a decimal string in `output`, returning the string
/// length (excluding the terminating NUL).  Always writes a terminating NUL.
pub fn sk_float_to_decimal(
    value: f32,
    output: &mut [u8; K_MAXIMUM_SK_FLOAT_TO_DECIMAL_LENGTH],
) -> usize {
    let mut value = value;
    let end = K_MAXIMUM_SK_FLOAT_TO_DECIMAL_LENGTH - 1;
    let mut index = 0usize;

    if value == f32::INFINITY {
        value = f32::MAX;
    }
    if value == f32::NEG_INFINITY {
        value = -f32::MAX;
    }
    if !value.is_finite() || value == 0.0 {
        output[0] = b'0';
        output[1] = 0;
        return 1;
    }
    if value < 0.0 {
        output[index] = b'-';
        index += 1;
        value = -value;
    }

    let binary_exponent = frexp_exponent(value);
    const K_LOG2: f64 = 0.301_029_995_663_981_2;
    let decimal_exponent = (K_LOG2 * binary_exponent as f64).floor() as i32;
    let mut decimal_shift = decimal_exponent - 8;
    let power = pow10(-decimal_shift);
    debug_assert!((value as f64) * power <= i32::MAX as f64);
    let mut d = ((value as f64) * power + 0.5) as i32;
    debug_assert!(d <= 999_999_999);
    if d > 167_772_159 {
        decimal_shift = decimal_exponent - 7;
        d = ((value as f64) * (power * 0.1) + 0.5) as i32;
        debug_assert!(d <= 99_999_999);
    }
    while d % 10 == 0 {
        d /= 10;
        decimal_shift += 1;
    }
    debug_assert!(d > 0);

    let mut buffer = [0u8; 9];
    let mut buffer_index = 0usize;
    loop {
        buffer[buffer_index] = (d % 10) as u8;
        buffer_index += 1;
        d /= 10;
        if d == 0 {
            break;
        }
    }

    if decimal_shift >= 0 {
        while buffer_index > 0 {
            buffer_index -= 1;
            output[index] = b'0' + buffer[buffer_index];
            index += 1;
        }
        for _ in 0..decimal_shift {
            output[index] = b'0';
            index += 1;
        }
    } else {
        let places_before_decimal = buffer_index as i32 + decimal_shift;
        if places_before_decimal > 0 {
            let mut places = places_before_decimal;
            while places > 0 {
                buffer_index -= 1;
                output[index] = b'0' + buffer[buffer_index];
                index += 1;
                places -= 1;
            }
            output[index] = b'.';
            index += 1;
        } else {
            output[index] = b'.';
            index += 1;
            let mut places_after_decimal = -places_before_decimal;
            while places_after_decimal > 0 {
                output[index] = b'0';
                index += 1;
                places_after_decimal -= 1;
            }
        }
        while buffer_index > 0 {
            buffer_index -= 1;
            output[index] = b'0' + buffer[buffer_index];
            index += 1;
            if index == end {
                break;
            }
        }
    }

    debug_assert!(index <= end);
    output[index] = 0;
    index
}

/// Convenience wrapper returning the exact bytes produced by `sk_float_to_decimal`.
pub fn sk_float_to_decimal_string(value: f32) -> String {
    let mut output = [0u8; K_MAXIMUM_SK_FLOAT_TO_DECIMAL_LENGTH];
    let len = sk_float_to_decimal(value, &mut output);
    String::from_utf8(output[..len].to_vec()).expect("ASCII decimal")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_values_and_round_trip() {
        assert_eq!(sk_float_to_decimal_string(f32::NAN), "0");
        assert_eq!(
            sk_float_to_decimal_string(f32::NEG_INFINITY),
            "-340282350000000000000000000000000000000"
        );
        assert_eq!(sk_float_to_decimal_string(1.0), "1");
        assert_eq!(sk_float_to_decimal_string(-0.0), "0");
        for &v in &[1.0f32, -1.0, 0.5, 1.0 / 3.0, f32::MIN_POSITIVE, f32::MAX] {
            let s = sk_float_to_decimal_string(v);
            assert!(s.parse::<f32>().unwrap() == v, "{v} -> {s}");
        }
    }
}
