use std::{path::PathBuf, error::Error};
use crate::Ersatz;
use super::{App, Command, AppError};

#[derive(Debug)]
pub struct Validate {
    glob_path:    String,
    do_abort:     bool,
    syntax_only:  bool,
    is_recursive: bool,
    verbosity:    u64,
}

impl Validate {
    pub(crate) fn new(app: &App) -> Self {
        let glob_path = app.value_of("GLOB_PATH").unwrap_or_else(|| unreachable!()).to_owned();
        let do_abort = app.is_present("abort");
        let syntax_only = app.is_present("syntax");
        let is_recursive = app.is_present("recursive");
        let verbosity = app.occurrences_of("verbose").max(app.occurrences_of("log"));

        Validate { glob_path, do_abort, syntax_only, is_recursive, verbosity }
    }

    pub fn new_command(app: &App) -> Box<dyn Command> {
        Box::new(Self::new(app))
    }
}

impl Command for Validate {
    fn name_of_log_file(&self) -> String {
        "ersatz-validation.log".to_owned()
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        debug!("{:?}", self);
        let mut glob_path = PathBuf::from(&self.glob_path);

        if self.is_recursive {
            glob_path.push("**");
        }
        glob_path.push("*.ers");

        let ref glob_pattern = glob_path.to_string_lossy();
        let mut glob_options = glob::MatchOptions::new();
        glob_options.case_sensitive = false;

        let mut num_all_files = 0;
        let mut num_bad_files = 0;

        match glob::glob_with(glob_pattern, glob_options) {
            Ok(path_list) => {
                for entry in path_list {
                    match entry {
                        Ok(ref path) => {
                            if self.verbosity >= 1 {
                                info!("> {}", path.display());
                            }

                            let result = Ersatz::from_file(path);

                            num_all_files += 1;

                            match result {
                                Ok(ersatz) => {
                                    if self.verbosity >= 2 {
                                        debug!("{:?}", ersatz);
                                    }

                                    if !self.syntax_only {
                                        // FIXME
                                    }
                                }
                                Err(err) => {
                                    if self.do_abort {
                                        warn!("Aborting on syntax error");
                                        return Err(err)
                                    } else {
                                        let ref header =
                                            format!("Syntax error in file '{}'...", path.display());
                                        AppError::report_with_header(err, header);
                                        num_bad_files += 1;
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            error!("Bad entry in path list: {}", err);
                        }
                    }
                }

                if num_bad_files > 0 {
                    println!(
                        "... Done ({} bad file{} out of {} checked).",
                        num_bad_files,
                        if num_bad_files == 1 { "" } else { "s" },
                        num_all_files,
                    );
                } else {
                    println!("... Done (no bad files out of {} checked).", num_all_files);
                }

                Ok(())
            }
            Err(err) => panic!("Invalid glob pattern: {}", err),
        }
    }
}
