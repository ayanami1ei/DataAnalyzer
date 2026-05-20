// 质量参数解析的单元测试
// 测试 parse_param 对上下界、定性描述等格式的解析能力，以及 recalc_isok 对合格判定的验证

use crate::sink::quality_inspection::quality_param::*;

#[test]
// 验证：解析 "0.200≤V≤0.550" 格式的双边界参数
fn parse_should_extract_both_bounds() {
    let pt = parse_param("0.200≤V≤0.550").expect("should parse");
    match pt {
        ParamType::BothBounds { lower, upper } => {
            assert!((lower - 0.200).abs() < 1e-9);
            assert!((upper - 0.550).abs() < 1e-9);
        }
        _ => panic!("expected BothBounds"),
    }
}

#[test]
// 验证：解析含单位的双边界参数（mm 后缀被忽略）
fn parse_should_extract_both_bounds_with_unit() {
    let pt = parse_param("1000≤V≤1010mm").expect("should parse");
    match pt {
        ParamType::BothBounds { lower, upper } => {
            assert!((lower - 1000.0).abs() < 1e-9);
            assert!((upper - 1010.0).abs() < 1e-9);
        }
        _ => panic!("expected BothBounds"),
    }
}

#[test]
// 验证：解析 "V≥2.5" 格式的下界参数
fn parse_should_extract_lower_bound() {
    let pt = parse_param("V≥2.5").expect("should parse");
    match pt {
        ParamType::LowerBound { lower } => {
            assert!((lower - 2.5).abs() < 1e-9);
        }
        _ => panic!("expected LowerBound"),
    }
}

#[test]
// 验证：解析 "V≤0.500" 格式的上界参数
fn parse_should_extract_upper_bound() {
    let pt = parse_param("V≤0.500").expect("should parse");
    match pt {
        ParamType::UpperBound { upper } => {
            assert!((upper - 0.500).abs() < 1e-9);
        }
        _ => panic!("expected UpperBound"),
    }
}

#[test]
// 验证：解析 "V=外护套应能与缆芯分离，并有明显阻力。" 格式的定性参数
fn parse_should_extract_qualitative() {
    let raw = "V=外护套应能与缆芯分离，并有明显阻力。";
    let pt = parse_param(raw).expect("should parse");
    match pt {
        ParamType::Qualitative { description } => {
            assert!(description.contains("外护套"));
        }
        _ => panic!("expected Qualitative"),
    }
}

#[test]
// 验证：空字符串或空白字符串返回解析错误
fn parse_should_fail_on_empty() {
    assert!(parse_param("").is_err());
    assert!(parse_param("   ").is_err());
}

#[test]
// 验证：双边界模式下，值在下界与上界之间时 recalc_isok 返回 true
fn recalc_isok_both_bounds_within() {
    let pt = ParamType::BothBounds {
        lower: 0.200,
        upper: 0.550,
    };
    assert_eq!(recalc_isok(&pt, "0.420").unwrap(), true);
}

#[test]
// 验证：双边界模式下，值低于下界时 recalc_isok 返回 false
fn recalc_isok_both_bounds_below() {
    let pt = ParamType::BothBounds {
        lower: 0.200,
        upper: 0.550,
    };
    assert_eq!(recalc_isok(&pt, "0.100").unwrap(), false);
}

#[test]
// 验证：双边界模式下，值高于上界时 recalc_isok 返回 false
fn recalc_isok_both_bounds_above() {
    let pt = ParamType::BothBounds {
        lower: 0.200,
        upper: 0.550,
    };
    assert_eq!(recalc_isok(&pt, "0.600").unwrap(), false);
}

#[test]
// 验证：下界模式下，值高于下界时 recalc_isok 返回 true
fn recalc_isok_lower_bound_above() {
    let pt = ParamType::LowerBound { lower: 2.5 };
    assert_eq!(recalc_isok(&pt, "3.0").unwrap(), true);
}

#[test]
// 验证：下界模式下，值低于下界时 recalc_isok 返回 false
fn recalc_isok_lower_bound_below() {
    let pt = ParamType::LowerBound { lower: 2.5 };
    assert_eq!(recalc_isok(&pt, "2.0").unwrap(), false);
}

#[test]
// 验证：上界模式下，值低于上界时 recalc_isok 返回 true
fn recalc_isok_upper_bound_below() {
    let pt = ParamType::UpperBound { upper: 0.500 };
    assert_eq!(recalc_isok(&pt, "0.300").unwrap(), true);
}

#[test]
// 验证：上界模式下，值高于上界时 recalc_isok 返回 false
fn recalc_isok_upper_bound_above() {
    let pt = ParamType::UpperBound { upper: 0.500 };
    assert_eq!(recalc_isok(&pt, "0.700").unwrap(), false);
}

#[test]
// 验证：定性参数模式下，值为 "OK" 时 recalc_isok 返回 true
fn recalc_isok_qualitative_ok() {
    let pt = ParamType::Qualitative {
        description: "外护套应能与缆芯分离".to_string(),
    };
    assert_eq!(recalc_isok(&pt, "OK").unwrap(), true);
}

#[test]
// 验证：定性参数模式下，值为 "NG" 时 recalc_isok 返回 false
fn recalc_isok_qualitative_not_ok() {
    let pt = ParamType::Qualitative {
        description: "外护套应能与缆芯分离".to_string(),
    };
    assert_eq!(recalc_isok(&pt, "NG").unwrap(), false);
}

#[test]
// 验证：空值或 "None" 字符串导致 recalc_isok 返回错误
fn recalc_isok_empty_value_should_fail() {
    let pt = ParamType::BothBounds {
        lower: 0.0,
        upper: 1.0,
    };
    assert!(recalc_isok(&pt, "").is_err());
    assert!(recalc_isok(&pt, "None").is_err());
}

#[test]
// 验证：定量参数类型（BothBounds / LowerBound / UpperBound）的 is_quantitative 返回 true
fn param_type_is_quantitative() {
    assert!(ParamType::BothBounds {
        lower: 0.0,
        upper: 1.0
    }
    .is_quantitative());
    assert!(ParamType::LowerBound { lower: 0.0 }.is_quantitative());
    assert!(ParamType::UpperBound { upper: 1.0 }.is_quantitative());
    assert!(!ParamType::Qualitative {
        description: "test".to_string()
    }
    .is_quantitative());
}

#[test]
// 验证：定性参数的 is_qualitative 返回 true，定量参数返回 false
fn param_type_is_qualitative() {
    assert!(ParamType::Qualitative {
        description: "test".to_string()
    }
    .is_qualitative());
    assert!(!ParamType::BothBounds {
        lower: 0.0,
        upper: 1.0
    }
    .is_qualitative());
}
