use super::errors::BankError;
use super::transaction::Transaction;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::{AddAssign, SubAssign, Add};
use std::path::Path;

pub type Name = String;
enum Operation {
    Deposit(i64),
    Withdraw(i64),
}

#[derive(Copy, Clone, Default, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct Balance(i64);

impl Balance {
    pub fn new(init: i64) -> Self {
        Self(init)
    }

    pub fn value(&self) -> i64 {
        self.0
    }

    fn apply_operations(&mut self, ops: &[Operation]) -> Vec<Operation> {
        let failed_ops = Vec::new();
        for op in ops {
            match op {
                Operation::Deposit(val) => {
                    self.0 += val;
                }
                Operation::Withdraw(val) => {
                    self.0 -= val;
                }
            }
        }
        failed_ops
    }
}

impl Display for Balance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AddAssign for Balance {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Balance {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

pub struct Storage {
    accounts: HashMap<Name, Balance>,
}

impl Storage {
    /// Создаёт новый пустой банк
    pub fn new() -> Self {
        Storage {
            accounts: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, name: Name) -> Option<Balance> {
        if self.accounts.contains_key(&name) {
            None
        } else {
            self.accounts.insert(name, Balance::default());
            Some(Balance::default())
        }
    }

    pub fn remove_user(&mut self, name: &Name) -> Option<Balance> {
        self.accounts.remove(name)
    }

    pub fn get_balance(&self, name: &Name) -> Option<Balance> {
        self.accounts.get(name).copied()
    }

    pub fn deposit(&mut self, name: &Name, amount: Balance) -> Result<(), BankError> {
        if let Some(balance) = self.accounts.get_mut(name) {
            balance.0 += amount.0;
            Ok(())
        } else {
            Err(BankError::UserNotFound)
        }
    }

    pub fn withdraw(&mut self, name: &Name, amount: Balance) -> Result<(), BankError> {
        if let Some(balance) = self.accounts.get_mut(name) {
            if balance.0 >= amount.0 {
                balance.0 -= amount.0;
                Ok(())
            } else {
                Err(BankError::FundsLimit)
            }
        } else {
            Err(BankError::UserNotFound)
        }
    }

    pub fn get_all(&self) -> Vec<(Name, Balance)> {
        self.accounts.iter().map(|(n, b)| (n.clone(), *b)).collect()
    }

    /// Загружает данные из CSV-файла или создаёт хранилище с дефолтными пользователями
    pub fn load_data(file: &str) -> Result<Storage, BankError> {
        let mut storage = Storage::new();

        // Проверяем, существует ли файл
        if Path::new(file).exists() {
            // Открываем файл
            let file = File::open(file)?;

            // Оборачиваем файл в BufReader
            // BufReader читает данные блоками и хранит их в буфере,
            // поэтому построчное чтение (lines()) работает быстрее, чем читать по байту
            let reader = BufReader::new(file);

            // Читаем файл построчно
            for line in reader.lines() {
                // Каждая строка — это Result<String>, поэтому делаем if let Ok
                if let Ok(line) = line {
                    // Разделяем строку по запятой: "Name,Balance"
                    let parts: Vec<&str> = line.trim().split(',').collect();

                    if parts.len() == 2 {
                        let name = parts[0].to_string();
                        // Пробуем преобразовать баланс из строки в число
                        let balance = Balance(parts[1].parse()?);

                        // Добавляем пользователя и выставляем баланс
                        storage.add_user(name.clone());
                        let _ = storage.deposit(&name, balance);
                    }
                }
            }
        } else {
            // если файла нет, создаём пользователей с нуля
            for u in ["John", "Alice", "Bob", "Vasya"] {
                storage.add_user(u.to_string());
            }
        }

        Ok(storage)
    }

    /// Сохраняет текущее состояние Storage в CSV-файл
    pub fn save(&self, file: &str) -> Result<(), BankError> {
        let mut data = String::new();

        // Собираем все данные в одну строку формата "Name,Balance"
        for (name, balance) in self.get_all() {
            data.push_str(&format!("{},{}\n", name, balance));
        }

        // Записываем в файл
        // Здесь мы не используем BufWriter, потому что сразу пишем всю строку целиком.
        fs::write(file, data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*; // подключаем всё из родительского модуля

    #[test]
    fn test_new_storage_is_empty() {
        let bank = Storage::new();
        assert_eq!(bank.accounts.len(), 0);
    }

    #[test]
    fn test_add_user() {
        let mut storage = Storage::new();
        assert_eq!(
            storage.add_user("Alice".to_string()),
            Some(Balance::default())
        ); // новый пользователь
        assert_eq!(storage.add_user("Alice".to_string()), None); // уже существует
    }

    #[test]
    fn test_remove_user() {
        let mut storage = Storage::new();
        storage.add_user("Bob".to_string());
        storage.deposit(&"Bob".to_string(), Balance(100)).unwrap();

        assert_eq!(storage.remove_user(&"Bob".to_string()), Some(Balance(100))); // удаляем и получаем баланс
        assert_eq!(storage.remove_user(&"Bob".to_string()), None); // второй раз — не найден
    }

    #[test]
    fn test_deposit_and_withdraw() {
        let mut storage = Storage::new();
        storage.add_user("Charlie".to_string());

        // Пополнение
        assert!(
            storage
                .deposit(&"Charlie".to_string(), Balance(200))
                .is_ok()
        );
        assert_eq!(
            storage.get_balance(&"Charlie".to_string()),
            Some(Balance(200))
        );

        // Успешное снятие
        assert!(
            storage
                .withdraw(&"Charlie".to_string(), Balance(150))
                .is_ok()
        );
        assert_eq!(
            storage.get_balance(&"Charlie".to_string()),
            Some(Balance(50))
        );

        // Ошибка: недостаточно средств
        assert!(
            storage
                .withdraw(&"Charlie".to_string(), Balance(100))
                .is_err()
        );
        assert_eq!(
            storage.get_balance(&"Charlie".to_string()),
            Some(Balance(50))
        );
    }

    #[test]
    fn test_nonexistent_user() {
        let mut storage = Storage::new();

        // Депозит несуществующему пользователю
        assert!(storage.deposit(&"Dana".to_string(), Balance(100)).is_err());

        // Снятие у несуществующего пользователя
        assert!(storage.withdraw(&"Dana".to_string(), Balance(50)).is_err());

        // Баланс у несуществующего пользователя
        assert_eq!(storage.get_balance(&"Dana".to_string()), None);
    }

    use std::io::{BufReader, BufWriter};
    use std::io::{Cursor, Write};

    #[test]
    fn test_load_data_existing_cursor() {
        // Создаём данные в памяти, как будто это CSV-файл
        let data = b"John,100\nAlice,200\nBob,50\n";
        let mut cursor = Cursor::new(&data[..]);

        // Читаем данные из Cursor
        let mut storage = Storage::new();
        let reader = BufReader::new(&mut cursor);
        for line in reader.lines() {
            let line = line.unwrap();
            let parts: Vec<&str> = line.trim().split(',').collect();
            if parts.len() == 2 {
                let name = parts[0].to_string();
                let balance = Balance(parts[1].parse().unwrap_or(0));
                storage.add_user(name.clone());
                storage.deposit(&name, balance).unwrap();
            }
        }

        assert_eq!(storage.get_balance(&"John".to_string()), Some(Balance(100)));
        assert_eq!(
            storage.get_balance(&"Alice".to_string()),
            Some(Balance(200))
        );
        assert_eq!(storage.get_balance(&"Bob".to_string()), Some(Balance(50)));
        assert_eq!(storage.get_balance(&"Vasya".to_string()), None); // нет в данных
    }

    #[test]
    fn test_save_writes_to_cursor_correctly() {
        // Создаём Storage и добавляем пользователей
        let mut storage = Storage::new();
        storage.add_user("John".to_string());
        storage.add_user("Alice".to_string());
        storage.deposit(&"John".to_string(), Balance(150)).unwrap();
        storage.deposit(&"Alice".to_string(), Balance(300)).unwrap();

        // Сохраняем в память через BufWriter
        let buffer = Vec::new();
        let mut cursor = Cursor::new(buffer);
        {
            let mut writer = BufWriter::new(&mut cursor);
            for (name, balance) in storage.get_all() {
                writeln!(writer, "{},{}", name, balance).unwrap();
            }
            writer.flush().unwrap();
        }

        // Читаем обратно из памяти
        cursor.set_position(0);
        let mut lines: Vec<String> = BufReader::new(cursor).lines().map(|l| l.unwrap()).collect();
        lines.sort(); // сортируем для сравнения

        assert_eq!(lines, vec!["Alice,300", "John,150"]);
    }

    #[test]
    fn test_apply_operations() {
        let mut balance = Balance::default();
        let ops = vec![
            Operation::Deposit(100),
            Operation::Deposit(200),
            Operation::Withdraw(250),
        ];

        let failed_ops = balance.apply_operations(&ops);
        assert!(failed_ops.is_empty());
        assert_eq!(balance, Balance(50));
    }
}
