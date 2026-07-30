use std::marker::PhantomData;
use std::os::raw::c_char;

use dear_imgui_sys::*;
use math::num_traits::Num;

type Vec2Raw = ImVec2_c;
type Vec4Raw = ImVec4_c;

const INLINE: usize = 128;

#[inline]
pub(crate) fn scratch<R>(s: &str, f: impl FnOnce(*const c_char) -> R) -> R {
    let b = s.as_bytes();
    if b.len() < INLINE {
        let mut tmp = [0u8; INLINE];
        tmp[..b.len()].copy_from_slice(b);
        f(tmp.as_ptr().cast())
    } else {
        let mut v = Vec::with_capacity(b.len() + 1);
        v.extend_from_slice(b);
        v.push(0);
        f(v.as_ptr().cast())
    }
}

#[inline]
pub(crate) fn scratch2<R>(
    a: &str,
    b: &str,
    f: impl FnOnce(*const c_char, *const c_char) -> R,
) -> R {
    scratch(a, |pa| scratch(b, |pb| f(pa, pb)))
}

pub trait AsVec2: Copy {
    fn as_vec2(self) -> Vec2Raw;
}

pub trait AsVec4: Copy {
    fn as_vec4(self) -> Vec4Raw;
}

#[inline]
const fn v2(x: f32, y: f32) -> Vec2Raw {
    Vec2Raw { x, y }
}

#[inline]
const fn v4(x: f32, y: f32, z: f32, w: f32) -> Vec4Raw {
    Vec4Raw { x, y, z, w }
}

#[inline]
fn from_v2(v: Vec2Raw) -> math::Vector2<f32> {
    math::Vector2::new(v.x, v.y)
}

#[inline]
fn from_v4(v: Vec4Raw) -> math::Vector4<f32> {
    math::Vector4::new(v.x, v.y, v.z, v.w)
}

impl AsVec2 for Vec2Raw {
    #[inline]
    fn as_vec2(self) -> Vec2Raw {
        self
    }
}

impl AsVec2 for [f32; 2] {
    #[inline]
    fn as_vec2(self) -> Vec2Raw {
        v2(self[0], self[1])
    }
}

impl AsVec2 for (f32, f32) {
    #[inline]
    fn as_vec2(self) -> Vec2Raw {
        v2(self.0, self.1)
    }
}

impl<T: Num + Copy + Into<f32>> AsVec2 for math::Vector2<T> {
    #[inline]
    fn as_vec2(self) -> Vec2Raw {
        v2(self.x.into(), self.y.into())
    }
}

impl<V: AsVec2> AsVec2 for &V {
    #[inline]
    fn as_vec2(self) -> Vec2Raw {
        (*self).as_vec2()
    }
}

impl AsVec4 for Vec4Raw {
    #[inline]
    fn as_vec4(self) -> Vec4Raw {
        self
    }
}

impl AsVec4 for [f32; 4] {
    #[inline]
    fn as_vec4(self) -> Vec4Raw {
        v4(self[0], self[1], self[2], self[3])
    }
}

impl AsVec4 for (f32, f32, f32, f32) {
    #[inline]
    fn as_vec4(self) -> Vec4Raw {
        v4(self.0, self.1, self.2, self.3)
    }
}

impl<T: Num + Copy + Into<f32>> AsVec4 for math::Vector4<T> {
    #[inline]
    fn as_vec4(self) -> Vec4Raw {
        v4(self.x.into(), self.y.into(), self.z.into(), self.w.into())
    }
}

impl<V: AsVec4> AsVec4 for &V {
    #[inline]
    fn as_vec4(self) -> Vec4Raw {
        (*self).as_vec4()
    }
}

const _: () = assert!(
    std::mem::size_of::<math::Vector2<f32>>() == std::mem::size_of::<Vec2Raw>(),
    "math::Vector2<f32> must be layout-compatible with ImVec2_c"
);
const _: () = assert!(
    std::mem::align_of::<math::Vector2<f32>>() == std::mem::align_of::<Vec2Raw>(),
    "math::Vector2<f32> must be layout-compatible with ImVec2_c"
);

