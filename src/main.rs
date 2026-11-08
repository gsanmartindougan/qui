use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{
    Alignment, Background, Border, Color, Element, Length, Shadow, Size, Subscription, Task, Theme,
};
use rfd::AsyncFileDialog;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const INK: Color = Color::from_rgb(0.067, 0.094, 0.153);
const MUTED: Color = Color::from_rgb(0.322, 0.384, 0.478);
const BLUE: Color = Color::from_rgb(0.145, 0.388, 0.922);
const PALE_BLUE: Color = Color::from_rgb(0.918, 0.945, 1.0);
const BORDER: Color = Color::from_rgb(0.82, 0.86, 0.92);
const BACKGROUND: Color = Color::from_rgb(0.953, 0.969, 0.988);
const ERROR: Color = Color::from_rgb(0.706, 0.137, 0.094);

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Qui")
        .window_size(Size::new(430.0, 610.0))
        .resizable(false)
        .centered()
        .theme(|_: &App| Theme::Light)
        .subscription(App::subscription)
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    Spanish,
    English,
}

fn tr(language: Language, spanish: &'static str, english: &'static str) -> &'static str {
    match language {
        Language::Spanish => spanish,
        Language::English => english,
    }
}

fn system_language() -> Language {
    let locale = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();
    if locale.to_ascii_lowercase().starts_with("es") {
        Language::Spanish
    } else {
        Language::English
    }
}

fn load_language() -> Language {
    match config_dir()
        .and_then(|directory| fs::read_to_string(directory.join("language")).ok())
        .as_deref()
        .map(str::trim)
    {
        Some("es") => Language::Spanish,
        Some("en") => Language::English,
        _ => system_language(),
    }
}

fn save_language(language: Language) -> Result<(), String> {
    let directory = config_dir().ok_or(tr(
        language,
        "No encuentro tu directorio de configuración.",
        "Could not find your configuration directory.",
    ))?;
    fs::create_dir_all(&directory).map_err(|error| {
        format!(
            "{} {}: {error}",
            tr(language, "No pude crear", "Could not create"),
            directory.display()
        )
    })?;
    let path = directory.join("language");
    let value = match language {
        Language::Spanish => b"es".as_slice(),
        Language::English => b"en".as_slice(),
    };
    atomic_write(&path, value).map_err(|error| {
        format!(
            "{} {}: {error}",
            tr(language, "No pude guardar", "Could not save"),
            path.display()
        )
    })
}

fn brand_logo() -> Element<'static, Message> {
    row![
        container(text("q").size(30).color(Color::WHITE))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fixed(44.0))
            .height(Length::Fixed(44.0))
            .style(|_| container::Style {
                background: Some(Background::Color(BLUE)),
                border: Border::default().rounded(10.0),
                ..Default::default()
            }),
        row![
            text("u").size(30).color(INK),
            text("i").size(30).color(BLUE)
        ]
        .spacing(0),
    ]
    .spacing(2)
    .align_y(Alignment::Center)
    .into()
}

