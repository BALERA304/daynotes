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
    pub scroll: u16,
    pub cursor: usize, // позиция курсора в активном поле (в символах)
    pub task: Task,
    pub field: TaskField,
    pub is_new: bool,
    pub dirty: bool,
    pub error: Option<String>,
}

impl TaskForm {
    pub fn new() -> Self {
        Self {
            scroll: 0,
            cursor: 0,
            task: Task::new(String::new()),
            field: TaskField::Title,
            is_new: true,
            dirty: false,
            error: None,
        }
    }

    pub fn edit(task: Task) -> Self {
        Self {
            scroll: 0,
            cursor: 0,
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
                self.cursor = self.current_text().chars().count();
                self.error = None;
            }
            KeyCode::BackTab => {
                self.field = self.field.prev();
                self.cursor = self.current_text().chars().count();
                self.error = None;
            }
            KeyCode::Left => {
                if self.field == TaskField::Priority {
                    self.task.priority = self.task.priority.prev();
                    self.dirty = true;
                } else if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.field == TaskField::Priority {
                    self.task.priority = self.task.priority.next();
                    self.dirty = true;
                } else {
                    let len = self.current_text().chars().count();
                    if self.cursor < len {
                        self.cursor += 1;
                    }
                }
            }
            KeyCode::Home => {
                if self.field != TaskField::Priority {
                    self.cursor = 0;
                }
            }
            KeyCode::End => {
                if self.field != TaskField::Priority {
                    self.cursor = self.current_text().chars().count();
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
                    self.cursor = self.current_text().chars().count();
                }
                TaskField::Description | TaskField::Comment => {
                    self.insert_char('\n');
                }
                TaskField::Priority => {
                    self.field = self.field.next();
                    self.cursor = self.current_text().chars().count();
                }
            },
            KeyCode::Backspace => {
                self.delete_char();
            }
            KeyCode::Char(c) => {
                self.insert_char(c);
            }
            _ => {}
        }
        None
    }

    fn current_text(&self) -> &str {
        match self.field {
            TaskField::Title => &self.task.title,
            TaskField::Description => &self.task.description,
            TaskField::Priority => "",
            TaskField::Milestone => &self.task.milestone,
            TaskField::Comment => &self.task.comment,
        }
    }

    fn current_text_mut(&mut self) -> &mut String {
        match self.field {
            TaskField::Title => &mut self.task.title,
            TaskField::Description => &mut self.task.description,
            TaskField::Priority => panic!("приоритет не редактируется текстом"),
            TaskField::Milestone => &mut self.task.milestone,
            TaskField::Comment => &mut self.task.comment,
        }
    }

    fn insert_char(&mut self, c: char) {
        self.dirty = true;
        let cursor = self.cursor;
        let text = self.current_text_mut();
        let byte_pos = text
            .char_indices()
            .nth(cursor)
            .map(|(i, _)| i)
            .unwrap_or(text.len());
        text.insert(byte_pos, c);
        self.cursor += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.dirty = true;
        let cursor = self.cursor;
        let text = self.current_text_mut();
        let byte_pos = text
            .char_indices()
            .nth(cursor - 1)
            .map(|(i, _)| i)
            .unwrap_or(0);
        text.remove(byte_pos);
        self.cursor -= 1;
    }
    /// Вычислить, на какой экранной строке находится курсор.
    fn cursor_screen_line(&self, width: u16) -> usize {
        if width == 0 {
            return 0;
        }
        let before: String = self.current_text().chars().take(self.cursor).collect();
        let mut line = 0usize;
        let mut col = 0u16;
        for c in before.chars() {
            if c == '\n' {
                line += 1;
                col = 0;
            } else {
                if col >= width {
                    line += 1;
                    col = 0;
                }
                col += 1;
            }
        }
        line
    }

    /// Пересчитать scroll так, чтобы курсор был виден.
    pub fn adjust_scroll(&mut self, visible_height: u16, width: u16) {
        if width == 0 || visible_height == 0 {
            return;
        }
        let cursor_line = self.cursor_screen_line(width);
        if cursor_line < self.scroll as usize {
            self.scroll = cursor_line as u16;
        } else if cursor_line >= self.scroll as usize + visible_height as usize {
            self.scroll = (cursor_line - visible_height as usize + 1) as u16;
        }
    }
}

