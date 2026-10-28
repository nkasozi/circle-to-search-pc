use iced::widget::image::Handle;

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: u32,
    pub app_name: String,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub is_minimized: bool,
    pub thumbnail: Option<Handle>,
}

impl WindowInfo {
    pub fn build(
        id: u32,
        app_name: String,
        title: String,
        width: u32,
        height: u32,
        is_minimized: bool,
        thumbnail: Option<Handle>,
    ) -> Self {
        Self {
            id,
            app_name,
            title,
            width,
            height,
            is_minimized,
            thumbnail,
        }
    }

    pub fn display_name(&self) -> String {
        match (self.title.is_empty(), self.app_name.is_empty()) {
            (true, _) => self.app_name.clone(),
            (false, true) => self.title.clone(),
            (false, false) => format!("{} - {}", self.app_name, self.title),
        }
    }
}
