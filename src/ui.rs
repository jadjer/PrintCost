use crate::{ActiveField, ActiveTab, TuiApp};
use ratatui::widgets::{Clear, List, ListItem, ListState};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, Tabs},
};

pub fn draw_ui(f: &mut Frame, app: &TuiApp) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let titles = vec![" Калькулятор стоимости ", " Настройки базы материалов "];
    let index = match app.active_tab {
        ActiveTab::Calculator => 0,
        ActiveTab::Settings => 1,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Меню (Переключение на Tab) "))
        .select(index)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[0]);

    match app.active_tab {
        ActiveTab::Calculator => draw_calculator_tab(f, chunks[1], app),
        ActiveTab::Settings => draw_settings_tab(f, chunks[1], app),
    }

    if app.dropdown_open && app.active_tab == ActiveTab::Calculator {
        draw_dropdown(f, app, " Выберите пластик для расчета [Enter] ");
    }
    if app.settings_dropdown_open && app.active_tab == ActiveTab::Settings {
        draw_dropdown(f, app, " Выберите пластик для изменения [Enter] ");
    }
}

fn draw_calculator_tab(f: &mut Frame, area: ratatui::layout::Rect, app: &TuiApp) {
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let input_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Материал
            Constraint::Length(3), // Вес
            Constraint::Length(3), // Часы
            Constraint::Length(3), // Копии
            Constraint::Length(3), // Коэффициент
            Constraint::Min(0),
        ])
        .split(main_layout[0]);

    let get_border_style = |field: ActiveField| {
        if app.active_field == field {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        }
    };

    f.render_widget(
        Paragraph::new(app.input_material_name.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Название пластика (из базы) ")
                .border_style(get_border_style(ActiveField::MaterialName)),
        ),
        input_chunks[0],
    );
    f.render_widget(
        Paragraph::new(app.input_weight.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Вес детали (в граммах) ")
                .border_style(get_border_style(ActiveField::Weight)),
        ),
        input_chunks[1],
    );
    f.render_widget(
        Paragraph::new(app.input_time.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Время печати (Часы Минуты, например '2 45' или '3') ")
                .border_style(get_border_style(ActiveField::Time)),
        ),
        input_chunks[2],
    );
    f.render_widget(
        Paragraph::new(app.input_copies.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Количество копий (шт) ")
                .border_style(get_border_style(ActiveField::Copies)),
        ),
        input_chunks[3],
    );
    f.render_widget(
        Paragraph::new(app.input_margin.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Коэффициент наценки (например: 1.5) ")
                .border_style(get_border_style(ActiveField::Margin)),
        ),
        input_chunks[4],
    );

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_layout[1]);

    let mat_name = app.input_material_name.value().trim().to_lowercase();
    let price_per_kg = app.config.materials.get(&mat_name).cloned().unwrap_or(0.0);

    let weight = app.input_weight.value().parse::<f64>().unwrap_or(0.0);
    let copies = app.input_copies.value().parse::<f64>().unwrap_or(1.0);
    let margin = app.input_margin.value().parse::<f64>().unwrap_or(1.0);

    let time_raw = app.input_time.value().trim();
    let parts: Vec<&str> = time_raw.split_whitespace().collect();
    let total_hours = match parts.as_slice() {
        [h_str, m_str] => {
            let h = h_str.parse::<f64>().unwrap_or(0.0);
            let m = m_str.parse::<f64>().unwrap_or(0.0);
            if m < 60.0 { h + (m / 60.0) } else { h }
        }
        [h_str] => h_str.parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    };

    let mat_cost_one = (weight / 1000.0) * price_per_kg;
    let time_cost_one = total_hours * app.config.price_per_hour;
    let total_cost_one = mat_cost_one + time_cost_one;
    let final_batch_cost = total_cost_one * copies * margin;

    let mut result_text = format!(
        "Цена часа печати:    {:.2}\n\
         Цена пластика за кг: {:.2} (для '{}')\n\n\
         --- Себестоимость 1 штуки ---\n\
         Пластик:             {:.2}\n\
         Время:               {:.2}\n\
         Итого за 1 шт:       {:.2}\n\n\
         =============================\n\
         ИТОГО ЗА {} шт. (с коэф. {:.2}): {:.2}",
        app.config.price_per_hour, price_per_kg, mat_name, mat_cost_one, time_cost_one, total_cost_one, copies, margin, final_batch_cost
    );

    if price_per_kg == 0.0 && !mat_name.is_empty() {
        result_text.push_str("\n\n⚠️ Внимание: Пластик не найден в базе данных! Расчет филамента равен 0.");
    }

    let results_block = Paragraph::new(result_text)
        .block(Block::default().borders(Borders::ALL).title(" Результаты расчета "))
        .style(Style::default().fg(Color::Green));
    f.render_widget(results_block, right_chunks[0]);

    let help_text = "Навигация: СТРЕЛКИ ВВЕРХ / ВНИЗ\n\
                      Переключение меню: TAB | Выход: ESC\n\n\
                      Нажмите [Пробел] в поле пластика,\n\
                      чтобы открыть выпадающий список.\n\n\
                      Изменить цены или добавить типы пластика\n\
                      можно на соседней вкладке настроек.";
    let help_block = Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title(" Справка по управлению "));
    f.render_widget(help_block, right_chunks[1]);
}

