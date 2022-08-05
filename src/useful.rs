pub fn hex(byte: u8) -> [char;2] {
    let l = byte / 16;
    let r = byte % 16;
    [hex_one_char(l).expect("l is less than 16"), hex_one_char(r).expect("r is less than 16")]
}

pub fn hex_one_char(byte: u8) -> Option<char> {
    match byte {
        0 => Some('0'),
        1 => Some('1'),
        2 => Some('2'),
        3 => Some('3'),
        4 => Some('4'),
        5 => Some('5'),
        6 => Some('6'),
        7 => Some('7'),
        8 => Some('8'),
        9 => Some('9'),
        10 => Some('A'),
        11 => Some('B'),
        12 => Some('C'),
        13 => Some('D'),
        14 => Some('E'),
        15 => Some('F'),
        _ => None,
    }
}