use std::env;
use std::fs;
use std::io::{stdout, Stdout};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueEnum};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};

const RU_WORDS: &[&str] = &[
    // Предлоги и короткие связки
    "и", "в", "не", "на", "с", "по", "к", "из", "у", "за", "от", "о", "до", "под", "над", "при", "про", "для", "или", "но",
    // Существительные
    "город", "дом", "окно", "дверь", "стол", "стул", "книга", "ручка", "время", "день",
    "ночь", "утро", "вечер", "земля", "небо", "солнце", "месяц", "звезда", "вода", "огонь",
    "воздух", "ветер", "дождь", "снег", "трава", "дерево", "цветок", "птица", "рыба", "зверь",
    "собака", "кошка", "лес", "поле", "река", "море", "озеро", "берег", "остров", "камень",
    "песок", "гора", "дорога", "путь", "улица", "поселок", "школа", "работа", "слово", "буква",
    "текст", "страница", "мысль", "голос", "звук", "песня", "музыка", "картина", "краска", "форма",
    "число", "номер", "часть", "центр", "край", "группа", "друг", "народ", "семья", "человек",
    // Глаголы
    "писать", "читать", "говорить", "слушать", "смотреть", "видеть", "думать", "знать", "понимать", "помнить",
    "забыть", "учить", "делать", "работать", "жить", "стоять", "сидеть", "лежать", "ходить", "бежать",
    "плавать", "брать", "давать", "купить", "продать", "строить", "открыть", "закрыть",
    // Прилагательные
    "большой", "маленький", "новый", "старый", "добрый", "быстрый", "медленный", "сильный", "слабый", "высокий",
    "низкий", "долгий", "короткий", "широкий", "узкий", "светлый", "темный", "горячий", "холодный", "белый",
    "черный", "красный", "синий", "зеленый", "первый", "последний", "хороший", "плохой", "главный", "простой",
    "сложный", "чистый", "тихий", "громкий", "каждый", "другой", "разный", "полный", "пустой", "важный"
];

const EN_WORDS: &[&str] = &[
    // Prepositions & short connectors
    "in", "on", "at", "to", "for", "with", "by", "from", "up", "about", "into", "over", "after", "and", "but", "or", "so", "if", "out", "as",
    // Nouns
    "time", "year", "people", "way", "day", "man", "thing", "woman", "life", "child",
    "world", "school", "state", "family", "student", "group", "country", "problem", "hand", "part",
    "place", "case", "week", "company", "system", "program", "question", "work", "number", "night",
    "point", "home", "water", "room", "mother", "area", "money", "story", "fact", "month",
    "lot", "right", "study", "book", "eye", "job", "word", "business", "issue", "side",
    // Verbs
    "have", "do", "say", "get", "make", "go", "know", "take", "see", "come",
    "think", "look", "want", "give", "use", "find", "tell", "ask", "work", "seem",
    "feel", "try", "leave", "call", "keep", "hold", "turn", "start", "show", "hear",
    "play", "run", "move", "like", "live", "believe", "hold", "bring", "happen", "write",
    // Adjectives
    "good", "new", "first", "last", "long", "great", "little", "own", "other", "old",
    "right", "big", "high", "different", "small", "large", "next", "early", "young", "important",
    "few", "public", "bad", "same", "able", "full", "easy", "hard", "clear", "recent"
];
const FIELD_LANG: usize = 0;
const FIELD_WORDS_LANG: usize = 1;
const FIELD_MODE: usize = 2;
const FIELD_AMOUNT: usize = 3;
const FIELD_USE_LLM: usize = 4;
const FIELD_PROMPT: usize = 5;
const FIELD_TEXT_LANG: usize = 6;
const FIELD_MODEL: usize = 7;
const FIELD_API_PROVIDER: usize = 8;
const FIELD_API_BASE: usize = 9;
const FIELD_API_KEY: usize = 10;
const FIELD_API_KEY_ENV: usize = 11;
const FIELD_LLM_WORDS: usize = 12;
const FIELD_RANDOM_WORDS: usize = 13;
const FIELD_REGENERATE: usize = 14;
const FIELD_SAVE: usize = 15;
const FIELD_CANCEL: usize = 16;
const FIELD_COUNT: usize = 17;

const RESULT_LOCK_DURATION: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Language {
    Ru,
    En,
}

