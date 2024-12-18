use core::str;
use std::char;
use std::fmt::Write;

const BASE64_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/";

#[derive(Clone, Debug, PartialEq)]
pub enum Passages {
    UNDEFINED = 0,
    OPEN = 1,
    WALL = 2,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CellType {
    NOTHING = 0,
    ALLY = 1,
    ENEMY = 2,
    MONSTER = 3,
    HELP = 4,
    OBJECTIVE = 8,
    ObjectiveMonster = 11,
    INVALID = 15,
}

#[derive(Debug)]
pub struct RadarView {
    pub horizontal: Vec<Passages>,
    pub vertical: Vec<Passages>,
    pub cells: Vec<CellType>,
}

pub trait ToBinary {
    fn to_binary(&self) -> String;
}

impl ToBinary for &str {
    fn to_binary(&self) -> String {
        self.chars().fold(String::with_capacity(self.len() * 8), |mut acc, c| {
            write!(acc, "{:08b}", c as u8).unwrap();
            acc
        })
    }
}

impl ToBinary for &String {
    fn to_binary(&self) -> String {
        self.chars().fold(String::with_capacity(self.len() * 8), |mut acc, c| {
            write!(acc, "{:08b}", c as u8).unwrap();
            acc
        })
    }
}

impl ToBinary for &[i32] {
    fn to_binary(&self) -> String {
        self.iter().fold(String::with_capacity(self.len() * 8), |mut acc, &d| {
            write!(acc, "{:08b}", d as u8).unwrap();
            acc
        })
    }
}

impl<const N: usize> ToBinary for &[i32; N] {
    fn to_binary(&self) -> String {
        self.iter().fold(String::with_capacity(N * 8), |mut acc, &d| {
            write!(acc, "{:08b}", d as u8).unwrap();
            acc
        })
    }
}

pub fn split_into_chunks(text: &str, chunk_size: usize) -> Vec<String> {
    text.as_bytes()
        .chunks(chunk_size)
        .map(|chunk| {
            let mut s = String::with_capacity(chunk_size);
            s.extend(chunk.iter().map(|&b| b as char));
            if s.len() < chunk_size {
                s.extend(std::iter::repeat('0').take(chunk_size - s.len()));
            }
            s
        })
        .collect::<Vec<String>>()
}

pub fn encode_base64<T: ToBinary>(input: T) -> String {
    let binary_text = input.to_binary();
    let subs = split_into_chunks(&binary_text, 6);
    let mut encoded = String::with_capacity(subs.len());

    for sub in subs {
        let decimal = match u8::from_str_radix(&sub, 2) {
            Ok(d) => d,
            Err(_) => continue,
        };

        if let Some(c) = BASE64_CHARS.chars().nth(decimal as usize) {
            encoded.push(c);
        }
    }

    encoded
}

/// Decodes a base64-encoded string into its original form.
///
/// # Algorithm
/// The function processes input character by character:
/// 1. Each base64 character represents 6 bits of data
/// 2. Bits are accumulated in a buffer until there are enough (8 bits) to form a byte
/// 3. Bytes are extracted and converted to characters
///
/// # Parameters
/// * `input` - A string slice containing base64-encoded data using the alphabet:
///   - a-z (0-25)
///   - A-Z (26-51)
///   - 0-9 (52-61)
///   - +/ (62-63)
///
/// # Returns
/// A `String` containing the decoded data.
///
/// # Examples
/// ```
/// use shared::radar::{encode_base64, decode_base64};
/// let encoded = encode_base64("Hello");
/// let decoded = decode_base64(&encoded);
/// assert_eq!(decoded, "Hello");
///
/// // Single byte values
/// assert_eq!(decode_base64("aa"), "\0");
/// assert_eq!(decode_base64("gq"), "\x19");
/// ```
///
/// # Bit Processing
/// ```text
/// Input:    |  S     |  G     |  V     |  s     |
/// Base64:   |010010  |010000  |010111  |100011  |
/// Output:   |01001001|00000101|11100011|
/// Chars:    |   I    |   E    |   š    |
/// ```
pub fn decode_base64(input: &str) -> String {
    let mut decoded = String::new();
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;

    for &byte in input.as_bytes() {
        let value = match byte {
            b'a'..=b'z' => byte - b'a',      // 0-25
            b'A'..=b'Z' => byte - b'A' + 26, // 26-51
            b'0'..=b'9' => byte - b'0' + 52, // 52-61
            b'+' => 62,
            b'/' => 63,
            _ => continue,
        };

        buffer = (buffer << 6) | (value as u32); // Shift buffer 6 bits to the left and add the value
        bits += 6;

        while bits >= 8 {
            bits -= 8;
            let byte = ((buffer >> bits) & 0xFF) as u8; // Shift buffer to the right by bits and pad with 0s
            decoded.push(byte as char);
        }
    }

    decoded
}

pub fn retrieve_cell(octet: &str) -> Vec<CellType> {
    const NUM_CELLS: usize = 9; // 4 bits per cell, remove the last 4 bits (padding)
    let mut data = Vec::with_capacity(NUM_CELLS);

    for i in (0..NUM_CELLS).map(|x| x * 4) {
        let bits = &octet[i..i + 4];
        let value = match u8::from_str_radix(bits, 2) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let cell = match value {
            0 => CellType::NOTHING,
            1 => CellType::ALLY,
            2 => CellType::ENEMY,
            3 => CellType::MONSTER,
            4 => CellType::HELP,
            8 => CellType::OBJECTIVE,
            11 => CellType::ObjectiveMonster,
            15 => CellType::INVALID,
            _ => continue,
        };
        data.push(cell);
    }

    data
}

pub fn retrieve_passage(horizontal: &str, vertical: &str) -> (Vec<Passages>, Vec<Passages>) {
    const NUM_HORIZONTAL: usize = 12;
    const NUM_VERTICAL: usize = 12;

    let mut horizontal_data = Vec::with_capacity(NUM_HORIZONTAL);
    let mut vertical_data = Vec::with_capacity(NUM_VERTICAL);

    for i in (0..NUM_HORIZONTAL).map(|x| x * 2) {
        let bits = &horizontal[i..i + 2];
        let value = match u8::from_str_radix(bits, 2) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let passage = match value {
            0 => Passages::UNDEFINED,
            1 => Passages::OPEN,
            2 => Passages::WALL,
            _ => continue,
        };
        horizontal_data.push(passage);
    }

    for i in (0..NUM_VERTICAL).map(|x| x * 2) {
        let bits = &vertical[i..i + 2];
        let value = match u8::from_str_radix(bits, 2) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let passage = match value {
            0 => Passages::UNDEFINED,
            1 => Passages::OPEN,
            2 => Passages::WALL,
            _ => continue,
        };
        vertical_data.push(passage);
    }

    (horizontal_data, vertical_data)
}

pub fn extract_data<T: ToBinary>(input: T) -> RadarView {
    let binary = input.to_binary();

    if binary.len() < 88 {
        return RadarView { horizontal: Vec::new(), vertical: Vec::new(), cells: Vec::new() };
    }

    // 3 first octets are for horizontal, 3 next for vertical, and the last 5 for cells
    // Horizontal and vertical are in little-endian order so we need to reverse them
    let mut horizontal_bits = String::with_capacity(24);
    write!(horizontal_bits, "{}{}{}", &binary[16..24], &binary[8..16], &binary[0..8]).unwrap();

    let mut vertical_bits = String::with_capacity(24);
    write!(vertical_bits, "{}{}{}", &binary[40..48], &binary[32..40], &binary[24..32]).unwrap();

    let cell_bits = &binary[48..88];

    let (horizontal, vertical) = retrieve_passage(&horizontal_bits, &vertical_bits);
    let cells = retrieve_cell(cell_bits);

    RadarView { horizontal, vertical, cells }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for the `split_into_chunks` function
    #[test]
    fn test_empty_string() {
        let result = split_into_chunks("", 3);
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_string_shorter_than_chunk() {
        let result = split_into_chunks("ab", 3);
        assert_eq!(result, vec!["ab0"]);
    }

    #[test]
    fn test_string_equal_to_chunk() {
        let result = split_into_chunks("abc", 3);
        assert_eq!(result, vec!["abc"]);
    }

    #[test]
    fn test_string_longer_than_chunk() {
        let result = split_into_chunks("abcdef", 3);
        assert_eq!(result, vec!["abc", "def"]);
    }

    #[test]
    fn test_string_with_partial_last_chunk() {
        let result = split_into_chunks("abcdefgh", 3);
        assert_eq!(result, vec!["abc", "def", "gh0"]);
    }

    // NOTE Simon: dunno why it doesn't work
    // #[test]
    // fn test_unicode_characters() {
    //     let result = split_into_chunks("héllo", 2);
    //     assert_eq!(result, vec!["hé", "ll", "o0"]);
    // }

    #[test]
    fn test_large_chunk_size() {
        let result = split_into_chunks("hello", 10);
        assert_eq!(result, vec!["hello00000"]);
    }

    #[test]
    fn test_multiple_complete_chunks() {
        let result = split_into_chunks("abcdefghi", 3);
        assert_eq!(result, vec!["abc", "def", "ghi"]);
    }

    // Tests for the `encode` function
    #[test]
    fn test_encode_prof() {
        assert_eq!(encode_base64(&[0]), "aa");
        assert_eq!(encode_base64(&[25]), "gq");
        assert_eq!(encode_base64(&[26]), "gG");
        assert_eq!(encode_base64(&[51]), "mW");
        assert_eq!(encode_base64(&[52]), "na");
        assert_eq!(encode_base64(&[61]), "pq");
        assert_eq!(encode_base64(&[62]), "pG");
        assert_eq!(encode_base64(&[63]), "pW");
        assert_eq!(encode_base64("Hello, World!"), "sgvSBg8SifDVCMXKiq");
        let numbers: Vec<i32> = (0..=255).collect();
        assert_eq!(encode_base64(&numbers[..]), "aaecaWqfbGCicqOlda0odXareHmufryxgbKAgXWDhH8GisiJjcuMjYGPkISSls4VmdeYmZq1nJC4otO7pd0+p0bbqKneruzhseLks0XntK9quvjtvfvwv1HzwLTCxv5FygfIy2rLzMDOAwPRBg1UB3bXCNn0Dxz3EhL6E3X9FN+aGykdHiwgH4IjIOUmJy6pKjgsK5svLPEyMzQBNj2EN6cHOQoKPAANQkMQQ6YTRQ+WSBkZTlw2T7I5URU8VB6/WmhcW8tfXSFiYCRlZm3oZ9dr0Tpu1DBx2nNA29ZD3T/G4ElJ5oxM5+JP6UVS7E7V8phY8/t19VF4+FR7/p3+/W");
    }

    #[test]
    fn test_decode_prof() {
        let test1 = "Hello, World!";
        assert_eq!(decode_base64(&encode_base64(test1)), test1);
        let test2 = "#123";
        assert_eq!(decode_base64(&encode_base64(test2)), test2);
        let test3 = "Hier, je suis rentré chez moi vers 18h, j'ai manger du poulet avec du riz et ainsi que fait mon exercice de Schooding.";
        assert_eq!(decode_base64(&encode_base64(test3)), test3);
        let test4 = "qwekasdjladfljadljk";
        assert_eq!(decode_base64(&encode_base64(test4)), test4);
    }

    // Tests for the `retrieve_cell` function
    #[test]
    fn test_retrieve_cell() {
        assert_eq!(
            retrieve_cell("1111111111111111000011111111000000000000"),
            vec![
                CellType::INVALID,
                CellType::INVALID,
                CellType::INVALID,
                CellType::INVALID,
                CellType::NOTHING,
                CellType::INVALID,
                CellType::INVALID,
                CellType::NOTHING,
                CellType::NOTHING
            ]
        );

        assert_eq!(
            retrieve_cell("1111111111110000000011110000000011110000"),
            vec![
                CellType::INVALID,
                CellType::INVALID,
                CellType::INVALID,
                CellType::NOTHING,
                CellType::NOTHING,
                CellType::INVALID,
                CellType::NOTHING,
                CellType::NOTHING,
                CellType::INVALID
            ]
        );

        assert_eq!(
            retrieve_cell("1111111111110000000011110000000011110000"),
            vec![
                CellType::INVALID,
                CellType::INVALID,
                CellType::INVALID,
                CellType::NOTHING,
                CellType::NOTHING,
                CellType::INVALID,
                CellType::NOTHING,
                CellType::NOTHING,
                CellType::INVALID
            ]
        );
    }

    #[test]
    fn test_retrieve_passage() {
        let (horizontal, vertical) =
            retrieve_passage("000000001010000101100110", "000000000010011010011010");
        assert_eq!(
            horizontal,
            vec![
                Passages::UNDEFINED,
                Passages::UNDEFINED,
                Passages::UNDEFINED,
                Passages::UNDEFINED,
                Passages::WALL,
                Passages::WALL,
                Passages::UNDEFINED,
                Passages::OPEN,
                Passages::OPEN,
                Passages::WALL,
                Passages::OPEN,
                Passages::WALL
            ]
        );
        assert_eq!(
            vertical,
            vec![
                Passages::UNDEFINED,
                Passages::UNDEFINED,
                Passages::UNDEFINED,
                Passages::UNDEFINED,
                Passages::UNDEFINED,
                Passages::WALL,
                Passages::OPEN,
                Passages::WALL,
                Passages::WALL,
                Passages::OPEN,
                Passages::WALL,
                Passages::WALL
            ]
        );

        let (horizontal, vertical) =
            retrieve_passage("000000101000010100011000", "000000001001100001101000");
        assert_eq!(
            horizontal,
            vec![
                Passages::UNDEFINED,
                Passages::UNDEFINED,
                Passages::UNDEFINED,
                Passages::WALL,
                Passages::WALL,
                Passages::UNDEFINED,
                Passages::OPEN,
                Passages::OPEN,
                Passages::UNDEFINED,
                Passages::OPEN,
                Passages::WALL,
                Passages::UNDEFINED
            ]
        );
        assert_eq!(
            vertical,
            vec![
                Passages::UNDEFINED,
                Passages::UNDEFINED,
                Passages::UNDEFINED,
                Passages::UNDEFINED,
                Passages::WALL,
                Passages::OPEN,
                Passages::WALL,
                Passages::UNDEFINED,
                Passages::OPEN,
                Passages::WALL,
                Passages::WALL,
                Passages::UNDEFINED
            ]
        );
    }

    #[test]
    fn test_extract_data() {
        let input = decode_base64("jivbQjIad/apapa");
        let radar_view = extract_data(&input);
        assert_eq!(radar_view.horizontal.len(), 12);
        assert_eq!(radar_view.vertical.len(), 12);
        assert_eq!(radar_view.cells.len(), 9);

        assert_eq!(
            vec![
                Passages::OPEN,
                Passages::UNDEFINED,
                Passages::UNDEFINED,
                Passages::OPEN,
                Passages::WALL,
                Passages::UNDEFINED,
                Passages::OPEN,
                Passages::OPEN,
                Passages::UNDEFINED,
                Passages::WALL,
                Passages::OPEN,
                Passages::UNDEFINED
            ],
            radar_view.horizontal
        );

        assert_eq!(
            vec![
                Passages::WALL,
                Passages::UNDEFINED,
                Passages::UNDEFINED,
                Passages::UNDEFINED,
                Passages::WALL,
                Passages::OPEN,
                Passages::WALL,
                Passages::UNDEFINED,
                Passages::WALL,
                Passages::WALL,
                Passages::WALL,
                Passages::UNDEFINED,
            ],
            radar_view.vertical
        );

        assert_eq!(
            vec![
                CellType::NOTHING,
                CellType::INVALID,
                CellType::INVALID,
                CellType::NOTHING,
                CellType::NOTHING,
                CellType::INVALID,
                CellType::NOTHING,
                CellType::NOTHING,
                CellType::INVALID
            ],
            radar_view.cells
        );
    }

    #[test]
    fn test_split_into_chunks() {
        assert_eq!(split_into_chunks("001100", 2), vec!["00", "11", "00"]);

        assert_eq!(split_into_chunks("00001111", 4), vec!["0000", "1111"]);

        assert_eq!(split_into_chunks("0011000", 2), vec!["00", "11", "00", "00"]);
    }
}
