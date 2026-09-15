use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::model::task::{Priority, Task};
use crate::view::tui::Confirm;

/// Какое поле формы в фокусе.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskField {
    Title,
    Description,
    Priority,
    Milestone,
    Comment,
}

impl TaskField {
    fn next(self) -> Self {
        match self {
            TaskField::Title => TaskField::Description,
            TaskField::Description => TaskField::Priority,
            TaskField::Priority => TaskField::Milestone,
            TaskField::Milestone => TaskField::Comment,
            TaskField::Comment => TaskField::Title,
        }
    }

    fn prev(self) -> Self {
        match self {
            TaskField::Title => TaskField::Comment,
            TaskField::Description => TaskField::Title,
            TaskField::Priority => TaskField::Description,
            TaskField::Milestone => TaskField::Priority,
            TaskField::Comment => TaskField::Milestone,
        }
    }
}

/// Что делать после обработки клавиши.
pub enum TaskAction {
    Save,
    Cancel,
}

/// Форма задачи.
pub struct TaskForm {
    pub task: Task,
    pub field: TaskField,
    pub is_new: bool,
    pub dirty: bool,
    pub error: Option<String>,
}

impl TaskForm {
    pub fn new() -> Self {
        Self {
            task: Task::new(String::new()),
            field: TaskField::Title,
            is_new: true,
            dirty: false,
            error: None,
        }
    }

    pub fn edit(task: Task) -> Self {
        Self {
            task,
            field: TaskField::Title,
            is_new: false,
            dirty: false,
            error: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<TaskAction> {
        match key.code {
            KeyCode::Esc => {
                if self.dirty {
                    self.error = Some("Есть изменения. F2 сохранить, Esc ещё раз отменить.".into());
                    self.dirty = false;
                    return None;
                }
                return Some(TaskAction::Cancel);
            }
            KeyCode::Tab => {
                self.field = self.field.next();
                self.error = None;
            }
            KeyCode::BackTab => {
                self.field = self.field.prev();
                self.error = None;
            }
            KeyCode::Left => {
                if self.field == TaskField::Priority {
                    self.task.priority = self.task.priority.prev();
                    self.dirty = true;
                }
            }
            KeyCode::Right => {
                if self.field == TaskField::Priority {
                    self.task.priority = self.task.priority.next();
                    self.dirty = true;
                }
            }
            KeyCode::F(2) => {
                if self.task.title.trim().is_empty() {
                    self.error = Some("Название не может быть пустым".into());
                    return None;
                }
                return Some(TaskAction::Save);
            }
            KeyCode::Enter => match self.field {
                TaskField::Title | TaskField::Milestone => {
                    self.field = self.field.next();
                }
                TaskField::Description | TaskField::Comment => {
                    self.push_char('\n');
                }
                TaskField::Priority => {
                    self.field = self.field.next();
                }
            },
            KeyCode::Backspace => {
                self.pop_char();
            }
            KeyCode::Char(c) => {
                self.push_char(c);
            }
            _ => {}
        }
        None
    }

    fn push_char(&mut self, c: char) {
        self.dirty = true;
        match self.field {
            TaskField::Title => self.task.title.push(c),
            TaskField::Description => self.task.description.push(c),
            TaskField::Priority => {}
            TaskField::Milestone => self.task.milestone.push(c),
            TaskField::Comment => self.task.comment.push(c),
        }
    }

    fn pop_char(&mut self) {
        self.dirty = true;
        match self.field {
            TaskField::Title => {
                self.task.title.pop();
            }
            TaskField::Description => {
                self.task.description.pop();
            }
            TaskField::Priority => {}
            TaskField::Milestone => {
                self.task.milestone.pop();
            }
            TaskField::Comment => {
                self.task.comment.pop();
            }
        }
    }
}

/// Отрисовать форму задачи.
pub fn draw_task_form(f: &mut Frame, form: &TaskForm, area: Rect) {
    let popup = centered_rect(80, 90, area);
    f.render_widget(Clear, popup);

    let title = if form.is_new {
        " Новая задача "
    } else {
        " Редактирование задачи "
    };

    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Length(3),
        ])
        .split(inner);

    draw_field(
        f,
        chunks[0],
        "Название",
        &form.task.title,
        form.field == TaskField::Title,
    );

    draw_field(
        f,
        chunks[1],
        "Описание",
        &form.task.description,
        form.field == TaskField::Description,
    );

    draw_priority_field(
        f,
        chunks[2],
        form.task.priority,
        form.field == TaskField::Priority,
    );

    draw_field(
        f,
        chunks[3],
        "Веха",
        &form.task.milestone,
        form.field == TaskField::Milestone,
    );

    draw_field(
        f,
        chunks[4],
        "Комментарий",
        &form.task.comment,
        form.field == TaskField::Comment,
    );

    let hint = if let Some(err) = &form.error {
        Span::styled(err.clone(), Style::default().fg(Color::Red))
    } else {
        Span::styled(
            "[F2] сохранить  [Esc] отмена",
            Style::default().fg(Color::DarkGray),
        )
    };
    let p = Paragraph::new(Line::from(hint));
    f.render_widget(p, chunks[5]);
}

