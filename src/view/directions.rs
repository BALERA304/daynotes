use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::model::direction::Direction;
use crate::view::tasks::{priority_color, truncate};
use crate::view::tui::{App, Focus};

/// Отрисовать окошко направлений.
pub fn draw_directions(f: &mut Frame, app: &mut App, area: Rect) {
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

    let inner_width = area.width.saturating_sub(4) as usize;
    let items: Vec<ListItem> = app
        .store
        .directions
        .iter()
        .map(|d| direction_item(d, inner_width))
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
fn direction_item(d: &Direction, width: usize) -> ListItem<'static> {
    let priority_style = Style::default()
        .fg(priority_color(d.priority))
        .add_modifier(Modifier::BOLD);

    let priority = d.priority.label();
    let fixed = 2 + 2 + priority.len() + 3;
    let remaining = width.saturating_sub(fixed);

    let name_max = (remaining * 55) / 100;
    let desc_max = remaining.saturating_sub(name_max);

    let name_short = truncate(&d.name, name_max);
    let desc_clean = d.description.replace('\n', " ");
    let desc_short = truncate(&desc_clean, desc_max);

    let mut spans = vec![
        Span::raw("● "),
        Span::styled(name_short, Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(priority.to_string(), priority_style),
    ];

    if !desc_short.is_empty() {
        spans.push(Span::raw(" · "));
        spans.push(Span::styled(
            desc_short,
            Style::default().fg(Color::DarkGray),
        ));
    }

    ListItem::new(Line::from(spans))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::task::Priority;

    #[test]
    fn test_direction_new() {
        let d = Direction::new("Здоровье".to_string());
        assert_eq!(d.name, "Здоровье");
        assert_eq!(d.description, "");
        assert_eq!(d.priority, Priority::Med);
        assert_eq!(d.id.len(), 8);
    }

    #[test]
    fn test_direction_new_unique_ids() {
        let d1 = Direction::new("A".to_string());
        let d2 = Direction::new("B".to_string());
        assert_ne!(d1.id, d2.id);
    }

    #[test]
    fn test_direction_empty_name() {
        // Пустое имя — валидно на уровне структуры,
        // валидация — на уровне формы.
        let d = Direction::new("".to_string());
        assert_eq!(d.name, "");
    }

    #[test]
    fn test_direction_serde_roundtrip() {
        let mut d = Direction::new("Здоровье".to_string());
        d.description = "Регулярные тренировки, сон 8 часов".to_string();
        d.priority = Priority::High;

        let json = serde_json::to_string(&d).unwrap();
        let parsed: Direction = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, d.id);
        assert_eq!(parsed.name, d.name);
        assert_eq!(parsed.description, d.description);
        assert_eq!(parsed.priority, d.priority);
    }

    #[test]
    fn test_direction_serde_defaults() {
        // Минимальный JSON без description и priority.
        let json = r#"{"id":"abc12345","name":"Здоровье"}"#;
        let d: Direction = serde_json::from_str(json).unwrap();
        assert_eq!(d.description, "");
        assert_eq!(d.priority, Priority::Med);
    }

    #[test]
    fn test_direction_serde_multiline_description() {
        let mut d = Direction::new("Здоровье".to_string());
        d.description = "Первая строка.\nВторая строка.\nТретья.".to_string();

        let json = serde_json::to_string(&d).unwrap();
        let parsed: Direction = serde_json::from_str(&json).unwrap();

        assert!(parsed.description.contains('\n'));
        assert_eq!(parsed.description.lines().count(), 3);
    }
}

#[cfg(test)]
mod tests1 {
    use super::*;
    use crate::model::task::Priority;

    #[test]
    fn test_direction_new() {
        let d = Direction::new("Здоровье".to_string());
        assert_eq!(d.name, "Здоровье");
        assert_eq!(d.description, "");
        assert_eq!(d.priority, Priority::Med);
        assert_eq!(d.id.len(), 8);
    }

    #[test]
    fn test_direction_new_unique_ids() {
        let d1 = Direction::new("A".to_string());
        let d2 = Direction::new("B".to_string());
        assert_ne!(d1.id, d2.id);
    }

    #[test]
    fn test_direction_empty_name() {
        // Пустое имя — валидно на уровне структуры,
        // валидация — на уровне формы.
        let d = Direction::new("".to_string());
        assert_eq!(d.name, "");
    }

    #[test]
    fn test_direction_serde_roundtrip() {
        let mut d = Direction::new("Здоровье".to_string());
        d.description = "Регулярные тренировки, сон 8 часов".to_string();
        d.priority = Priority::High;

        let json = serde_json::to_string(&d).unwrap();
        let parsed: Direction = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, d.id);
        assert_eq!(parsed.name, d.name);
        assert_eq!(parsed.description, d.description);
        assert_eq!(parsed.priority, d.priority);
    }

    #[test]
    fn test_direction_serde_defaults() {
        // Минимальный JSON без description и priority.
        let json = r#"{"id":"abc12345","name":"Здоровье"}"#;
        let d: Direction = serde_json::from_str(json).unwrap();
        assert_eq!(d.description, "");
        assert_eq!(d.priority, Priority::Med);
    }

    #[test]
    fn test_direction_serde_multiline_description() {
        let mut d = Direction::new("Здоровье".to_string());
        d.description = "Первая строка.\nВторая строка.\nТретья.".to_string();

        let json = serde_json::to_string(&d).unwrap();
        let parsed: Direction = serde_json::from_str(&json).unwrap();

        assert!(parsed.description.contains('\n'));
        assert_eq!(parsed.description.lines().count(), 3);
    }
}