#[derive(Debug, Clone)]
enum Message {
    EscapePressed,
    ToggleLanguage,
    Activate(String),
    SwitchFinished(SwitchOutcome),
    StartLogin,
    LoginStarted(String),
    OpenAddAccount,
    CancelAddAccount,
    FormNameChanged(String),
    FormLoginChanged(String),
    FormKeyPathChanged(String),
    FormKeyModeChanged(KeyMode),
    PickKeyPath,
    KeyPathPicked(Option<PathBuf>),
    SaveAccount,
    AccountSaved(Result<AccountSaved, String>),
    CopyPublicKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyMode {
    Existing,
    Create,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Selector,
    AddAccount,
    PublicKey,
}

struct App {
    language: Language,
    profiles: Vec<Profile>,
    accounts: Vec<GhAccount>,
    active_profile: Option<String>,
    selected_profile: Option<String>,
    previous_profile: Option<String>,
    busy: bool,
    message: Option<String>,
    login_needed: bool,
    login_command: Option<String>,
    load_error: Option<String>,
    screen: Screen,
    form_name: String,
    form_login: String,
    form_key_path: String,
    form_key_mode: KeyMode,
    form_error: Option<String>,
    public_key_path: Option<PathBuf>,
    public_key: String,
    copied_public_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Profile {
    id: String,
    name: String,
    #[serde(default)]
    user_name: String,
    #[serde(default)]
    email: String,
    ssh_identity: PathBuf,
    #[serde(default)]
    gh_login: Option<String>,
}

#[derive(Debug, Clone)]
struct AccountSaved {
    profile: Profile,
    public_key_path: PathBuf,
    public_key: String,
}

#[derive(Debug)]
struct SshAlias {
    host: String,
    identity: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct GhAccount {
    #[serde(default)]
    login: String,
    #[serde(default)]
    active: bool,
    #[serde(default)]
    state: String,
    error: Option<String>,
}

#[derive(Deserialize)]
struct GhStatus {
    hosts: HashMap<String, Vec<GhAccount>>,
}

#[derive(Debug, Clone)]
struct SwitchOutcome {
    profile_id: String,
    ssh_changed: bool,
    previous_profile: Option<String>,
    message: String,
    login_needed: bool,
    accounts: Vec<GhAccount>,
}

impl App {
    fn new() -> Self {
        let language = load_language();
        match load_profiles(language) {
            Ok(mut profiles) => {
                let accounts = load_gh_accounts();
                map_gh_accounts(&mut profiles, &accounts);
                let active_profile = read_active_profile(&profiles, &accounts);
                Self {
                    language,
                    profiles,
                    accounts,
                    active_profile,
                    selected_profile: None,
                    previous_profile: None,
                    busy: false,
                    message: None,
                    login_needed: false,
                    login_command: None,
                    load_error: None,
                    screen: Screen::Selector,
                    form_name: String::new(),
                    form_login: String::new(),
                    form_key_path: String::new(),
                    form_key_mode: KeyMode::Existing,
                    form_error: None,
                    public_key_path: None,
                    public_key: String::new(),
                    copied_public_key: false,
                }
            }
            Err(error) => Self {
                language,
                profiles: Vec::new(),
                accounts: Vec::new(),
                active_profile: None,
                selected_profile: None,
                previous_profile: None,
                busy: false,
                message: None,
                login_needed: false,
                login_command: None,
                load_error: Some(error),
                screen: Screen::Selector,
                form_name: String::new(),
                form_login: String::new(),
                form_key_path: String::new(),
                form_key_mode: KeyMode::Existing,
                form_error: None,
                public_key_path: None,
                public_key: String::new(),
                copied_public_key: false,
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let escape = if self.screen == Screen::Selector {
            iced::event::listen_with(|event, status, _window| match (event, status) {
                (
                    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                        ..
                    }),
                    iced::event::Status::Ignored,
                ) => Some(Message::EscapePressed),
                _ => None,
            })
        } else {
            Subscription::none()
        };

        escape
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EscapePressed => iced::exit(),
            Message::ToggleLanguage => {
                self.language = match self.language {
                    Language::Spanish => Language::English,
                    Language::English => Language::Spanish,
                };
                if let Err(error) = save_language(self.language) {
                    self.message = Some(format!(
                        "{}: {error}",
                        tr(
                            self.language,
                            "No se pudo guardar la preferencia de idioma",
                            "Could not save language preference"
                        )
                    ));
                }
                Task::none()
            }
            Message::Activate(id) if !self.busy => {
                let Some(profile) = self
                    .profiles
                    .iter()
                    .find(|profile| profile.id == id)
                    .cloned()
                else {
                    return Task::none();
                };
                let previous = self.active_profile.clone();
                self.selected_profile = Some(id);
                self.previous_profile = previous.clone();
                self.message = Some(format!(
                    "{} {}…",
                    tr(self.language, "Activando", "Activating"),
                    profile.name
                ));
                self.login_command = None;
                self.login_needed = false;
                self.busy = true;
                let language = self.language;
                Task::perform(
                    async move { activate_profile(profile, previous, language) },
                    Message::SwitchFinished,
                )
            }
            Message::Activate(_) => Task::none(),
            Message::SwitchFinished(outcome) => {
                self.busy = false;
                if outcome.ssh_changed {
                    self.active_profile = Some(outcome.profile_id.clone());
                    self.previous_profile = outcome.previous_profile;
                }
                self.message = Some(outcome.message);
                self.login_needed = outcome.login_needed;
                self.accounts = outcome.accounts;
                map_gh_accounts(&mut self.profiles, &self.accounts);
                Task::none()
            }
            Message::StartLogin => {
                let Some(id) = self.selected_profile.as_deref() else {
                    return Task::none();
                };
                let Some(profile) = self
                    .profiles
                    .iter()
                    .find(|profile| profile.id == id)
                    .cloned()
                else {
                    return Task::none();
                };
                let refresh = profile.gh_login.as_ref().is_some_and(|login| {
                    self.accounts
                        .iter()
                        .any(|account| account.login.eq_ignore_ascii_case(login))
                });
                let gh_login = profile.gh_login.clone();
                let language = self.language;
                Task::perform(
                    async move { start_gh_login(gh_login.as_deref(), refresh, language) },
                    Message::LoginStarted,
                )
            }
            Message::LoginStarted(message) => {
                self.message = Some(message);
                self.login_command = Some(tr(
                    self.language,
                    "Tras completar la autenticación, vuelve a activar el perfil para actualizar el estado.",
                    "After signing in, activate the profile again to refresh its status.",
                ).into());
                self.login_needed = false;
                Task::none()
            }
            Message::OpenAddAccount => {
                self.form_name.clear();
                self.form_login.clear();
                self.form_key_path.clear();
                self.form_key_mode = KeyMode::Existing;
                self.form_error = None;
                self.screen = Screen::AddAccount;
                Task::none()
            }
            Message::CancelAddAccount => {
                self.form_error = None;
                self.screen = Screen::Selector;
                Task::none()
            }
            Message::FormNameChanged(value) => {
                self.form_name = value;
                self.form_error = None;
                Task::none()
            }
            Message::FormLoginChanged(value) => {
                self.form_login = value;
                self.form_error = None;
                Task::none()
            }
            Message::FormKeyPathChanged(value) => {
                self.form_key_path = value;
                self.form_error = None;
                Task::none()
            }
            Message::FormKeyModeChanged(mode) => {
                self.form_key_mode = mode;
                self.form_error = None;
                Task::none()
            }
            Message::PickKeyPath => {
                let directory = home_dir()
                    .map(|home| home.join(".ssh"))
                    .filter(|path| path.is_dir());
                let title = if self.form_key_mode == KeyMode::Create {
                    tr(self.language, "Crear clave SSH", "Create SSH key")
                } else {
                    tr(
                        self.language,
                        "Seleccionar clave SSH privada",
                        "Select SSH private key",
                    )
                };
                let dialog = if let Some(directory) = directory {
                    AsyncFileDialog::new()
                        .set_title(title)
                        .set_directory(directory)
                } else {
                    AsyncFileDialog::new().set_title(title)
                };
                if self.form_key_mode == KeyMode::Create {
                    Task::perform(
                        async move {
                            dialog
                                .set_file_name("qui_ed25519")
                                .save_file()
                                .await
                                .map(|handle| handle.path().to_path_buf())
                        },
                        Message::KeyPathPicked,
                    )
                } else {
                    Task::perform(
                        async move {
                            dialog
                                .pick_file()
                                .await
                                .map(|handle| handle.path().to_path_buf())
                        },
                        Message::KeyPathPicked,
                    )
                }
            }
            Message::KeyPathPicked(Some(path)) => {
                self.form_key_path = path.to_string_lossy().into_owned();
                self.form_error = None;
                Task::none()
            }
            Message::KeyPathPicked(None) => Task::none(),
            Message::SaveAccount if !self.busy => {
                let name = self.form_name.trim().to_owned();
                let login = self.form_login.trim().to_owned();
                let key_path = expand_home(self.form_key_path.trim());
                let key_mode = self.form_key_mode;
                if name.is_empty() {
                    self.form_error = Some(
                        tr(
                            self.language,
                            "Escribe un nombre para la cuenta.",
                            "Enter a name for the account.",
                        )
                        .into(),
                    );
                    return Task::none();
                }
                if login.is_empty() || login.chars().any(char::is_whitespace) {
                    self.form_error = Some(
                        tr(
                            self.language,
                            "Escribe un login de GitHub válido, sin espacios.",
                            "Enter a valid GitHub username without spaces.",
                        )
                        .into(),
                    );
                    return Task::none();
                }
                if self.form_key_path.trim().is_empty() {
                    self.form_error = Some(
                        tr(
                            self.language,
                            "Elige la ruta de la clave SSH.",
                            "Choose the SSH key path.",
                        )
                        .into(),
                    );
                    return Task::none();
                }
                if self.profiles.iter().any(|profile| {
                    profile.name.eq_ignore_ascii_case(&name)
                        || profile
                            .gh_login
                            .as_deref()
                            .is_some_and(|existing| existing.eq_ignore_ascii_case(&login))
                }) {
                    self.form_error = Some(
                        tr(
                            self.language,
                            "Ya existe un perfil con ese nombre o login de GitHub.",
                            "A profile with that name or GitHub username already exists.",
                        )
                        .into(),
                    );
                    return Task::none();
                }
                self.busy = true;
                self.form_error = None;
                let language = self.language;
                Task::perform(
                    async move { register_account(name, login, key_path, key_mode, language) },
                    Message::AccountSaved,
                )
            }
            Message::SaveAccount => Task::none(),
            Message::AccountSaved(result) => {
                self.busy = false;
                match result {
                    Ok(saved) => {
                        self.profiles.push(saved.profile);
                        self.public_key_path = Some(saved.public_key_path);
                        self.public_key = saved.public_key;
                        self.copied_public_key = false;
                        self.load_error = None;
                        self.screen = Screen::PublicKey;
                    }
                    Err(error) => self.form_error = Some(error),
                }
                Task::none()
            }
            Message::CopyPublicKey => {
                self.copied_public_key = true;
                iced::clipboard::write(self.public_key.clone())
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Selector => self.selector_view(),
            Screen::AddAccount => self.add_account_view(),
            Screen::PublicKey => self.public_key_view(),
        }
    }

