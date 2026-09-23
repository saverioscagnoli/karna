use core::ffi::CStr;
use core::ffi::c_char;
use core::ffi::c_void;
use core::ptr;
use core::ptr::NonNull;
use core::slice;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::AcqRel;
use core::sync::atomic::Ordering::Release;

use alloc::ffi::CString;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use sdl3_shadercross_sys::SDL_SHADERCROSS_PROP_SHADER_CULL_UNUSED_BINDINGS_BOOLEAN;
use sdl3_shadercross_sys::SDL_SHADERCROSS_PROP_SHADER_DEBUG_ENABLE_BOOLEAN;
use sdl3_shadercross_sys::SDL_SHADERCROSS_PROP_SHADER_DEBUG_NAME_STRING;
use sdl3_shadercross_sys::SDL_SHADERCROSS_PROP_SPIRV_MSL_VERSION_STRING;
use sdl3_shadercross_sys::SDL_SHADERCROSS_SHADERSTAGE_FRAGMENT;
use sdl3_shadercross_sys::SDL_SHADERCROSS_SHADERSTAGE_VERTEX;
use sdl3_shadercross_sys::SDL_ShaderCross_CompileDXBCFromSPIRV;
use sdl3_shadercross_sys::SDL_ShaderCross_CompileDXILFromSPIRV;
use sdl3_shadercross_sys::SDL_ShaderCross_CompileGraphicsShaderFromSPIRV;
use sdl3_shadercross_sys::SDL_ShaderCross_CompileSPIRVFromHLSL;
use sdl3_shadercross_sys::SDL_ShaderCross_GetHLSLShaderFormats;
use sdl3_shadercross_sys::SDL_ShaderCross_GetSPIRVShaderFormats;
use sdl3_shadercross_sys::SDL_ShaderCross_GraphicsShaderResourceInfo;
use sdl3_shadercross_sys::SDL_ShaderCross_HLSL_Define;
use sdl3_shadercross_sys::SDL_ShaderCross_HLSL_Info;
use sdl3_shadercross_sys::SDL_ShaderCross_Init;
use sdl3_shadercross_sys::SDL_ShaderCross_Quit;
use sdl3_shadercross_sys::SDL_ShaderCross_ReflectGraphicsSPIRV;
use sdl3_shadercross_sys::SDL_ShaderCross_SPIRV_Info;
use sdl3_shadercross_sys::SDL_ShaderCross_ShaderStage;
use sdl3_shadercross_sys::SDL_ShaderCross_TranspileHLSLFromSPIRV;
use sdl3_shadercross_sys::SDL_ShaderCross_TranspileMSLFromSPIRV;
use sdl3_sys::SDL_CreateProperties;
use sdl3_sys::SDL_DestroyProperties;
use sdl3_sys::SDL_PropertiesID;
use sdl3_sys::SDL_SetBooleanProperty;
use sdl3_sys::SDL_SetStringProperty;
use sdl3_sys::SDL_free;
use sdl3_sys::SdlError;
use sdl3_sys::get_error;

use crate::gpu::CompiledShader;
use crate::gpu::Device;
use crate::gpu::Shader;
use crate::gpu::ShaderFormat;
use crate::gpu::ShaderResources;
use crate::gpu::ShaderStage;

static SHADERCROSS_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy)]
pub enum ShaderSource<'a> {
    Spirv(&'a [u8]),
    Hlsl(&'a str),
}

#[derive(Debug, Clone, Copy)]
pub struct Define<'a> {
    pub name: &'a str,
    pub value: Option<&'a str>,
}

impl<'a> Define<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name, value: None }
    }

    pub fn with_value(mut self, value: &'a str) -> Self {
        self.value = Some(value);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CompileOptions<'a> {
    pub stage: ShaderStage,
    pub entrypoint: &'a str,
    pub include_dir: Option<&'a str>,
    pub defines: &'a [Define<'a>],
    pub name: Option<&'a str>,
    pub debug: bool,
    pub cull_unused_bindings: bool,
    pub msl_version: Option<&'a str>,
}

impl<'a> CompileOptions<'a> {
    pub fn new(stage: ShaderStage) -> Self {
        Self {
            stage,
            entrypoint: "main",
            include_dir: None,
            defines: &[],
            name: None,
            debug: false,
            cull_unused_bindings: false,
            msl_version: None,
        }
    }

    pub fn vertex() -> Self {
        Self::new(ShaderStage::Vertex)
    }

    pub fn fragment() -> Self {
        Self::new(ShaderStage::Fragment)
    }

