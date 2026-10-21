use crate::math::left_shift_i32;
use crate::scalar::Scalar;

pub fn float_to_bits(value: Scalar) -> u32 {
    value.to_bits()
}

pub fn bits_to_float(bits: u32) -> Scalar {
    Scalar::from_bits(bits)
}

pub fn sign_bit_to_2s_compliment(mut x: i32) -> i32 {
    if x < 0 {
        x &= 0x7FFF_FFFF;
        x = -x;
    }
    x
}

pub fn twos_compliment_to_sign_bit(mut x: i32) -> i32 {
    let sign = x >> 31;
    x = (x ^ sign).wrapping_sub(sign);
    x |= left_shift_i32(sign, 31);
    x
}

pub fn float_as_2s_compliment(x: Scalar) -> i32 {
    sign_bit_to_2s_compliment(float_to_bits(x) as i32)
}

pub fn twos_compliment_as_float(x: i32) -> Scalar {
    bits_to_float(twos_compliment_to_sign_bit(x) as u32)
}

pub fn scalar_as_2s_compliment(x: Scalar) -> i32 {
    float_as_2s_compliment(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_sign_bit_conversions_round_trip() {
        for value in [
            0.0,
            1.0,
            -1.0,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
            0.5,
            -42.25,
        ] {
            let compliment = float_as_2s_compliment(value);
            let round_trip = twos_compliment_as_float(compliment);
            assert_eq!(float_to_bits(value), float_to_bits(round_trip));
        }
    }

    #[test]
    fn dev_sign_bit_conversions_match_reference_cases() {
        assert_eq!(sign_bit_to_2s_compliment(i32::MIN), 0);
        assert_eq!(twos_compliment_to_sign_bit(0), 0);
        assert_eq!(twos_compliment_to_sign_bit(-1), 0x8000_0001u32 as i32);
    }
}
