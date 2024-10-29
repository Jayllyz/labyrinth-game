fn main() {
    println!("Hello, server!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hello() {
        assert_eq!("Hello, server!", "Hello, server!");
    }
}
