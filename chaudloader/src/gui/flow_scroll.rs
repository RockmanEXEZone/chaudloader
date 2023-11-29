use fltk::prelude::*;

#[derive(Debug, Clone)]
pub struct FlowScroll {
    inner: fltk::group::Scroll,
}

impl Default for FlowScroll {
    fn default() -> Self {
        Self::new(0, 0, 0, 0, None)
    }
}
impl FlowScroll {
    pub fn new<T: Into<Option<&'static str>>>(x: i32, y: i32, w: i32, h: i32, label: T) -> Self {
        let mut inner = fltk::group::Scroll::new(x, y, w, h, label);
        inner.set_scrollbar_size(15);
        inner.set_type(fltk::group::ScrollType::VerticalAlways);

        // Dummy frame which will be positioned in the background
        // This keeps the scrollbars stable when using padding
        let mut dummy = fltk::frame::Frame::default();

        // Exclude any built-in children of the inner widget
        // Also want to exclude the dummy, so we do this after creating it
        let mut ignore_children = Vec::new();
        for i in 0..inner.children() {
            ignore_children.push(inner.child(i).unwrap());
        }

        inner.resize_callback(move |inner, x, y, w, _| {
            let mut h = 0;
            let w = w - if inner.scrollbar().visible() {
                inner.scrollbar_size()
            } else {
                0
            };

            for i in 0..inner.children() {
                let mut c = inner.child(i).unwrap();
                if ignore_children.contains(&c) {
                    // Ignore built-in children
                    continue;
                }

                // Call child's resize callback
                c.resize(x, y + h, w, c.height());
                // Child may resize itself further, so have to call c.height() again

                // Go to next row
                h += c.height();
            }

            dummy.set_frame(fltk::enums::FrameType::FlatBox);
            dummy.set_color(fltk::enums::Color::from_hex(0xFF7FFF));
            dummy.resize(x, y, w, h);
        });
        Self { inner }
    }

    pub fn end(&mut self) {
        self.inner.end();
        // Force a resize to ensure callback is called at least once before initial draw
        let x = self.x();
        let y = self.y();
        let w = self.w();
        let h = self.h();
        self.resize(x, y, w, h);
    }
}

fltk::widget_extends!(FlowScroll, fltk::group::Scroll, inner);
