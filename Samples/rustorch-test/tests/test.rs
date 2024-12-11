use rustorch_test::{parse, NUMBER_SEGMENT_TABLE};

#[test]
fn test_parse_normal() {
    assert_eq!(
        parse(&"0123".to_string()),
        Some([
            NUMBER_SEGMENT_TABLE[0],
            NUMBER_SEGMENT_TABLE[1],
            NUMBER_SEGMENT_TABLE[2],
            NUMBER_SEGMENT_TABLE[3],
        ])
    );
    assert_eq!(
        parse(&" 456".to_string()),
        Some([
            0x00,
            NUMBER_SEGMENT_TABLE[4],
            NUMBER_SEGMENT_TABLE[5],
            NUMBER_SEGMENT_TABLE[6],
        ])
    );
    assert_eq!(
        parse(&"789 ".to_string()),
        Some([
            NUMBER_SEGMENT_TABLE[7],
            NUMBER_SEGMENT_TABLE[8],
            NUMBER_SEGMENT_TABLE[9],
            0x00,
        ])
    );
    assert_eq!(
        parse(&"1".to_string()),
        Some([
            NUMBER_SEGMENT_TABLE[1],
            0x00, 0x00, 0x00,
        ])
    );
    assert_eq!(
        parse(&"123".to_string()),
        Some([
            NUMBER_SEGMENT_TABLE[1],
            NUMBER_SEGMENT_TABLE[2],
            NUMBER_SEGMENT_TABLE[3],
            0x00,
        ])
    );
    assert_eq!(
        parse(&" . . . .".to_string()),
        Some([ 0x01, 0x01, 0x01, 0x01 ])
    );
    assert_eq!(
        parse(&"1.2.3.4.".to_string()),
        Some([
            NUMBER_SEGMENT_TABLE[1] | 0x01,
            NUMBER_SEGMENT_TABLE[2] | 0x01,
            NUMBER_SEGMENT_TABLE[3] | 0x01,
            NUMBER_SEGMENT_TABLE[4] | 0x01,
        ])
    );
    assert_eq!(
        parse(&"99.99".to_string()),
        Some([
            NUMBER_SEGMENT_TABLE[9],
            NUMBER_SEGMENT_TABLE[9] | 0x01,
            NUMBER_SEGMENT_TABLE[9],
            NUMBER_SEGMENT_TABLE[9],
        ])
    );
    assert_eq!(
        parse(&"    ".to_string()),
        Some([ 0x00, 0x00, 0x00, 0x00 ])
    );
}

#[test]
fn test_parse_abnormal() {
    // 非対応文字
    assert_eq!(parse(&"x".to_string()), None);
    // 先頭の '.' は NG
    assert_eq!(parse(&".123".to_string()), None);
    // '.' の連続は NG
    assert_eq!(parse(&"12..34".to_string()), None);
}
