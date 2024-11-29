#![feature(let_chains)]

use std::{io, sync::Arc};

use digi_download_core::{
    digi4school::{session::Session, volume::Volume},
    error::ScraperError,
    lopdf::Document,
    merge_pdf,
};

use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use inquire::{MultiSelect, Password, Text};
use state::State;
use tokio::sync::Semaphore;
use validator::NonEmptyValidator;

mod state;
mod validator;

static CONCURRENT_DOWNLOADS: Semaphore = Semaphore::const_new(16);

async fn download_task(volume: Arc<Volume>, pb: ProgressBar) -> Result<Document, ScraperError> {
    let scraper = Arc::new(volume.get_scraper().await.map_err(ScraperError::Request)?);

    let page_count = scraper
        .fetch_page_count()
        .await
        .map_err(ScraperError::Request)?;

    assert!(page_count >= 1, "no pages to download");
    pb.set_length(page_count as u64);
    pb.inc(1);

    let (send, mut recv) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        for i in 1..=page_count {
            let scraper = scraper.clone();
            let send = send.clone();

            let _myperm = CONCURRENT_DOWNLOADS.acquire().await.unwrap();
            tokio::spawn(async move {
                match scraper.fetch_page_pdf(i).await {
                    Ok(p) => {
                        send.send(Ok((p, i))).unwrap();
                    }
                    Err(e) => {
                        send.send(Err(e)).unwrap();
                    }
                };
                drop(_myperm);
            });
        }
    });

    let mut merge_doc = None;
    let mut to_merge = Vec::with_capacity(32);
    let mut merge_next = 1;

    while merge_next <= page_count
        && let Some(res) = recv.recv().await
    {
        let (page, i) = res?;
        pb.inc(1);

        if i != merge_next {
            to_merge.push((page, i));
            continue;
        }

        if i == merge_next {
            match merge_doc {
                None => merge_doc = Some(page),
                Some(doc) => {
                    merge_doc = Some(merge_pdf(doc, page).map_err(ScraperError::PdfError)?)
                }
            }

            loop {
                merge_next += 1;
                let Some(index) = to_merge
                    .iter()
                    .enumerate()
                    .find(|(_, (_, i))| *i == merge_next)
                    .map(|(x, _)| x)
                else {
                    break;
                };

                let (doc, _) = to_merge.remove(index);
                merge_doc = Some(
                    merge_pdf(merge_doc.take().unwrap(), doc).map_err(ScraperError::PdfError)?,
                );
            }
        }
    }

    pb.finish();

    Ok(merge_doc.unwrap())
}

#[tokio::main]
async fn main() {
    let mut state = match State::load() {
        Ok(s) => s,
        Err(e) => {
            if e.kind() != io::ErrorKind::NotFound {
                eprintln!("failed to open state: {e}");
            }
            State::empty()
        }
    };

    let mut out_dir = std::env::temp_dir();
    out_dir.push("digi_download");

    if !out_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&out_dir) {
            eprintln!("Failed to create output directory: {e}");
            return;
        }
    }

    let email = Text::new("Enter your E-Mail:")
        .with_validator(validator::EmailValidator)
        .with_autocomplete(state.clone())
        .prompt()
        .expect("should be able to get email input");

    let password = Password::new("Enter your password:")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .without_confirmation()
        .prompt()
        .expect("should be able to get password input");

    let session = Session::new(email.clone(), password)
        .await
        .expect("should be able to log in");

    let mut books = session
        .get_books()
        .await
        .expect("should be able to get books");

    books.sort_unstable_by_key(|x| u16::MAX - x.year());

    let selected_books = MultiSelect::new("Choose the book:", books)
        .with_validator(NonEmptyValidator)
        .prompt()
        .expect("should be able to select books");

    let mut downloads = vec![];

    let mb = MultiProgress::with_draw_target(ProgressDrawTarget::hidden());
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap()
    .progress_chars("#*-");

    for book in selected_books {
        let volumes = book
            .get_volumes()
            .await
            .expect("should be able to get volumes");

        let x = if volumes.len() > 1 {
            let title = format!("Choose volume of {book}:");

            MultiSelect::new(&title, volumes)
                .prompt()
                .expect("should be able to select volumes")
        } else {
            eprintln!("There is only one volume available for {book}, downloading automatically");
            volumes
        };

        for volume in x {
            let volume = Arc::new(volume);
            let pb = ProgressBar::with_draw_target(None, ProgressDrawTarget::hidden());
            pb.set_message(volume.name().to_string());
            pb.set_style(sty.clone());
            mb.add(pb.clone());

            downloads.push((
                pb.clone(),
                volume.clone(),
                tokio::spawn(async move { download_task(volume, pb).await }),
            ));
        }
    }

    mb.set_draw_target(ProgressDrawTarget::stderr());

    let mut errors = vec![];
    let mut documents = vec![];
    for (pb, volume, task) in downloads {
        match task.await.unwrap() {
            Ok(d) => documents.push((volume, d)),
            Err(e) => {
                pb.abandon_with_message(format!("Failed: {}", volume.name()));
                errors.push((volume, e));
            }
        }
    }

    for (vol, doc) in &mut documents {
        // TODO: regex whitespace etc
        let mut file = vol.name().replace(['.', '+'], "").replace(' ', "_");
        file.push_str(".pdf");

        let mut path = out_dir.clone();
        path.push(file);

        if let Err(e) = doc.save(&path) {
            eprintln!("Failed to write document to {path:?}: {e}");
        }

        println!("{} has been saved to {:?}", vol.name(), path);
    }

    if !documents.is_empty() {
        let should_open = inquire::Confirm::new("Open output directory?")
            .with_default(true)
            .prompt()
            .expect("should be able to get confirm input");

        if should_open {
            if let Err(e) = open::that(out_dir) {
                eprintln!("Failed to open output directory: {e}");
            }
        }
    }

    for (vol, error) in errors {
        eprintln!("Failed to download {vol}: {error}!");
    }

    state.add_email(email);

    if let Err(e) = state.write() {
        eprintln!("failed to write state: {e}");
    }
}
