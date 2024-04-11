use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::{path::Path, process::exit};

mod app_state;
mod consts;
mod embedded;
mod exercise;
mod init;
mod list;
mod progress_bar;
mod run;
mod watch;

use self::{
    app_state::AppState,
    consts::WELCOME,
    exercise::InfoFile,
    init::init,
    list::list,
    run::run,
    watch::{watch, WatchExit},
};

/// Rustlings is a collection of small exercises to get you used to writing and reading Rust code
#[derive(Parser)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Option<Subcommands>,
}

#[derive(Subcommand)]
enum Subcommands {
    /// Initialize Rustlings
    Init,
    /// Same as just running `rustlings` without a subcommand.
    Watch,
    /// Run a single exercise. Runs the next pending exercise if the exercise name is not specified.
    Run {
        /// The name of the exercise
        name: Option<String>,
    },
    /// Reset a single exercise
    Reset {
        /// The name of the exercise
        name: String,
    },
    /// Return a hint for the given exercise
    Hint {
        /// The name of the exercise
        name: String,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    which::which("cargo").context(
        "Failed to find `cargo`.
Did you already install Rust?
Try running `cargo --version` to diagnose the problem.",
    )?;

    let mut info_file = InfoFile::parse()?;
    info_file.exercises.shrink_to_fit();
    let exercises = info_file.exercises;

    if matches!(args.command, Some(Subcommands::Init)) {
        init(&exercises).context("Initialization failed")?;
        println!(
            "\nDone initialization!\n
Run `cd rustlings` to go into the generated directory.
Then run `rustlings` for further instructions on getting started."
        );
        return Ok(());
    } else if !Path::new("exercises").is_dir() {
        println!(
            "
{WELCOME}

The `exercises` directory wasn't found in the current directory.
If you are just starting with Rustlings, run the command `rustlings init` to initialize it."
        );
        exit(1);
    }

    let mut app_state = AppState::new(exercises);

    match args.command {
        None | Some(Subcommands::Watch) => loop {
            match watch(&mut app_state)? {
                WatchExit::Shutdown => break,
                // It is much easier to exit the watch mode, launch the list mode and then restart
                // the watch mode instead of trying to pause the watch threads and correct the
                // watch state.
                WatchExit::List => list(&mut app_state)?,
            }
        },
        // `Init` is handled above.
        Some(Subcommands::Init) => (),
        Some(Subcommands::Run { name }) => {
            if let Some(name) = name {
                app_state.set_current_exercise_by_name(&name)?;
            }
            run(&mut app_state)?;
        }
        Some(Subcommands::Reset { name }) => {
            app_state.set_current_exercise_by_name(&name)?;
            app_state.set_pending(app_state.current_exercise_ind())?;
            let exercise = app_state.current_exercise();
            exercise.reset()?;
            println!("The exercise {exercise} has been reset!");
        }
        Some(Subcommands::Hint { name }) => {
            app_state.set_current_exercise_by_name(&name)?;
            println!("{}", app_state.current_exercise().hint);
        }
    }

    Ok(())
}
