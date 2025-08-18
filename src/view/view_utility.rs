use ratatui::{
    symbols::{self},
    widgets::List,
};

use super::*;

pub(crate) fn render_progress(
    progress: &Duration,
    area: Rect,
    buf: &mut Buffer,
    title: Block,
    duration: f64,
) {
    let progress = progress.as_secs_f64();
    let ratio = ((progress / duration) * 100.0).round() / 100.0;
    if ratio > 1.0 {
        return;
    }

    LineGauge::default()
        .block(title)
        .filled_style(Color::Green)
        .ratio(ratio)
        .label("")
        .line_set(symbols::line::THICK)
        .render(area, buf);
}

pub(crate) fn title_block<'a>(color: &'a Color, progress: &'a str) -> Block<'a> {
    let progress = Line::from(vec![Span::styled(
        progress,
        Style::default().fg(Color::Green),
    )]);

    Block::new()
        .borders(Borders::BOTTOM)
        .padding(Padding {
            left: (0),
            right: (1),
            top: (2),
            bottom: (1),
        })
        .title_bottom(progress)
        .title_alignment(ratatui::layout::Alignment::Right)
        .fg(*color)
        .bg(Color::DarkGray)
}

pub(crate) fn create_table(tracks: &[Audio]) -> Table<'_> {
    let header = Row::new(["Song", "Artist"])
        .style(Style::new().bold())
        .bottom_margin(1);

    //TODO: Refactor.
    let rows: Vec<Row> = tracks
        .iter()
        .map(|item| {
            let style = match item.is_playing {
                true => Style::default()
                    .fg(CUSTOM_LABEL_COLOR)
                    .add_modifier(Modifier::BOLD),
                _ => Style::default(),
            };

            Row::new([item.name.clone(), item.author.clone()]).style(style)
        })
        .collect();

    //let footer = Row::new(["Lemon", "Lemon Tree", "000"]);

    let widths = [Constraint::Percentage(50), Constraint::Fill(1)];

    Table::new(rows, widths)
        //.footer(footer.italic())
        //.style(Color::White)
        //.row_highlight_style(Style::new().on_black().bold())
        //.column_highlight_style(Color::Gray)
        //.cell_highlight_style(Style::new().reversed().yellow())
        .header(header)
        .column_spacing(1)
        .row_highlight_style(Style::new().fg(Color::Green))
        .highlight_symbol(">")
}

pub(crate) fn create_list(rows: Vec<Span>, highlight: Style) -> List {
    List::new(rows)
        .highlight_style(highlight)
        .highlight_symbol("  ")
        .repeat_highlight_symbol(true)
}

pub(crate) fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