impl Default for Language {
    fn default() -> Self {
        Language::Ru
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum OptionalLanguage {
    Auto,
    Ru,
    En,
}

impl Default for OptionalLanguage {
    fn default() -> Self {
        OptionalLanguage::Auto
    }
}

impl OptionalLanguage {
    fn to_option(self) -> Option<Language> {
        match self {
            OptionalLanguage::Auto => None,
            OptionalLanguage::Ru => Some(Language::Ru),
            OptionalLanguage::En => Some(Language::En),
        }
    }
}

fn optional_language_from(option: Option<Language>) -> OptionalLanguage {
    match option {
        None => OptionalLanguage::Auto,
        Some(Language::Ru) => OptionalLanguage::Ru,
        Some(Language::En) => OptionalLanguage::En,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ApiProvider {
    Auto,
    OpenAi,
    LmStudio,
    Ollama,
    Anthropic,
}

impl Default for ApiProvider {
    fn default() -> Self {
        ApiProvider::Auto
    }
}

fn provider_label(p: ApiProvider) -> String {
    match p {
        ApiProvider::Auto => "auto".to_string(),
        ApiProvider::OpenAi => "openai".to_string(),
        ApiProvider::LmStudio => "lmstudio".to_string(),
        ApiProvider::Ollama => "ollama".to_string(),
        ApiProvider::Anthropic => "anthropic".to_string(),
    }
}

fn next_provider(current: ApiProvider, forward: bool) -> ApiProvider {
    let order = [
        ApiProvider::Auto,
        ApiProvider::OpenAi,
        ApiProvider::LmStudio,
        ApiProvider::Ollama,
        ApiProvider::Anthropic,
    ];

    let pos = order.iter().position(|x| *x == current).unwrap_or(0);

    let new_pos = if forward {
        (pos + 1) % order.len()
    } else {
        (pos + order.len() - 1) % order.len()
    };

    order[new_pos]
}

fn detect_provider(api_base: &str) -> ApiProvider {
    let b = api_base.to_lowercase();

    if b.contains("lmstudio") || b.contains(":1234") {
        ApiProvider::LmStudio
    } else if b.contains("ollama") || b.contains(":11434") {
        ApiProvider::Ollama
    } else if b.contains("anthropic") {
        ApiProvider::Anthropic
    } else {
        ApiProvider::OpenAi
    }
}

#[derive(Parser, Clone)]
#[command(
    name = "RustTypingTrainer",
    version,
    about = "RustTypingTrainer — MonkeyType-like terminal typing test with in-app settings"
)]
struct Args {
    #[arg(long, default_value = "settings.toml")]
    config: PathBuf,

    #[arg(long, value_enum)]
    lang: Option<Language>,

    #[arg(long, value_enum)]
    words_lang: Option<OptionalLanguage>,

    #[arg(long)]
    mode: Option<String>,

    #[arg(long)]
    amount: Option<u32>,

    #[arg(long)]
    llm_words: Option<u32>,

    #[arg(long)]
    random_words: Option<u32>,

    #[arg(long)]
    words_file: Option<PathBuf>,

    #[arg(long)]
    prompt: Option<String>,

    #[arg(long)]
    llm: bool,

    #[arg(long)]
    random: bool,

    #[arg(long)]
    model: Option<String>,

    #[arg(long)]
    api_base: Option<String>,

    #[arg(long, value_enum)]
    api_provider: Option<ApiProvider>,

    #[arg(long)]
    api_key: Option<String>,

    #[arg(long)]
    api_key_env: Option<String>,

    #[arg(long, value_enum)]
    text_lang: Option<OptionalLanguage>,

    #[arg(long)]
    regenerate_on_restart: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct Config {
    language: Language,
    words_language: OptionalLanguage,
    mode: String,
    amount: u32,
    use_llm: bool,
    llm_words: u32,
    random_words: u32,
    words_file: Option<PathBuf>,
    regenerate_on_restart: bool,
    llm: LlmConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::Ru,
            words_language: OptionalLanguage::Auto,
            mode: "time".to_string(),
            amount: 30,
            use_llm: false,
            llm_words: 120,
            random_words: 300,
            words_file: None,
            regenerate_on_restart: false,
            llm: LlmConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct LlmConfig {
    prompt: String,
    text_language: OptionalLanguage,
    model: String,
    api_provider: ApiProvider,
    api_base: String,
    api_key: String,
    api_key_env: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            text_language: OptionalLanguage::Auto,
            model: "gpt-4o-mini".to_string(),
            api_provider: ApiProvider::Auto,
            api_base: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            api_key_env: "OPENAI_API_KEY".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct Settings {
    lang: Language,
    words_lang: Option<Language>,
    mode: String,
    amount: u32,
    use_llm: bool,
    llm_words: u32,
    random_words: u32,
    words_file: Option<PathBuf>,
    regenerate_on_restart: bool,
    llm: LlmSettings,
}

#[derive(Debug, Clone)]
struct LlmSettings {
    prompt: String,
    text_lang: Option<Language>,
    model: String,
    provider: ApiProvider,
    api_base: String,
    api_key: Option<String>,
    api_key_env: String,
}

const DEFAULT_CONFIG: &str = r#"language = "ru"
words_language = "auto"
mode = "time"
amount = 30
use_llm = false
llm_words = 120
random_words = 300
regenerate_on_restart = false

[llm]
prompt = ""
text_language = "auto"
model = "gpt-4o-mini"
api_provider = "auto"
api_base = "https://api.openai.com/v1"
api_key = ""
api_key_env = "OPENAI_API_KEY"
"#;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Screen {
    Test,
    Settings,
}

#[derive(Debug, Clone, Copy)]
struct Stats {
    elapsed: Duration,
    wpm: f64,
    raw: f64,
    acc: f64,
    correct_chars: usize,
    typed_chars: usize,
    raw_chars: usize,
}

struct App {
    settings: Settings,
    words: Vec<String>,
    typed_words: Vec<String>,
    current_word: usize,
    started: bool,
    finished: bool,
    start_time: Option<Instant>,
    end_time: Option<Instant>,
    time_limit: Option<Duration>,
    correct_keystrokes: u64,
    total_keystrokes: u64,
    message: String,

    screen: Screen,
    settings_index: usize,
    editing_field: Option<usize>,
    edit_buffer: String,
    config: Config,
    config_path: PathBuf,
    settings_message: String,

    result_lock_until: Option<Instant>,
}

impl App {
    fn new(settings: Settings, words: Vec<String>, config: Config, config_path: PathBuf) -> Self {
        let time_limit = if settings.mode == "time" {
            Some(Duration::from_secs(settings.amount as u64))
        } else {
            None
        };

        Self {
            settings,
            words,
            typed_words: vec![String::new()],
            current_word: 0,
            started: false,
            finished: false,
            start_time: None,
            end_time: None,
            time_limit,
            correct_keystrokes: 0,
            total_keystrokes: 0,
            message: String::new(),
            screen: Screen::Test,
            settings_index: 0,
            editing_field: None,
            edit_buffer: String::new(),
            config,
            config_path,
            settings_message: String::new(),
            result_lock_until: None,
        }
    }

    fn reset(&mut self, new_words: Option<Vec<String>>) {
        if let Some(words) = new_words {
            self.words = words;
        }

        self.typed_words = vec![String::new()];
        self.current_word = 0;
        self.started = false;
        self.finished = false;
        self.start_time = None;
        self.end_time = None;
        self.correct_keystrokes = 0;
        self.total_keystrokes = 0;
        self.message.clear();
        self.result_lock_until = None;
    }

    fn start_if_needed(&mut self) {
        if !self.started {
            self.started = true;
            self.start_time = Some(Instant::now());
        }
    }

    fn finish(&mut self) {
        if !self.finished {
            self.finished = true;
            self.end_time = Some(Instant::now());
            self.result_lock_until = Some(Instant::now() + RESULT_LOCK_DURATION);
        }
    }

    fn maybe_time_out(&mut self) {
        if self.finished || !self.started {
            return;
        }

        if let (Some(start), Some(limit)) = (self.start_time, self.time_limit) {
            if start.elapsed() >= limit {
                self.finish();
            }
        }
    }

    fn result_lock_remaining(&self) -> Option<Duration> {
        if !self.finished {
            return None;
        }

        self.result_lock_until
            .and_then(|until| until.checked_duration_since(Instant::now()))
    }

    fn result_locked(&self) -> bool {
        self.screen == Screen::Test && self.result_lock_remaining().is_some()
    }

    fn type_char(&mut self, c: char) {
        if self.finished || self.words.is_empty() || c.is_control() {
            return;
        }

        self.start_if_needed();

        if self.current_word >= self.typed_words.len() {
            self.typed_words.resize(self.current_word + 1, String::new());
        }

        self.total_keystrokes += 1;

        let pos = self.typed_words[self.current_word].chars().count();
        if let Some(target) = self.words.get(self.current_word) {
            if target.chars().nth(pos) == Some(c) {
                self.correct_keystrokes += 1;
            }
        }

        self.typed_words[self.current_word].push(c);
    }

    fn space(&mut self) {
        if self.finished || self.words.is_empty() {
            return;
        }

        if self.current_word >= self.typed_words.len() {
            self.typed_words.resize(self.current_word + 1, String::new());
        }

        let current = self.typed_words[self.current_word].clone();

        if current.is_empty() {
            return;
        }

        self.start_if_needed();
        self.total_keystrokes += 1;

        if let Some(target) = self.words.get(self.current_word) {
            if current == *target {
                self.correct_keystrokes += 1;
            }
        }

        if self.current_word + 1 >= self.words.len() {
            self.finish();
        } else {
            self.current_word += 1;

            if self.typed_words.len() <= self.current_word {
                self.typed_words.push(String::new());
            } else {
                self.typed_words[self.current_word].clear();
            }
        }
    }

    fn backspace(&mut self) {
        if self.finished || !self.started {
            return;
        }

        if self.current_word >= self.typed_words.len() {
            return;
        }

        if !self.typed_words[self.current_word].is_empty() {
            self.typed_words[self.current_word].pop();
            return;
        }

        if self.current_word > 0 {
            self.typed_words.pop();
            self.current_word -= 1;

            if self.typed_words.len() <= self.current_word {
                self.typed_words.push(String::new());
            }
        }
    }

    fn stats(&self) -> Stats {
        let now = Instant::now();

        let elapsed = match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => end.saturating_duration_since(start),
            (Some(start), None) => now.saturating_duration_since(start),
            _ => Duration::ZERO,
        };

        let typed_chars: usize = self
            .typed_words
            .iter()
            .map(|w| w.chars().count())
            .sum::<usize>()
            + self.typed_words.len().saturating_sub(1);

        let mut correct_chars = 0usize;

        for (i, typed) in self.typed_words.iter().enumerate() {
            if let Some(target) = self.words.get(i) {
                correct_chars += typed
                    .chars()
                    .zip(target.chars())
                    .filter(|(a, b)| a == b)
                    .count();
            }

            if i > 0 {
                if let (Some(prev_target), Some(prev_typed)) =
                    (self.words.get(i - 1), self.typed_words.get(i - 1))
                {
                    if prev_typed == prev_target {
                        correct_chars += 1;
                    }
                }
            }
        }

        let raw_chars = self.total_keystrokes as usize;
        let minutes = elapsed.as_secs_f64() / 60.0;

        let (wpm, raw) = if minutes > 0.0 {
            (
                (correct_chars as f64 / 5.0) / minutes,
                (raw_chars as f64 / 5.0) / minutes,
            )
        } else {
            (0.0, 0.0)
        };

        let acc = if self.total_keystrokes == 0 {
            100.0
        } else {
            (self.correct_keystrokes as f64 / self.total_keystrokes as f64) * 100.0
        };

        Stats {
            elapsed,
            wpm,
            raw,
            acc,
            correct_chars,
            typed_chars,
            raw_chars,
        }
    }
}

struct UiStrings {
    title: &'static str,
    settings_title: &'static str,
    active_help: &'static str,
    finished_help: &'static str,
    text_title: &'static str,
    results_title: &'static str,
    result_title: &'static str,
    wpm: &'static str,
    acc: &'static str,
    raw: &'static str,
    time_label: &'static str,
    chars_label: &'static str,
    errors_label: &'static str,
    generating: &'static str,
    gen_error: &'static str,
    no_words: &'static str,
    wpm_short: &'static str,
    acc_short: &'static str,
    raw_keys: &'static str,
    chars_correct: &'static str,
    chars_typed: &'static str,
    mode_time: &'static str,
    mode_words: &'static str,
    time_suffix: &'static str,
}

fn ui_strings(lang: Language) -> UiStrings {
    match lang {
        Language::Ru => UiStrings {
            title: "RustTypingTrainer",
            settings_title: "Настройки",
            active_help: "печать — старт | пробел — след. слово | backspace — исправить | tab — сброс | ctrl+r — новый текст | F2 — настройки | esc — выход",
            finished_help: "tab/enter — заново | n — новый текст | F2 — настройки | q/esc — выход",
            text_title: "текст",
            results_title: "результаты",
            result_title: "Результат",
            wpm: "Скорость (WPM)",
            acc: "Точность",
            raw: "Сырая скорость",
            time_label: "Время",
            chars_label: "Символы",
            errors_label: "Ошибки ввода",
            generating: "Генерация текста...",
            gen_error: "Ошибка генерации",
            no_words: "Слова не загружены",
            wpm_short: "сл/мин",
            acc_short: "точность",
            raw_keys: "клавиш",
            chars_correct: "верно",
            chars_typed: "введено",
            mode_time: "время",
            mode_words: "слова",
            time_suffix: "с",
        },
        Language::En => UiStrings {
            title: "RustTypingTrainer",
            settings_title: "Settings",
            active_help: "type to start | space — next word | backspace — fix | tab — reset | ctrl+r — new text | F2 — settings | esc — exit",
            finished_help: "tab/enter — restart | n — new text | F2 — settings | q/esc — quit",
            text_title: "text",
            results_title: "results",
            result_title: "Result",
            wpm: "WPM",
            acc: "Accuracy",
            raw: "Raw WPM",
            time_label: "Time",
            chars_label: "Chars",
            errors_label: "Input errors",
            generating: "Generating text...",
            gen_error: "Generation error",
            no_words: "No words loaded",
            wpm_short: "wpm",
            acc_short: "acc",
            raw_keys: "keys",
            chars_correct: "correct",
            chars_typed: "typed",
            mode_time: "time",
            mode_words: "words",
            time_suffix: "s",
        },
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config_path = args.config.clone();
    let file_config = load_or_create_config(&config_path)?;
    let mut settings = resolve_settings(args, file_config);

    if settings.mode != "time" && settings.mode != "words" {
        return Err(anyhow!("mode must be either 'time' or 'words'"));
    }

    settings.amount = settings.amount.max(1);

    let (words, initial_message) = generate_words_with_fallback(&mut settings);
    let initial_config = config_from_settings(&settings);

    let mut app = App::new(settings, words, initial_config, config_path);
    app.message = initial_message.unwrap_or_default();

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    loop {
        if app.screen == Screen::Test {
            app.maybe_time_out();
            terminal.draw(|f| ui(f, app))?;
        } else {
            terminal.draw(|f| settings_ui(f, app))?;
        }

        if app.result_locked() {
            let mut discarded = 0;

            while discarded < 512 && event::poll(Duration::from_millis(0))? {
                let _ = event::read()?;
                discarded += 1;
            }

            if event::poll(Duration::from_millis(20))? {
                let _ = event::read()?;
            }

            continue;
        }

        if !event::poll(Duration::from_millis(20))? {
            continue;
        }

        let evt = event::read()?;
        let Event::Key(key) = evt else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(());
        }

        match app.screen {
            Screen::Test => {
                if app.finished {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('й') => return Ok(()),
                        KeyCode::Tab | KeyCode::Enter => {
                            if app.settings.regenerate_on_restart && app.settings.use_llm {
                                let tr = ui_strings(app.settings.lang);
                                app.message = tr.generating.to_string();
                                terminal.draw(|f| ui(f, app))?;
                                regenerate_and_reset(app);
                            } else {
                                app.reset(None);
                            }
                        }
                        KeyCode::Char('n') | KeyCode::Char('т') => {
                            let tr = ui_strings(app.settings.lang);
                            app.message = tr.generating.to_string();
                            terminal.draw(|f| ui(f, app))?;
                            regenerate_and_reset(app);
                        }
                        KeyCode::F(2) => {
                            open_settings(app);
                        }
                        _ => {}
                    }
                } else {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        match key.code {
                            KeyCode::Char('r') | KeyCode::Char('к') => {
                                let tr = ui_strings(app.settings.lang);
                                app.message = tr.generating.to_string();
                                terminal.draw(|f| ui(f, app))?;
                                regenerate_and_reset(app);
                            }
                            _ => {}
                        }
                        continue;
                    }

                    match key.code {
                        KeyCode::Esc => return Ok(()),
                        KeyCode::Tab => app.reset(None),
                        KeyCode::F(2) => {
                            open_settings(app);
                        }
                        KeyCode::Char(' ') => app.space(),
                        KeyCode::Backspace => app.backspace(),
                        KeyCode::Char(c) => app.type_char(c),
                        _ => {}
                    }
                }
            }
            Screen::Settings => {
                handle_settings_key(terminal, app, key)?;
            }
        }
    }
}

fn open_settings(app: &mut App) {
    app.config = config_from_settings(&app.settings);
    app.settings_index = 0;
    app.editing_field = None;
    app.edit_buffer.clear();
    app.settings_message.clear();
    app.reset(None);
    app.screen = Screen::Settings;
}

fn handle_settings_key(
    _terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    key: KeyEvent,
) -> Result<()> {
    if let Some(field) = app.editing_field {
        match key.code {
            KeyCode::Enter => {
                let buffer = std::mem::take(&mut app.edit_buffer);
                apply_edit(app, field, buffer);
                app.editing_field = None;
            }
            KeyCode::Esc => {
                app.editing_field = None;
                app.edit_buffer.clear();
            }
            KeyCode::Backspace => {
                app.edit_buffer.pop();
            }
            KeyCode::Char(c) => {
                if is_numeric_field(field) {
                    if c.is_ascii_digit() {
                        app.edit_buffer.push(c);
                    }
                } else {
                    app.edit_buffer.push(c);
                }
            }
            _ => {}
        }

        return Ok(());
    }

    match key.code {
        KeyCode::Esc | KeyCode::F(2) => {
            app.screen = Screen::Test;
        }
        KeyCode::Up => {
            app.settings_index = (app.settings_index + FIELD_COUNT - 1) % FIELD_COUNT;
        }
        KeyCode::Down | KeyCode::Tab => {
            app.settings_index = (app.settings_index + 1) % FIELD_COUNT;
        }
        KeyCode::BackTab => {
            app.settings_index = (app.settings_index + FIELD_COUNT - 1) % FIELD_COUNT;
        }
        KeyCode::Enter => {
            activate_selected_setting(app);
        }
        KeyCode::Left => {
            adjust_selected_setting(app, false);
        }
        KeyCode::Right => {
            adjust_selected_setting(app, true);
        }
        KeyCode::Char(' ') => {
            if app.settings_index == FIELD_USE_LLM {
                app.config.use_llm = !app.config.use_llm;
            } else if app.settings_index == FIELD_REGENERATE {
                app.config.regenerate_on_restart = !app.config.regenerate_on_restart;
            }
        }
        _ => {}
    }

    Ok(())
}

fn activate_selected_setting(app: &mut App) {
    match app.settings_index {
        FIELD_SAVE => {
            let tr = ui_strings(app.config.language);
            app.settings_message = tr.generating.to_string();
            save_settings(app);
        }
        FIELD_CANCEL => {
            app.screen = Screen::Test;
        }
        FIELD_USE_LLM => {
            app.config.use_llm = !app.config.use_llm;
        }
        FIELD_REGENERATE => {
            app.config.regenerate_on_restart = !app.config.regenerate_on_restart;
        }
        FIELD_LANG | FIELD_WORDS_LANG | FIELD_MODE | FIELD_TEXT_LANG | FIELD_API_PROVIDER => {
            adjust_selected_setting(app, true);
        }
        _ => {
            start_editing(app, app.settings_index);
        }
    }
}

fn adjust_selected_setting(app: &mut App, forward: bool) {
    match app.settings_index {
        FIELD_LANG => {
            app.config.language = next_language(app.config.language);
        }
        FIELD_WORDS_LANG => {
            app.config.words_language = next_optional_language(app.config.words_language, forward);
        }
        FIELD_MODE => {
            app.config.mode = next_mode(&app.config.mode);
        }
        FIELD_TEXT_LANG => {
            app.config.llm.text_language =
                next_optional_language(app.config.llm.text_language, forward);
        }
        FIELD_API_PROVIDER => {
            app.config.llm.api_provider = next_provider(app.config.llm.api_provider, forward);
        }
        FIELD_USE_LLM => {
            app.config.use_llm = !app.config.use_llm;
        }
        FIELD_REGENERATE => {
            app.config.regenerate_on_restart = !app.config.regenerate_on_restart;
        }
        FIELD_AMOUNT => {
            if forward {
                app.config.amount = app.config.amount.saturating_add(1);
            } else if app.config.amount > 1 {
                app.config.amount -= 1;
            }
        }
        FIELD_LLM_WORDS => {
            if forward {
                app.config.llm_words = app.config.llm_words.saturating_add(1);
            } else if app.config.llm_words > 1 {
                app.config.llm_words -= 1;
            }
        }
        FIELD_RANDOM_WORDS => {
            if forward {
                app.config.random_words = app.config.random_words.saturating_add(1);
            } else if app.config.random_words > 1 {
                app.config.random_words -= 1;
            }
        }
        _ => {}
    }
}

fn start_editing(app: &mut App, field: usize) {
    app.edit_buffer = match field {
        FIELD_PROMPT => app.config.llm.prompt.clone(),
        FIELD_MODEL => app.config.llm.model.clone(),
        FIELD_API_BASE => app.config.llm.api_base.clone(),
        FIELD_API_KEY => app.config.llm.api_key.clone(),
        FIELD_API_KEY_ENV => app.config.llm.api_key_env.clone(),
        FIELD_AMOUNT => app.config.amount.to_string(),
        FIELD_LLM_WORDS => app.config.llm_words.to_string(),
        FIELD_RANDOM_WORDS => app.config.random_words.to_string(),
        _ => return,
    };

    app.editing_field = Some(field);
}

fn apply_edit(app: &mut App, field: usize, buffer: String) {
    let value = buffer.trim().to_string();

    match field {
        FIELD_PROMPT => {
            app.config.llm.prompt = value;
        }
        FIELD_MODEL => {
            if !value.is_empty() {
                app.config.llm.model = value;
            }
        }
        FIELD_API_BASE => {
            if !value.is_empty() {
                app.config.llm.api_base = value;
            }
        }
        FIELD_API_KEY => {
            app.config.llm.api_key = value;
        }
        FIELD_API_KEY_ENV => {
            if !value.is_empty() {
                app.config.llm.api_key_env = value;
            }
        }
        FIELD_AMOUNT => {
            if let Ok(n) = value.parse::<u32>() {
                app.config.amount = n.max(1);
            }
        }
        FIELD_LLM_WORDS => {
            if let Ok(n) = value.parse::<u32>() {
                app.config.llm_words = n.max(1);
            }
        }
        FIELD_RANDOM_WORDS => {
            if let Ok(n) = value.parse::<u32>() {
                app.config.random_words = n.max(1);
            }
        }
        _ => {}
    }
}

fn is_numeric_field(field: usize) -> bool {
    matches!(
        field,
        FIELD_AMOUNT | FIELD_LLM_WORDS | FIELD_RANDOM_WORDS
    )
}

fn next_language(current: Language) -> Language {
    match current {
        Language::Ru => Language::En,
        Language::En => Language::Ru,
    }
}

fn next_optional_language(current: OptionalLanguage, forward: bool) -> OptionalLanguage {
    let order = [
        OptionalLanguage::Auto,
        OptionalLanguage::Ru,
        OptionalLanguage::En,
    ];

    let pos = order.iter().position(|x| *x == current).unwrap_or(0);

    let new_pos = if forward {
        (pos + 1) % order.len()
    } else {
        (pos + order.len() - 1) % order.len()
    };

    order[new_pos]
}

fn next_mode(current: &str) -> String {
    if current == "time" {
        "words".to_string()
    } else {
        "time".to_string()
    }
}

fn settings_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(5),
        ])
        .split(f.area());

    let tr = ui_strings(app.config.language);
    let labels = setting_labels(app.config.language);

    let mut lines = Vec::new();

    for i in 0..FIELD_COUNT {
        let selected = i == app.settings_index;
        let editing = app.editing_field == Some(i);

        let label = labels[i];
        let value = if editing {
            format!("{}█", app.edit_buffer)
        } else {
            setting_value(app, i)
        };

        let marker = if selected { ">" } else { " " };

        let label_style = if selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let value_style = if editing {
            Style::default().fg(Color::Cyan)
        } else {
            match i {
                FIELD_SAVE => Style::default().fg(Color::Green),
                FIELD_CANCEL => Style::default().fg(Color::Red),
                _ => Style::default().fg(Color::White),
            }
        };

        lines.push(Line::from(vec![
            Span::raw(format!("{marker} ")),
            Span::styled(format!("{label}: "), label_style),
            Span::styled(value, value_style),
        ]));
    }

    let scroll_y = app.settings_index.saturating_sub(8) as u16;

    let list = Paragraph::new(Text::from(lines))
        .wrap(Wrap { trim: false })
        .scroll((scroll_y, 0))
        .block(Block::default().borders(Borders::ALL).title(tr.settings_title));

    let help = if app.editing_field.is_some() {
        settings_edit_help(app.config.language)
    } else {
        settings_help(app.config.language)
    };

    let bottom_lines = vec![
        Line::from(Span::styled(
            app.settings_message.clone(),
            Style::default().fg(Color::Red),
        )),
        Line::from(Span::styled(help, Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled(
            format!("config: {}", app.config_path.display()),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let bottom = Paragraph::new(Text::from(bottom_lines))
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).title("help"));

    let top = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} ", tr.settings_title),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            format!(" {} ", tr.title),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));

    f.render_widget(top, chunks[0]);
    f.render_widget(list, chunks[1]);
    f.render_widget(bottom, chunks[2]);
}

