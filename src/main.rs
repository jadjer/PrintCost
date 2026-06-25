use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write, Read};
use std::path::Path;
use serde::{Serialize, Deserialize};

const CONFIG_FILE: &str = "print_config.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Config {
    price_per_hour: f64,
    materials: HashMap<String, f64>,
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    if !Path::new(CONFIG_FILE).exists() {
        return Err("File does not exist".into());
    }
    let mut file = File::open(CONFIG_FILE)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = serde_json::from_str(&contents)?;
    Ok(config)
}

fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(CONFIG_FILE)?;
    let json = serde_json::to_string_pretty(config)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn read_string(prompt: &str) -> String {
    loop {
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Не удалось прочитать строку");
        let formatted = input.trim().to_lowercase();

        if !formatted.is_empty() {
            return formatted;
        } else {
            println!("Имя пластика не может быть пустым. Попробуйте еще раз.");
        }
    }
}

fn read_f64(prompt: &str) -> f64 {
    loop {
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Не удалось прочитать строку");

        match input.trim().parse::<f64>() {
            Ok(num) if num >= 0.0 => return num,
            _ => println!("Ошибка ввода. Пожалуйста, введите положительное число."),
        }
    }
}

fn read_time(prompt: &str) -> f64 {
    loop {
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Не удалось прочитать строку");

        // Разделяем строку по пробелам
        let parts: Vec<&str> = input.trim().split_whitespace().collect();

        match parts.as_slice() {
            // Если ввели два числа: Часы и Минуты
            [h_str, m_str] => {
                let h = h_str.parse::<u32>();
                let m = m_str.parse::<u32>();

                if let (Ok(hours), Ok(minutes)) = (h, m) {
                    if minutes < 60 {
                        // Переводим минуты в доли часа
                        return (hours as f64) + (minutes as f64 / 60.0);
                    }
                }
                println!("Ошибка. Минуты должны быть от 0 до 59. Пример ввода: 2 45");
            }
            // Если ввели только одно число (только часы)
            [h_str] => {
                if let Ok(hours) = h_str.parse::<f64>() {
                    if hours >= 0.0 {
                        return hours;
                    }
                }
                println!("Ошибка ввода. Введите положительное число.");
            }
            _ => {
                println!("Неверный формат. Введите либо одно число (часы), либо два через пробел (часы минуты).");
            }
        }
    }
}

fn main() {
    println!("=== Калькулятор стоимости 3D-печати ===");

    // 1. Загружаем существующий конфиг или создаем пустой
    let mut config = load_config().unwrap_or_else(|_| {
        println!("\n[Конфигурация не найдена. Создаем новый профиль]");
        let price_per_hour = read_f64("Введите стоимость одного часа печати: ");
        let cfg = Config {
            price_per_hour,
            materials: HashMap::new(),
        };
        save_config(&cfg).unwrap_or_else(|e| println!("Ошибка сохранения: {}", e));
        cfg
    });

    println!("\nСтоимость часа печати: {:.2}", config.price_per_hour);

    // 2. Ввод названия пластика
    println!("\n--- Выбор материала ---");
    let material_type = read_string("Введите название пластика (например: pla, tpu): ");

    let price_per_kg = match config.materials.get(&material_type) {
        Some(&price) => {
            println!("Используется сохраненная цена для {}: {:.2}/кг", material_type, price);
            price
        }
        None => {
            println!("Материал '{}' еще не сохранен в базе.", material_type);
            let price = read_f64(&format!("Введите цену за 1 кг для {}: ", material_type));

            config.materials.insert(material_type.clone(), price);
            if let Err(e) = save_config(&config) {
                println!("Предупреждение: не удалось обновить файл настроек: {}", e);
            } else {
                println!("Материал {} успешно сохранен в настройки.", material_type);
            }
            price
        }
    };

    // 3. Ввод параметров детали
    println!("\n--- Расчет для конкретной модели ---");
    let weight_grams = read_f64("Введите вес затраченного пластика (в граммах): ");

    // ИСПОЛЬЗУЕМ НОВУЮ ФУНКЦИЮ ДЛЯ ВВОДА ВРЕМЕНИ
    let hours = read_time("Введите фактическое время печати (Часы Минуты, например '2 45' или '0 30'): ");

    let copies = read_f64("Введите количество копий: ");
    let margin_factor = read_f64("Введите коэффициент стоимости (например, 1.5): ");

    // 4. Расчет стоимости
    let weight_kg = weight_grams / 1000.0;
    let material_cost_per_one = weight_kg * price_per_kg;
    let time_cost_per_one = hours * config.price_per_hour;

    let total_cost_per_one = material_cost_per_one + time_cost_per_one;
    let total_batch_cost = total_cost_per_one * copies * margin_factor;

    // 5. Вывод результатов
    println!("\n=== Результаты расчета ({}) ===", material_type.to_uppercase());
    println!("Расчетное время:                 {:.2} ч.", hours);
    println!("Себестоимость 1 копии (пластик): {:.2}", material_cost_per_one);
    println!("Себестоимость 1 копии (время):   {:.2}", time_cost_per_one);
    println!("Итоговая цена за {} шт. (с коэф. {:.2}): {:.2}", copies, margin_factor, total_batch_cost);
}