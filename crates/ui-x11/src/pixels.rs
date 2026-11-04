//! cairo の寸法（`f64`）を物理画素（`u32`）にする。

/// `value` を四捨五入して `u32` にする。負と NaN は 0、`u32` を超えれば `u32::MAX`。
///
/// 🔴 `as` は使えず（RS-007）、`f64` から整数への `TryFrom` は標準に無い。`u32` の範囲は `f64` で正確に表せるので、
/// 2 の冪を上から引いて組み立てる。字の寸法を測るときにだけ使う（毎フレームではない）。
pub(crate) fn to_pixels(value: f64) -> u32 {
    let rounded = value.round();
    if rounded.is_nan() || rounded <= 0.0_f64 {
        return 0;
    }
    let mut remaining = rounded.min(f64::from(u32::MAX));
    let mut result = 0_u32;
    for bit in (0..32_u32).rev() {
        let weight = f64::from(1_u32 << bit);
        if remaining >= weight {
            remaining -= weight;
            result |= 1_u32 << bit;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_and_saturates() {
        assert_eq!(to_pixels(0.0), 0);
        assert_eq!(to_pixels(7.49), 7);
        assert_eq!(to_pixels(7.5), 8);
        assert_eq!(to_pixels(16.0), 16);
        assert_eq!(to_pixels(-3.0), 0);
        assert_eq!(to_pixels(f64::NAN), 0);
        assert_eq!(to_pixels(1.0e12), u32::MAX);
        assert_eq!(to_pixels(f64::from(u32::MAX)), u32::MAX);
        assert_eq!(to_pixels(123_456_789.4), 123_456_789);
    }
}
