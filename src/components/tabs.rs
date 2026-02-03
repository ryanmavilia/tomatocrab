//! Tab navigation widget for switching between views

use ratatui::{
    layout::Rect,
    style::Modifier,
    text::Line,
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::app::View;
use crate::theme::{Theme, BORDER, PRIMARY, TEXT_MUTED};

/// Tab bar titles
const TAB_TITLES: [&str; 3] = ["Timer", "History", "Stats"];

/// Widget for displaying the tab bar
pub struct TabsWidget {
    current: View,
}

impl TabsWidget {
    pub fn new(current: View) -> Self {
        Self { current }
    }

    /// Render the tabs widget
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let titles: Vec<Line> = TAB_TITLES
            .iter()
            .enumerate()
            .map(|(i, t)| {
                if i == self.current.index() {
                    Line::styled(*t, Theme::tab_active())
                } else {
                    Line::styled(*t, Theme::tab_inactive())
                }
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(ratatui::style::Style::default().fg(BORDER))
                    .title(" TOMATOCRAB ")
                    .title_style(
                        ratatui::style::Style::default()
                            .fg(PRIMARY)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .select(self.current.index())
            .style(ratatui::style::Style::default().fg(TEXT_MUTED))
            .highlight_style(Theme::tab_active())
            .divider(" | ");

        frame.render_widget(tabs, area);
    }
}
