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
        Block, Borders, Clear, Paragraph, Widget, Wrap,
    },
    DefaultTerminal, Frame,
};
use rayon::prelude::*;
use scraper::{Html, Selector};
//use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
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
    content: Vec<String>,
    text: String,
    page: u16,
    pages: u16,
    path: String,
    wpm: u16,
    exit: bool,
    scroll_offset: u16,
    progress: HashMap<String, u16>,
    popup_text: Option<String>,
    show_metadata: Option<String>,
    metadata: HashMap<String, Vec<String>>,
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_width = r.width * percent_x / 100;
    let popup_height = r.height * percent_y / 100;
    let popup_x = (r.width - popup_width) / 2;
    let popup_y = (r.height - popup_height) / 2;
    Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    }
}

pub fn extract_text_from_xhtml(xhtml: &str) -> String {
    let document = Html::parse_document(xhtml);

    // Select the body of the HTML document
    let selector = Selector::parse("body").unwrap();
    let mut text = String::new();

    for element in document.select(&selector) {
        // Instead of joining with spaces, we join with newlines to preserve formatting
        text.push_str(&element.text().collect::<Vec<_>>().join("\n"));
    }

    text
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
        self.load_progress();
        let num_pages = {
            let epub = EpubDoc::new(&path).unwrap();
            epub.get_num_pages()
        };

        let metadata = {
            let epub = EpubDoc::new(&path).unwrap();
            epub.metadata
        };

        // Process pages in parallel
        let content: Vec<String> = (0..num_pages)
            .into_par_iter()
            .map(|i| {
                // Open a new instance of EpubDoc for each thread
                let mut epub = EpubDoc::new(&path).unwrap();
                epub.set_current_page(i);
                extract_text_from_xhtml(&epub.get_current_str().unwrap().0)
            })
            .collect();

        self.content = content;
        self.pages = num_pages as u16;
        self.page = *self.progress.get(&path).unwrap_or(&0);
        self.text = self.content[self.page as usize].clone();
        self.metadata = metadata;

        while !self.exit {
            self.path = path.clone();
            self.wpm = words_per_minute;
            terminal.draw(|frame| self.draw(frame))?;
            self.progress.insert(self.path.clone(), self.page);
            self.save_progress();
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());

        // If there's a popup to show, render it
        if let Some(ref popup_text) = self.popup_text {
            let popup_area = centered_rect(60, 20, frame.area()); // Center the popup
            frame.render_widget(Clear, popup_area); // Clear the background behind the popup
            let popup = Paragraph::new(popup_text.clone()) // Use popup_text here
            .block(Block::default().title("Reading Time").borders(Borders::ALL));
            frame.render_widget(popup, popup_area);
        }

        if let Some(ref show_metadata) = self.show_metadata {
            let popup_area = centered_rect(70, 60, frame.area());
            frame.render_widget(Clear, popup_area);

            // Render the metadata popup
            let popup = Paragraph::new(show_metadata.clone()).block(
                Block::default()
                    .title("Document Metadata")
                    .borders(Borders::ALL),
            );
            frame.render_widget(popup, popup_area);
        }
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
            KeyCode::Up => self.scroll_up(),
            KeyCode::Down => self.scroll_down(),
            KeyCode::Char('s') => self.show_reading_time(),
            KeyCode::Char('c') => {
                self.popup_text = None;
                self.show_metadata = None;
            }
            KeyCode::Char('m') => self.show_metadata(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    // This will handle going to next page
    fn next_page(&mut self) {
        if self.page != self.pages - 1 {
            self.page += 1;
            self.text = self.content[self.page as usize].clone();
            self.scroll_offset = 0;
        }
    }

    // This will handle going to the previous page, with 0 also being the lowest possible page
    fn previous_page(&mut self) {
        if self.page != 0 {
            self.page -= 1;
            self.text = self.content[self.page as usize].clone();
            self.scroll_offset = 0;
        }
    }

    fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    fn load_progress(&mut self) {
        if let Ok(data) = fs::read_to_string("progress.json") {
            self.progress = serde_json::from_str(&data).unwrap_or_default();
        }
    }

    fn save_progress(&self) {
        let data = serde_json::to_string(&self.progress).unwrap();
        fs::write("progress.json", data).unwrap();
    }

    // Calculate the estimated reading time for the current page based on WPM
    fn calculate_reading_time(&self) -> u32 {
        let word_count = self.text.split_whitespace().count();
        let reading_time = (word_count as f32 / self.wpm as f32 * 60.0).ceil() as u32;
        reading_time
    }

    // Show the estimated reading time for the current page
    fn show_reading_time(&mut self) {
        let reading_time = self.calculate_reading_time();
        self.popup_text = Some(format!(
            "Estimated reading time: {} seconds (WPM: {}) \n\n\n Press <C> to close pop-up!",
            reading_time, self.wpm
        ));
    }

    fn show_metadata(&mut self) {
        let metadata_str = self.format_metadata();
        self.show_metadata = Some(metadata_str);
    }

    // Helper function to format the metadata as a string
    fn format_metadata(&self) -> String {
        let mut result = String::new();
        for (key, values) in &self.metadata {
            result.push_str(&format!("\n {}:  {}\n", key, values.join(", ")));
        }
        result.push_str(&format!("\n\n\nPress <C> to close pop-up!"));
        result
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
            " Scroll up ".into(),
            "<Up>".blue().bold(),
            " Scroll down ".into(),
            "<Down>".blue().bold(),
            " Bonus ".into(),
            "<S> ".blue().bold(),
            " Metadata ".into(),
            "<M> ".blue().bold(),
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
            .skip(self.scroll_offset as usize) // Skip lines based on scroll_offset
            .take(area.height as usize) // Take only the visible lines
            .map(|line| Line::from(line.to_string().yellow()))
            .collect();

        let test_text = Text::from(text_lines);

        let _counter_text = Text::from(vec![
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
