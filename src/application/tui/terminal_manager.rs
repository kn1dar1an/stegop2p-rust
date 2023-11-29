use crate::application::event::Event;
use crossterm::event::{Event as CTEvent, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::{cursor, execute};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::{Frame, Terminal};
use std::io;
use std::io::Result;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::watch;

use super::ArcMutex;

pub struct TerminalManger {
    event_snd: mpsc::UnboundedSender<Event>,
    shutdown_watch_rcv: watch::Receiver<bool>,
    terminal: Terminal<CrosstermBackend<io::Stderr>>,
    tick_rate: Duration,
    render_rate: Duration,
}

impl TerminalManger {
    pub fn new(
        event_snd: mpsc::UnboundedSender<Event>,
        shutdown_watch_rcv: watch::Receiver<bool>,
        tick_rate: usize,
        render_rate: usize,
    ) -> Result<Self> {
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stderr()))?;
        terminal.clear()?;

        let tick_rate = Duration::from_secs_f64(1.0 / tick_rate as f64);
        let render_rate = Duration::from_secs_f64(1.0 / render_rate as f64);

        Ok(Self {
            event_snd,
            shutdown_watch_rcv,
            terminal,
            tick_rate,
            render_rate,
        })
    }

    pub fn draw<F>(&mut self, render_closure: F) -> Result<()>
    where
        F: Fn(&mut Frame),
    {
        self.terminal.draw(render_closure)?;
        Ok(())
    }

    pub async fn start(arc_self: ArcMutex<Self>) -> Result<()> {
        Self::enter()?;

        let mut term_eventstream = crossterm::event::EventStream::new();
        let mut t_interval;
        let mut r_interval;
        let event_snd;
        let mut shutdown_watch_rcv;

        match arc_self.lock() {
            Ok(self_guard) => {
                t_interval = tokio::time::interval(self_guard.tick_rate);
                r_interval = tokio::time::interval(self_guard.render_rate);
                event_snd = self_guard.event_snd.clone();
                shutdown_watch_rcv = self_guard.shutdown_watch_rcv.clone();
            },
            Err(err) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Could not unlock terminal manager mutex: {}", err)
                ));
            },
        }

        event_snd.send(Event::TerminalManagerInitialized).unwrap();

        loop {
            let t_instant = t_interval.tick();
            let r_instant = r_interval.tick();

            let _ = tokio::select! {
                shutdown_signal = shutdown_watch_rcv.changed() => {
                    match shutdown_signal {
                        Ok(_) => {
                            if *shutdown_watch_rcv.borrow_and_update() {
                                break;
                            }
                        },
                        Err(err) => { return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("Could not recv shutdown signal: {}", err)
                        )); }
                    }
                },
                term_event = term_eventstream.next() => {
                    match term_event {
                        Some(Ok(event)) => {
                            if let CTEvent::Key(key) = event {
                                if key.kind == KeyEventKind::Press {
                                    let _ = event_snd.send(Event::Key(key));
                                }
                            }
                        },
                        Some(Err(err)) => return Err(err),
                        None => {},
                    }
                },
                _ = t_instant => {
                    event_snd.send(Event::Tick).unwrap();
                },
                _ = r_instant => {
                    event_snd.send(Event::Render).unwrap();
                },
            };
        }

        Self::exit()?;
        Ok(())
    }

    fn enter() -> Result<()> {
        enable_raw_mode()?;
        execute!(std::io::stderr(), EnterAlternateScreen, cursor::Show)?;
        Ok(())
    }

    fn exit() -> Result<()> {
        if is_raw_mode_enabled()? {
            execute!(std::io::stderr(), LeaveAlternateScreen, cursor::Show)?;
            disable_raw_mode()?;
        }
        Ok(())
    }
}
