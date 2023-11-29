use crate::application::action::Action;
use crate::application::event::Event;
use crate::application::tui::components::Component;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph};
use std::io;

use super::message::{Message, Sender};

pub struct Chat {
    messages: Vec<Message>,
    input_buffer: String,
}

impl Chat {
    pub fn new() -> io::Result<Self> {
        let messages: Vec<Message> = vec![];
        let input_buffer = String::new();
        Ok(Self {
            messages,
            input_buffer,
        })
    }

    fn get_list_widget(&mut self) -> List<'_> {
        let block = Block::default()
            .title(String::from("StegoP2P"))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default())
            .border_type(BorderType::Plain);

        let mut items: Vec<ListItem> = vec![];
        for message in &self.messages {
            let list_item = ListItem::new(message.to_string());
            items.push(list_item);
        }
        List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>")
            .style(Style::default().fg(Color::White))
    }

    fn get_input_widget(&mut self) -> Paragraph<'_> {
        Paragraph::new(self.input_buffer.clone())
            .block(
                Block::default()
                    .title("Your message: ")
                    .title_alignment(Alignment::Left)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::LightYellow))
                    .border_type(BorderType::Plain),
            )
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left)
    }

    fn handle_new_message(&mut self, message: Message) -> io::Result<Option<Action>> {
        self.messages.push(message);

        Ok(None)
    }
}

impl Component for Chat {
    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> io::Result<()> {
        let rects = Layout::default()
            .constraints([Constraint::Percentage(100), Constraint::Min(3)].as_ref())
            .split(rect);

        f.render_widget(self.get_list_widget(), rects[0]);

        f.render_widget(self.get_input_widget(), rects[1]);

        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> io::Result<Option<Action>> {
        if let KeyEventKind::Press = key.kind {
            match key.code {
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }
                KeyCode::Enter => {
                    let str = self.input_buffer.clone();
                    self.input_buffer.clear();
                    return Ok(Some(Action::HandleOutgoing(str)));
                }
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                }
                _ => {}
            }
        }

        Ok(None)
    }

    fn handle_events(&mut self, event: Event) -> io::Result<Option<Action>> {
        let r = match event {
            Event::Key(key_event) => self.handle_key_events(key_event)?,
            _ => None,
        };
        Ok(r)
    }

    fn update(&mut self, action: Action) -> io::Result<Option<Action>> {
        match action {
            Action::HandleOutgoing(str) => {
                self.handle_new_message(Message::new(str, Sender::Outgoing))?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}
