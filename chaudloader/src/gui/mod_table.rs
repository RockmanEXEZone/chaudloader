use fltk::prelude::*;

#[derive(Debug, Clone, Default)]
struct ModTableData {
	padding_top: i32,
	padding_right: i32,
	padding_bottom: i32,
	padding_left: i32,
	spacing_horizontal: i32,
	spacing_vertical: i32,
}
#[derive(Debug, Clone)]
pub struct ModTable {
	group: fltk::group::Group,
	data: ModTableData,
}
impl Default for ModTable {
	fn default() -> Self {
		Self::new(0, 0, 0, 0, None)
	}
}

impl ModTable {
	pub fn new<T: Into<Option<&'static str>>>(x: i32, y: i32, w: i32, h: i32, label: T) -> Self {
		let mut group = fltk::group::Group::new(x, y, w, h, label);
		group.make_resizable(false);
		let mut mod_table = Self {
			group: group,
			data: ModTableData {
				..Default::default()
			},
		};
		mod_table
	}

	pub fn with_padding(mut self, top: i32, right: i32, bottom: i32, left: i32) -> Self {
		self.data.padding_top = top;
		self.data.padding_right = right;
		self.data.padding_bottom = bottom;
		self.data.padding_left = left;
		self
	}

	pub fn with_spacing(mut self, horizontal: i32, vertical: i32) -> Self {
		self.data.spacing_horizontal = horizontal;
		self.data.spacing_vertical = vertical;
		self
	}

	pub fn end(&mut self) {
		let data = self.data.clone();
		self.group.resize_callback(move |group, x, y, w, h| {
			let children = group.children();
			let mut cx = data.padding_left;
			let mut cy = data.padding_top;
			let mut ch = 0;
			for i in 0..children {
				let mut c = group.child(i).unwrap();

				// Wrap to next row
				// Skip if there are no sized elements on this row yet
				if ch != 0 && cx + c.w() + data.padding_right > w {
					cx = data.padding_left;
					cy += ch + data.spacing_vertical;
					ch = 0;
				}
				
				// Update max height for this row
				ch = std::cmp::max(ch, c.h());

				// Set position for child
				c.set_pos(x + cx, y + cy);

				// Go to next column
				cx += c.w() + data.spacing_horizontal;
			}
		});
		// Force a resize to ensure callback is called at least once before initial draw
		let x = self.x();
		let y = self.y();
		let w = self.w();
		let h = self.h();
		self.resize(x, y, w, h);
	}
}

fltk::widget_extends!(ModTable, fltk::group::Group, group);
