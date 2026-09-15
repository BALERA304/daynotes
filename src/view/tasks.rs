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

#[cfg(test)]
mod tests {
    use super::*;

    // --- Priority ---

    #[test]
    fn test_priority_weight() {
        assert_eq!(Priority::High.weight(), 3);
        assert_eq!(Priority::Med.weight(), 2);
        assert_eq!(Priority::Low.weight(), 1);
    }

    #[test]
    fn test_priority_label() {
        assert_eq!(Priority::High.label(), "high");
        assert_eq!(Priority::Med.label(), "med");
        assert_eq!(Priority::Low.label(), "low");
    }

    #[test]
    fn test_priority_next() {
        assert_eq!(Priority::Low.next(), Priority::Med);
        assert_eq!(Priority::Med.next(), Priority::High);
        assert_eq!(Priority::High.next(), Priority::Low);
    }

    #[test]
    fn test_priority_prev() {
        assert_eq!(Priority::Low.prev(), Priority::High);
        assert_eq!(Priority::High.prev(), Priority::Med);
        assert_eq!(Priority::Med.prev(), Priority::Low);
    }

    #[test]
    fn test_priority_next_prev_roundtrip() {
        for p in [Priority::Low, Priority::Med, Priority::High] {
            assert_eq!(p.next().prev(), p);
            assert_eq!(p.prev().next(), p);
        }
    }

    #[test]
    fn test_priority_default() {
        assert_eq!(Priority::default(), Priority::Med);
    }

    #[test]
    fn test_priority_serde_roundtrip() {
        for p in [Priority::Low, Priority::Med, Priority::High] {
            let json = serde_json::to_string(&p).unwrap();
            let parsed: Priority = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, p);
        }
    }

    #[test]
    fn test_priority_serde_lowercase() {
        let json = serde_json::to_string(&Priority::High).unwrap();
        assert_eq!(json, "\"high\"");
    }

    // --- Task ---

    #[test]
    fn test_task_new() {
        let task = Task::new("Тест".to_string());
        assert_eq!(task.title, "Тест");
        assert_eq!(task.description, "");
        assert_eq!(task.priority, Priority::Med);
        assert_eq!(task.milestone, "");
        assert_eq!(task.comment, "");
        assert_eq!(task.id.len(), 8);
    }

    #[test]
    fn test_task_new_unique_ids() {
        let t1 = Task::new("A".to_string());
        let t2 = Task::new("B".to_string());
        assert_ne!(t1.id, t2.id);
    }

    #[test]
    fn test_task_is_done_empty() {
        let task = Task::new("Тест".to_string());
        assert!(!task.is_done());
    }

    #[test]
    fn test_task_is_done_with_comment() {
        let mut task = Task::new("Тест".to_string());
        task.comment = "Сделал".to_string();
        assert!(task.is_done());
    }

    #[test]
    fn test_task_is_done_whitespace_only() {
        let mut task = Task::new("Тест".to_string());
        task.comment = "   ".to_string();
        assert!(!task.is_done());
    }

    #[test]
    fn test_task_is_done_newlines_only() {
        let mut task = Task::new("Тест".to_string());
        task.comment = "\n\n".to_string();
        assert!(!task.is_done());
    }

    #[test]
    fn test_task_serde_roundtrip() {
        let mut task = Task::new("Прочитать книгу".to_string());
        task.description = "Глава 1".to_string();
        task.priority = Priority::High;
        task.milestone = "Изучить Rust".to_string();
        task.comment = "Готово".to_string();

        let json = serde_json::to_string(&task).unwrap();
        let parsed: Task = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, task.id);
        assert_eq!(parsed.title, task.title);
        assert_eq!(parsed.description, task.description);
        assert_eq!(parsed.priority, task.priority);
        assert_eq!(parsed.milestone, task.milestone);
        assert_eq!(parsed.comment, task.comment);
    }

    #[test]
    fn test_task_serde_defaults() {
        let json = r#"{"id":"abc12345","title":"Тест"}"#;
        let task: Task = serde_json::from_str(json).unwrap();
        assert_eq!(task.description, "");
        assert_eq!(task.priority, Priority::Med);
        assert_eq!(task.milestone, "");
        assert_eq!(task.comment, "");
    }

    // --- truncate ---

    #[test]
    fn test_truncate_shorter() {
        assert_eq!(truncate("abc", 10), "abc");
    }

    #[test]
    fn test_truncate_equal() {
        assert_eq!(truncate("abc", 3), "abc");
    }

    #[test]
    fn test_truncate_longer() {
        assert_eq!(truncate("abcdef", 4), "abc…");
    }

    #[test]
    fn test_truncate_max_zero() {
        assert_eq!(truncate("abc", 0), "");
    }

    #[test]
    fn test_truncate_max_one() {
        assert_eq!(truncate("abcdef", 1), "…");
    }

    #[test]
    fn test_truncate_max_two() {
        assert_eq!(truncate("abcdef", 2), "a…");
    }

    #[test]
    fn test_truncate_empty() {
        assert_eq!(truncate("", 10), "");
        assert_eq!(truncate("", 0), "");
    }

    #[test]
    fn test_truncate_cyrillic() {
        assert_eq!(truncate("Привет", 4), "При…");
    }

    #[test]
    fn test_truncate_cyrillic_equal() {
        assert_eq!(truncate("Привет", 6), "Привет");
    }

    // --- priority_color ---

    #[test]
    fn test_priority_color_high() {
        assert_eq!(priority_color(Priority::High), Color::Red);
    }

    #[test]
    fn test_priority_color_med() {
        assert_eq!(priority_color(Priority::Med), Color::Blue);
    }

    #[test]
    fn test_priority_color_low() {
        assert_eq!(priority_color(Priority::Low), Color::Green);
    }
}
