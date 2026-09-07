//! The `ctx` object handed to every scene callback: `window`, `time`, `input`
//! and `assets`.
//!
//! None of these own anything. Each is a thin wrapper around a [`Slot`] that
//! the Rust side fills for the duration of one callback, so a script that keeps
//! `ctx.window` past the end of `update` gets a `TypeError` on next use rather
//! than reading freed memory.
//!
//! Every accessor here is a method, never a property. Reaching through a slot
//! into the engine is a real call — `ctx.time.delta()` should not read like a
//! cached field, because it costs a slot check and a call every time.
//!
//! The setters the engine exposes as `&mut` — `setTitle` and friends — are only
//! usable where the engine itself passes the handle mutably, that is everywhere
//! except `draw`.

use engine::AssetServer;
use engine::Input;
use engine::PresentMode;
use engine::SceneId;
use engine::Time;
use engine::WindowHandle;
use math as m;
use rquickjs::Ctx;
use rquickjs::Exception;
use rquickjs::JsLifetime;
use rquickjs::Result;
use rquickjs::class::Trace;
use rquickjs::function::Opt;

use crate::enums::JsButton;
use crate::enums::JsCursor;
use crate::enums::JsFont;
use crate::enums::JsImage;
use crate::enums::JsKey;
use crate::slot::Slot;
use crate::value::JsSize;
use crate::value::JsVec2;

#[derive(Trace, JsLifetime)]
#[rquickjs::class(rename = "Window")]
pub struct JsWindow {
    #[qjs(skip_trace)]
    pub slot: Slot<WindowHandle>,
}

#[rquickjs::methods]
impl JsWindow {
    pub fn title(&self, ctx: Ctx<'_>) -> Result<String> {
        Ok(self.slot.borrow(&ctx)?.title().to_string())
    }

    #[qjs(rename = "setTitle")]
    pub fn set_title(&self, ctx: Ctx<'_>, title: String) -> Result<()> {
        self.slot.borrow_mut(&ctx)?.set_title(title);
        Ok(())
    }

    pub fn size(&self, ctx: Ctx<'_>) -> Result<JsSize> {
        Ok(self.slot.borrow(&ctx)?.size().into())
    }

    #[qjs(rename = "setSize")]
    pub fn set_size(&self, ctx: Ctx<'_>, width: f32, height: f32) -> Result<()> {
        let size = m::Size::new(width.max(1.0) as u32, height.max(1.0) as u32);

        self.slot.borrow_mut(&ctx)?.set_size(size);
        Ok(())
    }

    pub fn width(&self, ctx: Ctx<'_>) -> Result<f32> {
        Ok(self.slot.borrow(&ctx)?.size().width as f32)
    }

    pub fn height(&self, ctx: Ctx<'_>) -> Result<f32> {
        Ok(self.slot.borrow(&ctx)?.size().height as f32)
    }

    pub fn resizable(&self, ctx: Ctx<'_>) -> Result<bool> {
        Ok(self.slot.borrow(&ctx)?.is_resizable())
    }

    #[qjs(rename = "setResizable")]
    pub fn set_resizable(&self, ctx: Ctx<'_>, value: bool) -> Result<()> {
        self.slot.borrow_mut(&ctx)?.set_resizable(value);
        Ok(())
    }

    #[qjs(rename = "mousePosition")]
    pub fn mouse_position(&self, ctx: Ctx<'_>) -> Result<JsVec2> {
        Ok(self.slot.borrow(&ctx)?.mouse_position().into())
    }

    /// Movement accumulated since the last frame, in pixels.
    #[qjs(rename = "mouseDelta")]
    pub fn mouse_delta(&self, ctx: Ctx<'_>) -> Result<JsVec2> {
        Ok(self.slot.borrow(&ctx)?.mouse_delta().into())
    }

    #[qjs(rename = "setCursor")]
    pub fn set_cursor(&self, ctx: Ctx<'_>, cursor: JsCursor) -> Result<()> {
        self.slot.borrow(&ctx)?.set_cursor(cursor.inner);
        Ok(())
    }

    /// `"vsync"`, `"immediate"` or `"mailbox"`.
    ///
    /// The request is ignored, with a log line, if the driver does not support
    /// the mode.
    #[qjs(rename = "setPresentMode")]
    pub fn set_present_mode(&self, ctx: Ctx<'_>, mode: String) -> Result<()> {
        let mode = match mode.as_str() {
            "vsync" => PresentMode::VSYNC,
            "immediate" => PresentMode::IMMEDIATE,
            "mailbox" => PresentMode::MAILBOX,
            other => {
                return Err(Exception::throw_type(
                    &ctx,
                    &format!(
                        "unknown present mode {other:?}, expected \"vsync\", \
                         \"immediate\" or \"mailbox\""
                    ),
                ));
            }
        };

        self.slot.borrow_mut(&ctx)?.set_present_mode(mode);
        Ok(())
    }