    fn selector_view(&self) -> Element<'_, Message> {
        let mut content = column![
            row![
                brand_logo(),
                text(tr(self.language, "Selector Git", "Git account switcher"))
                    .size(14)
                    .color(INK),
                Space::new().width(Length::Fill),
                language_button(self.language),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            text(tr(self.language, "PERFILES", "PROFILES"))
                .size(11)
                .color(MUTED),
        ]
        .spacing(10)
        .width(Length::Fill);

        if let Some(error) = &self.load_error {
            content = content.push(info_panel(error, true));
        }
        if self.profiles.is_empty() {
            content = content.push(info_panel(
                tr(self.language, "No hay cuentas guardadas todavía. Pulsa «Añadir cuenta» para crear el primer perfil.", "No accounts saved yet. Choose “Add account” to create your first profile."),
                false,
            ));
        } else {
            for profile in &self.profiles {
                content = content.push(profile_card(
                    profile,
                    self.active_profile.as_deref() == Some(profile.id.as_str()),
                    account_status(profile, &self.accounts, self.language),
                    self.busy,
                    self.language,
                ));
            }
        }

        content = content.push(
            button(tr(self.language, "+ Añadir cuenta", "+ Add account"))
                .width(Length::Fill)
                .style(button::primary)
                .on_press_maybe((!self.busy).then_some(Message::OpenAddAccount)),
        );
        content = content.push(
            container(
                column![
                    text(tr(
                        self.language,
                        "Autoría de commits",
                        "Commit author identity"
                    ))
                    .size(13)
                    .color(BLUE),
                    text(tr(
                        self.language,
                        "Se conserva por directorio según includeIf.",
                        "Remains directory-specific through includeIf."
                    ))
                    .size(12)
                    .color(BLUE),
                    text(tr(
                        self.language,
                        "SSH y gh cambian globalmente.",
                        "SSH and gh switch globally."
                    ))
                    .size(12)
                    .color(BLUE),
                ]
                .spacing(6),
            )
            .width(Length::Fill)
            .padding(14)
            .style(|_| container::Style {
                background: Some(Background::Color(PALE_BLUE)),
                border: Border::default().rounded(10.0),
                ..Default::default()
            }),
        );

        if let Some(message) = &self.message {
            content = content.push(info_panel(message, self.login_needed));
        }
        if self.login_needed {
            content = content.push(
                button(tr(self.language, "Iniciar sesión en gh", "Sign in to gh"))
                    .style(button::primary)
                    .on_press(Message::StartLogin),
            );
        }
        if let Some(command) = &self.login_command {
            content = content.push(info_panel(command, false));
        }
        if !self.busy {
            if let Some(previous) = self
                .previous_profile
                .as_deref()
                .and_then(|id| self.profile(id))
            {
                if self.active_profile.as_deref() != Some(previous.id.as_str()) {
                    content = content.push(
                        button(text(format!(
                            "{} {}",
                            tr(self.language, "Volver a", "Switch back to"),
                            previous.name
                        )))
                        .style(button::secondary)
                        .on_press(Message::Activate(previous.id.clone())),
                    );
                }
            }
        }
        content = content.push(
            text(tr(
                self.language,
                "Tab recorre · Enter activa · Esc cierra",
                "Tab navigates · Enter selects · Esc closes",
            ))
            .size(10)
            .color(MUTED),
        );

        container(app_scrollable(content.spacing(8).align_x(Alignment::Start)))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .style(|_| container::Style {
                background: Some(Background::Color(BACKGROUND)),
                text_color: Some(INK),
                ..Default::default()
            })
            .into()
    }

    fn profile(&self, id: &str) -> Option<&Profile> {
        self.profiles.iter().find(|profile| profile.id == id)
    }

    fn add_account_view(&self) -> Element<'_, Message> {
        let key_mode_buttons = row![
            button(tr(
                self.language,
                "Usar clave existente",
                "Use existing key"
            ))
            .style(if self.form_key_mode == KeyMode::Existing {
                button::primary
            } else {
                button::secondary
            })
            .on_press(Message::FormKeyModeChanged(KeyMode::Existing)),
            button(tr(
                self.language,
                "Crear clave Ed25519",
                "Create Ed25519 key"
            ))
            .style(if self.form_key_mode == KeyMode::Create {
                button::primary
            } else {
                button::secondary
            })
            .on_press(Message::FormKeyModeChanged(KeyMode::Create)),
        ]
        .spacing(10);

