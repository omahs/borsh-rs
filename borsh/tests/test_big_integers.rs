#[cfg(any(feature = "bigdecimal", feature = "num-bigint"))]
use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "num-bigint")]
use quickcheck::quickcheck;

#[cfg(feature = "bigdecimal")]
#[test]
fn test_bigdecimal() {
    use bigdecimal_dep::BigDecimal;
    let bigdecimals = [
        BigDecimal::from(0),
        "-0.0".parse().unwrap(),
        "3.14159265358979323846".parse().unwrap(),
        "-0000.000023".parse().unwrap(),
        BigDecimal::from(256),
        BigDecimal::from(666),
        BigDecimal::from(-42),
        "7".repeat(1024).parse().unwrap(),
    ];
    for bigdecimal in &bigdecimals {
        let serialized = bigdecimal.try_to_vec().unwrap();
        let deserialized =
            <BigDecimal>::try_from_slice(&serialized).expect("failed to deserialize BigDecimal");

        assert_eq!(&deserialized, bigdecimal);
    }
}

#[cfg(feature = "bigdecimal")]
#[test]
fn test_bigdecimal_vectors() {
    use bigdecimal_dep::BigDecimal;
    use num_bigint_dep::BigInt;

    fn assert_encoding(integer: i64, exponent: i64, vector: &[u8]) {
        let val = BigDecimal::new(BigInt::from(integer), exponent);
        let serialized = val.try_to_vec().unwrap();
        assert_eq!(&serialized, vector);
    }

    assert_encoding(0, 0, &[1, 0]);
    assert_encoding(-1, 1, &[0, 1, 0, 0, 0, 1, 2]);
    assert_encoding(-1, -1, &[0, 1, 0, 0, 0, 1, 1]);
    assert_encoding(1, -1, &[2, 1, 0, 0, 0, 1, 1]);
    assert_encoding(1, 1, &[2, 1, 0, 0, 0, 1, 2]);
}

#[cfg(feature = "num-bigint")]
#[test]
fn test_qc_bigint() {
    use num_bigint_dep::{BigInt, Sign};

    fn prop(a: Option<bool>, value: Vec<u32>) -> bool {
        let sign = match a {
            Some(true) => Sign::Plus,
            Some(false) => Sign::Minus,
            None => Sign::NoSign,
        };
        let value = BigInt::new(sign, value);
        let serialized = value.try_to_vec().unwrap();
        let deserialized =
            <BigInt>::try_from_slice(&serialized).expect("failed to deserialize BigDecimal");
        deserialized == value
    }

    quickcheck(prop as fn(Option<bool>, Vec<u32>) -> bool);
}

#[cfg(feature = "num-bigint")]
#[test]
fn test_qc_biguint() {
    use num_bigint_dep::BigUint;

    fn prop(value: Vec<u32>) -> bool {
        let value = BigUint::new(value);
        let serialized = value.try_to_vec().unwrap();
        let deserialized =
            <BigUint>::try_from_slice(&serialized).expect("failed to deserialize BigDecimal");
        deserialized == value
    }

    quickcheck(prop as fn(Vec<u32>) -> bool);
}