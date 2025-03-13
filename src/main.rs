use eframe::egui;
use std::fs::File;
use std::io::{self, BufRead, Read};
use std::path::Path;
use zip::read::ZipArchive;
use quick_xml::Reader;
use quick_xml::events::Event;
use pdf_extract::extract_text;
use rfd::FileDialog;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Word Counter",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

struct MyApp {
    file_path: String,
    word_count: Option<usize>,
    error_message: Option<String>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            word_count: None,
            error_message: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(egui::RichText::new("Підрахунок слів у файлі").size(24.0));

            ui.add_space(10.0);

            if ui.button(egui::RichText::new("📂 Вибрати файл").size(18.0)).clicked() {
                if let Some(path) = FileDialog::new().pick_file() {
                    self.file_path = path.to_string_lossy().to_string();
                    self.word_count = None;
                    self.error_message = None;
                }
            }

            ui.add_space(10.0);
            ui.label(egui::RichText::new(format!("Файл: {}", if self.file_path.is_empty() { "Не вибрано" } else { &self.file_path })).size(16.0));
            ui.add_space(10.0);

            if ui.button(egui::RichText::new("▶ Порахувати слова").size(18.0)).clicked() && !self.file_path.is_empty() {
                match count_words(&self.file_path) {
                    Ok(count) => {
                        self.word_count = Some(count);
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(e.to_string());
                        self.word_count = None;
                    }
                }
            }

            ui.add_space(10.0);
            if let Some(count) = self.word_count {
                ui.label(egui::RichText::new(format!("📝 Кількість слів: {}", count)).size(20.0).strong());
            }

            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, egui::RichText::new(format!("❌ Помилка: {}", error)).size(18.0));
            }
        });
    }
}

fn count_words(file_path: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

    match extension.as_str() {
        "txt" => count_words_in_txt(file_path),
        "docx" => count_words_in_docx(file_path),
        "pdf" => count_words_in_pdf(file_path),
        _ => Err("Непідтримуваний формат файлу".into()),
    }
}

fn count_words_in_txt(file_path: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);
    let mut word_count = 0;
    for line in reader.lines() {
        let line = line?;
        word_count += line.split_whitespace().count();
    }
    Ok(word_count)
}

fn count_words_in_docx(file_path: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut word_count = 0;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = file.name().to_string();

        if file_name == "word/document.xml" {
            let mut xml_content = String::new();
            file.read_to_string(&mut xml_content)?;
            let mut reader = Reader::from_str(&xml_content);
            reader.trim_text(true);

            while let Ok(event) = reader.read_event() {
                if let Event::Text(e) = event {
                    let text = e.unescape()?.into_owned();
                    word_count += text.split_whitespace().count();
                }
            }
        }
    }
    Ok(word_count)
}

fn count_words_in_pdf(file_path: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let text = extract_text(file_path)?;
    Ok(text.split_whitespace().count())
}
