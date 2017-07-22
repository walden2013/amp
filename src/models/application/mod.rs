pub mod modes;
mod clipboard;
mod preferences;

// Published API
pub use self::clipboard::ClipboardContent;
pub use self::preferences::Preferences;

use errors::*;
use std::env;
use std::path::Path;
use std::cell::RefCell;
use std::rc::Rc;
use input::{self, KeyMap};
use presenters;
use self::modes::*;
use scribe::{Buffer, Workspace};
use view::{self, StatusLineData, View};
use self::clipboard::Clipboard;
use git2::Repository;

pub enum Mode {
    Confirm(ConfirmMode),
    Command(CommandMode),
    Exit,
    Insert,
    Jump(JumpMode),
    LineJump(LineJumpMode),
    Normal,
    Open(OpenMode),
    Select(SelectMode),
    SelectLine(SelectLineMode),
    SearchInsert(SearchInsertMode),
    SymbolJump(SymbolJumpMode),
    Theme(ThemeMode),
}

pub struct Application {
    pub mode: Mode,
    pub workspace: Workspace,
    pub search_query: Option<String>,
    pub view: View,
    pub clipboard: Clipboard,
    pub repository: Option<Repository>,
    pub error: Option<Error>,
    pub preferences: Rc<RefCell<Preferences>>,
    key_map: KeyMap,
}

impl Application {
    pub fn new() -> Result<Application> {
        let current_dir = env::current_dir()?;

        // TODO: Log errors to disk.
        let preferences =
            Rc::new(RefCell::new(Preferences::load().unwrap_or_else(|_| Preferences::new(None))));

        // Set up a workspace in the current directory.
        let mut workspace = Workspace::new(&current_dir)?;

        // Try to open the specified file.
        // TODO: Handle non-existent files as new empty buffers.
        for path_arg in env::args().skip(1) {
            let path = Path::new(&path_arg);

            let argument_buffer = if path.exists() {
                // Load the buffer from disk.
                Buffer::from_file(path)?
            } else {
                // Build an empty buffer.
                let mut buffer = Buffer::new();

                // Point the buffer to the path, ensuring that it's absolute.
                if path.is_absolute() {
                    buffer.path = Some(path.to_path_buf());
                } else {
                    buffer.path = Some(workspace.path.join(path));
                }

                buffer
            };
            workspace.add_buffer(argument_buffer);
        }

        let view = View::new(preferences.clone())?;
        let clipboard = Clipboard::new();
        let mut key_map = KeyMap::default()?;

        // Merge user-defined keymaps into defaults.
        preferences.borrow().key_map().map(|user_defined_key_map_data| {
            KeyMap::from(user_defined_key_map_data).map(|user_defined_key_map| {
                key_map.merge(user_defined_key_map);
            })
        });

        Ok(Application {
               mode: Mode::Normal,
               workspace: workspace,
               search_query: None,
               view: view,
               clipboard: clipboard,
               repository: Repository::discover(&current_dir).ok(),
               error: None,
               preferences: preferences,
               key_map: key_map,
           })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            // Present the application state to the view.
            match self.mode {
                Mode::Confirm(_) => {
                    presenters::modes::confirm::display(&mut self.workspace,
                                                        &mut self.view)
                },
                Mode::Command(ref mut mode) => {
                    presenters::modes::search_select::display(&mut self.workspace,
                                                              mode,
                                                              &mut self.view)
                }
                Mode::Insert => {
                    presenters::modes::insert::display(&mut self.workspace,
                                                       &mut self.view)
                }
                Mode::Open(ref mut mode) => {
                    presenters::modes::search_select::display(&mut self.workspace,
                                                              mode,
                                                              &mut self.view)
                }
                Mode::SearchInsert(ref mode) => {
                    presenters::modes::search_insert::display(&mut self.workspace,
                                                              mode,
                                                              &mut self.view)
                }
                Mode::Jump(ref mut mode) => {
                    presenters::modes::jump::display(&mut self.workspace,
                                                     mode,
                                                     &mut self.view)
                }
                Mode::LineJump(ref mode) => {
                    presenters::modes::line_jump::display(&mut self.workspace,
                                                          mode,
                                                          &mut self.view)
                }
                Mode::SymbolJump(ref mut mode) => {
                    presenters::modes::search_select::display(&mut self.workspace,
                                                              mode,
                                                              &mut self.view)
                }
                Mode::Select(ref mode) => {
                    presenters::modes::select::display(&mut self.workspace,
                                                       mode,
                                                       &mut self.view)
                }
                Mode::SelectLine(ref mode) => {
                    presenters::modes::select_line::display(&mut self.workspace,
                                                            mode,
                                                            &mut self.view)
                }
                Mode::Normal => {
                    presenters::modes::normal::display(&mut self.workspace,
                                                       &mut self.view,
                                                       &self.repository)
                }
                Mode::Theme(ref mut mode) => {
                    presenters::modes::search_select::display(&mut self.workspace,
                                                              mode,
                                                              &mut self.view)
                }
                Mode::Exit => ()
            }

            // Display an error from previous command invocation, if one exists.
            if let Some(ref error) = self.error {
                self
                    .view
                    .draw_status_line(
                        &vec![StatusLineData{
                            content: error.description().to_string(),
                            style: view::Style::Bold,
                            colors: view::Colors::Warning,
                        }]
                    );
                self.view.present();
            }

            // Listen for and respond to user input.
            let command = self.view.listen().and_then(|key| {

                Application::mode_str(&self).and_then(|mode| {
                    self.key_map.command_for(&mode, &key)
                })
            });

            if let Some(com) = command {
                // Run the command and store its error output.
                self.error = com(self).err();
            }

            // Check if the command resulted in an exit, before
            // looping again and asking for input we won't use.
            if let Mode::Exit = self.mode {
                self.view.clear();
                break
            }
        }

        Ok(())
    }

    fn mode_str(application: &Application) -> Option<&'static str> {
        match application.mode {
            Mode::Command(ref mode) => if mode.insert_mode() {
                Some("search_select_insert")
            } else {
                Some("search_select")
            },
            Mode::SymbolJump(ref mode) => if mode.insert_mode() {
                Some("search_select_insert")
            } else {
                Some("search_select")
            },
            Mode::Open(ref mode) => if mode.insert_mode() {
                Some("search_select_insert")
            } else {
                Some("search_select")
            },
            Mode::Theme(ref mode) => if mode.insert_mode() {
                Some("search_select_insert")
            } else {
                Some("search_select")
            },
            Mode::Normal => Some("normal"),
            Mode::Confirm(_) => Some("confirm"),
            Mode::Insert => Some("insert"),
            Mode::Jump(_) => Some("jump"),
            Mode::LineJump(_) => Some("line_jump"),
            Mode::Select(_) => Some("select"),
            Mode::SelectLine(_) => Some("select_line"),
            Mode::SearchInsert(_) => Some("search_insert"),
            Mode::Exit => None,
        }
    }
}