fn setting_labels(lang: Language) -> [&'static str; FIELD_COUNT] {
    match lang {
        Language::Ru => [
            "Язык интерфейса",
            "Язык слов",
            "Режим",
            "Время/слова",
            "Генерация через LLM",
            "Промт для ИИ",
            "Язык генерации",
            "Модель",
            "Тип API",
            "API base",
            "API ключ",
            "Переменная ключа",
            "Слов для LLM",
            "Случайных слов",
            "Регенерация при сбросе",
            "Сохранить",
            "Отмена",
        ],
        Language::En => [
            "Interface language",
            "Words language",
            "Mode",
            "Time/words",
            "Use LLM",
            "AI prompt",
            "Text language",
            "Model",
            "API provider",
            "API base",
            "API key",
            "API key env",
            "LLM words",
            "Random words",
            "Regenerate on reset",
            "Save",
            "Cancel",
        ],
    }
}

fn setting_value(app: &App, index: usize) -> String {
    let lang = app.config.language;

    match index {
        FIELD_LANG => match app.config.language {
            Language::Ru => "ru".to_string(),
            Language::En => "en".to_string(),
        },
        FIELD_WORDS_LANG => optional_language_label(app.config.words_language),
        FIELD_MODE => app.config.mode.clone(),
        FIELD_AMOUNT => app.config.amount.to_string(),
        FIELD_USE_LLM => bool_label(app.config.use_llm, lang),
        FIELD_PROMPT => {
            if app.config.llm.prompt.trim().is_empty() {
                empty_label(lang).to_string()
            } else {
                truncate_string(&app.config.llm.prompt, 60)
            }
        }
        FIELD_TEXT_LANG => optional_language_label(app.config.llm.text_language),
        FIELD_MODEL => app.config.llm.model.clone(),
        FIELD_API_PROVIDER => provider_label(app.config.llm.api_provider),
        FIELD_API_BASE => app.config.llm.api_base.clone(),
        FIELD_API_KEY => {
            if app.config.llm.api_key.trim().is_empty() {
                empty_label(lang).to_string()
            } else {
                "********".to_string()
            }
        }
        FIELD_API_KEY_ENV => app.config.llm.api_key_env.clone(),
        FIELD_LLM_WORDS => app.config.llm_words.to_string(),
        FIELD_RANDOM_WORDS => app.config.random_words.to_string(),
        FIELD_REGENERATE => bool_label(app.config.regenerate_on_restart, lang),
        FIELD_SAVE => String::new(),
        FIELD_CANCEL => String::new(),
        _ => String::new(),
    }
}