        let mut form = column![
            row![
                brand_logo(),
                language_button(self.language),
                Space::new().width(Length::Fill),
                button(tr(self.language, "VOLVER AL SELECTOR", "BACK TO PROFILES"))
                    .style(button::text)
                    .on_press(Message::CancelAddAccount),
            ]
            .align_y(Alignment::Center),
            text(tr(self.language, "NUEVA CUENTA", "NEW ACCOUNT"))
                .size(11)
                .color(MUTED),
            text(tr(self.language, "Añadir cuenta", "Add account"))
                .size(24)
                .color(INK),
            text(tr(
                self.language,
                "Guarda los datos del perfil para elegirlo desde el selector.",
                "Save the profile details so you can select it from the switcher."
            ))
            .size(14)
            .color(MUTED),
            text(tr(self.language, "NOMBRE VISIBLE", "DISPLAY NAME"))
                .size(11)
                .color(MUTED),
            text_input(
                tr(self.language, "Ej. Personal", "e.g. Personal"),
                &self.form_name
            )
            .on_input(Message::FormNameChanged)
            .padding(12)
            .size(14),
            text(tr(
                self.language,
                "LOGIN DE GITHUB (gh)",
                "GITHUB USERNAME (gh)"
            ))
            .size(11)
            .color(MUTED),
            text_input(tr(self.language, "usuario", "username"), &self.form_login)
                .on_input(Message::FormLoginChanged)
                .padding(12)
                .size(14),
            text(tr(
                self.language,
                "Se usa el login de la cuenta; no se guardan credenciales de gh.",
                "The account username is used; gh credentials are not stored here."
            ))
            .size(12)
            .color(MUTED),
            text(tr(self.language, "CLAVE SSH PRIVADA", "SSH PRIVATE KEY"))
                .size(11)
                .color(MUTED),
            key_mode_buttons,
            row![
                text_input(
                    if self.form_key_mode == KeyMode::Create {
                        "~/.ssh/qui_ed25519"
                    } else {
                        "~/.ssh/id_ed25519"
                    },
                    &self.form_key_path,
                )
                .on_input(Message::FormKeyPathChanged)
                .padding(12)
                .size(14)
                .width(Length::Fill),
                button(if self.form_key_mode == KeyMode::Create {
                    tr(self.language, "Elegir ubicación", "Choose location")
                } else {
                    tr(self.language, "Elegir archivo", "Choose file")
                })
                .style(button::secondary)
                .on_press(Message::PickKeyPath),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(12)
        .width(Length::Fill);

        let key_help = if self.form_key_mode == KeyMode::Create {
            tr(
                self.language,
                "Qui crea la clave localmente con ssh-keygen en Kitty. Introduce allí la frase de paso; la clave privada nunca se lee ni se muestra.",
                "Qui creates the key locally with ssh-keygen in Kitty. Enter the passphrase there; the private key is never read or displayed.",
            )
        } else {
            tr(
                self.language,
                "Elige el archivo privado existente. Qui no lo lee ni muestra; se requiere su archivo público .pub para poder copiarlo a GitHub.",
                "Choose an existing private key. Qui never reads or displays it; its .pub file is needed to copy the public key to GitHub.",
            )
        };
        form = form.push(text(key_help).size(12).color(MUTED));
        form = form.push(info_panel(
            tr(self.language, "gh y SSH cambian globalmente. La autoría Git sigue por directorio según includeIf.", "gh and SSH switch globally. Git commit identity remains directory-specific through includeIf."),
            false,
        ));
        if let Some(error) = &self.form_error {
            form = form.push(info_panel(error.clone(), true));
        }
        form = form.push(
            row![
                Space::new().width(Length::Fill),
                button(tr(self.language, "Cancelar", "Cancel"))
                    .style(button::secondary)
                    .on_press(Message::CancelAddAccount),
                button(if self.busy {
                    tr(self.language, "Guardando…", "Saving…")
                } else {
                    tr(self.language, "Guardar cuenta", "Save account")
                })
                .style(button::primary)
                .on_press_maybe((!self.busy).then_some(Message::SaveAccount)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        );

        container(app_scrollable(form.spacing(18).align_x(Alignment::Start)))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .style(|_| container::Style {
                background: Some(Background::Color(BACKGROUND)),
                text_color: Some(INK),
                ..Default::default()
            })
            .into()
    }

    fn public_key_view(&self) -> Element<'_, Message> {
        let path = self
            .public_key_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut content = column![
            row![
                brand_logo(),
                language_button(self.language),
                Space::new().width(Length::Fill),
                button(tr(self.language, "VOLVER AL SELECTOR", "BACK TO PROFILES"))
                    .style(button::text)
                    .on_press(Message::CancelAddAccount),
            ]
            .align_y(Alignment::Center),
            text(tr(self.language, "PERFIL GUARDADO", "PROFILE SAVED")).size(11).color(MUTED),
            text(tr(self.language, "Clave pública lista", "Public key ready")).size(24).color(INK),
            text(tr(self.language, "Copia la línea completa y pégala en GitHub.", "Copy the full line and add it to GitHub."))
                .size(14)
                .color(MUTED),
            text(tr(self.language, "CLAVE PÚBLICA (.PUB)", "PUBLIC KEY (.PUB)")).size(11).color(MUTED),
            text_input("~/.ssh/qui_ed25519.pub", &path).padding(12).size(14),
            container(app_scrollable(
                text(&self.public_key).size(12).color(INK),
            ))
                .width(Length::Fill)
                .height(Length::Fixed(100.0))
                .padding(16)
                .style(|_| container::Style {
                    background: Some(Background::Color(Color::WHITE)),
                    border: Border::default().color(BORDER).width(1.0).rounded(8.0),
                    ..Default::default()
                }),
            button(if self.copied_public_key {
                tr(self.language, "Copiada", "Copied")
            } else {
                tr(self.language, "Copiar clave pública", "Copy public key")
            })
            .style(button::primary)
            .on_press(Message::CopyPublicKey),
            info_panel(
                tr(self.language, "Pégala en GitHub → Settings → SSH and GPG keys → New SSH key. La clave privada nunca se muestra ni se copia.", "Add it on GitHub under Settings → SSH and GPG keys → New SSH key. Your private key is never displayed or copied."),
                false,
            ),
        ]
        .spacing(18)
        .width(Length::Fill);
        if self.public_key.is_empty() {
            content = content.push(info_panel(
                tr(self.language, "No se pudo leer la clave pública. Comprueba que existe el archivo .pub junto a la clave privada.", "Could not read the public key. Check that the .pub file exists next to the private key."),
                true,
            ));
        }
        container(app_scrollable(content.align_x(Alignment::Start)))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .style(|_| container::Style {
                background: Some(Background::Color(BACKGROUND)),
                text_color: Some(INK),
                ..Default::default()
            })
            .into()
    }
}

fn profile_card<'a>(
    profile: &Profile,
    active: bool,
    gh_status: String,
    busy: bool,
    language: Language,
) -> Element<'a, Message> {
    let body = column![
        row![
            text(profile.name.clone())
                .size(15)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .color(INK),
            Space::new().width(Length::Fill),
            text(if active {
                tr(language, "ACTIVA", "ACTIVE")
            } else {
                ""
            })
            .size(10)
            .color(BLUE),
        ]
        .align_y(Alignment::Center),
        text(format!("SSH · {}", profile.name))
            .size(12)
            .color(MUTED),
        text(format!("gh · {gh_status}")).size(12).color(
            if gh_status.contains("no válido") || gh_status.contains("invalid token") {
                ERROR
            } else {
                MUTED
            }
        ),
    ]
    .spacing(2)
    .width(Length::Fill);
    button(body)
        .width(Length::Fill)
        .padding(10)
        .style(move |_, status| {
            let background = if active { PALE_BLUE } else { Color::WHITE };
            let border_color = if active || status == button::Status::Hovered {
                BLUE
            } else {
                BORDER
            };
            button::Style {
                background: Some(Background::Color(background)),
                text_color: INK,
                border: Border::default()
                    .color(border_color)
                    .width(1.0)
                    .rounded(8.0),
                shadow: Shadow::default(),
                snap: true,
            }
        })
        .on_press_maybe((!busy).then(|| Message::Activate(profile.id.clone())))
        .into()
}

fn language_button(language: Language) -> Element<'static, Message> {
    button(match language {
        Language::Spanish => "English",
        Language::English => "Español",
    })
    .style(button::text)
    .on_press(Message::ToggleLanguage)
    .into()
}

fn info_panel(message: impl Into<String>, error: bool) -> Element<'static, Message> {
    let message = message.into();
    container(
        text(message)
            .size(14)
            .color(if error { ERROR } else { INK }),
    )
    .width(Length::Fill)
    .padding(16)
    .style(|_| container::Style {
        background: Some(Background::Color(Color::WHITE)),
        border: Border::default().color(BORDER).width(1.0).rounded(8.0),
        ..Default::default()
    })
    .into()
}

fn app_scrollable<'a>(
    content: impl Into<Element<'a, Message>>,
) -> iced::widget::Scrollable<'a, Message> {
    let content = container(content)
        .width(Length::Fill)
        .padding(iced::Padding {
            right: 22.0,
            ..Default::default()
        });

    scrollable(content)
        .direction(iced::widget::scrollable::Direction::Vertical(
            iced::widget::scrollable::Scrollbar::new()
                .width(6)
                .margin(6)
                .scroller_width(6),
        ))
        .style(|theme, status| {
            let mut style = iced::widget::scrollable::default(theme, status);
            let rail = iced::widget::scrollable::Rail {
                background: Some(Background::Color(PALE_BLUE)),
                border: Border::default().rounded(3.0),
                scroller: iced::widget::scrollable::Scroller {
                    background: Background::Color(BLUE),
                    border: Border::default().rounded(3.0),
                },
            };
            style.vertical_rail = rail;
            style.horizontal_rail = rail;
            style
        })
}

