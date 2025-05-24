use crate::{
    args::Opts,
    tui::Renderer,
    util::{DisplayablePathBuf, Matcher},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct App {
    input: String,
    matcher: Matcher<DisplayablePathBuf>,
    renderer: Renderer,
    opts: Opts,
}

impl App {
    pub fn run() -> Result<()> {
        if !io::stderr().is_tty() {
            eprintln!("This is not a terminal!");
            exit(1);
        }

        let opts = Opts::from_args(env::args()).unwrap_or_else(|err| {
            eprintln!("Argument error: {err}.");
            usage();
            exit(1)
        });

        let entries = get_entries(&opts.base_path).unwrap_or_else(|err| {
            eprintln!("Error reading directory {err}");
            exit(1)
        });

        let matcher = Matcher::new(&entries)?;

        let Some(entry) = entries.get(0) else { exit(0) };

        let parent = fs::canonicalize(entry.get().parent().unwrap_or(&PathBuf::new()))
            .unwrap_or(PathBuf::default());

        let style = |bind: &&DisplayablePathBuf| match bind.get().is_dir() {
            true => bind.to_string().blue(),
            false => bind.to_string().stylize(),
        };

        let mut renderer = Renderer::new_fullscreen()?;

        let mut input = String::new();

        renderer.draw_list(
            &input,
            parent.to_string_lossy().yellow(),
            matcher.iter_all(),
            style,
        )?;

        while input.len() < 2 {
            match crossterm::event::read()? {
                Event::Key(KeyEvent {
                    code, modifiers, ..
                }) => {
                    if code.is_char('c') && modifiers == KeyModifiers::CONTROL {
                        renderer.restore()?;
                        return Ok(());
                    }

                    if let Some(key) = code.as_char() {
                        if modifiers.is_empty() {
                            input.push(key);

                            if !matcher.is_valid_prefix(&input) {
                                input.pop();
                            }

                            renderer.draw_list(
                                &input,
                                parent.to_string_lossy().yellow(),
                                matcher.iter_all(),
                                style,
                            )?;
                        }
                    }
                }
                _ => {}
            }
        }

        renderer.restore()?;

        if let Some(entry) = matcher.get_by_label(&input) {
            println!("{}", entry.get().to_string_lossy());
        } else {
            println!("Wrong label!");
        }
        Ok(())
    }

    pub fn draw() {
        todo!();
    }
}
