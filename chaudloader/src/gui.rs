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

pub fn run(
    gui_ready_sender: oneshot::Sender<()>,
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

    gui_ready_sender.send(()).unwrap();

    app.run()?;
    Ok(())
}
