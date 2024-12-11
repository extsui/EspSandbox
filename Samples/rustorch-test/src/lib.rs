pub const NUMBER_SEGMENT_TABLE: [u8; 10] = [
    0xFC,   // 0
    0x60,   // 1
    0xDA,   // 2
    0xF2,   // 3
    0x66,   // 4
    0xB6,   // 5
    0xBE,   // 6
    0xE4,   // 7
    0xFE,   // 8
    0xF6,   // 9
];

// LedDriver 用
pub fn parse(format: &String) -> Option<[u8; 4]> {
    let mut result: [u8; 4] = [ 0, 0, 0, 0 ];
    let mut index = 0 as usize;

    let mut ch_prev: Option<char> = None;
    for ch in format.chars() {
        match ch {
            '0'..='9' => {
                let number = (ch as u8 - b'0') as usize;
                result[index] = NUMBER_SEGMENT_TABLE[number];
                index += 1;
            },
            ' ' => {
                index += 1;
            },
            '.' => {
                // 先頭の '.' と連続の '.' は NG
                if (ch_prev == None) ||
                   (ch_prev.unwrap() == '.') {
                    return None
                } else {
                    result[index - 1] |= 0x01;
                }
            },
            _ => {
                return None
            },
        }
        ch_prev = Some(ch);
    }
    Some(result)
}
