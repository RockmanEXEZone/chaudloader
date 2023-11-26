use crate::mods;
use fltk::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct ModWidget {
    flex: fltk::group::Flex,
}

fltk::widget_extends!(ModWidget, fltk::group::Flex, flex);

impl ModWidget {
    pub fn new(mod_info: &mods::Info) -> Self {
        // Note: width is percentual but height is actual pixels!
        let mut flex = fltk::group::Flex::default().with_size(100, 80).row();
        flex.set_frame(fltk::enums::FrameType::FlatBox);
        flex.set_type(fltk::group::FlexType::Row);
        flex.set_color(fltk::enums::Color::from_hex(0xFF7F7F));
        flex.set_spacing(0);

        let mut icon_container = fltk::group::Group::default();
        icon_container.set_size(flex.height(), flex.height());
        icon_container.set_frame(fltk::enums::FrameType::BorderBox);
        icon_container.set_color(fltk::enums::Color::from_hex(0x007F00));
        flex.fixed(&icon_container, icon_container.width());
        {
            let mut icon = fltk::frame::Frame::default_fill();
            icon.set_frame(fltk::enums::FrameType::BorderBox);
            icon.set_color(fltk::enums::Color::from_hex(0x7F7FFF));
        }
        icon_container.end();

        let spacer = fltk::widget::Widget::default();
        flex.fixed(&spacer, 10);

        /*let mut enable_button = fltk::button::CheckButton::default()
            .with_label("Enabled")
            .with_size(15, 15)
            .with_pos(100 - 15, 100 - 15)
            .with_align(fltk::enums::Align::Left);
        enable_button.set_frame(fltk::enums::FrameType::FlatBox);
        enable_button.set_color(fltk::enums::Color::from_hex(0x7F7FFF));*/
        let mut mod_name_buffer = fltk::text::TextBuffer::default();
        mod_name_buffer.set_text(&mod_info.title);
        let mut mod_name = fltk::text::TextDisplay::default();
        mod_name.set_buffer(mod_name_buffer);
        mod_name.wrap_mode(fltk::text::WrapMode::AtBounds, 0);
        flex.end();
        Self { flex }
    }
}