    #[qjs(rename = "loadScene")]
    pub fn load_scene(&self, ctx: Ctx<'_>, name: String) -> Result<()> {
        self.slot.borrow(&ctx)?.load_scene(SceneId::new_str(&name));
        Ok(())
    }

    #[qjs(rename = "unloadScene")]
    pub fn unload_scene(&self, ctx: Ctx<'_>, name: String) -> Result<()> {
        self.slot
            .borrow(&ctx)?
            .unload_scene(SceneId::new_str(&name));
        Ok(())
    }

    #[qjs(rename = "activateScene")]
    pub fn activate_scene(&self, ctx: Ctx<'_>, name: String) -> Result<()> {
        self.slot
            .borrow(&ctx)?
            .activate_scene(SceneId::new_str(&name));
        Ok(())
    }

    #[qjs(rename = "deactivateScene")]
    pub fn deactivate_scene(&self, ctx: Ctx<'_>, name: String) -> Result<()> {
        self.slot
            .borrow(&ctx)?
            .deactivate_scene(SceneId::new_str(&name));
        Ok(())
    }

    #[qjs(rename = "startTextInput")]
    pub fn start_text_input(&self, ctx: Ctx<'_>) -> Result<()> {
        self.slot.borrow(&ctx)?.start_text_input();
        Ok(())
    }

    #[qjs(rename = "stopTextInput")]
    pub fn stop_text_input(&self, ctx: Ctx<'_>) -> Result<()> {
        self.slot.borrow(&ctx)?.stop_text_input();
        Ok(())
    }

    /// Tells the IME where the text being edited is, so its candidate window
    /// does not cover it.
    #[qjs(rename = "setTextInputArea")]
    pub fn set_text_input_area(
        &self,
        ctx: Ctx<'_>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        cursor: Opt<i32>,
    ) -> Result<()> {
        self.slot.borrow(&ctx)?.set_text_input_area(
            m::Vector2::new(x as i32, y as i32),
            m::Size::new(width.max(0.0) as u32, height.max(0.0) as u32),
            cursor.0.unwrap_or(0),
        );

        Ok(())
    }

    #[qjs(rename = "clearTextInputArea")]
    pub fn clear_text_input_area(&self, ctx: Ctx<'_>) -> Result<()> {
        self.slot.borrow(&ctx)?.clear_text_input_area();
        Ok(())
    }
}

#[derive(Trace, JsLifetime)]
#[rquickjs::class(rename = "Time")]
pub struct JsTime {
    #[qjs(skip_trace)]
    pub slot: Slot<Time>,
}

#[rquickjs::methods]
impl JsTime {
    /// Seconds since the previous frame.
    pub fn delta(&self, ctx: Ctx<'_>) -> Result<f32> {
        Ok(self.slot.borrow(&ctx)?.delta())
    }

    /// The fixed timestep, for use in `fixedUpdate`.
    #[qjs(rename = "fixedDelta")]
    pub fn fixed_delta(&self, ctx: Ctx<'_>) -> Result<f32> {
        Ok(self.slot.borrow(&ctx)?.fixed_delta())
    }

    pub fn fps(&self, ctx: Ctx<'_>) -> Result<f32> {
        Ok(self.slot.borrow(&ctx)?.fps())
    }

    /// How far the current frame sits between two fixed ticks, in `0..1` —
    /// the blend factor for interpolating `fixedUpdate` state when drawing.
    pub fn alpha(&self, ctx: Ctx<'_>) -> Result<f32> {
        Ok(self.slot.borrow(&ctx)?.alpha())
    }

    /// Wall-clock seconds the last frame took.
    pub fn frame(&self, ctx: Ctx<'_>) -> Result<f32> {
        Ok(self.slot.borrow(&ctx)?.frame().as_secs_f32())
    }

    #[qjs(rename = "setTargetFps")]
    pub fn set_target_fps(&self, ctx: Ctx<'_>, target: u32) -> Result<()> {
        self.slot.borrow(&ctx)?.set_target_fps(target);
        Ok(())
    }

    #[qjs(rename = "setTargetTps")]
    pub fn set_target_tps(&self, ctx: Ctx<'_>, target: u32) -> Result<()> {
        self.slot.borrow(&ctx)?.set_target_tps(target);
        Ok(())
    }
}

#[derive(Trace, JsLifetime)]
#[rquickjs::class(rename = "Input")]
pub struct JsInput {
    #[qjs(skip_trace)]
    pub slot: Slot<Input>,
}

#[rquickjs::methods]
impl JsInput {
    /// True for as long as the key is held.
    #[qjs(rename = "keyDown")]
    pub fn key_down(&self, ctx: Ctx<'_>, key: JsKey) -> Result<bool> {
        Ok(self.slot.borrow(&ctx)?.key_down(key.inner))
    }