fn draw_field(f: &mut Frame, area: Rect, label: &str, value: &str, focused: bool) {
    let style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", label))
        .border_style(style);

    let p = Paragraph::new(value.to_string())
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn draw_priority_field(f: &mut Frame, area: Rect, priority: Priority, focused: bool) {
    let style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Приоритет ")
        .border_style(style);

    let color = match priority {
        Priority::High => Color::Red,
        Priority::Med => Color::Blue,
        Priority::Low => Color::Green,
    };

    let p = Paragraph::new(Line::from(vec![
        Span::styled("← ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            priority.label().to_string(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" →", Style::default().fg(Color::DarkGray)),
    ]))
    .block(block);

    f.render_widget(p, area);
}

/// Прямоугольник по центру.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Отрисовать подтверждение.
pub fn draw_confirm(f: &mut Frame, confirm: &Confirm, area: Rect) {
    let text = match confirm {
        Confirm::DeleteTask(_) => "Удалить задачу? (y/n)",
        Confirm::DeleteDirection(_) => "Удалить направление? (y/n)",
        Confirm::Quit => "Выйти? (y/n)",
    };

    let popup = centered_rect(40, 20, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Подтверждение ");

    let p = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(p, popup);
}

/// Какое поле формы направления в фокусе.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionField {
    Name,
    Description,
    Priority,
}

impl DirectionField {
    fn next(self) -> Self {
        match self {
            DirectionField::Name => DirectionField::Description,
            DirectionField::Description => DirectionField::Priority,
            DirectionField::Priority => DirectionField::Name,
        }
    }

    fn prev(self) -> Self {
        match self {
            DirectionField::Name => DirectionField::Priority,
            DirectionField::Description => DirectionField::Name,
            DirectionField::Priority => DirectionField::Description,
        }
    }
}

/// Что делать после обработки клавиши в форме направления.
pub enum DirectionAction {
    Save,
    Cancel,
}

/// Форма направления.
pub struct DirectionForm {
    pub direction: crate::model::direction::Direction,
    pub field: DirectionField,
    pub is_new: bool,
    pub original_name: String,
    pub dirty: bool,
    pub error: Option<String>,
}

impl DirectionForm {
    pub fn new() -> Self {
        Self {
            direction: crate::model::direction::Direction::new(String::new()),
            field: DirectionField::Name,
            is_new: true,
            original_name: String::new(),
            dirty: false,
            error: None,
        }
    }

    pub fn edit(direction: crate::model::direction::Direction) -> Self {
        Self {
            original_name: direction.name.clone(),
            direction,
            field: DirectionField::Name,
            is_new: false,
            dirty: false,
            error: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<DirectionAction> {
        match key.code {
            KeyCode::Esc => {
                if self.dirty {
                    self.error = Some("Есть изменения. F2 сохранить, Esc ещё раз отменить.".into());
                    self.dirty = false;
                    return None;
                }
                return Some(DirectionAction::Cancel);
            }
            KeyCode::Tab => {
                self.field = self.field.next();
                self.error = None;
            }
            KeyCode::BackTab => {
                self.field = self.field.prev();
                self.error = None;
            }
            KeyCode::Left => {
                if self.field == DirectionField::Priority {
                    self.direction.priority = self.direction.priority.prev();
                    self.dirty = true;
                }
            }
            KeyCode::Right => {
                if self.field == DirectionField::Priority {
                    self.direction.priority = self.direction.priority.next();
                    self.dirty = true;
                }
            }
            KeyCode::F(2) => {
                if self.direction.name.trim().is_empty() {
                    self.error = Some("Название не может быть пустым".into());
                    return None;
                }
                return Some(DirectionAction::Save);
            }
            KeyCode::Enter => match self.field {
                DirectionField::Name => {
                    self.field = self.field.next();
                }
                DirectionField::Description => {
                    self.push_char('\n');
                }
                DirectionField::Priority => {
                    self.field = self.field.next();
                }
            },
            KeyCode::Backspace => {
                self.pop_char();
            }
            KeyCode::Char(c) => {
                self.push_char(c);
            }
            _ => {}
        }
        None
    }

    fn push_char(&mut self, c: char) {
        self.dirty = true;
        match self.field {
            DirectionField::Name => self.direction.name.push(c),
            DirectionField::Description => self.direction.description.push(c),
            DirectionField::Priority => {}
        }
    }

    fn pop_char(&mut self) {
        self.dirty = true;
        match self.field {
            DirectionField::Name => {
                self.direction.name.pop();
            }
            DirectionField::Description => {
                self.direction.description.pop();
            }
            DirectionField::Priority => {}
        }
    }
}

/// Отрисовать форму направления.
pub fn draw_direction_form(f: &mut Frame, form: &DirectionForm, area: Rect) {
    let popup = centered_rect(80, 90, area);
    f.render_widget(Clear, popup);

    let title = if form.is_new {
        " Новое направление "
    } else {
        " Редактирование направления "
    };

    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(15),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(inner);

    draw_field(
        f,
        chunks[0],
        "Название",
        &form.direction.name,
        form.field == DirectionField::Name,
    );

    draw_field(
        f,
        chunks[1],
        "Описание",
        &form.direction.description,
        form.field == DirectionField::Description,
    );

    draw_priority_field(
        f,
        chunks[2],
        form.direction.priority,
        form.field == DirectionField::Priority,
    );

    let hint = if let Some(err) = &form.error {
        Span::styled(err.clone(), Style::default().fg(Color::Red))
    } else {
        Span::styled(
            "[F2] сохранить  [Esc] отмена",
            Style::default().fg(Color::DarkGray),
        )
    };
    let p = Paragraph::new(Line::from(hint));
    f.render_widget(p, chunks[3]);
}