fn load_profiles(language: Language) -> Result<Vec<Profile>, String> {
    let output = Command::new("git")
        .args([
            "config",
            "--global",
            "--get-regexp",
            r"^includeif\..*\.path$",
        ])
        .output()
        .map_err(|error| {
            format!(
                "{}: {error}",
                tr(language, "No pude ejecutar git", "Could not run git")
            )
        })?;
    if !output.status.success() && output.status.code() != Some(1) {
        return Err(format!(
            "{}: {}",
            tr(
                language,
                "No pude leer las reglas includeIf de Git",
                "Could not read Git includeIf rules"
            ),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let mut profiles = Vec::new();
    let mut ssh_hosts = read_ssh_aliases();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let Some((condition, config_path)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let profile_id = if condition.to_lowercase().contains("personal")
            || config_path.to_lowercase().contains("personal")
        {
            "personal"
        } else if condition.to_lowercase().contains("work")
            || config_path.to_lowercase().contains("work")
        {
            "work"
        } else {
            continue;
        };
        let config_path = expand_home(config_path);
        let user_name = git_config_value(&config_path, "user.name", language)?;
        let email = git_config_value(&config_path, "user.email", language)?;
        let Some(host_index) = ssh_hosts
            .iter()
            .position(|host| host.host.to_lowercase().contains(profile_id))
        else {
            continue;
        };
        let ssh_host = ssh_hosts.remove(host_index);
        profiles.push(Profile {
            id: profile_id.into(),
            name: if profile_id == "work" {
                tr(language, "Trabajo", "Work")
            } else {
                "Personal"
            }
            .into(),
            user_name,
            email,
            ssh_identity: ssh_host.identity,
            gh_login: None,
        });
    }

    if let Some(path) = profiles_file() {
        match fs::read(&path) {
            Ok(contents) => {
                let saved: Vec<Profile> = serde_json::from_slice(&contents).map_err(|error| {
                    format!(
                        "{} {}: {error}",
                        tr(language, "No pude leer", "Could not read"),
                        path.display()
                    )
                })?;
                for profile in saved {
                    if profile.id.starts_with("account-")
                        && profile.gh_login.is_some()
                        && !profiles.iter().any(|existing| existing.id == profile.id)
                    {
                        profiles.push(profile);
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "{} {}: {error}",
                    tr(language, "No pude leer", "Could not read"),
                    path.display()
                ));
            }
        }
    }

    profiles.sort_by(|left, right| {
        let rank = |profile: &Profile| match profile.id.as_str() {
            "work" => (0, String::new()),
            "personal" => (1, String::new()),
            _ => (2, profile.name.to_lowercase()),
        };
        rank(left).cmp(&rank(right))
    });
    Ok(profiles)
}

fn profiles_file() -> Option<PathBuf> {
    config_dir().map(|path| path.join("profiles.json"))
}

fn save_profile(profile: &Profile, language: Language) -> Result<(), String> {
    let config = config_dir().ok_or(tr(
        language,
        "No encuentro tu directorio de configuración.",
        "Could not find your configuration directory.",
    ))?;
    let path = config.join("profiles.json");
    fs::create_dir_all(&config).map_err(|error| {
        format!(
            "{} {}: {error}",
            tr(language, "No pude crear", "Could not create"),
            config.display()
        )
    })?;
    let mut saved = match fs::read(&path) {
        Ok(contents) => serde_json::from_slice::<Vec<Profile>>(&contents).map_err(|error| {
            format!(
                "{} {}: {error}",
                tr(language, "No pude leer", "Could not read"),
                path.display()
            )
        })?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => {
            return Err(format!(
                "{} {}: {error}",
                tr(language, "No pude leer", "Could not read"),
                path.display()
            ));
        }
    };
    if saved.iter().any(|existing| existing.id == profile.id) {
        return Err(tr(
            language,
            "Ya existe un perfil con ese identificador.",
            "A profile with that identifier already exists.",
        )
        .into());
    }
    saved.push(profile.clone());
    let contents = serde_json::to_vec_pretty(&saved).map_err(|error| {
        format!(
            "{}: {error}",
            tr(
                language,
                "No pude guardar el perfil",
                "Could not serialize the profile"
            )
        )
    })?;
    atomic_write(&path, &contents).map_err(|error| {
        format!(
            "{} {}: {error}",
            tr(language, "No pude guardar", "Could not save"),
            path.display()
        )
    })
}

fn register_account(
    name: String,
    login: String,
    key_path: PathBuf,
    key_mode: KeyMode,
    language: Language,
) -> Result<AccountSaved, String> {
    if key_mode == KeyMode::Create {
        if key_path.exists() || public_key_path(&key_path).exists() {
            return Err(format!(
                "{} {}. {}.",
                tr(
                    language,
                    "Ya existe un archivo en",
                    "A file already exists at"
                ),
                key_path.display(),
                tr(
                    language,
                    "Elige otra ruta para no sobrescribir una clave",
                    "Choose another path to avoid overwriting a key"
                )
            ));
        }
        let parent = key_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .ok_or(tr(
                language,
                "Elige una ruta válida para la clave.",
                "Choose a valid key path.",
            ))?;
        let parent_missing = !parent.exists();
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "{} {}: {error}",
                tr(language, "No pude crear", "Could not create"),
                parent.display()
            )
        })?;
        #[cfg(unix)]
        if parent_missing {
            if let Err(error) = set_private_directory_permissions(parent) {
                return Err(format!(
                    "{} {}: {error}",
                    tr(language, "No pude proteger", "Could not secure"),
                    parent.display()
                ));
            }
        }
        run_ssh_keygen(&key_path, &login, language)?;
    } else {
        if !key_path.is_file() || key_path.extension().is_some_and(|ext| ext == "pub") {
            return Err(tr(
                language,
                "Selecciona un archivo de clave privada SSH existente.",
                "Select an existing SSH private key file.",
            )
            .into());
        }
    }

    let public_path = public_key_path(&key_path);
    let public_key = read_public_key(&public_path, language)?;
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            format!(
                "{}: {error}",
                tr(
                    language,
                    "No pude crear el identificador del perfil",
                    "Could not create a profile identifier"
                )
            )
        })?
        .as_nanos();
    let profile = Profile {
        id: format!("account-{unique}"),
        name,
        user_name: String::new(),
        email: String::new(),
        ssh_identity: key_path,
        gh_login: Some(login),
    };
    save_profile(&profile, language)?;
    Ok(AccountSaved {
        profile,
        public_key_path: public_path,
        public_key,
    })
}