    pub fn with_entrypoint(mut self, entrypoint: &'a str) -> Self {
        self.entrypoint = entrypoint;
        self
    }

    pub fn with_include_dir(mut self, dir: &'a str) -> Self {
        self.include_dir = Some(dir);
        self
    }

    pub fn with_defines(mut self, defines: &'a [Define<'a>]) -> Self {
        self.defines = defines;
        self
    }

    pub fn with_name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn with_cull_unused_bindings(mut self, cull: bool) -> Self {
        self.cull_unused_bindings = cull;
        self
    }

    pub fn with_msl_version(mut self, version: &'a str) -> Self {
        self.msl_version = Some(version);
        self
    }
}

pub struct ShaderCross(());

impl ShaderCross {
    pub fn init() -> Result<Self, SdlError> {
        if SHADERCROSS_ACTIVE.swap(true, AcqRel) {
            return Err(SdlError::new(
                "SDL_shadercross was initialized more than one time.",
            ));
        }

        if !unsafe { SDL_ShaderCross_Init() } {
            SHADERCROSS_ACTIVE.store(false, Release);
            return Err(get_error());
        }

        Ok(Self(()))
    }

    pub fn spirv_formats(&self) -> ShaderFormat {
        ShaderFormat::from_bits(unsafe { SDL_ShaderCross_GetSPIRVShaderFormats() })
    }

    pub fn hlsl_formats(&self) -> ShaderFormat {
        ShaderFormat::from_bits(unsafe { SDL_ShaderCross_GetHLSLShaderFormats() })
    }

    pub fn can_compile_hlsl(&self) -> bool {
        self.hlsl_formats().contains(ShaderFormat::SPIRV)
    }

    pub fn formats(&self, source: &ShaderSource<'_>) -> ShaderFormat {
        match source {
            ShaderSource::Spirv(_) => self.spirv_formats(),
            ShaderSource::Hlsl(_) if self.can_compile_hlsl() => self.spirv_formats(),
            ShaderSource::Hlsl(_) => ShaderFormat::NONE,
        }
    }

    pub fn spirv_from_hlsl(
        &self,
        source: &str,
        options: &CompileOptions<'_>,
    ) -> Result<Vec<u8>, SdlError> {
        if !self.can_compile_hlsl() {
            return Err(SdlError::new(
                "HLSL compilation needs SDL_shadercross built with DXC.",
            ));
        }

        let source = cstring(source)?;
        let entrypoint = cstring(options.entrypoint)?;
        let include_dir = options.include_dir.map(cstring).transpose()?;
        let defines = Defines::new(options.defines)?;
        let props = Props::new(options)?;

        let info = SDL_ShaderCross_HLSL_Info {
            source: source.as_ptr(),
            entrypoint: entrypoint.as_ptr(),
            include_dir: include_dir.as_ref().map_or(ptr::null(), |dir| dir.as_ptr()),
            defines: defines.as_ptr(),
            shader_stage: stage(options.stage),
            props: props.id(),
        };

        let mut size = 0;
        let code = unsafe { SDL_ShaderCross_CompileSPIRVFromHLSL(&info, &mut size) };

        unsafe { take_bytes(code, size) }
    }

    pub fn reflect(&self, spirv: &[u8]) -> Result<ShaderResources, SdlError> {
        let meta = unsafe { SDL_ShaderCross_ReflectGraphicsSPIRV(spirv.as_ptr(), spirv.len(), 0) };
        let meta = NonNull::new(meta).ok_or_else(get_error)?;

        let info = unsafe { &meta.as_ref().resource_info };
        let resources = ShaderResources {
            samplers: info.num_samplers,
            storage_textures: info.num_storage_textures,
            storage_buffers: info.num_storage_buffers,
            uniform_buffers: info.num_uniform_buffers,
        };

        unsafe { SDL_free(meta.as_ptr().cast()) };

        Ok(resources)
    }