/// Отрисовать форму задачи.
pub fn draw_task_form(f: &mut Frame, form: &mut TaskForm, area: Rect) {
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
    // Пересчитать скролл активного поля.
    match form.field {
        TaskField::Title => {
            let h = chunks[0].height.saturating_sub(2);
            let w = chunks[0].width.saturating_sub(2);
            form.adjust_scroll(h, w);
        }
        TaskField::Description => {
            let h = chunks[1].height.saturating_sub(2);
            let w = chunks[1].width.saturating_sub(2);
            form.adjust_scroll(h, w);
        }
        TaskField::Milestone => {
            let h = chunks[3].height.saturating_sub(2);
            let w = chunks[3].width.saturating_sub(2);
            form.adjust_scroll(h, w);
        }
        TaskField::Comment => {
            let h = chunks[4].height.saturating_sub(2);
            let w = chunks[4].width.saturating_sub(2);
            form.adjust_scroll(h, w);
        }
        TaskField::Priority => {}
    }
    let title_focused = form.field == TaskField::Title;
    draw_field(
        f,
        chunks[0],
        "Название",
        &form.task.title,
        title_focused,
        if title_focused {
            Some(form.cursor)
        } else {
            None
        },
        if title_focused { form.scroll } else { 0 },
    );

    let desc_focused = form.field == TaskField::Description;
    draw_field(
        f,
        chunks[1],
        "Описание",
        &form.task.description,
        desc_focused,
        if desc_focused {
            Some(form.cursor)
        } else {
            None
        },
        if desc_focused { form.scroll } else { 0 },
    );

    draw_priority_field(
        f,
        chunks[2],
        form.task.priority,
        form.field == TaskField::Priority,
    );

    let milestone_focused = form.field == TaskField::Milestone;
    draw_field(
        f,
        chunks[3],
        "Веха",
        &form.task.milestone,
        milestone_focused,
        if milestone_focused {
            Some(form.cursor)
        } else {
            None
        },
        if milestone_focused { form.scroll } else { 0 },
    );

    let comment_focused = form.field == TaskField::Comment;
    draw_field(
        f,
        chunks[4],
        "Комментарий",
        &form.task.comment,
        comment_focused,
        if comment_focused {
            Some(form.cursor)
        } else {
            None
        },
        if comment_focused { form.scroll } else { 0 },
    );

    let hint = if let Some(err) = &form.error {
        Span::styled(err.clone(), Style::default().fg(Color::Red))
    } else {
        Span::styled(
            "[F2] сохранить  [Esc] отмена  [←→] курсор  [Home/End] начало/конец",
            Style::default().fg(Color::DarkGray),
        )
    };
    let p = Paragraph::new(Line::from(hint));
    f.render_widget(p, chunks[5]);
}

