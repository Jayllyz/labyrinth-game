use std::fmt::Write;

pub trait ToBinary {
    fn to_binary(&self) -> Result<String, std::fmt::Error>;
}

impl ToBinary for &str {
    fn to_binary(&self) -> Result<String, std::fmt::Error> {
        self.chars().try_fold(String::with_capacity(self.len() * 8), |mut acc, c| {
            write!(acc, "{:08b}", c as u8)?;
            Ok(acc)
        })
    }
}

impl ToBinary for &String {
    fn to_binary(&self) -> Result<String, std::fmt::Error> {
        self.chars().try_fold(String::with_capacity(self.len() * 8), |mut acc, c| {
            write!(acc, "{:08b}", c as u8)?;
            Ok(acc)
        })
    }
}

impl ToBinary for &[i32] {
    fn to_binary(&self) -> Result<String, std::fmt::Error> {
        self.iter().try_fold(String::with_capacity(self.len() * 8), |mut acc, &d| {
            write!(acc, "{:08b}", d as u8)?;
            Ok(acc)
        })
    }
}

impl<const N: usize> ToBinary for &[i32; N] {
    fn to_binary(&self) -> Result<String, std::fmt::Error> {
        self.iter().try_fold(String::with_capacity(N * 8), |mut acc, &d| {
            write!(acc, "{:08b}", d as u8)?;
            Ok(acc)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_to_binary() {
        let input = "ABC";
        let result = input.to_binary().unwrap();
        assert_eq!(result, "010000010100001001000011");
    }

    #[test]
    fn test_string_to_binary() {
        let input = String::from("123");
        let result = (&input).to_binary().unwrap();
        assert_eq!(result, "001100010011001000110011");
    }

    #[test]
    fn test_empty_str_to_binary() {
        let input = "";
        let result = input.to_binary().unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_special_chars_to_binary() {
        let input = "!@#";
        let result = input.to_binary().unwrap();
        assert_eq!(result, "001000010100000000100011");
    }

    #[test]
    fn test_i32_slice_to_binary() {
        let input = [65, 66, 67];
        let result = (&input[..]).to_binary().unwrap();
        assert_eq!(result, "010000010100001001000011");
    }

    #[test]
    fn test_i32_array_to_binary() {
        let input = [49, 50, 51];
        let result = (&input).to_binary().unwrap();
        assert_eq!(result, "001100010011001000110011");
    }
}
