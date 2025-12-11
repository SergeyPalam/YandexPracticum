use super::storage::{Balance, Name, Storage};

pub fn compute_most_profitable(storage: &Storage) -> Option<(Name, Balance)> {
    storage
        .get_all()
        .into_iter()
        .max_by(|(_, lhs), (_, rhs)| lhs.value().cmp(&rhs.value()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_most_profitable() {
        let mut storage = Storage::new();
        storage.add_user("Alice".to_owned());
        storage
            .deposit(&"Alice".to_owned(), Balance::new(100))
            .unwrap();
        storage.add_user("Bob".to_owned());
        storage
            .deposit(&"Bob".to_owned(), Balance::new(200))
            .unwrap();

        let res = compute_most_profitable(&storage).unwrap();
        assert_eq!(res.0, "Bob");
        assert_eq!(res.1, Balance::new(200));
    }
}
