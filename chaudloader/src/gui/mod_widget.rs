use crate::mods;
use fltk::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct ModWidget {
    group: fltk::group::Group,
}

fltk::widget_extends!(ModWidget, fltk::group::Group, group);

impl ModWidget {
    pub fn new(mod_info: &mods::Info) -> Self {
        let mut group = fltk::group::Group::default().with_size(100, 80);

        // Note: width is percentual but height is actual pixels!
        let mut flex = fltk::group::Flex::default_fill().row();
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

        let mut mod_name = fltk::frame::Frame::default().with_label(&mod_info.title);
        mod_name.set_label_font(fltk::enums::Font::HelveticaBold);
        mod_name.set_align(
            fltk::enums::Align::TopLeft | fltk::enums::Align::Inside | fltk::enums::Align::Wrap,
        );

        let spacer = fltk::widget::Widget::default();
        flex.fixed(&spacer, 30);

        flex.end();

        let mut enable_button = fltk::button::CheckButton::default().with_size(20, 20);
        enable_button.set_frame(fltk::enums::FrameType::FlatBox);
        enable_button.set_color(fltk::enums::Color::from_hex(0x7F7FFF));
        let enable_button_w = enable_button.w();
        let enable_button_h = enable_button.h();

        // Anchor enable button
        group.resize_callback(move |group, x, y, w, h| {
            enable_button.resize(
                x + w - enable_button_w,
                y,
                enable_button_w,
                enable_button_h,
            );
        });

        group.end();
        Self { group }
    }
}
