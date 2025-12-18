//! Framebuffer Driver
//!
//! Provides low-level pixel operations for the video framebuffer.
//! The framebuffer is provided by the bootloader at a fixed address.

/// RGB Color representation
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

// Standard color palette
pub const COLOR_BLACK: Color = Color::new(0, 0, 0);
pub const COLOR_WHITE: Color = Color::new(255, 255, 255);
pub const COLOR_RED: Color = Color::new(255, 0, 0);
pub const COLOR_GREEN: Color = Color::new(0, 255, 0);
pub const COLOR_BLUE: Color = Color::new(0, 100, 255);
pub const COLOR_YELLOW: Color = Color::new(255, 255, 0);
pub const COLOR_CYAN: Color = Color::new(0, 255, 255);
pub const COLOR_MAGENTA: Color = Color::new(255, 0, 255);
pub const COLOR_GRAY: Color = Color::new(128, 128, 128);
pub const COLOR_DARK_GRAY: Color = Color::new(64, 64, 64);
pub const COLOR_LIGHT_GREEN: Color = Color::new(0, 200, 0);
pub const COLOR_ORANGE: Color = Color::new(255, 165, 0);

/// Framebuffer driver
pub struct Framebuffer {
    addr: usize,
    width: usize,
    height: usize,
    stride: usize,
    bytes_per_pixel: usize,
}

impl Framebuffer {
    /// Create a new framebuffer
    ///
    /// # Arguments
    /// * `addr` - Physical address of framebuffer (e.g., 0x80000000)
    /// * `width` - Width in pixels (e.g., 1280)
    /// * `height` - Height in pixels (e.g., 800)
    /// * `stride` - Pixels per line (usually same as width)
    pub const fn new(addr: usize, width: usize, height: usize, stride: usize) -> Self {
        Self {
            addr,
            width,
            height,
            stride,
            bytes_per_pixel: 4, // BGRA format
        }
    }

    /// Get framebuffer width
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Get framebuffer height
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Write a single pixel
    ///
    /// # Arguments
    /// * `x` - X coordinate (0 = left)
    /// * `y` - Y coordinate (0 = top)
    /// * `color` - RGB color to write
    pub fn write_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.width || y >= self.height {
            return; // Out of bounds
        }

        let offset = (y * self.stride + x) * self.bytes_per_pixel;
        let ptr = (self.addr + offset) as *mut u32;

        // BGRA format: Blue, Green, Red, Alpha
        let pixel = ((color.b as u32) << 0)
            | ((color.g as u32) << 8)
            | ((color.r as u32) << 16)
            | (0xFF << 24); // Alpha = 255 (opaque)

        unsafe {
            ptr.write_volatile(pixel);
        }
    }

    /// Clear entire screen with a color
    pub fn clear(&mut self, color: Color) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.write_pixel(x, y, color);
            }
        }
    }

    /// Fill a rectangle with a color
    ///
    /// # Arguments
    /// * `x` - Top-left X coordinate
    /// * `y` - Top-left Y coordinate
    /// * `w` - Width in pixels
    /// * `h` - Height in pixels
    /// * `color` - Fill color
    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        for dy in 0..h {
            for dx in 0..w {
                self.write_pixel(x + dx, y + dy, color);
            }
        }
    }

    /// Copy a region vertically (for scrolling)
    ///
    /// # Arguments
    /// * `src_y` - Source Y coordinate
    /// * `dst_y` - Destination Y coordinate
    /// * `height` - Height to copy in pixels
    pub fn copy_region(&mut self, src_y: usize, dst_y: usize, height: usize) {
        if src_y == dst_y {
            return;
        }

        let bytes_per_line = self.stride * self.bytes_per_pixel;
        let copy_bytes = height * bytes_per_line;

        let src_offset = src_y * bytes_per_line;
        let dst_offset = dst_y * bytes_per_line;

        unsafe {
            let src = (self.addr + src_offset) as *const u8;
            let dst = (self.addr + dst_offset) as *mut u8;

            // Use ptr::copy for fast bulk memory copy
            // This handles overlapping regions correctly
            core::ptr::copy(src, dst, copy_bytes);
        }
    }
}
