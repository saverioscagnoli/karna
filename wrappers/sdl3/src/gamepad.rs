use core::ffi::CStr;
use core::ffi::c_char;
use core::ptr;
use core::time::Duration;

use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec::Vec;
use sdl3_sys::*;

pub type GamepadId = SDL_JoystickID;

sdl_enum!(GamepadButton: SDL_GamepadButton {
    South => SDL_GAMEPAD_BUTTON_SOUTH,
    East => SDL_GAMEPAD_BUTTON_EAST,
    West => SDL_GAMEPAD_BUTTON_WEST,
    North => SDL_GAMEPAD_BUTTON_NORTH,
    Back => SDL_GAMEPAD_BUTTON_BACK,
    Guide => SDL_GAMEPAD_BUTTON_GUIDE,
    Start => SDL_GAMEPAD_BUTTON_START,
    LeftStick => SDL_GAMEPAD_BUTTON_LEFT_STICK,
    RightStick => SDL_GAMEPAD_BUTTON_RIGHT_STICK,
    LeftShoulder => SDL_GAMEPAD_BUTTON_LEFT_SHOULDER,
    RightShoulder => SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER,
    DpadUp => SDL_GAMEPAD_BUTTON_DPAD_UP,
    DpadDown => SDL_GAMEPAD_BUTTON_DPAD_DOWN,
    DpadLeft => SDL_GAMEPAD_BUTTON_DPAD_LEFT,
    DpadRight => SDL_GAMEPAD_BUTTON_DPAD_RIGHT,
    Misc1 => SDL_GAMEPAD_BUTTON_MISC1,
    RightPaddle1 => SDL_GAMEPAD_BUTTON_RIGHT_PADDLE1,
    LeftPaddle1 => SDL_GAMEPAD_BUTTON_LEFT_PADDLE1,
    RightPaddle2 => SDL_GAMEPAD_BUTTON_RIGHT_PADDLE2,
    LeftPaddle2 => SDL_GAMEPAD_BUTTON_LEFT_PADDLE2,
    Touchpad => SDL_GAMEPAD_BUTTON_TOUCHPAD,
    Misc2 => SDL_GAMEPAD_BUTTON_MISC2,
    Misc3 => SDL_GAMEPAD_BUTTON_MISC3,
    Misc4 => SDL_GAMEPAD_BUTTON_MISC4,
    Misc5 => SDL_GAMEPAD_BUTTON_MISC5,
    Misc6 => SDL_GAMEPAD_BUTTON_MISC6,
});

sdl_enum!(GamepadAxis: SDL_GamepadAxis {
    LeftX => SDL_GAMEPAD_AXIS_LEFTX,
    LeftY => SDL_GAMEPAD_AXIS_LEFTY,
    RightX => SDL_GAMEPAD_AXIS_RIGHTX,
    RightY => SDL_GAMEPAD_AXIS_RIGHTY,
    LeftTrigger => SDL_GAMEPAD_AXIS_LEFT_TRIGGER,
    RightTrigger => SDL_GAMEPAD_AXIS_RIGHT_TRIGGER,
});

sdl_enum!(GamepadType: SDL_GamepadType {
    Unknown => SDL_GAMEPAD_TYPE_UNKNOWN,
    Standard => SDL_GAMEPAD_TYPE_STANDARD,
    Xbox360 => SDL_GAMEPAD_TYPE_XBOX360,
    XboxOne => SDL_GAMEPAD_TYPE_XBOXONE,
    Ps3 => SDL_GAMEPAD_TYPE_PS3,
    Ps4 => SDL_GAMEPAD_TYPE_PS4,
    Ps5 => SDL_GAMEPAD_TYPE_PS5,
    SwitchPro => SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_PRO,
    JoyConLeft => SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_LEFT,
    JoyConRight => SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_RIGHT,
    JoyConPair => SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_PAIR,
    GameCube => SDL_GAMEPAD_TYPE_GAMECUBE,
});

sdl_enum!(GamepadButtonLabel: SDL_GamepadButtonLabel {
    Unknown => SDL_GAMEPAD_BUTTON_LABEL_UNKNOWN,
    A => SDL_GAMEPAD_BUTTON_LABEL_A,
    B => SDL_GAMEPAD_BUTTON_LABEL_B,
    X => SDL_GAMEPAD_BUTTON_LABEL_X,
    Y => SDL_GAMEPAD_BUTTON_LABEL_Y,
    Cross => SDL_GAMEPAD_BUTTON_LABEL_CROSS,
    Circle => SDL_GAMEPAD_BUTTON_LABEL_CIRCLE,
    Square => SDL_GAMEPAD_BUTTON_LABEL_SQUARE,
    Triangle => SDL_GAMEPAD_BUTTON_LABEL_TRIANGLE,
});

