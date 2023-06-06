use fltk::prelude::*;

use crate::{config, console, mods, path};

struct ConsoleWriter<'a>(&'a mut fltk::text::SimpleTerminal);

impl<'a> std::io::Write for ConsoleWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.append2(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

enum Message {}

pub struct Host {
    ready_sender: oneshot::Sender<()>,
    start_sender: oneshot::Sender<StartRequest>,
    message_receiver: fltk::app::Receiver<Message>,
}

pub struct Client {
    ready_receiver: Option<oneshot::Receiver<()>>,
    start_receiver: Option<oneshot::Receiver<StartRequest>>,
    message_sender: fltk::app::Sender<Message>,
}

impl Client {
    pub fn wait_for_ready(&mut self) {
        let r = if let Some(r) = self.ready_receiver.take() {
            r
        } else {
            return;
        };
        r.recv().unwrap()
    }

    pub fn wait_for_start(&mut self) -> StartRequest {
        self.start_receiver.take().unwrap().recv().unwrap()
    }
}

pub fn make_host_and_client() -> (Host, Client) {
    let (ready_sender, ready_receiver) = oneshot::channel();
    let (start_sender, start_receiver) = oneshot::channel();
    let (message_sender, message_receiver) = fltk::app::channel();
    (
        Host {
            ready_sender,
            start_sender,
            message_receiver,
        },
        Client {
            ready_receiver: Some(ready_receiver),
            start_receiver: Some(start_receiver),
            message_sender,
        },
    )
}

pub struct StartRequest {
    pub enabled_mods: Vec<(String, std::sync::Arc<mods::Mod>)>,
}

struct ModBinding {
    r#mod: std::sync::Arc<mods::Mod>,
    enabled: bool,
}

