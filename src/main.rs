pub mod config;
pub mod ui;

use config::{Config, load_config, save_config};
use ui::draw_ui;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use std::io;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveField {
    MaterialName,
    Weight,
    Time,
    Copies,
    Margin,
}

impl ActiveField {
    pub fn next(self) -> Self {
        match self {
            ActiveField::MaterialName => ActiveField::Weight,
            ActiveField::Weight => ActiveField::Time,
            ActiveField::Time => ActiveField::Copies,
            ActiveField::Copies => ActiveField::Margin,
            ActiveField::Margin => ActiveField::MaterialName,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            ActiveField::MaterialName => ActiveField::Margin,
            ActiveField::Weight => ActiveField::MaterialName,
            ActiveField::Time => ActiveField::Weight,
            ActiveField::Copies => ActiveField::Time,
            ActiveField::Margin => ActiveField::Copies,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Calculator,
    Settings,
}

pub struct TuiApp {
    pub config: Config,
    pub active_tab: ActiveTab,
    pub active_field: ActiveField,

    pub input_material_name: Input,
    pub input_weight: Input,
    pub input_time: Input,
    pub input_copies: Input,
    pub input_margin: Input,

    pub input_price_hour: Input,
    pub input_new_mat_name: Input,
    pub input_new_mat_price: Input,
    pub active_settings_field: usize,

    pub dropdown_open: bool,
    pub dropdown_selected: usize,
    pub settings_dropdown_open: bool,
}

impl TuiApp {
    fn new(config: Config) -> Self {
        Self {
            input_price_hour: Input::from(config.price_per_hour.to_string()),
            config,
            active_tab: ActiveTab::Calculator,
            active_field: ActiveField::MaterialName,
            input_material_name: Input::from(""),
            input_weight: Input::from(""),
            input_time: Input::from(""),
            input_copies: Input::from(""),
            input_margin: Input::from(""),
            input_new_mat_name: Input::from(""),
            input_new_mat_price: Input::from(""),
            active_settings_field: 0,
            dropdown_open: false,
            dropdown_selected: 0,
            settings_dropdown_open: false,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let config = load_config().unwrap_or_default();
    let mut app = TuiApp::new(config);

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Ошибка работы TUI: {:?}", err);
    }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut TuiApp) -> io::Result<()>
where
    std::io::Error: From<<B as Backend>::Error>,
{
    loop {
        terminal.draw(|f| draw_ui(f, app))?;

        if let Event::Key(key) = event::read().map_err(|e| io::Error::new(io::ErrorKind::Other, e))? {
            if app.dropdown_open {
                let materials_count = app.config.materials.len();
                match key.code {
                    KeyCode::Esc => {
                        app.dropdown_open = false;
                    }
                    KeyCode::Down => {
                        if materials_count > 0 {
                            app.dropdown_selected = (app.dropdown_selected + 1) % materials_count;
                        }
                    }
                    KeyCode::Up => {
                        if materials_count > 0 {
                            app.dropdown_selected = if app.dropdown_selected == 0 {
                                materials_count - 1
                            } else {
                                app.dropdown_selected - 1
                            };
                        }
                    }
                    KeyCode::Enter => {
                        // Подтверждаем выбор из списка
                        let mut sorted_keys: Vec<&String> = app.config.materials.keys().collect();
                        sorted_keys.sort();
                        if let Some(&chosen_material) = sorted_keys.get(app.dropdown_selected) {
                            app.input_material_name = Input::from(chosen_material.clone());
                        }
                        app.dropdown_open = false;
                    }
                    _ => {}
                }
                continue; // Пропускаем стандартную обработку ввода, пока открыт дропдаун
            }

            if app.settings_dropdown_open {
                let materials_count = app.config.materials.len();
                match key.code {
                    KeyCode::Esc => app.settings_dropdown_open = false,
                    KeyCode::Down => {
                        if materials_count > 0 {
                            app.dropdown_selected = (app.dropdown_selected + 1) % materials_count;
                        }
                    }
                    KeyCode::Up => {
                        if materials_count > 0 {
                            app.dropdown_selected = if app.dropdown_selected == 0 {
                                materials_count - 1
                            } else {
                                app.dropdown_selected - 1
                            };
                        }
                    }
                    KeyCode::Enter => {
                        let mut sorted_keys: Vec<&String> = app.config.materials.keys().collect();
                        sorted_keys.sort();
                        if let Some(&chosen_material) = sorted_keys.get(app.dropdown_selected) {
                            app.input_new_mat_name = Input::from(chosen_material.clone());
                            // Автоматически подставляем текущую цену для этого имени, если она есть
                            if let Some(price) = app.config.materials.get(chosen_material) {
                                app.input_new_mat_price = Input::from(price.to_string());
                            }
                            app.active_settings_field = 2; // Перекидываем фокус сразу на ввод цены
                        }
                        app.settings_dropdown_open = false;
                    }
                    _ => {}
                }
                continue;
            }

            match key.code {
                KeyCode::Esc => return Ok(()),
                KeyCode::Tab => {
                    app.active_tab = match app.active_tab {
                        ActiveTab::Calculator => ActiveTab::Settings,
                        ActiveTab::Settings => ActiveTab::Calculator,
                    };
                }
                KeyCode::Char(' ') if app.active_tab == ActiveTab::Calculator && app.active_field == ActiveField::MaterialName => {
                    app.dropdown_open = true;
                    app.dropdown_selected = 0;
                }
                KeyCode::Char(' ') if app.active_tab == ActiveTab::Settings && app.active_settings_field == 1 => {
                    app.settings_dropdown_open = true;
                    app.dropdown_selected = 0;
                }
                KeyCode::Down => {
                    if app.active_tab == ActiveTab::Calculator {
                        app.active_field = app.active_field.next();
                    } else {
                        app.active_settings_field = (app.active_settings_field + 1) % 3;
                    }
                }
                KeyCode::Up => {
                    if app.active_tab == ActiveTab::Calculator {
                        app.active_field = app.active_field.prev();
                    } else {
                        app.active_settings_field = if app.active_settings_field == 0 { 2 } else { app.active_settings_field - 1 };
                    }
                }
                KeyCode::Enter => {
                    if app.active_tab == ActiveTab::Settings {
                        if app.active_settings_field == 0 {
                            if let Ok(val) = app.input_price_hour.value().parse::<f64>() {
                                app.config.price_per_hour = val;
                                let _ = save_config(&app.config);
                            }
                        } else if app.active_settings_field == 2 || app.active_settings_field == 1 {
                            let name = app.input_new_mat_name.value().trim().to_lowercase();
                            let price = app.input_new_mat_price.value().parse::<f64>().unwrap_or(0.0);
                            if !name.is_empty() && price > 0.0 {
                                app.config.materials.insert(name, price);
                                let _ = save_config(&app.config);
                                app.input_new_mat_name = Input::from("");
                                app.input_new_mat_price = Input::from("");
                                app.active_settings_field = 1;
                            }
                        }
                    }
                }
                _ => {
                    if app.active_tab == ActiveTab::Calculator {
                        match app.active_field {
                            ActiveField::MaterialName => {
                                app.input_material_name.handle_event(&Event::Key(key));
                            }
                            ActiveField::Weight => {
                                app.input_weight.handle_event(&Event::Key(key));
                            }
                            ActiveField::Time => {
                                app.input_time.handle_event(&Event::Key(key));
                            } // ОБЪЕДИНЕНО
                            ActiveField::Copies => {
                                app.input_copies.handle_event(&Event::Key(key));
                            }
                            ActiveField::Margin => {
                                app.input_margin.handle_event(&Event::Key(key));
                            }
                        }
                    } else {
                        match app.active_settings_field {
                            0 => {
                                app.input_price_hour.handle_event(&Event::Key(key));
                            }
                            1 => {
                                app.input_new_mat_name.handle_event(&Event::Key(key));
                            }
                            2 => {
                                app.input_new_mat_price.handle_event(&Event::Key(key));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
