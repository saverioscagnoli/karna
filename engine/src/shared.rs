use assets::AssetServer;
use imgui::SharedImgui;
use logging::debug;

#[derive(Clone)]
pub struct SharedResources {
    pub assets: AssetServer,
    pub imgui: SharedImgui,
}

impl SharedResources {
    pub fn new() -> Self {
        let assets = AssetServer::_new();
        debug!("Asset server initialized");

        let imgui = SharedImgui::new();
        debug!("Imgui context manager initialized");

        Self { assets, imgui }
    }
}