fn public_key_path(private_key_path: &Path) -> PathBuf {
    let mut path = private_key_path.as_os_str().to_os_string();
    path.push(".pub");
    PathBuf::from(path)
}

fn read_public_key(path: &Path, language: Language) -> Result<String, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "{} {}: {error}",
            tr(
                language,
                "No pude leer la clave pública",
                "Could not read public key"
            ),
            path.display()
        )
    })?;
    let public_key = contents.trim();
    let valid = public_key.lines().count() == 1
        && public_key.split_whitespace().count() >= 2
        && (public_key.starts_with("ssh-")
            || public_key.starts_with("ecdsa-")
            || public_key.starts_with("sk-"));
    if !valid {
        return Err(format!(
            "{} {}",
            path.display(),
            tr(
                language,
                "no parece una clave pública SSH válida.",
                "does not look like a valid SSH public key."
            )
        ));
    }
    Ok(public_key.to_owned())
}

fn run_ssh_keygen(path: &Path, comment: &str, language: Language) -> Result<(), String> {
    let mut command = Command::new("kitty");
    command
        .args([
            "--wait-for-single-instance-window-close",
            "--hold",
            "ssh-keygen",
        ])
        .args(["-t", "ed25519", "-C", comment, "-f"])
        .arg(path);
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(_) => {
            let mut fallback = Command::new("x-terminal-emulator");
            fallback
                .arg("-e")
                .arg("ssh-keygen")
                .args(["-t", "ed25519", "-C", comment, "-f"])
                .arg(path);
            fallback.spawn().map_err(|error| {
                format!(
                    "{}: {error}",
                    tr(
                        language,
                        "No pude abrir Kitty ni un emulador de terminal para crear la clave",
                        "Could not open Kitty or a terminal emulator to create the key"
                    )
                )
            })?
        }
    };
    let status = child.wait().map_err(|error| {
        format!(
            "{}: {error}",
            tr(
                language,
                "No pude esperar a ssh-keygen",
                "Could not wait for ssh-keygen"
            )
        )
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(tr(
            language,
            "No se creó la clave SSH. Revisa Kitty y vuelve a intentarlo.",
            "The SSH key was not created. Check Kitty and try again.",
        )
        .into())
    }
}

