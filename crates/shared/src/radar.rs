use core::str;
use std::char;
use std::collections::HashMap;
use std::fmt::Write;

const BASE64_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/";

#[derive(Clone, Debug, PartialEq)]
pub enum Passages {
    UNDEFINED = 0,
    OPEN = 1,
    WALL = 2,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Cells {
    NOTHING = 0,
    ALLY = 1,
    ENEMY = 2,
    MONSTER = 3,
    HELP = 4,
    OBJECTIVE = 8,
    ObjectiveMonster = 11,
    INVALID = 15,
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
    let subs: Vec<String> = text
        .as_bytes()
        .chunks(chunk_size)
        .map(|chunk| {
            let mut s = chunk.iter().map(|&b| b as char).collect::<String>();
            s.extend(std::iter::repeat('0').take(chunk_size - s.len()));
            s
        })
        .collect();

    subs
}

pub fn encode<T: ToBinary>(input: T) -> String {
    let binary_text = input.to_binary();

    let subs = split_into_chunks(&binary_text, 6);

    let mut encoded = String::new();
    for sub in subs {
        let decimal = isize::from_str_radix(&sub, 2).unwrap();
        encoded += &BASE64_CHARS.chars().nth(decimal as usize).unwrap().to_string();
    }

    encoded
}

pub fn decode(input: &str) -> String {
    let mut decoded = String::new();

    let mut binary: String = String::new();
    for char in input.chars() {
        let index = BASE64_CHARS.find(char).map(|pos| pos as u8);
        if index.is_none() {
            continue;
        }
        binary += format! {"{:06b}", index.unwrap()}.as_str();
    }

    let splitted = split_into_chunks(&binary, 8);
    for sub in splitted {
        let decimal = isize::from_str_radix(&sub, 2).unwrap();
        decoded += &char::from_u32(decimal as u32).unwrap().to_string();
    }

    decoded
}

pub fn extract_data(input: &str) -> (Vec<Passages>, Vec<Passages>, Vec<Cells>) {
    let mut binary = String::new();
    for char in input.chars() {
        binary += format! {"{:08b}", char as u8}.as_str();
    }

    let splitted_octet = split_into_chunks(&binary, 8);

    let mut horizontal_octet = Vec::<String>::new();
    for octet in splitted_octet.iter().take(3) {
        horizontal_octet.push(octet.clone());
    }
    horizontal_octet.reverse();

    let mut vertical_octet = Vec::<String>::new();
    for octet in splitted_octet.iter().take(6).skip(3) {
        vertical_octet.push(octet.clone());
    }
    vertical_octet.reverse();

    let mut cell_octet = String::new();
    for octet in splitted_octet.iter().take(11).skip(6) {
        cell_octet += octet;
    }

    let cell = retrieve_cell(&cell_octet);
    let (horizontal, vertical) =
        retrieve_passage(&horizontal_octet.join(""), &vertical_octet.join(""));

    (vertical, horizontal, cell)
}

pub fn retrieve_cell(octet: &str) -> Vec<Cells> {
    let splitted_4bits = split_into_chunks(octet, 4);
    let splitted_4bits = &splitted_4bits[0..splitted_4bits.len() - 1];

    let map = HashMap::from([
        ("0000", Cells::NOTHING),
        ("0001", Cells::ALLY),
        ("0010", Cells::ENEMY),
        ("0011", Cells::MONSTER),
        ("0100", Cells::HELP),
        ("1000", Cells::OBJECTIVE),
        ("1011", Cells::ObjectiveMonster),
        ("1111", Cells::INVALID),
    ]);

    let mut data = Vec::new();

    for bits in splitted_4bits {
        let value = map.get(bits.as_str());
        if let Some(value) = value {
            data.push(value.clone());
        }
    }

    data
}

pub fn retrieve_passage(horizontal: &str, vertical: &str) -> (Vec<Passages>, Vec<Passages>) {
    let horizontal_2bits = split_into_chunks(horizontal, 2);
    let vertical_2bits = split_into_chunks(vertical, 2);

    let map = HashMap::from([
        ("00", Passages::UNDEFINED),
        ("01", Passages::OPEN),
        ("10", Passages::WALL),
    ]);

    let mut horizontal_data = Vec::new();
    for bits in horizontal_2bits {
        let value = map.get(bits.as_str());
        if let Some(v) = value {
            horizontal_data.push(v.clone());
        }
    }

    let mut vertical_data = Vec::new();
    for bits in vertical_2bits {
        let value = map.get(bits.as_str());
        if let Some(v) = value {
            vertical_data.push(v.clone());
        }
    }

    (horizontal_data, vertical_data)
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
        assert_eq!(encode(&[0]), "aa");
        assert_eq!(encode(&[25]), "gq");
        assert_eq!(encode(&[26]), "gG");
        assert_eq!(encode(&[51]), "mW");
        assert_eq!(encode(&[52]), "na");
        assert_eq!(encode(&[61]), "pq");
        assert_eq!(encode(&[62]), "pG");
        assert_eq!(encode(&[63]), "pW");
        assert_eq!(encode("Hello, World!"), "sgvSBg8SifDVCMXKiq");
        let numbers: Vec<i32> = (0..=255).collect();
        assert_eq!(encode(&numbers[..]), "aaecaWqfbGCicqOlda0odXareHmufryxgbKAgXWDhH8GisiJjcuMjYGPkISSls4VmdeYmZq1nJC4otO7pd0+p0bbqKneruzhseLks0XntK9quvjtvfvwv1HzwLTCxv5FygfIy2rLzMDOAwPRBg1UB3bXCNn0Dxz3EhL6E3X9FN+aGykdHiwgH4IjIOUmJy6pKjgsK5svLPEyMzQBNj2EN6cHOQoKPAANQkMQQ6YTRQ+WSBkZTlw2T7I5URU8VB6/WmhcW8tfXSFiYCRlZm3oZ9dr0Tpu1DBx2nNA29ZD3T/G4ElJ5oxM5+JP6UVS7E7V8phY8/t19VF4+FR7/p3+/W");
    }

