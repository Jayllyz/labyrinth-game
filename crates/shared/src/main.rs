fn main() {
    println!("Hello, shared!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hello() {
        assert_eq!("Hello, shared!", "Hello, shared!");
    }
}
