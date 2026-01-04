use macros::Get;
use utils::Label;

#[derive(Get)]
pub struct SceneChanger {
    changed_to: Option<Label>,
}

impl SceneChanger {
    pub(crate) fn new() -> Self {
        Self { changed_to: None }
    }

    #[inline]
    pub fn request_change(&mut self, label: Label) {
        self.changed_to = Some(label);
    }

    #[inline]
    pub(crate) fn changed_to(&mut self) -> Option<Label> {
        self.changed_to.take()
    }
}
