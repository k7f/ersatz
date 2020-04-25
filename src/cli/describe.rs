use std::{str::FromStr, path::PathBuf, error::Error};
use crate::Ersatz;
use super::{App, Command};

#[derive(Debug)]
pub struct Describe {
    ersatz:       Ersatz,
    main_path:    String,
    trigger_name: Option<String>,
    verbosity:    u64,
}

impl Describe {
    pub(crate) fn new(app: &mut App) -> Self {
        let mut ersatz = Ersatz::new();
        let main_path = app.value_of("MAIN_PATH").unwrap_or_else(|| unreachable!()).into();
        let trigger_name = app.value_of("TRIGGER").map(Into::into);
        let verbosity = app.occurrences_of("verbose").max(app.occurrences_of("log"));

        app.apply_props(&mut ersatz);
        app.accept_selectors(&["TRIGGER", "MAX_STEPS"]);

        Describe { ersatz, main_path, trigger_name, verbosity }
    }

    pub fn new_command(app: &mut App) -> Box<dyn Command> {
        Box::new(Self::new(app))
    }
}

impl Command for Describe {
    fn name_of_log_file(&self) -> String {
        if let Ok(mut path) = PathBuf::from_str(&self.main_path) {
            if path.set_extension("log") {
                if let Some(file_name) = path.file_name() {
                    return file_name.to_str().unwrap().to_owned()
                } else {
                }
            } else {
            }
        } else {
        }

        "ersatz.log".to_owned()
    }

    fn console_level(&self) -> Option<log::LevelFilter> {
        Some(match self.verbosity {
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        })
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        debug!("{:?}", self);
        info!("Using script \"{}\"", self.main_path);

        self.ersatz.add_from_file(self.main_path.as_str())?;
        println!("{:?}", self.ersatz);

        Ok(())
    }
}
