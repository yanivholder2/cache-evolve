pub fn get_index(hash: u64, size: usize) -> usize {
    (hash as usize) % size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_index() {
        let hash = 123;
        let size = 10;
        let index = get_index(hash, size);
        // 123 % 10 = 3
        assert_eq!(index, 3);
    }
}