fn optional_language_label(value: OptionalLanguage) -> String {
    match value {
        OptionalLanguage::Auto => "auto".to_string(),
        OptionalLanguage::Ru => "ru".to_string(),
        OptionalLanguage::En => "en".to_string(),
    }
}

fn bool_label(value: bool, lang: Language) -> String {
    match lang {
        Language::Ru => {
            if value {
                "да".to_string()
            } else {
                "нет".to_string()
            }
        }
        Language::En => {
            if value {
                "yes".to_string()
            } else {
                "no".to_string()
            }
        }
    }
}

fn empty_label(lang: Language) -> &'static str {
    match lang {
        Language::Ru => "(пусто)",
        Language::En => "(empty)",
    }
}

fn truncate_string(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();

    if chars.len() <= max {
        s.to_string()
    } else {
        let mut out: String = chars[..max].iter().collect();
        out.push('…');
        out
    }
}

fn settings_help(lang: Language) -> &'static str {
    match lang {
        Language::Ru => "↑/↓ выбор | ←/→ изменить | enter переключить/редактировать | пробел да/нет | esc отмена | F2 назад",
        Language::En => "↑/↓ select | ←/→ change | enter toggle/edit | space yes/no | esc cancel | F2 back",
    }
}

fn settings_edit_help(lang: Language) -> &'static str {
    match lang {
        Language::Ru => "ввод — применить | esc — отмена",
        Language::En => "enter — apply | esc — cancel",
    }
}

