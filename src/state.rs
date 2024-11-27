use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

use inquire::{autocompletion::Replacement, Autocomplete};

#[derive(Clone)]
pub struct State {
    previous_emails: Vec<String>,
}

fn get_state_file() -> io::Result<PathBuf> {
    let mut state = dirs::state_dir().ok_or(io::Error::new(
        io::ErrorKind::Other,
        "couldn't identify state directory",
    ))?;

    state.push("digi_download");

    if !state.exists() {
        std::fs::create_dir_all(&state)?;
    }

    state.push("state");

    Ok(state)
}

impl State {
    pub fn load() -> io::Result<Self> {
        let state = get_state_file()?;

        let file = BufReader::new(File::open(state)?);

        let previous_emails: Vec<String> = file
            .lines()
            .map(|x| x.map(|x| x.trim().to_string()))
            .collect::<Result<_, _>>()?;

        Ok(Self { previous_emails })
    }

    pub fn empty() -> Self {
        State {
            previous_emails: vec![],
        }
    }

    pub fn add_email(&mut self, email: String) {
        if !self.previous_emails.contains(&email) {
            self.previous_emails.push(email);
        }
    }

    pub fn write(self) -> io::Result<()> {
        let state_file = get_state_file()?;

        let mut writer = BufWriter::new(File::create(state_file)?);

        for line in self.previous_emails {
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
        }

        Ok(())
    }
}

impl Autocomplete for State {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        let suggestions = self
            .previous_emails
            .iter()
            .filter(|x| x.starts_with(input))
            .cloned()
            .collect();

        Ok(suggestions)
    }

    fn get_completion(
        &mut self,
        _input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        if let Some(suggestion) = highlighted_suggestion {
            return Ok(Replacement::Some(suggestion));
        }

        Ok(Replacement::None)
    }
}
