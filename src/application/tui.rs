use crate::application::tui::components::chat::Chat;
use crate::application::tui::components::Component;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use std::io;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};
use terminal_manager::TerminalManger;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::sync::watch;

use super::action::Action;
use super::event::Event;

pub mod components;
pub mod terminal_manager;

type ArcMutex<T> = Arc<Mutex<T>>;

macro_rules! new_arc_mutex {
    ($e:expr) => {
        Arc::new(Mutex::new(($e)))
    };
}

type BoxVec<T> = Vec<Box<T>>;
type DynComponent = dyn Component + Sync + Send;

#[derive(Debug, PartialEq)]
enum TuiStatus {
    Running,
    Stopped,
    ShouldStop,
}

pub struct Tui {
    rt_handle: Handle,
    action_snd: UnboundedSender<Action>,
    action_rcv: UnboundedReceiver<Action>,
    event_snd: UnboundedSender<Event>,
    event_rcv: UnboundedReceiver<Event>,
    shutdown_watch_snd: watch::Sender<bool>,
    arc_terminal_manager: ArcMutex<TerminalManger>,
    views: ArcMutex<BoxVec<DynComponent>>,
    app_state: AppState,
}

#[derive(Debug)]
struct AppState {
    status: TuiStatus,
}

impl Tui {
    pub fn new(rt_handle: Handle) -> io::Result<Self> {
        let (action_snd, action_rcv) = mpsc::unbounded_channel::<Action>();
        let (event_snd, event_rcv) = mpsc::unbounded_channel::<Event>();
        let (shutdown_watch_snd, shutdown_watch_rcv) = watch::channel(false);
        let arc_terminal_manager = new_arc_mutex!(TerminalManger::new(
            event_snd.clone(),
            shutdown_watch_rcv.clone(),
            60usize,
            60usize
        )?);
        let views: ArcMutex<BoxVec<DynComponent>> =
            new_arc_mutex!((vec![Box::new(Chat::new()?)]));

        let app_state = AppState {
            status: TuiStatus::Stopped,
        };

        Ok(Self {
            rt_handle,
            action_snd,
            action_rcv,
            event_snd,
            event_rcv,
            shutdown_watch_snd,
            arc_terminal_manager,
            views,
            app_state,
        })
    }

    pub async fn run(mut self) -> io::Result<()> {
        // Start terminal manager task
        self.rt_handle
            .spawn(TerminalManger::start(self.arc_terminal_manager.clone()));

        loop {
            // Check for events
            let event_opt = self.event_rcv.recv().await;
            if let Some(event) = event_opt {
                match event.clone() {
                    Event::TerminalManagerInitialized => {
                        let _ = self.action_snd.send(Action::Start);
                    }
                    Event::Error(message) => {
                        let _ = self.action_snd.send(Action::Error(message));
                    }
                    Event::Tick => {
                        let _ = self.action_snd.send(Action::Tick);
                    }
                    Event::Render => {
                        let _ = self.action_snd.send(Action::Render);
                    }
                    Event::Key(k) => {
                        if let KeyEventKind::Press = k.kind {
                            match k.code {
                                // KeyCode::Backspace => todo!(),
                                // KeyCode::Enter => todo!(),
                                // KeyCode::Left => todo!(),
                                // KeyCode::Right => todo!(),
                                // KeyCode::Up => todo!(),
                                // KeyCode::Down => todo!(),
                                // KeyCode::Home => todo!(),
                                // KeyCode::End => todo!(),
                                // KeyCode::PageUp => todo!(),
                                // KeyCode::PageDown => todo!(),
                                // KeyCode::Tab => todo!(),
                                // KeyCode::BackTab => todo!(),
                                // KeyCode::Delete => todo!(),
                                // KeyCode::Insert => todo!(),
                                // KeyCode::F(_) => todo!(),
                                // Char(c) => todo!(),
                                // KeyCode::Null => todo!(),
                                KeyCode::Esc => {
                                    let _ = self.action_snd.send(Action::Stop);
                                }
                                // KeyCode::CapsLock => todo!(),
                                // KeyCode::ScrollLock => todo!(),
                                // KeyCode::NumLock => todo!(),
                                // KeyCode::PrintScreen => todo!(),
                                // KeyCode::Pause => todo!(),
                                // KeyCode::Menu => todo!(),
                                // KeyCode::KeypadBegin => todo!(),
                                // KeyCode::Media(_) => todo!(),
                                // KeyCode::Modifier(_) => todo!(),
                                _ => {}
                            }
                        }
                    }
                    Event::CapturedInput(str) => {
                        let _ = self.action_snd.send(Action::HandleOutgoing(str));
                    }
                    _ => {}
                }
                
                self.handle_component_events(event)?;
            }

            // Execute Actions
            while let Ok(action) = self.action_rcv.try_recv() {
                match action.clone() {
                    Action::Start => self.app_state.status = TuiStatus::Running,
                    Action::Tick => {}
                    Action::Render => {
                        self.render()?;
                    }
                    Action::Stop => self.app_state.status = TuiStatus::ShouldStop,
                    Action::Error(_) => {}
                    Action::HandleOutgoing(_) => {
                        // send message
                    }
                }
                self.update_components(action)?;
            }

            if TuiStatus::ShouldStop == self.app_state.status {
                break;
            }
        }

        self.stop();

        Ok(())
    }

    pub fn stop(&mut self) {
        let _ = self.shutdown_watch_snd.send(true);

        self.app_state.status = TuiStatus::Stopped;
    }

    fn render(&mut self) -> io::Result<()> {
        if let Ok(mut term_mgr) = self.arc_terminal_manager.lock() {
            let component_mtx = self.views.clone();
            term_mgr.draw(move |frame| {
                if let Ok(mut components) = component_mtx.lock() {
                    for component in components.iter_mut() {
                        let _ = component.draw(frame, frame.size());
                    }
                }
            })?;
        } else {
            return Err(io::Error::new(
                ErrorKind::Other,
                "Could not unlock terminal manager mutex",
            ));
        };

        Ok(())
    }

    fn handle_component_events(&mut self, event: Event) -> io::Result<()> {
        if let Ok(mut views) = self.views.lock() {
            for view in views.iter_mut() {
                if let Some(side_effect) = view.handle_events(event.clone())? {
                    let _ = self.action_snd.send(side_effect);
                }
            }
        } else {
            return Err(io::Error::new(
                ErrorKind::Other,
                "Could not unlock terminal manager mutex",
            ));
        };

        Ok(())
    }

    fn update_components(&mut self, action: Action) -> io::Result<()> {
        if let Ok(mut views) = self.views.lock() {
            for view in views.iter_mut() {
                if let Some(side_effect) = view.update(action.clone())? {
                    let _ = self.action_snd.send(side_effect);
                }
            }
        } else {
            return Err(io::Error::new(
                ErrorKind::Other,
                "Could not unlock terminal manager mutex",
            ));
        };

        Ok(())
    }
}
