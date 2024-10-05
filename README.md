# **EPUB Reader CLI**

In the digital age, reading is both an intellectual journey and a form of meditation. This CLI EPUB Reader is built to enhance that journey, providing a streamlined reading experience directly in the terminal. Powered by Rust, it leverages modern libraries to create a minimalist but efficient interface, allowing you to focus on the content while keeping your reading progress intact.

## **Features**
- **Text-Based EPUB Reading**: This tool allows you to view and navigate EPUB files directly in the terminal. While the focus is on simple text content, the current beta version does not yet fully support complex elements such as tables or code blocks. These are displayed, but not with perfect fidelity.
- **Progress Tracking**: Your reading progress is automatically saved in a JSON file using a Rust `HashMap`, ensuring that when you reopen a book, you continue right where you left off.
- **Parallel EPUB Processing**: The application utilizes **Rayon** to process the EPUB file in parallel at launch, making the loading experience faster and more responsive, even for large files.
- **Estimated Reading Time**: Press 's' to calculate and display the estimated time required to finish the current page, based on your words-per-minute (WPM) reading speed.
- **Customizable Reading Speed**: Set your reading speed with the command-line argument to match your preferred pace.

## **Installation**

1. Clone the repository:
   ```bash
   git clone https://github.com/Juan-LukeKlopper/epub_reader.rs.git
   ```

2. Build the project:
   ```bash
   cd epub_reader.rs
   cargo build --release
   ```

3. Navigate to build folder:
   ```bash
   cd target/release
   ```

4. Run the application:
   ```bash
   epub-reader-cli --path /path/to/your/book.epub
   ```

## **Usage**

### **Command-line Arguments**

```bash
USAGE:
    epub-reader-cli [OPTIONS] --path <PATH>

OPTIONS:
    -p, --path <PATH>              Path of the EPUB file to open
    -w, --words-per-minute <WPM>    Set reading speed in words per minute (default: 238)
    -h, --help                      Show help information
    -v, --version                   Show version information
```

### **Keyboard Controls**

- **Left/Right Arrow**: Turn to the previous/next page.
- **Up/Down Arrow**: Scroll through the current page.
- **S**: Show the estimated reading time for the current page.
- **M**: Show the document metadata.
- **Q**: Quit the reader.

### **Demo**

[![asciicast](https://asciinema.org/a/dJTP1vVIyIAcRBRl0FRXpydBh.svg)](https://asciinema.org/a/dJTP1vVIyIAcRBRl0FRXpydBh)

## **How it Works**

### **CLI Design**
This EPUB reader is built using Rust’s powerful ecosystem of libraries:
- **`clap`** for parsing command-line arguments.
- **`crossterm`** for cross-platform terminal input and output.
- **`ratatui`** for handling the user interface within the terminal, displaying content and handling interactions.

### **Progress Tracking**
Your progress is saved after each page flip. The application maintains a JSON file where it tracks your position in each book you open. Using a `HashMap`, it stores the current page number for each book path, allowing you to resume reading from where you left off.

### **Parallel Processing**
The reader uses the **Rayon** crate to speed up EPUB processing by leveraging parallelism. This ensures that even large books are loaded quickly, giving you an efficient and responsive reading experience.