fn save_settings(app: &mut App) {
    if let Err(e) = save_config(&app.config_path, &app.config) {
        app.settings_message = match app.config.language {
            Language::Ru => format!("Ошибка сохранения настроек: {e}"),
            Language::En => format!("Failed to save settings: {e}"),
        };
        return;
    }

    let mut new_settings = settings_from_config(app.config.clone());
    let (words, error_message) = generate_words_with_fallback(&mut new_settings);

    app.settings = new_settings;
    app.reset(Some(words));
    app.message = error_message.unwrap_or_default();
    app.screen = Screen::Test;
}

fn save_config(path: &Path, config: &Config) -> Result<()> {
    let body = toml::to_string_pretty(config).context("failed to serialize config")?;
    let content = format!("# RustTypingTrainer settings (saved by app)\n\n{body}");

    fs::write(path, content)
        .with_context(|| format!("failed to write config file: {}", path.display()))?;

    Ok(())
}

fn regenerate_and_reset(app: &mut App) {
    let (words, error_message) = generate_words_with_fallback(&mut app.settings);
    app.reset(Some(words));
    app.message = error_message.unwrap_or_default();
}

fn generate_words_with_fallback(settings: &mut Settings) -> (Vec<String>, Option<String>) {
    let tr = ui_strings(settings.lang);

    match generate_words(settings) {
        Ok(words) => (words, None),
        Err(e) => {
            let message = format!("{}: {e}", tr.gen_error);
            let mut fallback = settings.clone();
            fallback.use_llm = false;

            match generate_words(&fallback) {
                Ok(words) => {
                    *settings = fallback;
                    (words, Some(message))
                }
                Err(_) => {
                    let lang = fallback.words_lang.unwrap_or(fallback.lang);
                    let words = random_words(120, None, lang);
                    *settings = fallback;
                    (words, Some(message))
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(f.area());

    let tr = ui_strings(app.settings.lang);
    let stats = app.stats();

    let mode = if app.settings.mode == "time" {
        format!(
            " {} {}{} ",
            tr.mode_time, app.settings.amount, tr.time_suffix
        )
    } else {
        format!(" {} {} ", tr.mode_words, app.words.len())
    };

    let time = if let Some(limit) = app.time_limit {
        let remaining = limit.checked_sub(stats.elapsed).unwrap_or(Duration::ZERO);
        format!("{:.0}{}", remaining.as_secs_f64(), tr.time_suffix)
    } else {
        format!("{:.1}{}", stats.elapsed.as_secs_f64(), tr.time_suffix)
    };

    let top_line = Line::from(vec![
        Span::styled(mode, Style::default().fg(Color::Cyan)),
        Span::styled(format!(" {time} "), Style::default().fg(Color::Yellow)),
        Span::styled(
            format!(" {} {:.0} ", tr.wpm_short, stats.wpm),
            Style::default().fg(Color::Green),
        ),
        Span::styled(
            format!(" {} {:.1}% ", tr.acc_short, stats.acc),
            Style::default().fg(Color::Magenta),
        ),
        Span::styled(
            format!(" {}", app.message),
            Style::default().fg(Color::Red),
        ),
    ]);

    let top = Paragraph::new(top_line)
        .block(Block::default().borders(Borders::ALL).title(tr.title));

    let middle = if app.finished {
        results_paragraph(app, &stats)
    } else {
        words_paragraph(app)
    };

    let help = if app.result_locked() {
        match app.settings.lang {
            Language::Ru => "подождите, результат заблокирован...",
            Language::En => "please wait, result locked...",
        }
    } else if app.finished {
        tr.finished_help
    } else {
        tr.active_help
    };

    let bottom = Paragraph::new(help)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title("keys"));

    f.render_widget(top, chunks[0]);
    f.render_widget(middle, chunks[1]);
    f.render_widget(bottom, chunks[2]);
}

fn words_paragraph(app: &App) -> Paragraph<'static> {
    let tr = ui_strings(app.settings.lang);
    let mut spans: Vec<Span> = Vec::new();

    if app.words.is_empty() {
        return Paragraph::new(tr.no_words)
            .block(Block::default().borders(Borders::ALL).title(tr.text_title));
    }

    let window_start = app.current_word.saturating_sub(40);
    let window_end = (window_start + 140).min(app.words.len());

    if window_start > 0 {
        spans.push(Span::raw("… "));
    }

    let mut first = true;

    for i in window_start..window_end {
        if !first {
            spans.push(Span::raw(" "));
        }
        first = false;

        let target = &app.words[i];
        let typed = app
            .typed_words
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("");

        let target_chars: Vec<char> = target.chars().collect();
        let typed_chars: Vec<char> = typed.chars().collect();

        for (j, target_char) in target_chars.iter().enumerate() {
            let mut style = if j < typed_chars.len() {
                if typed_chars[j] == *target_char {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                }
            } else {
                Style::default().fg(Color::DarkGray)
            };

            if !app.finished && i == app.current_word && j == typed_chars.len() {
                style = style.add_modifier(Modifier::UNDERLINED);
            }

            spans.push(Span::styled(target_char.to_string(), style));
        }

        if typed_chars.len() > target_chars.len() {
            for (j, extra_char) in typed_chars.iter().enumerate() {
                if j < target_chars.len() {
                    continue;
                }

                spans.push(Span::styled(
                    extra_char.to_string(),
                    Style::default().fg(Color::Red),
                ));
            }
        }

        if !app.finished && i == app.current_word && typed_chars.len() >= target_chars.len() {
            spans.push(Span::styled("▌", Style::default().fg(Color::White)));
        }
    }

    if window_end < app.words.len() {
        spans.push(Span::raw(" …"));
    }

    let progress = format!(
        "{}/{}",
        app.current_word.min(app.words.len().saturating_sub(1)) + 1,
        app.words.len()
    );

    Paragraph::new(Line::from(spans))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{} {progress}", tr.text_title)),
        )
}

fn results_paragraph(app: &App, stats: &Stats) -> Paragraph<'static> {
    let tr = ui_strings(app.settings.lang);
    let errors = app.total_keystrokes.saturating_sub(app.correct_keystrokes);

    let mut lines = vec![
        Line::from(Span::styled(
            tr.result_title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw(format!("{}: {:.0}", tr.wpm, stats.wpm))),
        Line::from(Span::raw(format!("{}: {:.1}%", tr.acc, stats.acc))),
        Line::from(Span::raw(format!(
            "{}: {:.0} ({} {})",
            tr.raw, stats.raw, stats.raw_chars, tr.raw_keys
        ))),
        Line::from(Span::raw(format!(
            "{}: {:.1}{}",
            tr.time_label,
            stats.elapsed.as_secs_f64(),
            tr.time_suffix
        ))),
        Line::from(Span::raw(format!(
            "{}: {} {} / {} {}",
            tr.chars_label, stats.correct_chars, tr.chars_correct, stats.typed_chars, tr.chars_typed
        ))),
        Line::from(Span::raw(format!("{}: {}", tr.errors_label, errors))),
    ];

    if let Some(remaining) = app.result_lock_remaining() {
        let secs = remaining.as_secs_f64().ceil();

        let msg = match app.settings.lang {
            Language::Ru => format!("Результат заблокирован: {:.0} с", secs),
            Language::En => format!("Result locked: {:.0}s", secs),
        };

        lines.push(Line::from(Span::styled(
            msg,
            Style::default().fg(Color::Yellow),
        )));
    }

    Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title(tr.results_title))
}