#[cfg(unix)]
fn set_private_directory_permissions(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

fn git_config_value(path: &Path, key: &str, language: Language) -> Result<String, String> {
    let output = Command::new("git")
        .args(["config", "--file"])
        .arg(path)
        .arg("--get")
        .arg(key)
        .output()
        .map_err(|error| {
            format!(
                "{} {}: {error}",
                tr(language, "No pude leer", "Could not read"),
                path.display()
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "{} {key} {}.",
            tr(language, "Falta", "Missing"),
            path.display()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn read_ssh_aliases() -> Vec<SshAlias> {
    let Some(home) = home_dir() else {
        return Vec::new();
    };
    let Ok(config) = fs::read_to_string(home.join(".ssh/config")) else {
        return Vec::new();
    };
    let mut aliases = Vec::new();
    let mut host = None;
    for line in config.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        if key.eq_ignore_ascii_case("host") {
            host = value
                .split_whitespace()
                .find(|name| {
                    let name = name.to_lowercase();
                    !name.contains('*')
                        && !name.starts_with('!')
                        && (name.contains("work") || name.contains("personal"))
                })
                .map(str::to_owned);
        } else if key.eq_ignore_ascii_case("identityfile") {
            if let Some(host) = host.take() {
                aliases.push(SshAlias {
                    host,
                    identity: expand_home(value.trim().trim_matches('"')),
                });
            }
        }
    }
    aliases
}

fn load_gh_accounts() -> Vec<GhAccount> {
    let Ok(output) = Command::new("gh")
        .args([
            "auth",
            "status",
            "--hostname",
            "github.com",
            "--json",
            "hosts",
        ])
        .output()
    else {
        return Vec::new();
    };
    let Ok(status) = serde_json::from_slice::<GhStatus>(&output.stdout) else {
        return Vec::new();
    };
    status.hosts.get("github.com").cloned().unwrap_or_default()
}

fn map_gh_accounts(profiles: &mut [Profile], accounts: &[GhAccount]) {
    for profile in profiles.iter_mut() {
        if profile.gh_login.is_some() {
            continue;
        }
        profile.gh_login = accounts
            .iter()
            .find(|account| account.login.eq_ignore_ascii_case(&profile.user_name))
            .map(|account| account.login.clone());
    }
    let unmatched_profiles = profiles
        .iter()
        .filter(|profile| profile.gh_login.is_none())
        .count();
    let unmatched_accounts = accounts
        .iter()
        .filter(|account| {
            !profiles
                .iter()
                .any(|profile| profile.gh_login.as_deref() == Some(account.login.as_str()))
        })
        .count();
    if unmatched_profiles == 1 && unmatched_accounts == 1 {
        let Some(login) = accounts
            .iter()
            .find(|account| {
                !profiles
                    .iter()
                    .any(|profile| profile.gh_login.as_deref() == Some(account.login.as_str()))
            })
            .map(|account| account.login.clone())
        else {
            return;
        };
        if let Some(profile) = profiles
            .iter_mut()
            .find(|profile| profile.gh_login.is_none())
        {
            profile.gh_login = Some(login);
        }
    }
}

fn account_status(profile: &Profile, accounts: &[GhAccount], language: Language) -> String {
    let Some(login) = profile.gh_login.as_deref() else {
        return tr(language, "sin iniciar sesión", "not signed in").into();
    };
    let Some(account) = accounts
        .iter()
        .find(|account| account.login.eq_ignore_ascii_case(login))
    else {
        return tr(language, "sin iniciar sesión", "not signed in").into();
    };
    if account.state == "success" && account.error.is_none() {
        tr(language, "sesión válida", "signed in").into()
    } else if account.error.as_deref().is_some_and(|error| {
        let error = error.to_lowercase();
        error.contains("invalid")
            || error.contains("unauthorized")
            || error.contains("401")
            || error.contains("403")
    }) {
        tr(language, "token no válido", "invalid token").into()
    } else {
        tr(language, "no se pudo verificar", "could not verify").into()
    }
}

fn read_active_profile(profiles: &[Profile], accounts: &[GhAccount]) -> Option<String> {
    let active_file = config_dir()?.join("ssh-profile.gitconfig");
    if let Ok(contents) = fs::read_to_string(active_file) {
        if let Some(id) = contents
            .lines()
            .find_map(|line| line.strip_prefix("# Qui profile: "))
        {
            if profiles.iter().any(|profile| profile.id == id) {
                return Some(id.to_owned());
            }
        }
    }
    accounts
        .iter()
        .find(|account| account.active)
        .and_then(|account| {
            profiles
                .iter()
                .find(|profile| profile.user_name.eq_ignore_ascii_case(&account.login))
                .map(|profile| profile.id.clone())
        })
}

fn activate_profile(
    profile: Profile,
    previous_profile: Option<String>,
    language: Language,
) -> SwitchOutcome {
    let ssh_result = install_ssh_rewrite(&profile, language);
    let ssh_changed = ssh_result.is_ok();
    let (message, login_needed, accounts) = match ssh_result {
        Ok(()) => {
            let gh_result = profile.gh_login.as_deref().map(|login| {
                Command::new("gh")
                    .args([
                        "auth",
                        "switch",
                        "--hostname",
                        "github.com",
                        "--user",
                        login,
                    ])
                    .output()
            });
            let accounts = load_gh_accounts();
            let status = account_status(&profile, &accounts, language);
            let gh_switched = matches!(&gh_result, Some(Ok(output)) if output.status.success());
            let login_needed = !gh_switched || status != tr(language, "sesión válida", "signed in");
            let message = match gh_result {
                Some(Ok(output)) if output.status.success() => {
                    format!(
                        "{} {}. {}: {}.",
                        tr(
                            language,
                            "SSH y gh seleccionados para",
                            "SSH and gh switched to"
                        ),
                        profile.name,
                        tr(language, "Estado de gh", "gh status"),
                        status
                    )
                }
                Some(Err(error)) => format!(
                    "{} {}. {}: {error}",
                    tr(language, "SSH cambiado a", "SSH switched to"),
                    profile.name,
                    tr(language, "No pude ejecutar gh", "Could not run gh")
                ),
                Some(Ok(_)) => format!(
                    "{} {}. {}: {}.",
                    tr(language, "SSH cambiado a", "SSH switched to"),
                    profile.name,
                    tr(
                        language,
                        "gh no pudo cambiar de cuenta; estado",
                        "gh could not switch accounts; status"
                    ),
                    status
                ),
                None => format!(
                    "{} {}. {}: {}.",
                    tr(language, "SSH cambiado a", "SSH switched to"),
                    profile.name,
                    tr(
                        language,
                        "gh no tiene una cuenta asociada; estado",
                        "no gh account is associated; status"
                    ),
                    status
                ),
            };
            (message, login_needed, accounts)
        }
        Err(error) => (error, false, load_gh_accounts()),
    };
    SwitchOutcome {
        profile_id: profile.id,
        ssh_changed,
        previous_profile,
        message,
        login_needed,
        accounts,
    }
}

fn install_ssh_rewrite(profile: &Profile, language: Language) -> Result<(), String> {
    let config_dir = config_dir().ok_or(tr(
        language,
        "No encuentro tu directorio de configuración.",
        "Could not find your configuration directory.",
    ))?;
    let git_file = config_dir.join("ssh-profile.gitconfig");
    let ssh_file = config_dir.join("ssh-profile.conf");
    let ssh_config = home_dir()
        .ok_or(tr(
            language,
            "No encuentro tu directorio personal.",
            "Could not find your home directory.",
        ))?
        .join(".ssh/config");
    fs::create_dir_all(&config_dir).map_err(|error| {
        format!(
            "{} {}: {error}",
            tr(language, "No pude crear", "Could not create"),
            config_dir.display()
        )
    })?;
    if let Some(parent) = ssh_config.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "{} {}: {error}",
                tr(language, "No pude crear", "Could not create"),
                parent.display()
            )
        })?;
    }

    let previous_git = read_optional_file(&git_file, language)?;
    let previous_ssh = read_optional_file(&ssh_file, language)?;
    let previous_ssh_config = read_optional_file(&ssh_config, language)?;
    let mut prefixes = HashSet::new();
    for alias in read_ssh_aliases() {
        prefixes.insert(format!("git@{}:", alias.host));
        prefixes.insert(format!("ssh://git@{}/", alias.host));
    }
    let mut git_content = format!("# Qui profile: {}\n[url \"git@github.com:\"]\n", profile.id);
    for prefix in prefixes {
        git_content.push_str(&format!("\tinsteadOf = {prefix}\n"));
    }
    let ssh_content = format!(
        "# Qui profile: {}\nHost github.com\n  HostName github.com\n  User git\n  IdentityFile {}\n  IdentitiesOnly yes\n",
        profile.id,
        ssh_quote_path(&profile.ssh_identity)
    );

    atomic_write(&ssh_file, ssh_content.as_bytes()).map_err(|error| {
        format!(
            "{}: {error}",
            tr(
                language,
                "No pude guardar la configuración SSH",
                "Could not save SSH configuration"
            )
        )
    })?;
    if let Err(error) = ensure_ssh_include(&ssh_config, &ssh_file, language) {
        restore_file(&ssh_file, previous_ssh.as_deref());
        return Err(error);
    }
    if let Err(error) = atomic_write(&git_file, git_content.as_bytes()) {
        restore_file(&ssh_config, previous_ssh_config.as_deref());
        restore_file(&ssh_file, previous_ssh.as_deref());
        return Err(format!(
            "{}: {error}",
            tr(
                language,
                "No pude guardar la configuración Git",
                "Could not save Git configuration"
            )
        ));
    }
    if let Err(error) = ensure_git_include(&git_file, language) {
        restore_file(&ssh_config, previous_ssh_config.as_deref());
        restore_file(&ssh_file, previous_ssh.as_deref());
        restore_file(&git_file, previous_git.as_deref());
        return Err(error);
    }
    Ok(())
}