    /// True only on the callback where the key went down.
    #[qjs(rename = "keyPressed")]
    pub fn key_pressed(&self, ctx: Ctx<'_>, key: JsKey) -> Result<bool> {
        Ok(self.slot.borrow(&ctx)?.key_pressed(key.inner))
    }

    #[qjs(rename = "keyReleased")]
    pub fn key_released(&self, ctx: Ctx<'_>, key: JsKey) -> Result<bool> {
        Ok(self.slot.borrow(&ctx)?.key_released(key.inner))
    }

    #[qjs(rename = "mouseDown")]
    pub fn mouse_down(&self, ctx: Ctx<'_>, button: JsButton) -> Result<bool> {
        Ok(self.slot.borrow(&ctx)?.mouse_down(button.inner))
    }

    #[qjs(rename = "mousePressed")]
    pub fn mouse_pressed(&self, ctx: Ctx<'_>, button: JsButton) -> Result<bool> {
        Ok(self.slot.borrow(&ctx)?.mouse_pressed(button.inner))
    }

    #[qjs(rename = "mouseReleased")]
    pub fn mouse_released(&self, ctx: Ctx<'_>, button: JsButton) -> Result<bool> {
        Ok(self.slot.borrow(&ctx)?.mouse_released(button.inner))
    }

    #[qjs(rename = "mouseWheel")]
    pub fn mouse_wheel(&self, ctx: Ctx<'_>) -> Result<JsVec2> {
        Ok(self.slot.borrow(&ctx)?.mouse_wheel().into())
    }

    /// Text committed since the last frame, once text input is started.
    pub fn text(&self, ctx: Ctx<'_>) -> Result<String> {
        Ok(self.slot.borrow(&ctx)?.text().to_string())
    }

    /// The IME's in-progress composition, not yet committed.
    pub fn preedit(&self, ctx: Ctx<'_>) -> Result<String> {
        Ok(self.slot.borrow(&ctx)?.preedit().to_string())
    }

    #[qjs(rename = "preeditCursor")]
    pub fn preedit_cursor(&self, ctx: Ctx<'_>) -> Result<i32> {
        Ok(self.slot.borrow(&ctx)?.preedit_cursor())
    }
}

#[derive(Trace, JsLifetime)]
#[rquickjs::class(rename = "Assets")]
pub struct JsAssets {
    #[qjs(skip_trace)]
    pub slot: Slot<AssetServer>,
}

#[rquickjs::methods]
impl JsAssets {
    /// Queues `path` for loading and returns its handle immediately.
    ///
    /// Paths are relative to the app's asset root. Drawing the handle before
    /// the load finishes draws the placeholder, so there is no need to wait.
    #[qjs(rename = "loadImage")]
    pub fn load_image(&self, ctx: Ctx<'_>, path: String) -> Result<JsImage> {
        Ok(self.slot.borrow_mut(&ctx)?.load_image(path).into())
    }

    #[qjs(rename = "imageSize")]
    pub fn image_size(&self, ctx: Ctx<'_>, image: JsImage) -> Result<JsSize> {
        Ok(self.slot.borrow(&ctx)?.get_image(image.inner).size.into())
    }

    #[qjs(rename = "isImagePending")]
    pub fn is_image_pending(&self, ctx: Ctx<'_>, image: JsImage) -> Result<bool> {
        Ok(self.slot.borrow(&ctx)?.is_image_pending(image.inner))
    }

    #[qjs(rename = "isImageReady")]
    pub fn is_image_ready(&self, ctx: Ctx<'_>, image: JsImage) -> Result<bool> {
        Ok(self.slot.borrow(&ctx)?.is_image_ready(image.inner))
    }

    #[qjs(rename = "placeholderImage")]
    pub fn placeholder_image(&self, ctx: Ctx<'_>) -> Result<JsImage> {
        Ok(self.slot.borrow(&ctx)?.placeholder_image().into())
    }

    #[qjs(rename = "loadFont")]
    pub fn load_font(&self, ctx: Ctx<'_>, path: String) -> Result<JsFont> {
        Ok(self.slot.borrow_mut(&ctx)?.load_font(path).into())
    }

    /// Looks the font up by family name through the system's font config.
    #[qjs(rename = "systemFont")]
    pub fn system_font(&self, ctx: Ctx<'_>, name: String) -> Result<JsFont> {
        Ok(self.slot.borrow_mut(&ctx)?.system_font(name).into())
    }

    #[qjs(rename = "fontFamily")]
    pub fn font_family(&self, ctx: Ctx<'_>, font: JsFont) -> Result<Option<String>> {
        Ok(self
            .slot
            .borrow(&ctx)?
            .font_family(font.inner)
            .map(str::to_string))
    }
}