fn load_or_create_config(path: &Path) -> Result<Config> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).ok();
            }
        }

        fs::write(path, DEFAULT_CONFIG)
            .with_context(|| format!("failed to write default config to {}", path.display()))?;

        eprintln!("Создан файл настроек: {}", path.display());
    }

    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file: {}", path.display()))?;

    let config: Config = toml::from_str(&text)
        .with_context(|| format!("failed to parse config file: {}", path.display()))?;

    Ok(config)
}

fn resolve_settings(args: Args, config: Config) -> Settings {
    let lang = args.lang.unwrap_or(config.language);

    let words_lang = args
        .words_lang
        .unwrap_or(config.words_language)
        .to_option();

    let mode = args.mode.unwrap_or(config.mode);
    let amount = args.amount.unwrap_or(config.amount).max(1);
    let llm_words = args.llm_words.unwrap_or(config.llm_words).max(1);
    let random_words = args.random_words.unwrap_or(config.random_words).max(1);

    let words_file = args
        .words_file
        .clone()
        .or(config.words_file)
        .filter(|p| !p.as_os_str().is_empty());

    let use_llm = if args.prompt.is_some() || args.llm {
        true
    } else if args.random {
        false
    } else {
        config.use_llm
    };

    let prompt = args.prompt.clone().unwrap_or(config.llm.prompt.clone());
    let prompt = if prompt.trim().is_empty() {
        default_prompt(lang)
    } else {
        prompt.trim().to_string()
    };

    let text_lang = args
        .text_lang
        .unwrap_or(config.llm.text_language)
        .to_option();

    let model = args.model.unwrap_or(config.llm.model);
    let provider = args.api_provider.unwrap_or(config.llm.api_provider);
    let api_base = args.api_base.unwrap_or(config.llm.api_base);
    let api_key_env = args.api_key_env.unwrap_or(config.llm.api_key_env);

    let api_key = args.api_key.clone().or_else(|| {
        let key = config.llm.api_key.trim().to_string();
        if key.is_empty() {
            None
        } else {
            Some(key)
        }
    });

    let regenerate_on_restart = args.regenerate_on_restart || config.regenerate_on_restart;

    Settings {
        lang,
        words_lang,
        mode,
        amount,
        use_llm,
        llm_words,
        random_words,
        words_file,
        regenerate_on_restart,
        llm: LlmSettings {
            prompt,
            text_lang,
            model,
            provider,
            api_base,
            api_key,
            api_key_env,
        },
    }
}

