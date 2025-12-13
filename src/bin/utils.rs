use bank_system::storage::{Balance, Name, Storage};
use bank_system::transaction::{Transaction, Deposit, Withdraw, Transfer};
use std::io::{self, BufRead, Write};

fn main() {
    let mut storage = match Storage::load_data("balance.csv") {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Невозможно загрузить данные {e}");
            return;
        }
    };

    println!("=== Bank CLI Utils ===");
    println!("Команды:");
    println!("  add <name> <balance>      - добавить пользователя");
    println!("  remove <name>             - удалить пользователя");
    println!("  deposit <name> <amount>   - пополнить баланс");
    println!("  withdraw <name> <amount>  - снять со счёта");
    println!("  balance <name>            - показать баланс");
    println!("  exit                      - выйти");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush().unwrap(); // показываем приглашение

        let mut input = String::new();
        if stdin.lock().read_line(&mut input).unwrap() == 0 {
            break; // EOF
        }

        let args: Vec<&str> = input.split_whitespace().collect();
        if args.is_empty() {
            continue;
        }

        match args[0] {
            "add" => {
                if args.len() != 3 {
                    println!("Пример: add John 100");
                    continue;
                }
                let name: Name = args[1].to_string();
                let balance = match args[2].parse::<i64>() {
                    Ok(b) => Balance::new(b),
                    Err(_) => {
                        println!("Сумма должна быть числом");
                        continue;
                    }
                };
                if storage.add_user(name.clone()).is_some() {
                    let _ = storage.deposit(&name, balance);
                    println!("Пользователь {} добавлен с балансом {}", name, balance);
                    if let Err(e) = storage.save("balance.csv") {
                        eprintln!("Невозможно сохранить данные: {e}");
                    }
                } else {
                    println!("Пользователь {} уже существует", name);
                }
            }
            "remove" => {
                if args.len() != 2 {
                    println!("Пример: remove John");
                    continue;
                }
                let name = args[1];
                if storage.remove_user(&name.to_string()).is_some() {
                    println!("Пользователь {} удалён", name);
                    if let Err(e) = storage.save("balance.csv") {
                        eprintln!("Невозможно сохрнить данные: {e}");
                    }
                } else {
                    println!("Пользователь {} не найден", name);
                }
            }
            "deposit" => {
                if args.len() != 3 {
                    println!("Пример: deposit John 100");
                    continue;
                }
                let name = args[1].to_string();
                let amount: i64 = match args[2].parse() {
                    Ok(a) => a,
                    Err(_) => {
                        println!("Сумма должна быть числом");
                        continue;
                    }
                };

                let tx = Deposit::new(&name, Balance::new(amount));
                // Применяем транзакцию 
                match tx.apply(&mut storage) {
                    Ok(_) => {
                        println!("Транзакция: депозит {} на {}", name, amount);
                        if let Err(e) = storage.save("balance.csv") {
                            eprintln!("Невозможно сохрнить данные: {e}");
                        }
                    }
                    Err(e) => println!("Ошибка транзакции: {:?}", e),
                }
            }
            "transfer" => {
                if args.len() != 4 {
                    println!("Пример: tx_transfer Alice Bob 50");
                    continue;
                }
                let from = args[1].to_string();
                let to = args[2].to_string();
                let amount: i64 = match args[3].parse() {
                    Ok(a) => a,
                    Err(_) => {
                        println!("Сумма должна быть числом");
                        continue;
                    }
                };

                let tx = Transfer::new(&from, &to, Balance::new(amount));
                // Применяем транзакцию 
                match tx.apply(&mut storage) {
                    Ok(_) => {
                        println!("Транзакция: перевод средств {} от {} на {}", amount, from, to);
                        if let Err(e) = storage.save("balance.csv") {
                            eprintln!("Невозможно сохрнить данные: {e}");
                        }
                    }
                    Err(e) => println!("Ошибка транзакции: {:?}", e),
                }
            }
            "withdraw" => {
                if args.len() != 3 {
                    println!("Пример: withdraw John 100");
                    continue;
                }
                let name = args[1].to_string();
                let amount = match args[2].parse::<i64>() {
                    Ok(a) => Balance::new(a),
                    Err(_) => {
                        println!("Сумма должна быть числом");
                        continue;
                    }
                };
                let tx = Withdraw::new(&name, amount);
                // Применяем транзакцию 
                match tx.apply(&mut storage) {
                    Ok(_) => {
                        println!("Транзакция: депозит {} на {}", name, amount);
                        if let Err(e) = storage.save("balance.csv") {
                            eprintln!("Невозможно сохрнить данные: {e}");
                        }
                    }
                    Err(e) => println!("Ошибка транзакции: {:?}", e),
                }
            }
            "balance" => {
                if args.len() != 2 {
                    println!("Пример: balance John");
                    continue;
                }
                let name = args[1].to_string();
                let amount = if let Some(val) = storage.get_balance(&name) {
                    val
                } else {
                    println!("Пользователь: {} не найден", name);
                    continue;
                };
                println!("Баланс пользователя {} = {}", name, amount);
            }
            "+" => {
                if args.len() != 8 {
                    println!(
                        "Пример: + deposit Alice 100 transfer Alice Bob 30: cur {}",
                        args.len()
                    );
                    continue;
                }

                let name = args[2].to_string();
                let amount: i64 = match args[3].parse() {
                    Ok(a) => a,
                    Err(_) => {
                        println!("Сумма должна быть числом");
                        continue;
                    }
                };

                let deposit = Deposit::new(&name, Balance::new(amount));

                let from = args[5].to_string();
                let to = args[6].to_string();
                let amount: i64 = match args[7].parse() {
                    Ok(a) => a,
                    Err(_) => {
                        println!("Сумма должна быть числом");
                        continue;
                    }
                };

                let transfer = Transfer::new(&from, &to, Balance::new(amount));

                // Здесь мы используем оператор +
                let combined_tx = deposit + transfer;

                match combined_tx.apply(&mut storage) {
                    Ok(_) => println!("Транзакции выполнены!"),
                    Err(e) => println!("Ошибка при выполнении: {:?}", e),
                }

                if let Err(e) = storage.save("balance.csv") {
                    eprintln!("Невозможно сохрнить данные: {e}");
                }
            }
            "exit" => break,
            _ => println!("Неизвестная команда"),
        }
    }

    println!("Выход из CLI, все изменения сохранены.");
}