sdl_enum!(PowerState: SDL_PowerState {
    Error => SDL_POWERSTATE_ERROR,
    Unknown => SDL_POWERSTATE_UNKNOWN,
    OnBattery => SDL_POWERSTATE_ON_BATTERY,
    NoBattery => SDL_POWERSTATE_NO_BATTERY,
    Charging => SDL_POWERSTATE_CHARGING,
    Charged => SDL_POWERSTATE_CHARGED,
});

sdl_enum!(ConnectionState: SDL_JoystickConnectionState {
    Invalid => SDL_JOYSTICK_CONNECTION_INVALID,
    Unknown => SDL_JOYSTICK_CONNECTION_UNKNOWN,
    Wired => SDL_JOYSTICK_CONNECTION_WIRED,
    Wireless => SDL_JOYSTICK_CONNECTION_WIRELESS,
});

sdl_enum!(SensorKind: SDL_SensorType {
    Accel => SDL_SENSOR_ACCEL,
    Gyro => SDL_SENSOR_GYRO,
    AccelLeft => SDL_SENSOR_ACCEL_L,
    GyroLeft => SDL_SENSOR_GYRO_L,
    AccelRight => SDL_SENSOR_ACCEL_R,
    GyroRight => SDL_SENSOR_GYRO_R,
});

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Power {
    pub state: PowerState,
    pub percent: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchpadFinger {
    pub down: bool,
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
}

pub(crate) fn normalize_axis(value: i16) -> f32 {
    (value as f32 / SDL_JOYSTICK_AXIS_MAX as f32).clamp(-1.0, 1.0)
}

fn check(ok: bool) -> Result<(), SdlError> {
    if ok { Ok(()) } else { Err(get_error()) }
}

unsafe fn opt_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }

    unsafe { CStr::from_ptr(ptr) }.to_str().ok()
}

fn unit_to_u16(v: f32) -> u16 {
    (v.clamp(0.0, 1.0) * u16::MAX as f32) as u16
}

fn duration_ms(d: Duration) -> u32 {
    u32::try_from(d.as_millis()).unwrap_or(u32::MAX)
}

pub fn gamepads() -> Vec<GamepadId> {
    let mut count = 0;
    let ptr = unsafe { SDL_GetGamepads(&mut count) };

    if ptr.is_null() {
        return Vec::new();
    }

    let ids = unsafe { core::slice::from_raw_parts(ptr, count.max(0) as usize) }.to_vec();
    unsafe { SDL_free(ptr.cast()) };
    ids
}

pub fn has_gamepad() -> bool {
    unsafe { SDL_HasGamepad() }
}

pub fn is_gamepad(id: GamepadId) -> bool {
    unsafe { SDL_IsGamepad(id) }
}

pub fn name_for_id(id: GamepadId) -> Option<String> {
    unsafe { opt_str(SDL_GetGamepadNameForID(id)) }.map(String::from)
}

pub fn type_for_id(id: GamepadId) -> GamepadType {
    GamepadType::from_raw(unsafe { SDL_GetGamepadTypeForID(id) }).unwrap_or(GamepadType::Unknown)
}

pub fn player_index_for_id(id: GamepadId) -> Option<u32> {
    u32::try_from(unsafe { SDL_GetGamepadPlayerIndexForID(id) }).ok()
}

pub fn add_mapping(mapping: &str) -> Result<bool, SdlError> {
    let c = CString::new(mapping.replace('\0', "")).unwrap_or_default();

    match unsafe { SDL_AddGamepadMapping(c.as_ptr()) } {
        -1 => Err(get_error()),
        added => Ok(added == 1),
    }
}

pub fn set_events_enabled(enabled: bool) {
    unsafe { SDL_SetGamepadEventsEnabled(enabled) }
}

pub fn events_enabled() -> bool {
    unsafe { SDL_GamepadEventsEnabled() }
}

pub fn update() {
    unsafe { SDL_UpdateGamepads() }
}

pub struct Gamepad {
    raw: ptr::NonNull<SDL_Gamepad>,
}

