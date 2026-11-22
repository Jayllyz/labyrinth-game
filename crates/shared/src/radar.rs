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
    ObjectiveAlly = 9,
    ObjectiveEnemy = 10,
    ObjectiveMonster = 11,
    INVALID = 15,
}

impl CellType {
    /// The exit, whoever stands on it.
    pub fn is_exit(&self) -> bool {
        matches!(
            self,
            Self::OBJECTIVE | Self::ObjectiveAlly | Self::ObjectiveEnemy | Self::ObjectiveMonster
        )
    }

    /// Cells worth entering even when walled on three sides.
    pub fn is_goal_or_help(&self) -> bool {
        self.is_exit() || *self == Self::HELP
    }
}

#[derive(Debug)]
pub struct Radar {
    pub horizontal: Vec<Passages>,
    pub vertical: Vec<Passages>,
    pub cells: Vec<CellType>,
}

/// Encodes bytes to Base64 using the custom alphabet, without padding characters.
pub fn encode_base64(bytes: &[u8]) -> String {
    let alphabet = BASE64_CHARS.as_bytes();
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;

    for &byte in bytes {
        buffer = (buffer << 8) | u32::from(byte);
        bits += 8;
        while bits >= 6 {
            bits -= 6;
            encoded.push(char::from(alphabet[((buffer >> bits) & 0x3F) as usize]));
        }
    }
    if bits > 0 {
        encoded.push(char::from(alphabet[((buffer << (6 - bits)) & 0x3F) as usize]));
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
/// The decoded bytes.
///
/// # Examples
/// ```
/// use shared::radar::{encode_base64, decode_base64};
/// let encoded = encode_base64(b"Hello");
/// assert_eq!(decode_base64(&encoded), b"Hello");
///
/// // Single byte values
/// assert_eq!(decode_base64("aa"), [0]);
/// assert_eq!(decode_base64("gq"), [0x19]);
/// ```
///
/// # Bit Processing
/// ```text
/// Input:    |  S     |  G     |  V     |  s     |
/// Base64:   |010010  |010000  |010111  |100011  |
/// Output:   |01001001|00000101|11100011|
/// Chars:    |   I    |   E    |   š    |
/// ```
pub fn decode_base64(input: &str) -> Vec<u8> {
    let mut decoded = Vec::with_capacity(input.len() * 3 / 4);
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
            decoded.push(((buffer >> bits) & 0xFF) as u8);
        }
    }

    decoded
}

/// Reads the 9 cells of the radar, 4 bits each, from the high bits of a 40-bit field.
pub fn retrieve_cell(bits: u64) -> Vec<CellType> {
    (0..9)
        .map(|i| match (bits >> (36 - 4 * i)) & 0xF {
            0 => CellType::NOTHING,
            1 => CellType::ALLY,
            2 => CellType::ENEMY,
            3 => CellType::MONSTER,
            // A hint with someone standing on it is still a hint.
            4..=7 => CellType::HELP,
            8 => CellType::OBJECTIVE,
            9 => CellType::ObjectiveAlly,
            10 => CellType::ObjectiveEnemy,
            11 => CellType::ObjectiveMonster,
            // Skipping a cell would shift every following one, so unknown values stay in place.
            _ => CellType::INVALID,
        })
        .collect()
}

/// Reads 12 passages, 2 bits each, from the high bits of a 24-bit field.
fn passages(bits: u32) -> Vec<Passages> {
    (0..12)
        .filter_map(|i| match (bits >> (22 - 2 * i)) & 0b11 {
            0 => Some(Passages::UNDEFINED),
            1 => Some(Passages::OPEN),
            2 => Some(Passages::WALL),
            _ => None,
        })
        .collect()
}

/// Reads the horizontal and vertical passages from their 24-bit fields.
pub fn retrieve_passage(horizontal: u32, vertical: u32) -> (Vec<Passages>, Vec<Passages>) {
    (passages(horizontal), passages(vertical))
}

