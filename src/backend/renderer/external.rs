//! Compositor-provided `wl_buffer` integration.
//!
//! This module lets compositors expose custom buffers to Smithay's renderer
//! helpers without teaching Smithay about the buffer's native allocation API.

use std::{any::Any, error::Error};

#[cfg(feature = "renderer_gl")]
use crate::{utils::Rectangle, wayland::compositor::SurfaceData};
use crate::utils::{Buffer as BufferCoord, Size};
use wayland_server::protocol::wl_buffer;

/// Error returned by compositor-provided external buffer import hooks.
pub type ExternalBufferImportError = Box<dyn Error + Send + Sync>;

/// Metadata for a compositor-provided `wl_buffer`.
pub trait ExternalBuffer: Any + Send + Sync {
    /// Buffer dimensions in buffer coordinates.
    fn dimensions(&self) -> Size<i32, BufferCoord>;

    /// Whether the buffer format may contain alpha.
    fn has_alpha(&self) -> Option<bool> {
        None
    }

    /// Whether the buffer contents are y-inverted.
    fn y_inverted(&self) -> Option<bool> {
        None
    }

    /// Expose the concrete type for compositors that need to recover it.
    fn as_any(&self) -> &dyn Any;

    /// Import the buffer into the provided GLES renderer, if supported.
    #[cfg(feature = "renderer_gl")]
    fn import_gles(
        &self,
        _renderer: &mut crate::backend::renderer::gles::GlesRenderer,
        _surface: Option<&SurfaceData>,
        _damage: &[Rectangle<i32, BufferCoord>],
    ) -> Option<Result<crate::backend::renderer::gles::GlesTexture, ExternalBufferImportError>> {
        None
    }
}

/// `wl_buffer` user data for compositor-provided renderable buffers.
pub struct ExternalBufferData {
    inner: Box<dyn ExternalBuffer>,
}

impl ExternalBufferData {
    /// Wrap compositor-specific buffer data.
    pub fn new<T>(data: T) -> Self
    where
        T: ExternalBuffer + 'static,
    {
        Self {
            inner: Box::new(data),
        }
    }

    /// Recover the compositor-specific buffer data.
    pub fn get<T: Any>(&self) -> Option<&T> {
        self.inner.as_any().downcast_ref()
    }

    /// Buffer dimensions in buffer coordinates.
    pub fn dimensions(&self) -> Size<i32, BufferCoord> {
        self.inner.dimensions()
    }

    /// Whether the buffer format may contain alpha.
    pub fn has_alpha(&self) -> Option<bool> {
        self.inner.has_alpha()
    }

    /// Whether the buffer contents are y-inverted.
    pub fn y_inverted(&self) -> Option<bool> {
        self.inner.y_inverted()
    }

    #[cfg(feature = "renderer_gl")]
    pub(crate) fn import_gles(
        &self,
        renderer: &mut crate::backend::renderer::gles::GlesRenderer,
        surface: Option<&SurfaceData>,
        damage: &[Rectangle<i32, BufferCoord>],
    ) -> Option<Result<crate::backend::renderer::gles::GlesTexture, ExternalBufferImportError>> {
        self.inner.import_gles(renderer, surface, damage)
    }
}

impl std::fmt::Debug for ExternalBufferData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalBufferData")
            .field("dimensions", &self.inner.dimensions())
            .finish()
    }
}

/// Return compositor-provided buffer data attached to `buffer`, if present.
#[cfg(feature = "wayland_frontend")]
pub fn external_buffer(buffer: &wl_buffer::WlBuffer) -> Option<&ExternalBufferData> {
    use wayland_server::Resource;

    buffer.data::<ExternalBufferData>()
}
