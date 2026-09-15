use crate::controller;
use crate::model::date;
use crate::model::day::Day;
use crate::model::direction::Direction;
use crate::model::store::Store;
use crate::model::task::Priority;
use crate::view::forms::{self, DirectionAction, DirectionForm, TaskAction, TaskField, TaskForm};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction as LayoutDirection, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io;

/// Какое окошко в фокусе.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Tasks,
    Directions,
}

#[derive(Debug, Clone)]
pub enum Confirm {
    DeleteTask(String),
    DeleteDirection(String),
    Quit,
}
/// Нормализовать клавишу: русская раскладка → латинская.
///
/// Работает только для командных клавиш (a, e, d, q, j, k, g, G).
/// В формах не применяется, чтобы можно было вводить русский текст.
fn normalize_key(code: KeyCode) -> KeyCode {
    match code {
        KeyCode::Char('ф') => KeyCode::Char('a'),
        KeyCode::Char('у') => KeyCode::Char('e'),
        KeyCode::Char('в') => KeyCode::Char('d'),
        KeyCode::Char('й') => KeyCode::Char('q'),
        KeyCode::Char('о') => KeyCode::Char('j'),
        KeyCode::Char('л') => KeyCode::Char('k'),
        KeyCode::Char('п') => KeyCode::Char('g'),
        KeyCode::Char('П') => KeyCode::Char('G'),
        KeyCode::Char('н') => KeyCode::Char('y'),
        KeyCode::Char('т') => KeyCode::Char('n'),
        other => other,
    }
}
/// Состояние TUI.
pub struct App {
    pub direction_form: Option<DirectionForm>,
    pub confirm: Option<Confirm>,
    pub form: Option<TaskForm>,
    pub store: Store,
    pub date: String,
    pub focus: Focus,
    pub tasks_state: ListState,
    pub directions_state: ListState,
    pub should_quit: bool,
}

impl App {
    pub fn new(store: Store, date: String) -> Self {
        let mut tasks_state = ListState::default();
        if !store
            .get_day(&date)
            .map(|d| d.tasks.is_empty())
            .unwrap_or(true)
        {
            tasks_state.select(Some(0));
        }

        let mut directions_state = ListState::default();
        if !store.directions.is_empty() {
            directions_state.select(Some(0));
        }

        Self {
            direction_form: None,
            confirm: None,
            form: None,
            store,
            date,
            focus: Focus::Tasks,
            tasks_state,
            directions_state,
            should_quit: false,
        }
    }

    /// Текущий день.
    pub fn day(&self) -> Option<&Day> {
        self.store.get_day(&self.date)
    }

    /// Переключить фокус.
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Tasks => Focus::Directions,
            Focus::Directions => Focus::Tasks,
        };
    }

    /// Перемещение вверх.
    pub fn move_up(&mut self) {
        match self.focus {
            Focus::Tasks => {
                let len = self.day().map(|d| d.tasks.len()).unwrap_or(0);
                if len == 0 {
                    return;
                }
                let i = self.tasks_state.selected().unwrap_or(0);
                let next = if i == 0 { len - 1 } else { i - 1 };
                self.tasks_state.select(Some(next));
            }
            Focus::Directions => {
                let len = self.store.directions.len();
                if len == 0 {
                    return;
                }
                let i = self.directions_state.selected().unwrap_or(0);
                let next = if i == 0 { len - 1 } else { i - 1 };
                self.directions_state.select(Some(next));
            }
        }
    }

    /// Перемещение вниз.
    pub fn move_down(&mut self) {
        match self.focus {
            Focus::Tasks => {
                let len = self.day().map(|d| d.tasks.len()).unwrap_or(0);
                if len == 0 {
                    return;
                }
                let i = self.tasks_state.selected().unwrap_or(0);
                let next = (i + 1) % len;
                self.tasks_state.select(Some(next));
            }
            Focus::Directions => {
                let len = self.store.directions.len();
                if len == 0 {
                    return;
                }
                let i = self.directions_state.selected().unwrap_or(0);
                let next = (i + 1) % len;
                self.directions_state.select(Some(next));
            }
        }
    }

    /// В начало списка.
    pub fn move_home(&mut self) {
        match self.focus {
            Focus::Tasks => {
                if self.day().map(|d| !d.tasks.is_empty()).unwrap_or(false) {
                    self.tasks_state.select(Some(0));
                }
            }
            Focus::Directions => {
                if !self.store.directions.is_empty() {
                    self.directions_state.select(Some(0));
                }
            }
        }
    }

    /// В конец списка.
    pub fn move_end(&mut self) {
        match self.focus {
            Focus::Tasks => {
                let len = self.day().map(|d| d.tasks.len()).unwrap_or(0);
                if len > 0 {
                    self.tasks_state.select(Some(len - 1));
                }
            }
            Focus::Directions => {
                let len = self.store.directions.len();
                if len > 0 {
                    self.directions_state.select(Some(len - 1));
                }
            }
        }
    }
}