impl Gamepad {
    pub fn open(id: GamepadId) -> Result<Self, SdlError> {
        ptr::NonNull::new(unsafe { SDL_OpenGamepad(id) })
            .map(|raw| Self { raw })
            .ok_or_else(get_error)
    }

    pub fn as_ptr(&self) -> *mut SDL_Gamepad {
        self.raw.as_ptr()
    }

    pub fn id(&self) -> GamepadId {
        unsafe { SDL_GetGamepadID(self.as_ptr()) }
    }

    pub fn name(&self) -> &str {
        unsafe { opt_str(SDL_GetGamepadName(self.as_ptr())) }.unwrap_or("Unknown Gamepad")
    }

    pub fn serial(&self) -> Option<&str> {
        unsafe { opt_str(SDL_GetGamepadSerial(self.as_ptr())) }
    }

    pub fn kind(&self) -> GamepadType {
        GamepadType::from_raw(unsafe { SDL_GetGamepadType(self.as_ptr()) })
            .unwrap_or(GamepadType::Unknown)
    }

    pub fn real_kind(&self) -> GamepadType {
        GamepadType::from_raw(unsafe { SDL_GetRealGamepadType(self.as_ptr()) })
            .unwrap_or(GamepadType::Unknown)
    }

    pub fn vendor(&self) -> u16 {
        unsafe { SDL_GetGamepadVendor(self.as_ptr()) }
    }

    pub fn product(&self) -> u16 {
        unsafe { SDL_GetGamepadProduct(self.as_ptr()) }
    }

    pub fn product_version(&self) -> u16 {
        unsafe { SDL_GetGamepadProductVersion(self.as_ptr()) }
    }

    pub fn connected(&self) -> bool {
        unsafe { SDL_GamepadConnected(self.as_ptr()) }
    }

    pub fn connection(&self) -> ConnectionState {
        ConnectionState::from_raw(unsafe { SDL_GetGamepadConnectionState(self.as_ptr()) })
            .unwrap_or(ConnectionState::Unknown)
    }

    pub fn power(&self) -> Power {
        let mut percent = -1;
        let state = unsafe { SDL_GetGamepadPowerInfo(self.as_ptr(), &mut percent) };

        Power {
            state: PowerState::from_raw(state).unwrap_or(PowerState::Unknown),
            percent: u8::try_from(percent).ok(),
        }
    }

    pub fn player_index(&self) -> Option<u32> {
        u32::try_from(unsafe { SDL_GetGamepadPlayerIndex(self.as_ptr()) }).ok()
    }

    pub fn set_player_index(&mut self, index: Option<u32>) -> Result<(), SdlError> {
        let index = index.and_then(|i| i32::try_from(i).ok()).unwrap_or(-1);
        check(unsafe { SDL_SetGamepadPlayerIndex(self.as_ptr(), index) })
    }

    pub fn has_button(&self, button: GamepadButton) -> bool {
        unsafe { SDL_GamepadHasButton(self.as_ptr(), button.raw()) }
    }

    pub fn button(&self, button: GamepadButton) -> bool {
        unsafe { SDL_GetGamepadButton(self.as_ptr(), button.raw()) }
    }

    pub fn button_label(&self, button: GamepadButton) -> GamepadButtonLabel {
        GamepadButtonLabel::from_raw(unsafe {
            SDL_GetGamepadButtonLabel(self.as_ptr(), button.raw())
        })
        .unwrap_or(GamepadButtonLabel::Unknown)
    }

    pub fn has_axis(&self, axis: GamepadAxis) -> bool {
        unsafe { SDL_GamepadHasAxis(self.as_ptr(), axis.raw()) }
    }

    pub fn axis(&self, axis: GamepadAxis) -> f32 {
        normalize_axis(self.axis_raw(axis))
    }

    pub fn axis_raw(&self, axis: GamepadAxis) -> i16 {
        unsafe { SDL_GetGamepadAxis(self.as_ptr(), axis.raw()) }
    }

    fn capability(&self, name: &[u8]) -> bool {
        let props = unsafe { SDL_GetGamepadProperties(self.as_ptr()) };
        props != 0 && unsafe { SDL_GetBooleanProperty(props, name.as_ptr().cast(), false) }
    }

    pub fn has_rumble(&self) -> bool {
        self.capability(SDL_PROP_GAMEPAD_CAP_RUMBLE_BOOLEAN)
    }

