pub fn maze_parser(input: &str) -> Vec<Vec<u8>> {
    if input.is_empty() {
        return Vec::new();
    }

    let lines: Vec<&str> = input.lines().collect();
    let (height, width) = (lines.len(), lines[0].len());

    let mut maze = vec![vec![0u8; width]; height];

    for (y, line) in lines.iter().enumerate() {
        for (x, char) in line.chars().enumerate() {
            maze[y][x] = match char {
                ' ' => 0,
                _ => 1,
            };
        }
    }

    maze
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maze_parser() {
        let input = "###\n# #\n###";
        let expected = vec![vec![1, 1, 1], vec![1, 0, 1], vec![1, 1, 1]];
        assert_eq!(maze_parser(input), expected);

        let input = "#### \n#  #|\n#### ";
        let expected = vec![vec![1, 1, 1, 1, 0], vec![1, 0, 0, 1, 1], vec![1, 1, 1, 1, 0]];
        assert_eq!(maze_parser(input), expected);

        let input = "#  # \n#  # \n#  # ";
        let expected = vec![vec![1, 0, 0, 1, 0], vec![1, 0, 0, 1, 0], vec![1, 0, 0, 1, 0]];
        assert_eq!(maze_parser(input), expected);

        assert_eq!(maze_parser(""), Vec::<Vec<u8>>::new());
    }
}