fn draw_dropdown(f: &mut Frame, app: &TuiApp, title: &str) {
    let area = f.area();

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(35), Constraint::Percentage(35)])
        .split(area);

    let target_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(40), Constraint::Percentage(30)])
        .split(popup_layout[1])[1]; // Берем центральный квадрант

    f.render_widget(Clear, target_area);

    // Сортируем ключи базы материалов, чтобы список был стабильным
    let mut sorted_materials: Vec<(&String, &f64)> = app.config.materials.iter().collect();
    sorted_materials.sort_by(|a, b| a.0.cmp(b.0));

    let items: Vec<ListItem> = sorted_materials
        .iter()
        .enumerate()
        .map(|(i, (name, price))| {
            let text = format!(" {} ({:.0} руб/кг)", name.to_uppercase(), price);
            if i == app.dropdown_selected {
                ListItem::new(text).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            } else {
                ListItem::new(text).style(Style::default().fg(Color::White))
            }
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    let mut list_state = ListState::default().with_selected(Some(app.dropdown_selected));
    f.render_stateful_widget(list, target_area, &mut list_state);
}

fn draw_settings_tab(f: &mut Frame, area: ratatui::layout::Rect, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let edit_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Цена часа
            Constraint::Length(3), // Название нового мат.
            Constraint::Length(3), // Цена нового мат.
            Constraint::Length(4), // Инфо блок
            Constraint::Min(0),
        ])
        .split(chunks[0]);

    let get_settings_border = |index: usize| {
        if app.active_settings_field == index {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        }
    };

    f.render_widget(
        Paragraph::new(app.input_price_hour.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" [Поле 1] Цена за 1 час работы принтера ")
                .border_style(get_settings_border(0)),
        ),
        edit_chunks[0],
    );
    f.render_widget(
        Paragraph::new(app.input_new_mat_name.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" [Поле 2] Добавить пластик: Название (например: tpu) ")
                .border_style(get_settings_border(1)),
        ),
        edit_chunks[1],
    );
    f.render_widget(
        Paragraph::new(app.input_new_mat_price.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" [Поле 3] Добавить пластик: Цена за 1 кг ")
                .border_style(get_settings_border(2)),
        ),
        edit_chunks[2],
    );

    let info_msg = " Нажмите [ENTER] внутри активного поля, чтобы\n сохранить/применить внесенные изменения в базу JSON.";
    f.render_widget(
        Paragraph::new(info_msg).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 💾 Как сохранить? ")
                .border_style(Style::default().fg(Color::Cyan)),
        ),
        edit_chunks[3],
    );

    let mut rows = Vec::new();
    for (name, price) in &app.config.materials {
        rows.push(Row::new(vec![name.clone(), format!("{:.2}", price)]));
    }

    let table = Table::new(rows, [Constraint::Percentage(50), Constraint::Percentage(50)])
        .header(Row::new(vec!["Материал", "Цена за 1 кг"]).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
        .block(Block::default().borders(Borders::ALL).title(" 📦 Текущая база пластиков в JSON "));

    f.render_widget(table, chunks[1]);
}