fn settings_from_config(config: Config) -> Settings {
    let lang = config.language;
    let words_lang = config.words_language.to_option();

    let prompt = if config.llm.prompt.trim().is_empty() {
        default_prompt(lang)
    } else {
        config.llm.prompt.trim().to_string()
    };

    let api_key = if config.llm.api_key.trim().is_empty() {
        None
    } else {
        Some(config.llm.api_key.trim().to_string())
    };

    Settings {
        lang,
        words_lang,
        mode: config.mode,
        amount: config.amount.max(1),
        use_llm: config.use_llm,
        llm_words: config.llm_words.max(1),
        random_words: config.random_words.max(1),
        words_file: config.words_file.filter(|p| !p.as_os_str().is_empty()),
        regenerate_on_restart: config.regenerate_on_restart,
        llm: LlmSettings {
            prompt,
            text_lang: config.llm.text_language.to_option(),
            model: config.llm.model,
            provider: config.llm.api_provider,
            api_base: config.llm.api_base,
            api_key,
            api_key_env: config.llm.api_key_env,
        },
    }
}

fn config_from_settings(settings: &Settings) -> Config {
    Config {
        language: settings.lang,
        words_language: optional_language_from(settings.words_lang),
        mode: settings.mode.clone(),
        amount: settings.amount,
        use_llm: settings.use_llm,
        llm_words: settings.llm_words,
        random_words: settings.random_words,
        words_file: settings.words_file.clone(),
        regenerate_on_restart: settings.regenerate_on_restart,
        llm: LlmConfig {
            prompt: settings.llm.prompt.clone(),
            text_language: optional_language_from(settings.llm.text_lang),
            model: settings.llm.model.clone(),
            api_provider: settings.llm.provider,
            api_base: settings.llm.api_base.clone(),
            api_key: settings.llm.api_key.clone().unwrap_or_default(),
            api_key_env: settings.llm.api_key_env.clone(),
        },
    }
}