#[inline]
pub fn as_imvec2_slice(p: &[math::Vector2<f32>]) -> &[Vec2Raw] {
    unsafe { std::slice::from_raw_parts(p.as_ptr().cast(), p.len()) }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Condition {
    Always = ImGuiCond_Always as i32,
    Once = ImGuiCond_Once as i32,
    FirstUseEver = ImGuiCond_FirstUseEver as i32,
    Appearing = ImGuiCond_Appearing as i32,
}

impl Condition {
    #[inline]
    const fn bits(self) -> ImGuiCond {
        self as i32 as ImGuiCond
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct WindowFlags(ImGuiWindowFlags);

macro_rules! window_flags {
    ($($name:ident => $raw:ident),* $(,)?) => {
        impl WindowFlags {
            pub const NONE: Self = Self(0);
            $(pub const $name: Self = Self($raw as ImGuiWindowFlags);)*
        }
    };
}

window_flags! {
    NO_TITLE_BAR                => ImGuiWindowFlags_NoTitleBar,
    NO_RESIZE                   => ImGuiWindowFlags_NoResize,
    NO_MOVE                     => ImGuiWindowFlags_NoMove,
    NO_SCROLLBAR                => ImGuiWindowFlags_NoScrollbar,
    NO_SCROLL_WITH_MOUSE        => ImGuiWindowFlags_NoScrollWithMouse,
    NO_COLLAPSE                 => ImGuiWindowFlags_NoCollapse,
    ALWAYS_AUTO_RESIZE          => ImGuiWindowFlags_AlwaysAutoResize,
    NO_BACKGROUND               => ImGuiWindowFlags_NoBackground,
    NO_SAVED_SETTINGS           => ImGuiWindowFlags_NoSavedSettings,
    NO_MOUSE_INPUTS             => ImGuiWindowFlags_NoMouseInputs,
    MENU_BAR                    => ImGuiWindowFlags_MenuBar,
    HORIZONTAL_SCROLLBAR        => ImGuiWindowFlags_HorizontalScrollbar,
    NO_FOCUS_ON_APPEARING       => ImGuiWindowFlags_NoFocusOnAppearing,
    NO_BRING_TO_FRONT_ON_FOCUS  => ImGuiWindowFlags_NoBringToFrontOnFocus,
    ALWAYS_VERTICAL_SCROLLBAR   => ImGuiWindowFlags_AlwaysVerticalScrollbar,
    ALWAYS_HORIZONTAL_SCROLLBAR => ImGuiWindowFlags_AlwaysHorizontalScrollbar,
    NO_NAV_INPUTS               => ImGuiWindowFlags_NoNavInputs,
    NO_NAV_FOCUS                => ImGuiWindowFlags_NoNavFocus,
    UNSAVED_DOCUMENT            => ImGuiWindowFlags_UnsavedDocument,
}

impl WindowFlags {
    #[inline]
    pub const fn bits(self) -> ImGuiWindowFlags {
        self.0
    }
    #[inline]
    pub const fn from_bits_retain(bits: ImGuiWindowFlags) -> Self {
        Self(bits)
    }
    #[inline]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    #[inline]
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl std::ops::BitOr for WindowFlags {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl std::ops::BitOrAssign for WindowFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for WindowFlags {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::Not for WindowFlags {
    type Output = Self;
    #[inline]
    fn not(self) -> Self {
        Self(!self.0)
    }
}

pub struct Ui<'frame> {
    frame: PhantomData<*mut &'frame ()>,
}

impl<'frame> Ui<'frame> {
    #[inline]
    pub(crate) fn new() -> Self {
        Self { frame: PhantomData }
    }

    #[inline]
    pub fn window<'ui, 'p, N: AsRef<str>>(&'ui self, name: N) -> WindowBuilder<'ui, 'frame, 'p, N> {
        WindowBuilder {
            ui: self,
            name,
            open: None,
            flags: WindowFlags::NONE,
            pos: None,
            size: None,
            size_constraints: None,
            collapsed: None,
            focused: false,
            bg_alpha: None,
        }
    }

    pub fn text(&self, s: impl AsRef<str>) {
        let s = s.as_ref();
        unsafe {
            let start = s.as_ptr() as *const c_char;
            igTextUnformatted(start, start.add(s.len()));
        }
    }

    pub fn text_colored(&self, color: impl AsVec4, s: impl AsRef<str>) {
        unsafe {
            igPushStyleColor_Vec4(ImGuiCol_Text as i32, color.as_vec4());
        }
        self.text(s);
        unsafe {
            igPopStyleColor(1);
        }
    }

    pub fn text_disabled(&self, s: impl AsRef<str>) {
        unsafe {
            igBeginDisabled(true);
        }
        self.text(s);
        unsafe {
            igEndDisabled();
        }
    }

    pub fn text_wrapped(&self, s: impl AsRef<str>) {
        unsafe {
            igPushTextWrapPos(0.0);
        }
        self.text(s);
        unsafe {
            igPopTextWrapPos();
        }
    }

    pub fn label(&self, label: impl AsRef<str>, value: impl AsRef<str>) {
        scratch2(label.as_ref(), value.as_ref(), |l, v| unsafe {
            igLabelText(l, v)
        })
    }

    pub fn separator(&self) {
        unsafe { igSeparator() }
    }

    pub fn separator_text(&self, label: impl AsRef<str>) {
        scratch(label.as_ref(), |l| unsafe { igSeparatorText(l) })
    }

    pub fn same_line(&self) {
        unsafe { igSameLine(0.0, -1.0) }
    }

    pub fn same_line_with(&self, offset_from_start: f32, spacing: f32) {
        unsafe { igSameLine(offset_from_start, spacing) }
    }

    pub fn spacing(&self) {
        unsafe { igSpacing() }
    }

    pub fn new_line(&self) {
        unsafe { igNewLine() }
    }

    pub fn indent(&self, width: f32) {
        unsafe { igIndent(width) }
    }

    pub fn unindent(&self, width: f32) {
        unsafe { igUnindent(width) }
    }

    pub fn dummy(&self, size: impl AsVec2) {
        unsafe { igDummy(size.as_vec2()) }
    }

    pub fn cursor_screen_pos(&self) -> math::Vector2<f32> {
        unsafe { from_v2(igGetCursorScreenPos()) }
    }

    pub fn set_cursor_screen_pos(&self, pos: impl AsVec2) {
        unsafe { igSetCursorScreenPos(pos.as_vec2()) }
    }

    pub fn cursor_pos(&self) -> math::Vector2<f32> {
        unsafe { from_v2(igGetCursorPos()) }
    }

    pub fn set_cursor_pos(&self, pos: impl AsVec2) {
        unsafe { igSetCursorPos(pos.as_vec2()) }
    }

    pub fn content_region_avail(&self) -> math::Vector2<f32> {
        unsafe { from_v2(igGetContentRegionAvail()) }
    }

    pub fn window_pos(&self) -> math::Vector2<f32> {
        unsafe { from_v2(igGetWindowPos()) }
    }

    pub fn window_size(&self) -> math::Vector2<f32> {
        unsafe { from_v2(igGetWindowSize()) }
    }

    pub fn mouse_pos(&self) -> math::Vector2<f32> {
        unsafe { from_v2(igGetMousePos()) }
    }

    pub fn item_rect_min(&self) -> math::Vector2<f32> {
        unsafe { from_v2(igGetItemRectMin()) }
    }

    pub fn item_rect_max(&self) -> math::Vector2<f32> {
        unsafe { from_v2(igGetItemRectMax()) }
    }

    pub fn item_rect_size(&self) -> math::Vector2<f32> {
        unsafe { from_v2(igGetItemRectSize()) }
    }

    pub fn style_color(&self, idx: ImGuiCol) -> math::Vector4<f32> {
        unsafe { from_v4(*igGetStyleColorVec4(idx as i32)) }
    }

    pub fn calc_text_size(&self, s: impl AsRef<str>) -> math::Vector2<f32> {
        let s = s.as_ref();
        unsafe {
            let start = s.as_ptr() as *const c_char;
            from_v2(igCalcTextSize(start, start.add(s.len()), false, -1.0))
        }
    }

    pub fn button(&self, label: impl AsRef<str>, size: impl AsVec2) -> bool {
        scratch(label.as_ref(), |l| unsafe { igButton(l, size.as_vec2()) })
    }

    pub fn small_button(&self, label: impl AsRef<str>) -> bool {
        scratch(label.as_ref(), |l| unsafe { igSmallButton(l) })
    }

    pub fn invisible_button(&self, id: impl AsRef<str>, size: impl AsVec2) -> bool {
        scratch(id.as_ref(), |i| unsafe {
            igInvisibleButton(i, size.as_vec2(), 0)
        })
    }

    pub fn checkbox(&self, label: impl AsRef<str>, value: &mut bool) -> bool {
        scratch(label.as_ref(), |l| unsafe { igCheckbox(l, value) })
    }

    pub fn radio_button(&self, label: impl AsRef<str>, active: bool) -> bool {
        scratch(label.as_ref(), |l| unsafe { igRadioButton_Bool(l, active) })
    }

    pub fn slider_f32(&self, label: impl AsRef<str>, value: &mut f32, min: f32, max: f32) -> bool {
        scratch2(label.as_ref(), "%.3f", |l, fmt| unsafe {
            igSliderFloat(l, value, min, max, fmt, 0)
        })
    }

    pub fn drag_f32(&self, label: impl AsRef<str>, value: &mut f32, speed: f32) -> bool {
        scratch2(label.as_ref(), "%.3f", |l, fmt| unsafe {
            igDragFloat(l, value, speed, 0.0, 0.0, fmt, 0)
        })
    }

    pub fn drag_vec2(
        &self,
        label: impl AsRef<str>,
        value: &mut math::Vector2<f32>,
        speed: f32,
    ) -> bool {
        let mut tmp = [value.x, value.y];
        let changed = scratch2(label.as_ref(), "%.3f", |l, fmt| unsafe {
            igDragFloat2(l, tmp.as_mut_ptr(), speed, 0.0, 0.0, fmt, 0)
        });
        if changed {
            value.x = tmp[0];
            value.y = tmp[1];
        }
        changed
    }

    pub fn drag_vec3(
        &self,
        label: impl AsRef<str>,
        value: &mut math::Vector3<f32>,
        speed: f32,
    ) -> bool {
        let mut tmp = [value.x, value.y, value.z];
        let changed = scratch2(label.as_ref(), "%.3f", |l, fmt| unsafe {
            igDragFloat3(l, tmp.as_mut_ptr(), speed, 0.0, 0.0, fmt, 0)
        });
        if changed {
            value.x = tmp[0];
            value.y = tmp[1];
            value.z = tmp[2];
        }
        changed
    }

    pub fn color_edit4(&self, label: impl AsRef<str>, value: &mut math::Vector4<f32>) -> bool {
        let mut tmp = [value.x, value.y, value.z, value.w];
        let changed = scratch(label.as_ref(), |l| unsafe {
            igColorEdit4(l, tmp.as_mut_ptr(), 0)
        });
        if changed {
            value.x = tmp[0];
            value.y = tmp[1];
            value.z = tmp[2];
            value.w = tmp[3];
        }
        changed
    }

    /// Dear imgui's built-in widget gallery. Handy to eyeball the backend.
    pub fn show_demo_window(&self, open: &mut bool) {
        unsafe { igShowDemoWindow(open) }
    }

    pub fn show_metrics_window(&self, open: &mut bool) {
        unsafe { igShowMetricsWindow(open) }
    }

    pub fn is_item_hovered(&self) -> bool {
        unsafe { igIsItemHovered(0) }
    }

    pub fn is_item_active(&self) -> bool {
        unsafe { igIsItemActive() }
    }

    pub fn is_item_clicked(&self) -> bool {
        unsafe { igIsItemClicked(0) }
    }

    pub fn is_window_hovered(&self) -> bool {
        unsafe { igIsWindowHovered(0) }
    }

    pub fn tooltip<R>(&self, f: impl FnOnce(&Ui<'frame>) -> R) -> R {
        unsafe { igBeginTooltip() };
        let _guard = EndTooltip;
        f(self)
    }

    pub fn popup<R>(&self, id: impl AsRef<str>, f: impl FnOnce(&Ui<'frame>) -> R) -> Option<R> {
        let opened = scratch(id.as_ref(), |i| unsafe { igBeginPopup(i, 0) });
        if !opened {
            return None;
        }
        let _guard = EndPopup;
        Some(f(self))
    }

    pub fn open_popup(&self, id: impl AsRef<str>) {
        scratch(id.as_ref(), |i| unsafe { igOpenPopup_Str(i, 0) })
    }

    pub fn child<R>(
        &self,
        id: impl AsRef<str>,
        size: impl AsVec2,
        border: bool,
        f: impl FnOnce(&Ui<'frame>) -> R,
    ) -> Option<R> {
        let child_flags = if border {
            ImGuiChildFlags_Borders as ImGuiChildFlags
        } else {
            0
        };
        let visible = scratch(id.as_ref(), |i| unsafe {
            igBeginChild_Str(i, size.as_vec2(), child_flags, 0)
        });
        let _guard = EndChild;
        if visible { Some(f(self)) } else { None }
    }

    pub fn with_id<R>(&self, id: i32, f: impl FnOnce(&Ui<'frame>) -> R) -> R {
        unsafe { igPushID_Int(id) };
        let _guard = PopId;
        f(self)
    }

    pub fn with_item_width<R>(&self, width: f32, f: impl FnOnce(&Ui<'frame>) -> R) -> R {
        unsafe { igPushItemWidth(width) };
        let _guard = PopItemWidth;
        f(self)
    }
}

struct EndWindow;

impl Drop for EndWindow {
    #[inline]
    fn drop(&mut self) {
        unsafe { igEnd() }
    }
}

struct EndChild;
impl Drop for EndChild {
    #[inline]
    fn drop(&mut self) {
        unsafe { igEndChild() }
    }
}

struct EndTooltip;
impl Drop for EndTooltip {
    #[inline]
    fn drop(&mut self) {
        unsafe { igEndTooltip() }
    }
}

struct EndPopup;
impl Drop for EndPopup {
    #[inline]
    fn drop(&mut self) {
        unsafe { igEndPopup() }
    }
}

struct PopId;
impl Drop for PopId {
    #[inline]
    fn drop(&mut self) {
        unsafe { igPopID() }
    }
}

struct PopItemWidth;
impl Drop for PopItemWidth {
    #[inline]
    fn drop(&mut self) {
        unsafe { igPopItemWidth() }
    }
}

#[must_use = "a WindowBuilder does nothing until .build() or .begin() is called"]
pub struct WindowBuilder<'ui, 'frame, 'p, N: AsRef<str>> {
    ui: &'ui Ui<'frame>,
    name: N,
    open: Option<&'p mut bool>,
    flags: WindowFlags,
    pos: Option<(Vec2Raw, Vec2Raw, Condition)>,
    size: Option<(Vec2Raw, Condition)>,
    size_constraints: Option<(Vec2Raw, Vec2Raw)>,
    collapsed: Option<(bool, Condition)>,
    focused: bool,
    bg_alpha: Option<f32>,
}

impl<'ui, 'frame, 'p, N: AsRef<str>> WindowBuilder<'ui, 'frame, 'p, N> {
    #[inline]
    pub fn position(mut self, pos: impl AsVec2, cond: Condition) -> Self {
        self.pos = Some((pos.as_vec2(), v2(0.0, 0.0), cond));
        self
    }

    #[inline]
    pub fn position_pivot(mut self, pos: impl AsVec2, pivot: impl AsVec2, cond: Condition) -> Self {
        self.pos = Some((pos.as_vec2(), pivot.as_vec2(), cond));
        self
    }

    #[inline]
    pub fn size(mut self, size: impl AsVec2, cond: Condition) -> Self {
        self.size = Some((size.as_vec2(), cond));
        self
    }

    #[inline]
    pub fn size_constraints(mut self, min: impl AsVec2, max: impl AsVec2) -> Self {
        self.size_constraints = Some((min.as_vec2(), max.as_vec2()));
        self
    }

    #[inline]
    pub fn collapsed(mut self, collapsed: bool, cond: Condition) -> Self {
        self.collapsed = Some((collapsed, cond));
        self
    }

    #[inline]
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    #[inline]
    pub fn bg_alpha(mut self, alpha: f32) -> Self {
        self.bg_alpha = Some(alpha);
        self
    }

    #[inline]
    pub fn opened(mut self, open: &'p mut bool) -> Self {
        self.open = Some(open);
        self
    }

    #[inline]
    pub fn flags(mut self, flags: WindowFlags) -> Self {
        self.flags = flags;
        self
    }

    #[inline]
    pub fn no_decoration(self) -> Self {
        let f = self.flags
            | WindowFlags::NO_TITLE_BAR
            | WindowFlags::NO_RESIZE
            | WindowFlags::NO_SCROLLBAR
            | WindowFlags::NO_COLLAPSE;
        self.flags(f)
    }

    pub fn build<R>(self, f: impl FnOnce(&Ui<'frame>) -> R) -> Option<R> {
        let (ui, visible, _guard) = self.begin_raw();
        if visible { Some(f(ui)) } else { None }
    }

    pub fn begin(self) -> Option<WindowToken<'ui>> {
        let (_ui, visible, guard) = self.begin_raw();
        if visible {
            Some(WindowToken {
                _guard: guard,
                _ui: PhantomData,
            })
        } else {
            None
        }
    }

    fn begin_raw(self) -> (&'ui Ui<'frame>, bool, EndWindow) {
        unsafe {
            if let Some((pos, pivot, cond)) = self.pos {
                igSetNextWindowPos(pos, cond.bits(), pivot);
            }
            if let Some((size, cond)) = self.size {
                igSetNextWindowSize(size, cond.bits());
            }
            if let Some((min, max)) = self.size_constraints {
                igSetNextWindowSizeConstraints(min, max, None, std::ptr::null_mut());
            }
            if let Some((collapsed, cond)) = self.collapsed {
                igSetNextWindowCollapsed(collapsed, cond.bits());
            }
            if self.focused {
                igSetNextWindowFocus();
            }
            if let Some(alpha) = self.bg_alpha {
                igSetNextWindowBgAlpha(alpha);
            }
        }

        let mut open = self.open;
        let open_ptr = open
            .as_deref_mut()
            .map_or(std::ptr::null_mut(), |b| b as *mut bool);

        let visible = scratch(self.name.as_ref(), |name| unsafe {
            igBegin(name, open_ptr, self.flags.bits())
        });

        (self.ui, visible, EndWindow)
    }
}

#[must_use = "dropping the token immediately ends the window"]
pub struct WindowToken<'ui> {
    _guard: EndWindow,
    _ui: PhantomData<*mut &'ui ()>,
}

impl<'ui> WindowToken<'ui> {
    #[inline]
    pub fn end(self) {}
}