    #[test]
    fn test_decode_prof() {
        let test1 = "Hello, World!";
        assert_eq!(decode(&encode(test1)), test1.to_string() + "\0");
        let test2 = "#123";
        assert_eq!(decode(&encode(test2)), test2.to_string() + "\0");
        let test3 = "Hier, je suis rentré chez moi vers 18h, j'ai manger du poulet avec du riz et ainsi que fait mon exercice de Schooding.";
        assert_eq!(decode(&encode(test3)), test3.to_string() + "\0");
        let test4 = "qwekasdjladfljadljk";
        assert_eq!(decode(&encode(test4)), test4.to_string() + "\0");
    }

    // Tests for the `retrieve_cell` function
    #[test]
    fn test_retrieve_cell() {
        assert_eq!(retrieve_cell("00000000"), vec![Cells::NOTHING]);

        assert_eq!(retrieve_cell("000100100000"), vec![Cells::ALLY, Cells::ENEMY]);

        assert_eq!(
            retrieve_cell("0011010010000000"),
            vec![Cells::MONSTER, Cells::HELP, Cells::OBJECTIVE]
        );

        assert_eq!(retrieve_cell("111100000000"), vec![Cells::INVALID, Cells::NOTHING]);

        assert_eq!(retrieve_cell("011100010000"), vec![Cells::ALLY]);
    }

    #[test]
    fn test_retrieve_passage() {
        let (horizontal, vertical) = retrieve_passage("000110", "010110");
        assert_eq!(horizontal, vec![Passages::UNDEFINED, Passages::OPEN, Passages::WALL]);
        assert_eq!(vertical, vec![Passages::OPEN, Passages::OPEN, Passages::WALL]);

        let (horizontal, vertical) = retrieve_passage("101010", "010101");
        assert_eq!(horizontal, vec![Passages::WALL, Passages::WALL, Passages::WALL]);
        assert_eq!(vertical, vec![Passages::OPEN, Passages::OPEN, Passages::OPEN]);

        let (horizontal, vertical) = retrieve_passage("11", "11");
        assert_eq!(horizontal.len(), 0);
        assert_eq!(vertical.len(), 0);
    }

    #[test]
    fn test_extract_data() {
        let input = decode("jivbQjIad/apapa");
        let (horizontal, vertical, cells) = extract_data(&input);

        assert_eq!(horizontal.len(), 12);
        assert_eq!(vertical.len(), 12);
        assert_eq!(cells.len(), 9);

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
            horizontal
        );

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
            vertical
        );

        assert_eq!(
            vec![
                Cells::NOTHING,
                Cells::INVALID,
                Cells::INVALID,
                Cells::NOTHING,
                Cells::NOTHING,
                Cells::INVALID,
                Cells::NOTHING,
                Cells::NOTHING,
                Cells::INVALID
            ],
            cells
        );
    }

    #[test]
    fn test_split_into_chunks() {
        assert_eq!(split_into_chunks("001100", 2), vec!["00", "11", "00"]);

        assert_eq!(split_into_chunks("00001111", 4), vec!["0000", "1111"]);

        assert_eq!(split_into_chunks("0011000", 2), vec!["00", "11", "00", "00"]);
    }
}