/// Запустить TUI.
pub fn run(store: Store, date: String) -> Result<()> {
    // Настройка терминала.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Создаём App.
    let mut app = App::new(store, date);

    // Главный цикл.
    let res = run_loop(&mut terminal, &mut app);

    // Восстановление терминала.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    println!();
    if let Err(err) = res {
        eprintln!("Ошибка TUI: {:?}", err);
    }
    Ok(())
}

/// Главный цикл обработки событий.
fn run_loop<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            // Если открыта форма — обрабатываем её.
            if let Some(form) = &mut app.form {
                if let Some(action) = form.handle_key(key) {
                    match action {
                        TaskAction::Save => {
                            // Сохраняем задачу.
                            let task = form.task.clone();
                            let is_new = form.is_new;

                            if is_new {
                                controller::actions::add_task(&app.date, task)?;
                            } else {
                                controller::actions::update_task(&app.date, task)?;
                            }

                            // Перезагружаем store.
                            app.store = crate::model::storage::load()?;
                            app.form = None;
                        }
                        TaskAction::Cancel => {
                            app.form = None;
                        }
                    }
                }
                continue;
            }
            // Если открыта форма направления — обрабатываем её.
            if let Some(form) = &mut app.direction_form {
                if let Some(action) = form.handle_key(key) {
                    match action {
                        DirectionAction::Save => {
                            let direction = form.direction.clone();
                            let is_new = form.is_new;
                            let original_name = form.original_name.clone();

                            if is_new {
                                controller::actions::add_direction(direction)?;
                            } else {
                                controller::actions::update_direction(&original_name, direction)?;
                            }

                            app.store = crate::model::storage::load()?;
                            app.direction_form = None;
                        }
                        DirectionAction::Cancel => {
                            app.direction_form = None;
                        }
                    }
                }
                continue;
            }

            // Если открыто подтверждение — обрабатываем его.
            if let Some(confirm) = &app.confirm {
                match normalize_key(key.code) {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        match confirm {
                            Confirm::DeleteTask(id) => {
                                let id = id.clone();
                                controller::actions::delete_task(&app.date, &id)?;
                                app.store = crate::model::storage::load()?;
                            }
                            Confirm::DeleteDirection(name) => {
                                let name = name.clone();
                                controller::actions::delete_direction(&name)?;
                                app.store = crate::model::storage::load()?;
                            }
                            Confirm::Quit => {
                                app.should_quit = true;
                            }
                        }
                        app.confirm = None;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        app.confirm = None;
                    }
                    _ => {}
                }
                continue;
            }

            // Иначе — обычная навигация.
            match normalize_key(key.code) {
                KeyCode::Char('q') => {
                    app.confirm = Some(Confirm::Quit);
                }
                KeyCode::Tab => app.toggle_focus(),
                KeyCode::BackTab => app.toggle_focus(),
                KeyCode::Up => app.move_up(),
                KeyCode::Down => app.move_down(),
                KeyCode::Char('k') => app.move_up(),
                KeyCode::Char('j') => app.move_down(),
                KeyCode::Char('g') => app.move_home(),
                KeyCode::Char('G') => app.move_end(),
                KeyCode::Char('a') => {
                    // Создать задачу или направление в зависимости от фокуса.
                    match app.focus {
                        Focus::Tasks => {
                            app.form = Some(TaskForm::new());
                        }
                        Focus::Directions => {
                            app.direction_form = Some(DirectionForm::new());
                        }
                    }
                }
                KeyCode::Char('e') | KeyCode::Enter => {
                    // Редактировать выбранное.
                    match app.focus {
                        Focus::Tasks => {
                            if let Some(day) = app.day() {
                                if let Some(i) = app.tasks_state.selected() {
                                    if let Some(task) = day.tasks.get(i) {
                                        app.form = Some(TaskForm::edit(task.clone()));
                                    }
                                }
                            }
                        }
                        Focus::Directions => {
                            if let Some(i) = app.directions_state.selected() {
                                if let Some(d) = app.store.directions.get(i) {
                                    app.direction_form = Some(DirectionForm::edit(d.clone()));
                                }
                            }
                        }
                    }
                }
                KeyCode::Char('d') => {
                    // Открыть форму редактирования с фокусом на комментарии.
                    match app.focus {
                        Focus::Tasks => {
                            if let Some(day) = app.day() {
                                if let Some(i) = app.tasks_state.selected() {
                                    if let Some(task) = day.tasks.get(i) {
                                        let mut form = TaskForm::edit(task.clone());
                                        form.field = TaskField::Comment;
                                        app.form = Some(form);
                                    }
                                }
                            }
                        }
                        Focus::Directions => {
                            // Пока не нужно.
                        }
                    }
                }
                KeyCode::Delete => match app.focus {
                    Focus::Tasks => {
                        if let Some(day) = app.day() {
                            if let Some(i) = app.tasks_state.selected() {
                                if let Some(task) = day.tasks.get(i) {
                                    let id = task.id.clone();
                                    app.confirm = Some(Confirm::DeleteTask(id));
                                }
                            }
                        }
                    }
                    Focus::Directions => {
                        if let Some(i) = app.directions_state.selected() {
                            if let Some(d) = app.store.directions.get(i) {
                                let name = d.name.clone();
                                app.confirm = Some(Confirm::DeleteDirection(name));
                            }
                        }
                    }
                },
                _ => {}
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

/// Отрисовать весь экран.
fn draw(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(LayoutDirection::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(size);

    draw_header(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_status(f, app, chunks[2]);

    // Форма поверх всего.
    // Форма задачи поверх всего.
    if let Some(form) = &app.form {
        forms::draw_task_form(f, form, size);
    }

    // Форма направления.
    if let Some(form) = &app.direction_form {
        forms::draw_direction_form(f, form, size);
    }

    // Подтверждение — поверх всего.
    if let Some(confirm) = &app.confirm {
        forms::draw_confirm(f, confirm, size);
    }
}

/// Заголовок.
fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let parsed = date::parse(&app.date).ok();
    let weekday = parsed.map(date::weekday_ru).unwrap_or("?");
    let title = format!(" dn ─ {} ({}) ", app.date, weekday);

    let block = Block::default().borders(Borders::ALL).title(title);

    f.render_widget(block, area);
}

/// Контент: слева задачи (60%), справа направления (40%).
fn draw_content(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(LayoutDirection::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    draw_tasks(f, app, chunks[0]);
    draw_directions(f, app, chunks[1]);
}

/// Окошко задач.
fn draw_tasks(f: &mut Frame, app: &mut App, area: Rect) {
    let title = " Задачи ";
    let is_focused = app.focus == Focus::Tasks;

    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    let tasks = match app.day() {
        Some(day) => &day.tasks,
        None => {
            let p = Paragraph::new("День не найден").block(block);
            f.render_widget(p, area);
            return;
        }
    };

    if tasks.is_empty() {
        let p = Paragraph::new("Задач нет. Нажми [a], чтобы добавить.")
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = tasks
        .iter()
        .map(|task| {
            let done = task.is_done();
            let base_style = if done {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let priority_style = Style::default()
                .fg(priority_color(task.priority))
                .add_modifier(Modifier::BOLD);

            let mut lines = vec![Line::from(vec![Span::styled(
                task.title.clone(),
                base_style,
            )])];

            // Строка с метаданными: приоритет, веха.
            let mut meta = vec![Span::styled(
                task.priority.label().to_string(),
                priority_style,
            )];
            if !task.milestone.is_empty() {
                meta.push(Span::raw(" · "));
                meta.push(Span::styled(
                    task.milestone.clone(),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            lines.push(Line::from(meta));

            // Комментарий.
            if done {
                lines.push(Line::from(vec![
                    Span::raw("  └ "),
                    Span::styled(task.comment.clone(), base_style),
                ]));
            }

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.tasks_state);
}

/// Окошко направлений.
fn draw_directions(f: &mut Frame, app: &mut App, area: Rect) {
    let title = " Направления ";
    let is_focused = app.focus == Focus::Directions;

    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    if app.store.directions.is_empty() {
        let p = Paragraph::new("Направлений нет. Нажми [a], чтобы добавить.")
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = app
        .store
        .directions
        .iter()
        .map(|d| direction_item(d))
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.directions_state);
}

/// Один элемент направления.
fn direction_item(d: &Direction) -> ListItem<'static> {
    let priority_style = Style::default()
        .fg(priority_color(d.priority))
        .add_modifier(Modifier::BOLD);

    let mut lines = vec![Line::from(vec![
        Span::raw("● "),
        Span::styled(
            d.name.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(d.priority.label().to_string(), priority_style),
    ])];

    if !d.description.is_empty() {
        for line in d.description.lines() {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(line.to_string(), Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    ListItem::new(lines)
}

/// Статус-бар.
fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let text = match app.focus {
        Focus::Tasks => {
            " [a] добавить  [e] редактировать  [d] комментарий  [Del] удалить  [Tab] направления  [q] выход "
        }
        Focus::Directions => {
            " [a] добавить  [e] редактировать  [Del] удалить  [Tab] задачи  [q] выход "
        }
    };

    let p = Paragraph::new(text).block(Block::default().borders(Borders::ALL));

    f.render_widget(p, area);
}

/// Цвет для приоритета.
fn priority_color(p: Priority) -> Color {
    match p {
        Priority::High => Color::Red,
        Priority::Med => Color::Blue,
        Priority::Low => Color::Green,
    }
}
