use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use epub::doc::EpubDoc;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Paragraph, Widget,
    },
    DefaultTerminal, Frame,
};
use std::io;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path of the epub file
    #[arg(short, long)]
    path: String,

    /// words per minute used to calculate estimated reading time
    /// 238 is the Adult Average Reading Speed so is a sensible default
    #[arg(short, long, default_value_t = 238)]
    words_per_minute: u16,
}

#[derive(Debug, Default)]
pub struct App {
    page: u16,
    path: String,
    wpm: u16,
    exit: bool,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        Args {
            path,
            words_per_minute,
        }: Args,
    ) -> io::Result<()> {
        self.path = path.clone();
        self.wpm = words_per_minute;
        let mut epub = EpubDoc::new(&path).unwrap();
        println!(
            "Metadata: {:?}\n\nTable of centent: {:?}\n\nNum of pages: {:?} \n\n current page: {:?} \n\n spine: {:?} current string: {:?} ",
            epub.metadata.clone(),
            epub.toc.clone(),
            epub.get_num_pages(),
            epub.get_current_page(),
            epub.spine.clone(),
            epub.get_current_str().unwrap(),
        );
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.previous_page(),
            KeyCode::Right => self.next_page(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    // This will handle going to next page
    fn next_page(&mut self) {
        self.page += 1;
    }

    // This will handle going to the previous page, with 0 also being the lowest possible page
    fn previous_page(&mut self) {
        if self.page != 0 {
            self.page -= 1;
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Epub reader application ".bold());
        let instructions = Title::from(Line::from(vec![
            " Previous page ".into(),
            "<Left>".blue().bold(),
            " Next page ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let text_lines: Vec<Line> = self
            .text
            .lines() // Split text by newlines
            .take(area.height as usize) // Take only the visible lines
            .map(|line| Line::from(line.to_string().yellow()))
            .collect();

        let test_text = Text::from(text_lines);

        let counter_text = Text::from(vec![
            Line::from(vec!["Page: ".into(), self.page.to_string().yellow()]),
            Line::from(vec!["Path: ".into(), self.path.clone().yellow()]),
            Line::from(vec!["text: ".into(), self.text.clone().yellow()]),
        ]);

        Paragraph::new(test_text)
            .wrap(Wrap { trim: true })
            .block(block)
            .render(area, buf);
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = App::default().run(&mut terminal, args);
    ratatui::restore();
    app_result
}