fn make_main_tile(
    game_env: &mods::GameEnv,
    config: &config::Config,
    mut on_start: impl FnMut(&mut fltk::group::Tile, StartRequest) + 'static,
) -> impl WidgetBase {
    let tile = fltk::group::Tile::default_fill();

    // Left browser.
    let left_group = fltk::group::Group::default().with_size(230, tile.height());

    let toolbar_group = fltk::group::Group::default().with_size(left_group.width(), 25);

    let mut refresh_button = fltk::button::Button::default()
        .with_label("Refresh") // TODO: Localize.
        .with_size(toolbar_group.width() / 2, toolbar_group.height());

    let mut open_folder_button = fltk::button::Button::default()
        .with_label("Open Folder") // TODO: Localize.
        .with_size(toolbar_group.width() / 2, toolbar_group.height())
        .with_pos(toolbar_group.width() / 2, 0);

    toolbar_group.end();

    let mut browser = fltk::browser::HoldBrowser::default()
        .with_size(left_group.width(), left_group.height() - 50)
        .with_pos(0, toolbar_group.height());
    left_group.resizable(&browser);

    let mod_bindings = std::rc::Rc::new(std::cell::RefCell::new(std::collections::BTreeMap::<
        String,
        ModBinding,
    >::new()));

    let mut play_button = fltk::button::Button::default()
        .with_label("Play")
        .with_size(left_group.width(), 25)
        .with_pos(0, toolbar_group.height() + browser.height());
    left_group.end();

    // Right browser.
    let right_group = fltk::group::Group::default()
        .with_size(tile.width() - left_group.width(), tile.height())
        .with_pos(left_group.width(), 0);

    let mut enabled_checkbox = fltk::button::CheckButton::default()
        .with_size(right_group.width(), 25)
        .with_pos(right_group.x(), right_group.y())
        .with_label("Enable");

    let mut help_view = fltk::misc::HelpView::default()
        .with_size(right_group.width(), right_group.height() - 25 - 25)
        .with_pos(right_group.x(), right_group.y() + 25);
    help_view.set_text_font(fltk::enums::Font::Helvetica);
    help_view.set_text_size(16);

    help_view.set_value("No mod selected."); // TODO: Localize.

    right_group.resizable(&help_view);

    right_group.end();

    tile.end();

    let set_selection = {
        let mut enabled_checkbox = enabled_checkbox.clone();
        let game_env = game_env.clone();
        let mut help_view = help_view.clone();

        move |selection: Option<(&String, &ModBinding)>| {
            let (mod_name, binding) = if let Some((mod_name, binding)) = selection {
                (mod_name, binding)
            } else {
                enabled_checkbox.set(false);
                enabled_checkbox.hide();
                help_view.set_value(&maud::html! { "No mod selected." }.into_string());
                return;
            };

            let compatibility = mods::check_compatibility(&game_env, &binding.r#mod.info);
            if compatibility.is_compatible() {
                enabled_checkbox.show();
                enabled_checkbox.set(binding.enabled);
            } else {
                enabled_checkbox.hide();
            }

            let base_path = std::path::Path::new("mods").join(mod_name);

            let readme_parser =
                pulldown_cmark::Parser::new(&binding.r#mod.readme).map(|event| match event {
                    pulldown_cmark::Event::Start(pulldown_cmark::Tag::Image(
                        link_type,
                        url,
                        title,
                    )) => pulldown_cmark::Event::Start(pulldown_cmark::Tag::Image(
                        link_type,
                        match url::Url::parse(&url) {
                            Err(url::ParseError::RelativeUrlWithoutBase) => {
                                // Use relative paths for images.
                                pulldown_cmark::CowStr::Boxed(
                                    path::ensure_safe(std::path::Path::new(&*url))
                                        .and_then(|path| {
                                            base_path.join(path).to_str().map(|s| {
                                                s.to_string()
                                                    .split(std::path::MAIN_SEPARATOR_STR)
                                                    .collect::<Vec<_>>()
                                                    .join("/")
                                            })
                                        })
                                        .unwrap_or_else(|| "".to_string())
                                        .into_boxed_str(),
                                )
                            }
                            _ => {
                                // Don't allow any other kinds of images.
                                pulldown_cmark::CowStr::Borrowed("")
                            }
                        },
                        title,
                    )),

                    event => event,
                });

            let mut readme = String::new();
            pulldown_cmark::html::push_html(&mut readme, readme_parser);
            help_view.set_value(
                &maud::html! {
                    (if !compatibility.is_compatible() {
                        maud::html! {
                            table width="100%" {
                                tr {
                                    td {
                                        p {
                                            font color="red" {
                                                "This mod is not compatible and cannot be loaded:"
                                            }
                                        }
                                        ul {
                                            (if !compatibility.loader_version {
                                                maud::html! {
                                                    li {
                                                        font color="red" {
                                                            "Loader version ("
                                                            (*crate::VERSION)
                                                            ") does not match mod requirement's ("
                                                            (&binding.r#mod.info.requires_loader_version)
                                                            ")"
                                                        }
                                                    }
                                                }
                                            } else {
                                                maud::html! { }
                                            })

                                            (if !compatibility.exe_crc32 {
                                                maud::html! {
                                                    li {
                                                        font color="red" {
                                                            "Game CRC32 ("
                                                            (format!("{:08x}", game_env.exe_crc32))
                                                            ") does not match mod requirement's (one of "
                                                            (binding.r#mod.info.requires_exe_crc32.as_ref().map(|reqs| reqs.iter().map(|v| format!("{:08x}", v)).collect::<Vec<_>>().join(", ")).unwrap_or_else(|| "".to_string()))
                                                            ")"
                                                        }
                                                    }
                                                }
                                            } else {
                                                maud::html! { }
                                            })

                                            (if !compatibility.game {
                                                maud::html! {
                                                    li {
                                                        font color="red" {
                                                            "Game name ("
                                                            (serde_plain::to_string(&game_env.volume).unwrap())
                                                             ") does not match mod requirement's (one of "
                                                            (binding.r#mod.info.requires_game.as_ref().map(|reqs| reqs.iter().map(|v| serde_plain::to_string(v).unwrap()).collect::<Vec<_>>().join(", ")).unwrap_or_else(|| "".to_string()))
                                                            ")"
                                                        }
                                                    }
                                                }
                                            } else {
                                                maud::html! { }
                                            })
                                        }
                                    }
                                }
                            }
                            hr { }
                        }
                    } else {
                        maud::html! {}
                    })
                    table {
                        tr {
                            th { "Title"}
                            td { (binding.r#mod.info.title) }
                        }
                        tr {
                            th { "Version"}
                            td { (binding.r#mod.info.version) }
                        }
                        tr {
                            th { "Authors"}
                            td { (binding.r#mod.info.authors.join(", ")) }
                        }
                        (if let Some(url) = binding.r#mod.info.url.as_ref() {
                            maud::html! {
                                tr {
                                    th { "Link"}
                                    td { a href=(url) { (url) } }
                                }
                            }
                        } else {
                            maud::html! { }
                        })
                    }
                    hr { }
                    (maud::PreEscaped(readme))
                }
                .into_string(),
            );
        }
    };

    let mut update_browser_items = {
        let mut browser = browser.clone();
        let game_env = game_env.clone();
        move |mod_bindings: &std::collections::BTreeMap<String, ModBinding>| {
            let current_selection = browser
                .selected_items()
                .first()
                .and_then(|i| mod_bindings.keys().nth((i - 1) as usize))
                .cloned();

            browser.clear();
            for (i, (name, binding)) in mod_bindings.iter().enumerate() {
                browser.add(&format!(
                    "{}@.{} {} v{}",
                    if !mods::check_compatibility(&game_env, &binding.r#mod.info).is_compatible() {
                        "@B88"
                    } else {
                        ""
                    },
                    if binding.enabled { "☑️" } else { "🔲" },
                    name,
                    binding.r#mod.info.version
                ));
                if current_selection
                    .as_ref()
                    .map(|selection| selection == name)
                    .unwrap_or(false)
                {
                    browser.select((i + 1) as i32);
                }
            }
        }
    };

    let refresh_browser = {
        let mod_bindings = std::rc::Rc::clone(&mod_bindings);
        let game_env = game_env.clone();
        let mut update_browser_items = update_browser_items.clone();
        move || {
            let mut mod_bindings = mod_bindings.borrow_mut();

            let currently_enabled = mod_bindings
                .iter()
                .filter(|(_, binding)| binding.enabled)
                .map(|(name, _)| name.clone())
                .collect::<std::collections::HashSet<_>>();

            *mod_bindings = match mods::scan() {
                Ok(mod_bindings) => mod_bindings
                    .into_iter()
                    .map(|(name, r#mod)| {
                        (
                            name.clone(),
                            ModBinding {
                                r#mod: std::sync::Arc::clone(&r#mod),
                                enabled: mods::check_compatibility(&game_env, &r#mod.info)
                                    .is_compatible()
                                    && currently_enabled.contains(&name),
                            },
                        )
                    })
                    .collect(),
                Err(_e) => {
                    return;
                }
            };
            update_browser_items(&*mod_bindings);
        }
    };

    browser.set_callback({
        let mod_bindings = std::rc::Rc::clone(&mod_bindings);
        let browser = browser.clone();
        let mut set_selection = set_selection.clone();
        move |_| {
            let mod_bindings = mod_bindings.borrow();

            set_selection(
                browser
                    .selected_items()
                    .first()
                    .and_then(|i| mod_bindings.keys().nth((i - 1) as usize))
                    .and_then(|mod_name| {
                        mod_bindings
                            .get(mod_name)
                            .map(|binding| (mod_name, binding))
                    }),
            );
        }
    });

    open_folder_button.set_callback({
        let mod_bindings = std::rc::Rc::clone(&mod_bindings);
        let browser = browser.clone();
        move |_| {
            let mod_bindings = mod_bindings.borrow();

            let base_path = std::path::Path::new("mods");
            let path = if let Some(current_selection) = browser
                .selected_items()
                .first()
                .and_then(|i| mod_bindings.keys().nth((i - 1) as usize))
            {
                base_path.join(current_selection)
            } else {
                base_path.to_path_buf()
            };

            opener::open(path).unwrap();
        }
    });

    {
        let mut mod_bindings = mod_bindings.borrow_mut();
        *mod_bindings = match mods::scan() {
            Ok(mod_bindings) => mod_bindings
                .into_iter()
                .map(|(name, r#mod)| {
                    (
                        name.clone(),
                        ModBinding {
                            r#mod: std::sync::Arc::clone(&r#mod),
                            enabled: mods::check_compatibility(&game_env, &r#mod.info)
                                .is_compatible()
                                && config.enabled_mods.contains(&name),
                        },
                    )
                })
                .collect(),
            Err(_e) => std::collections::BTreeMap::new(),
        };
        update_browser_items(&*mod_bindings);
    }

    refresh_button.set_callback({
        let mut refresh_browser = refresh_browser.clone();
        move |_| {
            refresh_browser();
        }
    });

    enabled_checkbox.set_callback({
        let mod_bindings = std::rc::Rc::clone(&mod_bindings);
        let browser = browser.clone();
        let mut update_browser_items = update_browser_items.clone();

        move |cbox| {
            let mut mod_bindings = mod_bindings.borrow_mut();

            let binding = if let Some(binding) = browser
                .selected_items()
                .first()
                .and_then(|i| mod_bindings.values_mut().nth((i - 1) as usize))
            {
                binding
            } else {
                return;
            };

            binding.enabled = cbox.value();
            update_browser_items(&*mod_bindings);
        }
    });

    play_button.set_callback({
        let mut tile = tile.clone();
        let mod_bindings = std::rc::Rc::clone(&mod_bindings);
        move |_| {
            let mod_bindings = mod_bindings.borrow();
            on_start(
                &mut tile,
                StartRequest {
                    enabled_mods: mod_bindings
                        .iter()
                        .filter(|(_, binding)| binding.enabled)
                        .map(|(name, binding)| {
                            (name.to_string(), std::sync::Arc::clone(&binding.r#mod))
                        })
                        .collect(),
                },
            );
        }
    });

    tile
}

fn make_window(
    game_env: &mods::GameEnv,
    start_sender: oneshot::Sender<StartRequest>,
    config: &config::Config,
) -> fltk::window::Window {
    let mut wind = fltk::window::Window::default()
        .with_size(800, 600)
        .with_label(&format!(
            "chaudloader v{}: {} (crc32: {:08x})",
            *crate::VERSION,
            serde_plain::to_string(&game_env.volume).unwrap(),
            game_env.exe_crc32
        ));
    wind.make_resizable(true);

    let mut console = fltk::text::SimpleTerminal::default_fill();
    wind.resizable(&console);
    console.set_ansi(true);
    console.set_stay_at_bottom(true);
    console.hide();

    let main_tile = make_main_tile(game_env, config, {
        let mut start_sender = Some(start_sender);
        move |main_tile, start_request| {
            let start_sender = if let Some(start_sender) = start_sender.take() {
                start_sender
            } else {
                return;
            };

            main_tile.hide();

            let mut console_reader = console::Console::hijack().unwrap();
            std::thread::spawn({
                let mut console = console.clone();
                move || {
                    std::io::copy(&mut console_reader, &mut ConsoleWriter(&mut console)).unwrap();
                }
            });
            console.show();

            start_sender.send(start_request).unwrap();
        }
    });
    wind.resizable(&main_tile);

    wind.end();

    wind
}

pub fn run(
    host: Host,
    game_env: mods::GameEnv,
    config: config::Config,
) -> Result<(), anyhow::Error> {
    let Host {
        ready_sender,
        start_sender,
        message_receiver,
    } = host;

    let app = fltk::app::App::default();
    fltk_theme::WidgetTheme::new(fltk_theme::ThemeType::Metro).apply();

    let mut wind = make_window(&game_env, start_sender, &config);
    wind.show();

    ready_sender.send(()).unwrap();

    while app.wait() {
        if let Some(message) = message_receiver.recv() {
            match message {}
        }
    }

    Ok(())
}
