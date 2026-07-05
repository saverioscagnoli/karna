use assets::AssetServer;
use imgui::SharedImgui;

#[derive(Clone)]
pub struct SharedResources {
    pub assets: AssetServer,
    pub imgui: SharedImgui,
}

impl SharedResources {
    pub fn new() -> Self {
        Self {
            assets: AssetServer::_new(),
            imgui: SharedImgui::new(),
        }
    }
}
