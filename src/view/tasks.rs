use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::model::task::{Priority, Task};
use crate::view::tui::{App, Focus};

/// Отрисовать окошко задач.
pub fn draw_tasks(f: &mut Frame, app: &mut App, area: Rect) {
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

    let inner_width = area.width.saturating_sub(4) as usize;
    let items: Vec<ListItem> = tasks
        .iter()
        .map(|task| task_item(task, inner_width))
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

/// Один элемент задачи.
fn task_item(task: &Task, width: usize) -> ListItem<'static> {
    let done = task.is_done();
    let base_style = if done {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let priority_style = Style::default()
        .fg(priority_color(task.priority))
        .add_modifier(Modifier::BOLD);

    let title_short = truncate(&task.title, width);

    let mut lines = vec![Line::from(vec![Span::styled(title_short, base_style)])];

    let priority = task.priority.label();
    let mut meta_spans = vec![Span::styled(priority.to_string(), priority_style)];

    if !task.milestone.is_empty() {
        let used = priority.len() + 3;
        let remaining = width.saturating_sub(used);
        let milestone_short = truncate(&task.milestone, remaining);
        meta_spans.push(Span::raw(" · "));
        meta_spans.push(Span::styled(
            milestone_short,
            Style::default().fg(Color::DarkGray),
        ));
    }
    lines.push(Line::from(meta_spans));

    if done {
        let comment_short = truncate(&task.comment, width.saturating_sub(4));
        lines.push(Line::from(vec![
            Span::raw("  └ "),
            Span::styled(comment_short, base_style),
        ]));
    }

    ListItem::new(lines)
}

/// Цвет для приоритета.
pub fn priority_color(p: Priority) -> Color {
    match p {
        Priority::High => Color::Red,
        Priority::Med => Color::Blue,
        Priority::Low => Color::Green,
    }
}

/// Обрезать строку до `max` символов, добавив `…`, если обрезали.
pub fn truncate(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    if max == 0 {
        return String::new();
    }
    if max == 1 {
        return "…".to_string();
    }
    let mut result: String = s.chars().take(max - 1).collect();
    result.push('…');
    result
}
