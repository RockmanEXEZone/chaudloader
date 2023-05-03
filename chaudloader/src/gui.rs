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

enum Message {
    SetExeCrc32(u32),
}

pub struct Host {
    ready_sender: oneshot::Sender<()>,
    message_receiver: fltk::app::Receiver<Message>,
}

pub struct Client {
    ready_receiver: Option<oneshot::Receiver<()>>,
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

    pub fn set_exe_crc32(&self, exe_crc32: u32) {
        self.message_sender.send(Message::SetExeCrc32(exe_crc32));
    }
}

pub fn make_host_and_client() -> (Host, Client) {
    let (ready_sender, ready_receiver) = oneshot::channel();
    let (message_sender, message_receiver) = fltk::app::channel();
    (
        Host {
            ready_sender,
            message_receiver,
        },
        Client {
            ready_receiver: Some(ready_receiver),
            message_sender,
        },
    )
}

pub fn run(
    host: Host,
    game_volume: crate::GameVolume,
    mut console_reader: impl std::io::Read + Send + 'static,
) -> Result<(), anyhow::Error> {
    use fltk::prelude::*;

    let app = fltk::app::App::default();
    fltk_theme::WidgetTheme::new(fltk_theme::ThemeType::Greybird).apply();

    let mut wind = fltk::window::Window::new(100, 100, 800, 600, "chaudloader");
    wind.make_resizable(true);

    let mut console = fltk::text::SimpleTerminal::new(0, 0, wind.width(), wind.height(), "console");
    console.set_ansi(true);
    console.set_stay_at_bottom(true);
    wind.resizable(&console);

    std::thread::spawn(move || {
        let _ = std::io::copy(&mut console_reader, &mut ConsoleWriter(&mut console));
    });

    wind.end();
    wind.show();

    host.ready_sender.send(()).unwrap();

    while app.wait() {
        if let Some(message) = host.message_receiver.recv() {
            match message {
                Message::SetExeCrc32(exe_crc32) => {
                    // TODO
                }
            }
        }
    }

    Ok(())
}
