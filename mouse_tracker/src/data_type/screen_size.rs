use anyhow::bail;
use display_info::DisplayInfo;
use errors::Error::ScreenSizeError;

#[derive(Debug, Clone)]
pub struct ScreenSize {
    pub max_width: u32,
    pub max_height: u32
}

impl ScreenSize {
    pub fn new() -> anyhow::Result<Self> {
        let display_infos = DisplayInfo::all();

        match display_infos {
            Ok(infos) => {
                match infos.iter().find(|display_info| { display_info.is_primary }) {
                    Some(display_info) => {
                        Ok(ScreenSize{ max_width: display_info.width, max_height: display_info.height })
                    },
                    None => { bail!(ScreenSizeError) }
                }
            },
            Err(_err) => { bail!(ScreenSizeError) }
        }
    }

}