fn default_prompt(lang: Language) -> String {
    match lang {
        Language::Ru => "Напиши короткий связный текст для тренировки печати.".to_string(),
        Language::En => "Write a short coherent typing test passage.".to_string(),
    }
}

fn generate_words(settings: &Settings) -> Result<Vec<String>> {
    let target_words = if settings.mode == "words" {
        settings.amount.max(1) as usize
    } else if settings.use_llm {
        settings.llm_words.max(1) as usize
    } else {
        settings.random_words.max(1) as usize
    };

    if settings.use_llm {
        let text = llm_generate(settings, target_words as u32)?;

        let mut words: Vec<String> = text
            .split_whitespace()
            .map(sanitize_word)
            .filter(|w| !w.is_empty())
            .collect();

        if words.is_empty() {
            return Err(anyhow!("LLM returned no usable words"));
        }

        if words.len() > target_words {
            words.truncate(target_words);
        }

        return Ok(words);
    }

    let custom_words = match &settings.words_file {
        Some(path) => Some(load_word_file(path)?),
        None => None,
    };

    let lang = settings.words_lang.unwrap_or(settings.lang);

    Ok(random_words(target_words, custom_words.as_deref(), lang))
}

fn load_word_file(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("cannot read word file: {}", path.display()))?;

    Ok(content
        .split_whitespace()
        .map(|s| s.to_string())
        .collect())
}

fn random_words(n: usize, custom: Option<&[String]>, lang: Language) -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut out = Vec::with_capacity(n);

    for _ in 0..n {
        let word = match custom {
            Some(list) if !list.is_empty() => list[rng.gen_range(0..list.len())].clone(),
            _ => {
                let list: &[&str] = match lang {
                    Language::Ru => RU_WORDS,
                    Language::En => EN_WORDS,
                };

                list[rng.gen_range(0..list.len())].to_string()
            }
        };

        out.push(word);
    }

    out
}

fn sanitize_word(word: &str) -> String {
    word.trim_matches(|c: char| {
        c.is_whitespace()
            || matches!(
                c,
                '.' | ',' | ';' | ':' | '!' | '?' | '(' | ')' | '[' | ']' | '{' | '}'
                    | '"' | '\'' | '\u{2018}' | '\u{2019}' | '\u{201C}' | '\u{201D}'
                    | '\u{00AB}' | '\u{00BB}' | '\u{201E}' | '\u{2014}' | '\u{2013}'
            )
    })
    .to_string()
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageContent,
}

#[derive(Deserialize)]
struct ChatMessageContent {
    content: Option<String>,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<AnthropicMessage>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicBlock>,
}

#[derive(Deserialize)]
struct AnthropicBlock {
    text: Option<String>,
}

fn llm_generate(settings: &Settings, target_words: u32) -> Result<String> {
    let mut api_key = settings.llm.api_key.clone().unwrap_or_default();

    if api_key.is_empty() {
        api_key = env::var(&settings.llm.api_key_env).unwrap_or_default();
    }

    let provider = match settings.llm.provider {
        ApiProvider::Auto => detect_provider(&settings.llm.api_base),
        p => p,
    };

    if api_key.is_empty() {
        let local = settings.llm.api_base.contains("127.0.0.1")
            || settings.llm.api_base.contains("localhost");

        if local || provider == ApiProvider::LmStudio || provider == ApiProvider::Ollama {
            api_key = "rusttype-local".to_string();
        } else {
            let message = match settings.lang {
                Language::Ru => format!(
                    "API-ключ не найден. Укажи его в настройках (F2), в переменной {} или через --api-key.",
                    settings.llm.api_key_env
                ),
                Language::En => format!(
                    "API key not found. Set it in settings (F2), in {} environment variable, or pass --api-key.",
                    settings.llm.api_key_env
                ),
            };

            return Err(anyhow!(message));
        }
    }

    let base = settings.llm.api_base.trim().trim_end_matches('/');
    let base_v1 = if base.ends_with("/v1") {
        base.to_string()
    } else {
        format!("{base}/v1")
    };

    let language_instruction = match settings.llm.text_lang {
        Some(Language::Ru) => "Write in Russian.",
        Some(Language::En) => "Write in English.",
        None => "Write in the language of the prompt.",
    };

    let system = format!(
        "You generate text for a typing test. Return ONLY plain text without markdown, numbering, quotes or lists. Prefer exactly {target_words} words separated by single spaces. {language_instruction}"
    );

    let user = format!(
        "Prompt: {}\n\nGenerate the typing test text now.",
        settings.llm.prompt
    );

    let max_tokens = target_words.saturating_mul(6).max(128).min(4096);

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .context("failed to build HTTP client")?;

    if provider == ApiProvider::Anthropic {
        let url = format!("{base_v1}/messages");

        let request = AnthropicRequest {
            model: settings.llm.model.clone(),
            max_tokens,
            system,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: user,
            }],
        };

        let response = client
            .post(&url)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .context("failed to send LLM request")?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("LLM API returned {status}: {body}"));
        }

        let parsed: AnthropicResponse = response.json().context("failed to parse LLM JSON")?;

        let content: String = parsed
            .content
            .iter()
            .filter_map(|b| b.text.clone())
            .collect::<Vec<_>>()
            .join("");

        return Ok(content.trim().to_string());
    }

    let url = format!("{base_v1}/chat/completions");

    let request = ChatRequest {
        model: settings.llm.model.clone(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system,
            },
            ChatMessage {
                role: "user".to_string(),
                content: user,
            },
        ],
        temperature: 0.7,
        max_tokens,
    };

    let response = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .context("failed to send LLM request")?;

    let status = response.status();

    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        return Err(anyhow!("LLM API returned {status}: {body}"));
    }

    let parsed: ChatResponse = response.json().context("failed to parse LLM JSON")?;

    let content = parsed
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("LLM response has no choices"))?
        .message
        .content
        .unwrap_or_default();

    Ok(content.trim().to_string())
}