fn read_optional_file(path: &Path, language: Language) -> Result<Option<Vec<u8>>, String> {
    match fs::read(path) {
        Ok(contents) => Ok(Some(contents)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!(
            "{} {}: {error}",
            tr(language, "No pude leer", "Could not read"),
            path.display()
        )),
    }
}

fn ensure_ssh_include(config: &Path, included: &Path, language: Language) -> Result<(), String> {
    let existing = read_optional_file(config, language)?;
    let current = existing
        .as_deref()
        .map(|contents| String::from_utf8_lossy(contents).into_owned())
        .unwrap_or_default();
    let expected = included.to_string_lossy();
    let already_included = current.lines().any(|line| {
        let line = line.trim();
        if line.starts_with('#') {
            return false;
        }
        let Some((key, value)) = line.split_once(char::is_whitespace) else {
            return false;
        };
        key.eq_ignore_ascii_case("include") && value.trim().trim_matches('"') == expected
    });
    if already_included {
        return Ok(());
    }
    let updated = format!("Include {}\n{}", ssh_quote_path(included), current);
    atomic_write(config, updated.as_bytes()).map_err(|error| {
        format!(
            "{} {}: {error}",
            tr(
                language,
                "No pude añadir el Include a",
                "Could not add the Include to"
            ),
            config.display()
        )
    })
}

fn ensure_git_include(config_file: &Path, language: Language) -> Result<(), String> {
    let include_paths = Command::new("git")
        .args(["config", "--global", "--get-all", "include.path"])
        .output()
        .map_err(|error| {
            format!(
                "{}: {error}",
                tr(
                    language,
                    "No pude ejecutar git config",
                    "Could not run git config"
                )
            )
        })?;
    if !include_paths.status.success() && include_paths.status.code() != Some(1) {
        return Err(format!(
            "{}: {}",
            tr(
                language,
                "No pude leer los include.path de Git",
                "Could not read Git include.path values"
            ),
            String::from_utf8_lossy(&include_paths.stderr).trim()
        ));
    }
    let config_path = config_file.to_string_lossy().into_owned();
    let already_included = String::from_utf8_lossy(&include_paths.stdout)
        .lines()
        .any(|path| path == config_path);
    if already_included {
        return Ok(());
    }
    let output = Command::new("git")
        .args(["config", "--global", "--add", "include.path"])
        .arg(config_path)
        .output()
        .map_err(|error| {
            format!(
                "{}: {error}",
                tr(
                    language,
                    "No pude registrar la configuración Git",
                    "Could not register Git configuration"
                )
            )
        })?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{}: {}",
            tr(
                language,
                "No pude registrar la configuración Git",
                "Could not register Git configuration"
            ),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn ssh_quote_path(path: &Path) -> String {
    format!(
        "\"{}\"",
        path.to_string_lossy()
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
    )
}

fn restore_file(path: &Path, previous: Option<&[u8]>) {
    if let Some(previous) = previous {
        let _ = atomic_write(path, previous);
    } else {
        let _ = fs::remove_file(path);
    }
}

fn start_gh_login(login: Option<&str>, refresh: bool, language: Language) -> String {
    let (program, args): (&str, &[&str]) = if refresh {
        ("gh", &["auth", "refresh", "--hostname", "github.com"])
    } else {
        (
            "gh",
            &[
                "auth",
                "login",
                "--hostname",
                "github.com",
                "--git-protocol",
                "ssh",
                "--web",
            ],
        )
    };
    if refresh {
        let Some(login) = login else {
            return tr(
                language,
                "No hay una cuenta de gh para renovar. Iníciala con gh auth login.",
                "There is no gh account to refresh. Sign in with gh auth login.",
            )
            .into();
        };
        let switched = Command::new("gh")
            .args([
                "auth",
                "switch",
                "--hostname",
                "github.com",
                "--user",
                login,
            ])
            .status();
        if !switched.is_ok_and(|status| status.success()) {
            return format!(
                "{} {login}. {} `gh auth login --hostname github.com --git-protocol ssh --web`.",
                tr(
                    language,
                    "No pude seleccionar la cuenta",
                    "Could not select account"
                ),
                tr(language, "Iníciala con", "Sign in with")
            );
        }
    }
    let kitty = Command::new("kitty")
        .args(["--detach", "--hold", program])
        .args(args)
        .spawn();
    if kitty.is_ok() {
        return tr(language, "Se abrió Kitty para autenticar gh. Al terminar, pulsa de nuevo el perfil para actualizar el estado.", "Kitty opened for gh sign-in. When it is done, select the profile again to refresh its status.").into();
    }

    let mut terminal = Command::new("x-terminal-emulator");
    terminal.arg("-e").arg(program).args(args);
    match terminal.spawn() {
        Ok(_) => tr(language, "Se abrió una terminal para autenticar gh. Al terminar, pulsa de nuevo el perfil para actualizar el estado.", "A terminal opened for gh sign-in. When it is done, select the profile again to refresh its status.").into(),
        Err(_) => format!("{}. `gh {}`", tr(language, "No encontré Kitty ni un emulador de terminal. Ejecuta en una terminal", "Kitty and no terminal emulator were found. Run this in a terminal"), args.join(" ")),
    }
}

fn config_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|home| home.join(".config")))
        .map(|path| path.join("qui"))
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn expand_home(path: &str) -> PathBuf {
    if path == "~" {
        return home_dir().unwrap_or_else(|| PathBuf::from(path));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return home_dir().unwrap_or_else(|| PathBuf::from("~")).join(rest);
    }
    PathBuf::from(path)
}

fn atomic_write(path: &Path, content: &[u8]) -> std::io::Result<()> {
    let temp = path.with_extension("tmp");
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temp)?;
    file.write_all(content)?;
    file.sync_all()?;
    fs::rename(temp, path)
}