    pub fn has_trigger_rumble(&self) -> bool {
        self.capability(SDL_PROP_GAMEPAD_CAP_TRIGGER_RUMBLE_BOOLEAN)
    }

    pub fn has_rgb_led(&self) -> bool {
        self.capability(SDL_PROP_GAMEPAD_CAP_RGB_LED_BOOLEAN)
    }

    pub fn has_mono_led(&self) -> bool {
        self.capability(SDL_PROP_GAMEPAD_CAP_MONO_LED_BOOLEAN)
    }

    pub fn has_player_led(&self) -> bool {
        self.capability(SDL_PROP_GAMEPAD_CAP_PLAYER_LED_BOOLEAN)
    }

    pub fn rumble(&mut self, low: f32, high: f32, duration: Duration) -> Result<(), SdlError> {
        check(unsafe {
            SDL_RumbleGamepad(
                self.as_ptr(),
                unit_to_u16(low),
                unit_to_u16(high),
                duration_ms(duration),
            )
        })
    }

    pub fn rumble_triggers(
        &mut self,
        left: f32,
        right: f32,
        duration: Duration,
    ) -> Result<(), SdlError> {
        check(unsafe {
            SDL_RumbleGamepadTriggers(
                self.as_ptr(),
                unit_to_u16(left),
                unit_to_u16(right),
                duration_ms(duration),
            )
        })
    }

    pub fn stop_rumble(&mut self) -> Result<(), SdlError> {
        self.rumble(0.0, 0.0, Duration::ZERO)
    }

    pub fn set_led(&mut self, r: u8, g: u8, b: u8) -> Result<(), SdlError> {
        check(unsafe { SDL_SetGamepadLED(self.as_ptr(), r, g, b) })
    }

    pub fn has_sensor(&self, sensor: SensorKind) -> bool {
        unsafe { SDL_GamepadHasSensor(self.as_ptr(), sensor.raw()) }
    }

    pub fn sensor_enabled(&self, sensor: SensorKind) -> bool {
        unsafe { SDL_GamepadSensorEnabled(self.as_ptr(), sensor.raw()) }
    }

    pub fn set_sensor_enabled(
        &mut self,
        sensor: SensorKind,
        enabled: bool,
    ) -> Result<(), SdlError> {
        check(unsafe { SDL_SetGamepadSensorEnabled(self.as_ptr(), sensor.raw(), enabled) })
    }

    pub fn sensor_rate(&self, sensor: SensorKind) -> f32 {
        unsafe { SDL_GetGamepadSensorDataRate(self.as_ptr(), sensor.raw()) }
    }

    pub fn sensor_data(&self, sensor: SensorKind) -> Result<[f32; 3], SdlError> {
        let mut data = [0.0; 3];

        check(unsafe {
            SDL_GetGamepadSensorData(self.as_ptr(), sensor.raw(), data.as_mut_ptr(), 3)
        })?;

        Ok(data)
    }

    pub fn touchpads(&self) -> u32 {
        unsafe { SDL_GetNumGamepadTouchpads(self.as_ptr()) }.max(0) as u32
    }

    pub fn touchpad_fingers(&self, touchpad: u32) -> u32 {
        unsafe { SDL_GetNumGamepadTouchpadFingers(self.as_ptr(), touchpad as i32) }.max(0) as u32
    }

    pub fn touchpad_finger(&self, touchpad: u32, finger: u32) -> Option<TouchpadFinger> {
        let mut f = TouchpadFinger {
            down: false,
            x: 0.0,
            y: 0.0,
            pressure: 0.0,
        };

        let ok = unsafe {
            SDL_GetGamepadTouchpadFinger(
                self.as_ptr(),
                touchpad as i32,
                finger as i32,
                &mut f.down,
                &mut f.x,
                &mut f.y,
                &mut f.pressure,
            )
        };

        ok.then_some(f)
    }

    pub fn mapping(&self) -> Option<String> {
        let ptr = unsafe { SDL_GetGamepadMapping(self.as_ptr()) };
        let mapping = unsafe { opt_str(ptr) }.map(String::from);

        if !ptr.is_null() {
            unsafe { SDL_free(ptr.cast()) };
        }

        mapping
    }
}

impl Drop for Gamepad {
    fn drop(&mut self) {
        unsafe { SDL_CloseGamepad(self.as_ptr()) };
    }
}