/// Отрисовать текстовое поле с курсором.
///
/// `cursor` — позиция курсора в символах, `None` если поле не в фокусе.
fn draw_field(
    f: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    focused: bool,
    cursor: Option<usize>,
    scroll: u16,
) {
    let style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", label))
        .border_style(style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let p = Paragraph::new(value.to_string())
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(p, inner);

    // Установить курсор, если поле в фокусе.
    if let Some(cursor) = cursor {
        if let Some((x, y)) = cursor_position(value, cursor, inner, scroll) {
            f.set_cursor_position((x, y));
        }
    }
}

/// Вычислить позицию курсора на экране.
///
/// Возвращает (x, y) — абсолютные координаты в терминале.
fn cursor_position(value: &str, cursor: usize, area: Rect, scroll: u16) -> Option<(u16, u16)> {
    let width = area.width as usize;
    if width == 0 {
        return None;
    }

    let before: String = value.chars().take(cursor).collect();

    let mut screen_line: usize = 0;
    let mut col: usize = 0;

    for c in before.chars() {
        if c == '\n' {
            screen_line += 1;
            col = 0;
        } else {
            if col >= width {
                screen_line += 1;
                col = 0;
            }
            col += 1;
        }
    }

    let visible_line = screen_line.saturating_sub(scroll as usize);

    // Если курсор за пределами — прижимаем к границе.
    let y = if visible_line >= area.height as usize {
        area.y + area.height - 1
    } else {
        area.y + visible_line as u16
    };

    let x = area.x + (col as u16).min(area.width.saturating_sub(1));

    Some((x, y))
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

// ============================================================
// Форма направления
// ============================================================

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
    pub scroll: u16,
    pub cursor: usize,
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
            scroll: 0,
            cursor: 0,
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
            scroll: 0,
            cursor: 0,
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
                self.cursor = self.current_text().chars().count();
                self.error = None;
            }
            KeyCode::BackTab => {
                self.field = self.field.prev();
                self.cursor = self.current_text().chars().count();
                self.error = None;
            }
            KeyCode::Left => {
                if self.field == DirectionField::Priority {
                    self.direction.priority = self.direction.priority.prev();
                    self.dirty = true;
                } else if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.field == DirectionField::Priority {
                    self.direction.priority = self.direction.priority.next();
                    self.dirty = true;
                } else {
                    let len = self.current_text().chars().count();
                    if self.cursor < len {
                        self.cursor += 1;
                    }
                }
            }
            KeyCode::Home => {
                if self.field != DirectionField::Priority {
                    self.cursor = 0;
                }
            }
            KeyCode::End => {
                if self.field != DirectionField::Priority {
                    self.cursor = self.current_text().chars().count();
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
                    self.cursor = self.current_text().chars().count();
                }
                DirectionField::Description => {
                    self.insert_char('\n');
                }
                DirectionField::Priority => {
                    self.field = self.field.next();
                    self.cursor = self.current_text().chars().count();
                }
            },
            KeyCode::Backspace => {
                self.delete_char();
            }
            KeyCode::Char(c) => {
                self.insert_char(c);
            }
            _ => {}
        }
        None
    }

    fn current_text(&self) -> &str {
        match self.field {
            DirectionField::Name => &self.direction.name,
            DirectionField::Description => &self.direction.description,
            DirectionField::Priority => "",
        }
    }

    fn current_text_mut(&mut self) -> &mut String {
        match self.field {
            DirectionField::Name => &mut self.direction.name,
            DirectionField::Description => &mut self.direction.description,
            DirectionField::Priority => panic!("приоритет не редактируется текстом"),
        }
    }

    fn insert_char(&mut self, c: char) {
        self.dirty = true;
        let cursor = self.cursor;
        let text = self.current_text_mut();
        let byte_pos = text
            .char_indices()
            .nth(cursor)
            .map(|(i, _)| i)
            .unwrap_or(text.len());
        text.insert(byte_pos, c);
        self.cursor += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.dirty = true;
        let cursor = self.cursor;
        let text = self.current_text_mut();
        let byte_pos = text
            .char_indices()
            .nth(cursor - 1)
            .map(|(i, _)| i)
            .unwrap_or(0);
        text.remove(byte_pos);
        self.cursor -= 1;
    }
    fn cursor_screen_line(&self, width: u16) -> usize {
        if width == 0 {
            return 0;
        }
        let before: String = self.current_text().chars().take(self.cursor).collect();
        let mut line = 0usize;
        let mut col = 0u16;
        for c in before.chars() {
            if c == '\n' {
                line += 1;
                col = 0;
            } else {
                if col >= width {
                    line += 1;
                    col = 0;
                }
                col += 1;
            }
        }
        line
    }

    pub fn adjust_scroll(&mut self, visible_height: u16, width: u16) {
        if width == 0 || visible_height == 0 {
            return;
        }
        let cursor_line = self.cursor_screen_line(width);
        if cursor_line < self.scroll as usize {
            self.scroll = cursor_line as u16;
        } else if cursor_line >= self.scroll as usize + visible_height as usize {
            self.scroll = (cursor_line - visible_height as usize + 1) as u16;
        }
    }
}

