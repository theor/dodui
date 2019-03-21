
// use crate::transform::{GlobalTransform, Transform};
// use specs::prelude::*;
// use crate::rendering::Text;

// use cgmath::Point2;
// use hashbrown::HashMap;

use crate::manager::*;

#[derive(Debug)]
pub struct BitmapFont(pub gfx_text::BitmapFont);

impl BitmapFont {
    pub fn measure(&self, text: &str) -> (i32, i32) {
        let mut width = 0;
        let mut last_char = None;

        for ch in text.chars() {
            let ch_info = match self.0.find_char(ch) {
                Some(info) => info,
                None => continue,
            };
            last_char = Some(ch_info);

            width += ch_info.x_advance;
        }

        match last_char {
            Some(info) => width += info.x_offset + info.width - info.x_advance,
            None => (),
        }

        (width, self.0.get_font_height() as i32)
    }
}

impl Load<Ctx, SimpleKey> for BitmapFont {
  type Error = Error;

  fn load(
    key: SimpleKey,
    _storage: &mut warmy::Storage<Ctx, SimpleKey>,
    _ctx: &mut Ctx,
  ) -> Result<Loaded<Self, SimpleKey>, Error> {
        match key {
      SimpleKey::Path(path) => {
        println!("Load BitmapFont {}", path.display());
        let bitmap = gfx_text::BitmapFont::from_path(path.to_str().unwrap(), 16, None).map_err(Error::FontError)?;
        // storage.get::<ShaderSet>(&dep, ctx).unwrap();
        Ok(Loaded::without_dep(
          BitmapFont(bitmap)
          .into()
        ))
      }

      SimpleKey::Logical(_) => Err(Error::CannotLoadFromLogical),
    }
  }
}

// pub struct MeasureSystem {
    
// }

// impl MeasureSystem {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

// impl<'a> System<'a> for MeasureSystem
// {
//     type SystemData = (
//         ReadExpect<'a, crate::manager::ResourceManager>,
//         ReadStorage<'a, Dimension>,
//         WriteStorage<'a, Transform>,
//         ReadStorage<'a, Text>,
//         // Read<'a, Screen>,
//     );
//     fn run(&mut self, (store, dim, mut tr, text): Self::SystemData) {
//         let key = SimpleKey::Path(("style/NotoSans-Regular.ttf").into());
        
//         let font = store.get::<BitmapFont>(&key);
//         let font = match font {
//             Ok(ref font) => font,
//             _ => return,
//         };
//         for (dim, mut tr, text) in (dim.maybe(), &mut tr, text.maybe()).join() {
//             if let Some(dim) = dim {
//                 tr.size = dim.size;
//             } else if let Some(text) = text {
//                 let (w, h) = font.borrow().measure(&text.text);
//                 tr.size = (w as f32, h as f32).into();
//             }
//         }
//     }
// }