    pub fn compile(
        &self,
        source: ShaderSource<'_>,
        format: ShaderFormat,
        options: &CompileOptions<'_>,
    ) -> Result<CompiledShader, SdlError> {
        if !format.is_single() {
            return Err(SdlError::new(format!(
                "Expected exactly one target shader format, got {:?}.",
                format
            )));
        }

        if !self.formats(&source).contains(format) {
            return Err(SdlError::new(format!(
                "Cannot compile {} to {:?} with this SDL_shadercross build.",
                source_kind(&source),
                format
            )));
        }

        let owned;
        let spirv = match source {
            ShaderSource::Spirv(spirv) => spirv,
            ShaderSource::Hlsl(hlsl) => {
                owned = self.spirv_from_hlsl(hlsl, options)?;
                &owned
            }
        };

        let resources = self.reflect(spirv)?;
        let entrypoint = cstring(options.entrypoint)?;
        let props = Props::new(options)?;
        let info = spirv_info(spirv, &entrypoint, options.stage, &props);

        let code = match format {
            ShaderFormat::SPIRV => spirv.to_vec(),
            ShaderFormat::MSL => unsafe {
                take_string(SDL_ShaderCross_TranspileMSLFromSPIRV(&info))?.into_bytes()
            },
            ShaderFormat::DXBC => {
                let mut size = 0;
                unsafe { take_bytes(SDL_ShaderCross_CompileDXBCFromSPIRV(&info, &mut size), size)? }
            }
            ShaderFormat::DXIL => {
                let mut size = 0;
                unsafe { take_bytes(SDL_ShaderCross_CompileDXILFromSPIRV(&info, &mut size), size)? }
            }
            _ => {
                return Err(SdlError::new(format!(
                    "SDL_shadercross cannot output {:?}.",
                    format
                )));
            }
        };

        let entrypoint = match format {
            ShaderFormat::MSL => msl_entrypoint(options.entrypoint),
            _ => options.entrypoint.to_string(),
        };

        Ok(CompiledShader {
            format,
            stage: options.stage,
            entrypoint,
            code,
            resources,
        })
    }

    pub fn compile_all(
        &self,
        source: ShaderSource<'_>,
        formats: ShaderFormat,
        options: &CompileOptions<'_>,
    ) -> Result<Vec<CompiledShader>, SdlError> {
        let owned;
        let source = match source {
            ShaderSource::Hlsl(hlsl) => {
                owned = self.spirv_from_hlsl(hlsl, options)?;
                ShaderSource::Spirv(&owned)
            }
            spirv => spirv,
        };

        formats
            .iter()
            .map(|format| self.compile(source, format, options))
            .collect()
    }

    pub fn hlsl_from_spirv(
        &self,
        spirv: &[u8],
        options: &CompileOptions<'_>,
    ) -> Result<String, SdlError> {
        let entrypoint = cstring(options.entrypoint)?;
        let props = Props::new(options)?;
        let info = spirv_info(spirv, &entrypoint, options.stage, &props);

        unsafe { take_string(SDL_ShaderCross_TranspileHLSLFromSPIRV(&info)) }
    }

    pub fn create_shader(
        &self,
        device: Device,
        source: ShaderSource<'_>,
        options: &CompileOptions<'_>,
    ) -> Result<Shader, SdlError> {
        let owned;
        let spirv = match source {
            ShaderSource::Spirv(spirv) => spirv,
            ShaderSource::Hlsl(hlsl) => {
                owned = self.spirv_from_hlsl(hlsl, options)?;
                &owned
            }
        };

        let resources = self.reflect(spirv)?;
        let entrypoint = cstring(options.entrypoint)?;
        let props = Props::new(options)?;
        let info = spirv_info(spirv, &entrypoint, options.stage, &props);

        let resource_info = SDL_ShaderCross_GraphicsShaderResourceInfo {
            num_samplers: resources.samplers,
            num_storage_textures: resources.storage_textures,
            num_storage_buffers: resources.storage_buffers,
            num_uniform_buffers: resources.uniform_buffers,
        };

        let raw = unsafe {
            SDL_ShaderCross_CompileGraphicsShaderFromSPIRV(
                device.as_ptr(),
                &info,
                &resource_info,
                0,
            )
        };
        let raw = NonNull::new(raw).ok_or_else(get_error)?;

        Ok(Shader::from_raw(device, raw, options.stage))
    }
}

impl Drop for ShaderCross {
    fn drop(&mut self) {
        unsafe { SDL_ShaderCross_Quit() };

        SHADERCROSS_ACTIVE.store(false, Release);
    }
}

struct Props(SDL_PropertiesID);

impl Props {
    fn new(options: &CompileOptions<'_>) -> Result<Self, SdlError> {
        let needed = options.debug
            || options.cull_unused_bindings
            || options.name.is_some()
            || options.msl_version.is_some();

        if !needed {
            return Ok(Self(0));
        }

        let id = unsafe { SDL_CreateProperties() };

        if id == 0 {
            return Err(get_error());
        }

        let props = Self(id);

        if options.debug {
            props.set_bool(SDL_SHADERCROSS_PROP_SHADER_DEBUG_ENABLE_BOOLEAN, true)?;
        }

        if options.cull_unused_bindings {
            props.set_bool(
                SDL_SHADERCROSS_PROP_SHADER_CULL_UNUSED_BINDINGS_BOOLEAN,
                true,
            )?;
        }

        if let Some(name) = options.name {
            props.set_str(SDL_SHADERCROSS_PROP_SHADER_DEBUG_NAME_STRING, name)?;
        }

        if let Some(version) = options.msl_version {
            props.set_str(SDL_SHADERCROSS_PROP_SPIRV_MSL_VERSION_STRING, version)?;
        }

        Ok(props)
    }

