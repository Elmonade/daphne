use crate::Audio;
use crate::PlayerState;
use crate::SinkState;
use crate::order::Order;
use color_eyre::owo_colors::OwoColorize;
use number_drawer::NumberDrawer;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Flex;
use ratatui::layout::Rect;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::palette::tailwind;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::LineGauge;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::widgets::{Block, BorderType, Padding};
use ratatui::widgets::{Row, Table};
use std::time::Duration;
mod number_drawer;
mod view_utility;

const CUSTOM_LABEL_COLOR: Color = tailwind::SKY.c200;
const BY_COLOR: Color = tailwind::RED.c300;
const GAUGE_COLOR: Color = tailwind::GREEN.c800;
const PAUSED_COLOR: Color = Color::Gray;

pub(crate) fn render(frame: &mut Frame, state: &PlayerState, sink: &SinkState) {
    let settings = Block::default()
        .fg(Color::Green)
        .padding(Padding::uniform(4))
        .title("PLAYBACK ORDER")
        .borders(Borders::TOP | Borders::BOTTOM);

    if state.is_choosing {
        // List seciton
        let left_top_block = Block::default().borders(Borders::NONE).fg(Color::Yellow);
        frame.render_widget(left_top_block, frame.area());

        let table = view_utility::create_table(&state.tracks);
        let mut table_state = state.table_state.clone();
        frame.render_stateful_widget(table, frame.area(), &mut table_state);
    } else {
        // Player Section
        let [top, bottom] = Layout::vertical([Constraint::Length(6), Constraint::Fill(1)])
            .vertical_margin(1)
            .flex(ratatui::layout::Flex::Center)
            .areas(frame.area());
        let player_color = match sink.is_playing {
            true => Color::Yellow,
            false => Color::Magenta,
        };

        let mut index = 1; // Point at something on startup.
        if let Some(current_index) = state.current_track_index {
            index = current_index;
        }
        if let Some(music) = state.tracks.get(index) {
            let progress_label = format!(" {}/{}", sink.position.as_secs(), music.length);
            let progress_block = view_utility::title_block(&player_color, &progress_label);
            view_utility::render_progress(
                &sink.position,
                top,
                frame.buffer_mut(),
                progress_block,
                music.length as f64,
            );

            let name = Line::from(vec![Span::styled(
                &music.name,
                Style::default().fg(player_color),
            )])
            .right_aligned();
            let author = Line::from(vec![
                Span::styled(&music.author, Style::default().fg(Color::Green)),
            ])
            .right_aligned();

            let info_para = Paragraph::new(name)
                .wrap(ratatui::widgets::Wrap { trim: true })
                .alignment(ratatui::layout::Alignment::Left)
                .block(
                    Block::default()
                        .borders(Borders::all())
                        .padding(Padding::top(2))
                        .style(Style::default().fg(player_color))
                        .title_bottom(author),
                );
            frame.render_widget(info_para, bottom);
        }
    }

    /*
        // Config Section
        let highlight = if state.is_configuring {
            Style::new().reversed()
        } else {
            Style::new()
        };

        let options = [
            Order::Shuffle.to_string(),
            Order::Album.to_string(),
            Order::Artist.to_string(),
            Order::Track.to_string(),
        ];

        // TODO: This should be inside view_utility.
        let rows: Vec<Span> = options
            .iter()
            .map(|item| {
                let style = match *item == state.playback_order.to_string() {
                    true => Style::default().add_modifier(Modifier::UNDERLINED),
                    _ => Style::default(),
                };

                Span::from(item).style(style)
            })
            .collect();

        let list = view_utility::create_list(rows, highlight);
        let mut list_state = state.list_state.clone();
        frame.render_stateful_widget(list.block(settings), right, &mut list_state);

        // Search Section
        if state.is_searching {
            frame.render_widget(Clear, right);
            Paragraph::new(state.keyword.as_str())
                .block(
                    Block::bordered()
                        .fg(Color::Green)
                        .border_type(BorderType::Rounded)
                        .padding(Padding::uniform(1))
                        .title("SEARCH"),
                )
                .render(right, frame.buffer_mut());
        }

    */

    // Volume Section
    if state.is_adjusting {
        let volume = (sink.volume * 10.0) as u32;
        let mut string_volume = volume.to_string();
        if volume < 10 {
            string_volume = format!("0{volume}")
        }

        let enlarged_volume = NumberDrawer::draw(&string_volume);

        let centered_area = view_utility::center(
            frame.area(),
            Constraint::Percentage(100),
            Constraint::Percentage(100),
        );

        let mut spacer = 0;
        if centered_area.width > 10 && centered_area.height > 10 {
            spacer = 5;
        }

        let volume_paragraph = Paragraph::new(enlarged_volume)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::new().borders(Borders::NONE).padding(Padding::new(
                centered_area.width / 2 - spacer,
                0,
                centered_area.height / 2 - spacer + 1,
                0,
            )));

        frame.render_widget(Clear, frame.area());
        frame.render_widget(volume_paragraph, centered_area);
    }
}