/// Отрисовать форму направления.
pub fn draw_direction_form(f: &mut Frame, form: &mut DirectionForm, area: Rect) {
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
            Constraint::Min(15),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(inner);
    // Пересчитать скролл активного поля.
    match form.field {
        DirectionField::Name => {
            let h = chunks[0].height.saturating_sub(2);
            let w = chunks[0].width.saturating_sub(2);
            form.adjust_scroll(h, w);
        }
        DirectionField::Description => {
            let h = chunks[1].height.saturating_sub(2);
            let w = chunks[1].width.saturating_sub(2);
            form.adjust_scroll(h, w);
        }
        DirectionField::Priority => {}
    }

    let name_focused = form.field == DirectionField::Name;
    draw_field(
        f,
        chunks[0],
        "Название",
        &form.direction.name,
        name_focused,
        if name_focused {
            Some(form.cursor)
        } else {
            None
        },
        if name_focused { form.scroll } else { 0 },
    );

    let desc_focused = form.field == DirectionField::Description;
    draw_field(
        f,
        chunks[1],
        "Описание",
        &form.direction.description,
        desc_focused,
        if desc_focused {
            Some(form.cursor)
        } else {
            None
        },
        if desc_focused { form.scroll } else { 0 },
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
            "[F2] сохранить  [Esc] отмена  [←→] курсор  [Home/End] начало/конец",
            Style::default().fg(Color::DarkGray),
        )
    };
    let p = Paragraph::new(Line::from(hint));
    f.render_widget(p, chunks[3]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::task::Priority;

    // --- TaskField ---

    #[test]
    fn test_task_field_next() {
        assert_eq!(TaskField::Title.next(), TaskField::Description);
        assert_eq!(TaskField::Description.next(), TaskField::Priority);
        assert_eq!(TaskField::Priority.next(), TaskField::Milestone);
        assert_eq!(TaskField::Milestone.next(), TaskField::Comment);
        assert_eq!(TaskField::Comment.next(), TaskField::Title);
    }

    #[test]
    fn test_task_field_prev() {
        assert_eq!(TaskField::Title.prev(), TaskField::Comment);
        assert_eq!(TaskField::Comment.prev(), TaskField::Milestone);
        assert_eq!(TaskField::Milestone.prev(), TaskField::Priority);
        assert_eq!(TaskField::Priority.prev(), TaskField::Description);
        assert_eq!(TaskField::Description.prev(), TaskField::Title);
    }

    #[test]
    fn test_task_field_roundtrip() {
        for f in [
            TaskField::Title,
            TaskField::Description,
            TaskField::Priority,
            TaskField::Milestone,
            TaskField::Comment,
        ] {
            assert_eq!(f.next().prev(), f);
            assert_eq!(f.prev().next(), f);
        }
    }

    // --- DirectionField ---

    #[test]
    fn test_direction_field_next() {
        assert_eq!(DirectionField::Name.next(), DirectionField::Description);
        assert_eq!(DirectionField::Description.next(), DirectionField::Priority);
        assert_eq!(DirectionField::Priority.next(), DirectionField::Name);
    }

    #[test]
    fn test_direction_field_prev() {
        assert_eq!(DirectionField::Name.prev(), DirectionField::Priority);
        assert_eq!(DirectionField::Priority.prev(), DirectionField::Description);
        assert_eq!(DirectionField::Description.prev(), DirectionField::Name);
    }

    #[test]
    fn test_direction_field_roundtrip() {
        for f in [
            DirectionField::Name,
            DirectionField::Description,
            DirectionField::Priority,
        ] {
            assert_eq!(f.next().prev(), f);
            assert_eq!(f.prev().next(), f);
        }
    }

    // --- TaskForm::new / edit ---

    #[test]
    fn test_task_form_new() {
        let form = TaskForm::new();
        assert!(form.is_new);
        assert_eq!(form.field, TaskField::Title);
        assert_eq!(form.cursor, 0);
        assert_eq!(form.scroll, 0);
        assert!(!form.dirty);
        assert!(form.error.is_none());
        assert_eq!(form.task.title, "");
    }

    #[test]
    fn test_task_form_edit() {
        let mut task = Task::new("Тест".to_string());
        task.priority = Priority::High;

        let form = TaskForm::edit(task.clone());
        assert!(!form.is_new);
        assert_eq!(form.field, TaskField::Title);
        assert_eq!(form.cursor, 0);
        assert!(!form.dirty);
        assert_eq!(form.task.title, "Тест");
        assert_eq!(form.task.priority, Priority::High);
    }

    // --- TaskForm::insert_char / delete_char ---

    #[test]
    fn test_task_insert_char_at_end() {
        let mut form = TaskForm::new();
        // Курсор на 0, поле Title.
        form.insert_char('A');
        form.insert_char('B');
        form.insert_char('C');

        assert_eq!(form.task.title, "ABC");
        assert_eq!(form.cursor, 3);
        assert!(form.dirty);
    }

    #[test]
    fn test_task_insert_char_in_middle() {
        let mut form = TaskForm::new();
        form.task.title = "AC".to_string();
        form.cursor = 1; // между A и C
        form.insert_char('B');

        assert_eq!(form.task.title, "ABC");
        assert_eq!(form.cursor, 2);
    }

    #[test]
    fn test_task_insert_char_at_start() {
        let mut form = TaskForm::new();
        form.task.title = "BC".to_string();
        form.cursor = 0;
        form.insert_char('A');

        assert_eq!(form.task.title, "ABC");
        assert_eq!(form.cursor, 1);
    }

    #[test]
    fn test_task_delete_char_at_end() {
        let mut form = TaskForm::new();
        form.task.title = "ABC".to_string();
        form.cursor = 3;
        form.delete_char();

        assert_eq!(form.task.title, "AB");
        assert_eq!(form.cursor, 2);
    }

    #[test]
    fn test_task_delete_char_in_middle() {
        let mut form = TaskForm::new();
        form.task.title = "ABC".to_string();
        form.cursor = 2; // после B
        form.delete_char();

        assert_eq!(form.task.title, "AC");
        assert_eq!(form.cursor, 1);
    }

    #[test]
    fn test_task_delete_char_at_start() {
        let mut form = TaskForm::new();
        form.task.title = "ABC".to_string();
        form.cursor = 0;
        form.delete_char();

        // Ничего не удаляется.
        assert_eq!(form.task.title, "ABC");
        assert_eq!(form.cursor, 0);
    }

    #[test]
    fn test_task_delete_char_empty() {
        let mut form = TaskForm::new();
        form.cursor = 0;
        form.delete_char();
        assert_eq!(form.task.title, "");
    }

    #[test]
    fn test_task_insert_cyrillic() {
        let mut form = TaskForm::new();
        form.insert_char('П');
        form.insert_char('р');
        form.insert_char('и');
        form.insert_char('в');
        form.insert_char('е');
        form.insert_char('т');

        assert_eq!(form.task.title, "Привет");
        assert_eq!(form.cursor, 6);
    }

    #[test]
    fn test_task_insert_newline() {
        let mut form = TaskForm::new();
        form.field = TaskField::Description;
        form.insert_char('A');
        form.insert_char('\n');
        form.insert_char('B');

        assert_eq!(form.task.description, "A\nB");
        assert_eq!(form.cursor, 3);
    }

    #[test]
    fn test_task_insert_into_milestone() {
        let mut form = TaskForm::new();
        form.field = TaskField::Milestone;
        form.insert_char('X');

        assert_eq!(form.task.milestone, "X");
        assert_eq!(form.task.title, "");
    }

    #[test]
    fn test_task_insert_into_comment() {
        let mut form = TaskForm::new();
        form.field = TaskField::Comment;
        form.insert_char('Y');

        assert_eq!(form.task.comment, "Y");
    }

    #[test]
    fn test_task_dirty_flag() {
        let mut form = TaskForm::new();
        assert!(!form.dirty);
        form.insert_char('A');
        assert!(form.dirty);
    }

    #[test]
    fn test_task_delete_sets_dirty() {
        let mut form = TaskForm::new();
        form.task.title = "A".to_string();
        form.cursor = 1;
        form.delete_char();
        assert!(form.dirty);
    }

    // --- DirectionForm ---

    #[test]
    fn test_direction_form_new() {
        let form = DirectionForm::new();
        assert!(form.is_new);
        assert_eq!(form.field, DirectionField::Name);
        assert_eq!(form.cursor, 0);
        assert_eq!(form.scroll, 0);
        assert!(!form.dirty);
        assert!(form.error.is_none());
        assert_eq!(form.direction.name, "");
        assert_eq!(form.original_name, "");
    }

    #[test]
    fn test_direction_form_edit() {
        let mut d = crate::model::direction::Direction::new("Здоровье".to_string());
        d.priority = Priority::High;

        let form = DirectionForm::edit(d.clone());
        assert!(!form.is_new);
        assert_eq!(form.field, DirectionField::Name);
        assert_eq!(form.cursor, 0);
        assert!(!form.dirty);
        assert_eq!(form.direction.name, "Здоровье");
        assert_eq!(form.direction.priority, Priority::High);
        assert_eq!(form.original_name, "Здоровье");
    }

    #[test]
    fn test_direction_insert_char() {
        let mut form = DirectionForm::new();
        form.insert_char('A');
        form.insert_char('B');

        assert_eq!(form.direction.name, "AB");
        assert_eq!(form.cursor, 2);
        assert!(form.dirty);
    }

    #[test]
    fn test_direction_delete_char() {
        let mut form = DirectionForm::new();
        form.direction.name = "ABC".to_string();
        form.cursor = 3;
        form.delete_char();

        assert_eq!(form.direction.name, "AB");
        assert_eq!(form.cursor, 2);
    }

    #[test]
    fn test_direction_insert_into_description() {
        let mut form = DirectionForm::new();
        form.field = DirectionField::Description;
        form.insert_char('X');

        assert_eq!(form.direction.description, "X");
        assert_eq!(form.direction.name, "");
    }

    #[test]
    fn test_direction_insert_cyrillic() {
        let mut form = DirectionForm::new();
        for c in "Здоровье".chars() {
            form.insert_char(c);
        }
        assert_eq!(form.direction.name, "Здоровье");
        assert_eq!(form.cursor, 8);
    }

    #[test]
    fn test_direction_delete_at_start() {
        let mut form = DirectionForm::new();
        form.direction.name = "ABC".to_string();
        form.cursor = 0;
        form.delete_char();

        assert_eq!(form.direction.name, "ABC");
    }
}