    fn id(&self) -> SDL_PropertiesID {
        self.0
    }

    fn set_bool(&self, key: &[u8], value: bool) -> Result<(), SdlError> {
        if !unsafe { SDL_SetBooleanProperty(self.0, key.as_ptr().cast(), value) } {
            return Err(get_error());
        }

        Ok(())
    }

    fn set_str(&self, key: &[u8], value: &str) -> Result<(), SdlError> {
        let value = cstring(value)?;

        if !unsafe { SDL_SetStringProperty(self.0, key.as_ptr().cast(), value.as_ptr()) } {
            return Err(get_error());
        }

        Ok(())
    }
}

impl Drop for Props {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe { SDL_DestroyProperties(self.0) };
        }
    }
}

struct Defines {
    _strings: Vec<CString>,
    raw: Vec<SDL_ShaderCross_HLSL_Define>,
}

impl Defines {
    fn new(defines: &[Define<'_>]) -> Result<Self, SdlError> {
        if defines.is_empty() {
            return Ok(Self {
                _strings: Vec::new(),
                raw: Vec::new(),
            });
        }

        let mut strings = Vec::with_capacity(defines.len() * 2);
        let mut raw = Vec::with_capacity(defines.len() + 1);

        for define in defines {
            let name = cstring(define.name)?;
            let value = define.value.map(cstring).transpose()?;

            raw.push(SDL_ShaderCross_HLSL_Define {
                name: name.as_ptr().cast_mut(),
                value: value
                    .as_ref()
                    .map_or(ptr::null_mut(), |v| v.as_ptr().cast_mut()),
            });

            strings.push(name);
            strings.extend(value);
        }

        raw.push(SDL_ShaderCross_HLSL_Define::default());

        Ok(Self {
            _strings: strings,
            raw,
        })
    }

    fn as_ptr(&self) -> *mut SDL_ShaderCross_HLSL_Define {
        if self.raw.is_empty() {
            ptr::null_mut()
        } else {
            self.raw.as_ptr().cast_mut()
        }
    }
}

fn spirv_info(
    spirv: &[u8],
    entrypoint: &CStr,
    shader_stage: ShaderStage,
    props: &Props,
) -> SDL_ShaderCross_SPIRV_Info {
    SDL_ShaderCross_SPIRV_Info {
        bytecode: spirv.as_ptr(),
        bytecode_size: spirv.len(),
        entrypoint: entrypoint.as_ptr(),
        shader_stage: stage(shader_stage),
        props: props.id(),
    }
}

fn stage(stage: ShaderStage) -> SDL_ShaderCross_ShaderStage {
    match stage {
        ShaderStage::Vertex => SDL_SHADERCROSS_SHADERSTAGE_VERTEX,
        ShaderStage::Fragment => SDL_SHADERCROSS_SHADERSTAGE_FRAGMENT,
    }
}

fn source_kind(source: &ShaderSource<'_>) -> &'static str {
    match source {
        ShaderSource::Spirv(_) => "SPIR-V",
        ShaderSource::Hlsl(_) => "HLSL",
    }
}

fn msl_entrypoint(entrypoint: &str) -> String {
    match entrypoint {
        "main" => String::from("main0"),
        other => other.to_string(),
    }
}

fn cstring(s: &str) -> Result<CString, SdlError> {
    CString::new(s).map_err(|_| SdlError::new("InteriorNul"))
}

unsafe fn take_bytes(ptr: *mut c_void, size: usize) -> Result<Vec<u8>, SdlError> {
    let ptr = NonNull::new(ptr).ok_or_else(get_error)?;
    let bytes = unsafe { slice::from_raw_parts(ptr.as_ptr().cast::<u8>(), size) }.to_vec();

    unsafe { SDL_free(ptr.as_ptr()) };

    Ok(bytes)
}

unsafe fn take_string(ptr: *mut c_void) -> Result<String, SdlError> {
    let ptr = NonNull::new(ptr).ok_or_else(get_error)?;
    let string = unsafe { CStr::from_ptr(ptr.as_ptr().cast::<c_char>()) }
        .to_string_lossy()
        .into_owned();

    unsafe { SDL_free(ptr.as_ptr()) };

    Ok(string)
}