/// Extracts the radar from its decoded bytes.
///
/// Layout (11 bytes):
/// - bytes 0..3: horizontal passages, little-endian
/// - bytes 3..6: vertical passages, little-endian
/// - bytes 6..11: cells
///
/// Shorter input gives an empty radar.
pub fn extract_data(bytes: &[u8]) -> Radar {
    let Some(bytes) = bytes.get(..11) else {
        return Radar { horizontal: Vec::new(), vertical: Vec::new(), cells: Vec::new() };
    };
    let little_endian_24 = |b: &[u8]| u32::from_le_bytes([b[0], b[1], b[2], 0]);
    let cells = bytes[6..11].iter().fold(0u64, |acc, &b| (acc << 8) | u64::from(b));

    let (horizontal, vertical) =
        retrieve_passage(little_endian_24(&bytes[0..3]), little_endian_24(&bytes[3..6]));
    Radar { horizontal, vertical, cells: retrieve_cell(cells) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bits_u64(bits: &str) -> u64 {
        u64::from_str_radix(bits, 2).unwrap()
    }

    fn bits_u32(bits: &str) -> u32 {
        u32::from_str_radix(bits, 2).unwrap()
    }

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
        assert_eq!(encode_base64(b"Hello, World!"), "sgvSBg8SifDVCMXKiq");
        let numbers: Vec<u8> = (0..=255).collect();
        assert_eq!(
            encode_base64(&numbers),
            "aaecaWqfbGCicqOlda0odXareHmufryxgbKAgXWDhH8GisiJjcuMjYGPkISSls4VmdeYmZq1nJC4otO7pd0+p0bbqKneruzhseLks0XntK9quvjtvfvwv1HzwLTCxv5FygfIy2rLzMDOAwPRBg1UB3bXCNn0Dxz3EhL6E3X9FN+aGykdHiwgH4IjIOUmJy6pKjgsK5svLPEyMzQBNj2EN6cHOQoKPAANQkMQQ6YTRQ+WSBkZTlw2T7I5URU8VB6/WmhcW8tfXSFiYCRlZm3oZ9dr0Tpu1DBx2nNA29ZD3T/G4ElJ5oxM5+JP6UVS7E7V8phY8/t19VF4+FR7/p3+/W"
        );
    }

    #[test]
    fn test_decode_prof() {
        let test1 = "Hello, World!";
        assert_eq!(decode_base64(&encode_base64(test1.as_bytes())), test1.as_bytes());
        let test2 = "#123";
        assert_eq!(decode_base64(&encode_base64(test2.as_bytes())), test2.as_bytes());
        let test3 = "Hier, je suis rentré chez moi vers 18h, j'ai manger du poulet avec du riz et ainsi que fait mon exercice de Schooding.";
        assert_eq!(decode_base64(&encode_base64(test3.as_bytes())), test3.as_bytes());
        let test4 = "qwekasdjladfljadljk";
        assert_eq!(decode_base64(&encode_base64(test4.as_bytes())), test4.as_bytes());
    }

    #[test]
    fn test_retrieve_cell_keeps_every_cell_in_place() {
        // A hint and an exit with an opponent standing on them (values 6 and 10).
        let cells = retrieve_cell(bits_u64("1111111111110110101011111111000011110000"));

        assert_eq!(cells.len(), 9);
        assert_eq!(cells[3], CellType::HELP);
        assert_eq!(cells[4], CellType::ObjectiveEnemy);
        assert_eq!(cells[7], CellType::NOTHING);
    }

    #[test]
    fn test_retrieve_cell() {
        assert_eq!(
            retrieve_cell(bits_u64("1111111111111111000011111111000000000000")),
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
            retrieve_cell(bits_u64("1111111111110000000011110000000011110000")),
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
            retrieve_cell(bits_u64("1111111111110000000011110000000011110000")),
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
        let (horizontal, vertical) = retrieve_passage(
            bits_u32("000000001010000101100110"),
            bits_u32("000000000010011010011010"),
        );
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

        let (horizontal, vertical) = retrieve_passage(
            bits_u32("000000101000010100011000"),
            bits_u32("000000001001100001101000"),
        );
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
        let radar_view = extract_data(&decode_base64("jivbQjIad/apapa"));
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
}
