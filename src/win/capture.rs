// Screen and window capture. A window is captured with Windows.Graphics.Capture (works when it's
// covered by other windows), falling back to PrintWindow(PW_RENDERFULLCONTENT); a display with a
// GDI BitBlt of the screen. Images are BGRA in physical pixels; callers convert to logical.

use super::*;
use windows::core::Interface;
use windows::Graphics::Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE, D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP};
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC;
use windows::Win32::Graphics::Dxgi::IDXGIDevice;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS};
use windows::Win32::System::WinRT::Direct3D11::{CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess};
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;

#[derive(Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

impl Image {
    pub fn crop(&self, x: i32, y: i32, w: i32, h: i32) -> Image {
        let x0 = x.clamp(0, self.width as i32) as u32;
        let y0 = y.clamp(0, self.height as i32) as u32;
        let x1 = (x + w).clamp(0, self.width as i32) as u32;
        let y1 = (y + h).clamp(0, self.height as i32) as u32;
        let (cw, ch) = (x1.saturating_sub(x0), y1.saturating_sub(y0));
        let mut out = Vec::with_capacity((cw * ch * 4) as usize);
        for row in y0..y1 {
            let s = ((row * self.width + x0) * 4) as usize;
            out.extend_from_slice(&self.bgra[s..s + (cw * 4) as usize]);
        }
        Image { width: cw, height: ch, bgra: out }
    }

    /// Box-filter downscale to `w` wide (keeps aspect ratio).
    pub fn resize_to_width(&self, w: u32) -> Image {
        if w >= self.width || w == 0 {
            return self.clone();
        }
        let h = ((self.height as f64) * (w as f64) / (self.width as f64)).round().max(1.0) as u32;
        let sx = self.width as f64 / w as f64;
        let sy = self.height as f64 / h as f64;
        let mut out = vec![0u8; (w * h * 4) as usize];
        for oy in 0..h {
            let y0 = (oy as f64 * sy) as u32;
            let y1 = (((oy + 1) as f64 * sy).ceil() as u32).min(self.height).max(y0 + 1);
            for ox in 0..w {
                let x0 = (ox as f64 * sx) as u32;
                let x1 = (((ox + 1) as f64 * sx).ceil() as u32).min(self.width).max(x0 + 1);
                let mut acc = [0u32; 4];
                let mut n = 0u32;
                for yy in y0..y1 {
                    for xx in x0..x1 {
                        let i = ((yy * self.width + xx) * 4) as usize;
                        for c in 0..4 {
                            acc[c] += self.bgra[i + c] as u32;
                        }
                        n += 1;
                    }
                }
                let o = ((oy * w + ox) * 4) as usize;
                for c in 0..4 {
                    out[o + c] = (acc[c] / n.max(1)) as u8;
                }
            }
        }
        Image { width: w, height: h, bgra: out }
    }

    pub fn distinct_colors(&self, step: usize) -> usize {
        let mut set = std::collections::HashSet::new();
        let mut i = 0;
        while i + 3 < self.bgra.len() {
            set.insert((self.bgra[i], self.bgra[i + 1], self.bgra[i + 2]));
            i += 4 * step.max(1);
            if set.len() > 64 {
                break;
            }
        }
        set.len()
    }

    pub fn save_png(&self, path: &str) -> Result<(), String> {
        let f = std::fs::File::create(path).map_err(|e| format!("Failed to save screenshot to {}: {}", path, e))?;
        let mut enc = png::Encoder::new(std::io::BufWriter::new(f), self.width, self.height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().map_err(|e| format!("Failed to save screenshot to {}: {}", path, e))?;
        let mut rgba = self.bgra.clone();
        for px in rgba.chunks_mut(4) {
            px.swap(0, 2);
            px[3] = 255;
        }
        w.write_image_data(&rgba).map_err(|e| format!("Failed to save screenshot to {}: {}", path, e))
    }

    pub fn load_png(path: &str) -> Result<Image, String> {
        let f = std::fs::File::open(path).map_err(|_| format!("Could not read image: {}", path))?;
        let mut dec = png::Decoder::new(std::io::BufReader::new(f));
        dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut r = dec.read_info().map_err(|_| format!("Could not read image: {}", path))?;
        let mut buf = vec![0u8; r.output_buffer_size()];
        let info = r.next_frame(&mut buf).map_err(|_| format!("Could not read image: {}", path))?;
        let (w, h) = (info.width, info.height);
        let mut bgra = Vec::with_capacity((w * h * 4) as usize);
        let data = &buf[..info.buffer_size()];
        match info.color_type {
            png::ColorType::Rgba => {
                for px in data.chunks(4) {
                    bgra.extend_from_slice(&[px[2], px[1], px[0], 255]);
                }
            }
            png::ColorType::Rgb => {
                for px in data.chunks(3) {
                    bgra.extend_from_slice(&[px[2], px[1], px[0], 255]);
                }
            }
            png::ColorType::GrayscaleAlpha => {
                for px in data.chunks(2) {
                    bgra.extend_from_slice(&[px[0], px[0], px[0], 255]);
                }
            }
            _ => {
                for px in data {
                    bgra.extend_from_slice(&[*px, *px, *px, 255]);
                }
            }
        }
        Ok(Image { width: w, height: h, bgra })
    }

    fn put(&mut self, x: i32, y: i32, c: [u8; 3]) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let i = ((y as u32 * self.width + x as u32) * 4) as usize;
        self.bgra[i] = c[2];
        self.bgra[i + 1] = c[1];
        self.bgra[i + 2] = c[0];
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 3], thick: i32) {
        for t in 0..thick {
            for xx in x..x + w {
                self.put(xx, y + t, c);
                self.put(xx, y + h - 1 - t, c);
            }
            for yy in y..y + h {
                self.put(x + t, yy, c);
                self.put(x + w - 1 - t, yy, c);
            }
        }
    }

    pub fn fill(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 3]) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.put(xx, yy, c);
            }
        }
    }

    /// Draw a number label (white on the box color) at the top-left of a box.
    pub fn label(&mut self, x: i32, y: i32, n: usize, c: [u8; 3], scale: i32) {
        let s = n.to_string();
        let (cw, ch) = (4 * scale, 7 * scale);
        let w = s.len() as i32 * (cw + scale) + scale * 2;
        let h = ch + scale * 2;
        self.fill(x, y, w, h, c);
        for (i, d) in s.chars().enumerate() {
            let glyph = DIGITS[d.to_digit(10).unwrap_or(0) as usize];
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..4 {
                    if bits & (0b1000 >> col) != 0 {
                        self.fill(x + scale + i as i32 * (cw + scale) + col * scale, y + scale + row as i32 * scale, scale, scale, [255, 255, 255]);
                    }
                }
            }
        }
    }
}

const DIGITS: [[u8; 7]; 10] = [
    [0b0110, 0b1001, 0b1001, 0b1001, 0b1001, 0b1001, 0b0110],
    [0b0010, 0b0110, 0b0010, 0b0010, 0b0010, 0b0010, 0b0111],
    [0b0110, 0b1001, 0b0001, 0b0010, 0b0100, 0b1000, 0b1111],
    [0b1110, 0b0001, 0b0001, 0b0110, 0b0001, 0b0001, 0b1110],
    [0b0010, 0b0110, 0b1010, 0b1111, 0b0010, 0b0010, 0b0010],
    [0b1111, 0b1000, 0b1110, 0b0001, 0b0001, 0b1001, 0b0110],
    [0b0110, 0b1000, 0b1000, 0b1110, 0b1001, 0b1001, 0b0110],
    [0b1111, 0b0001, 0b0010, 0b0010, 0b0100, 0b0100, 0b0100],
    [0b0110, 0b1001, 0b1001, 0b0110, 0b1001, 0b1001, 0b0110],
    [0b0110, 0b1001, 0b1001, 0b0111, 0b0001, 0b0001, 0b0110],
];

fn dib_read(hdc: HDC, bmp: HBITMAP, w: i32, h: i32) -> Option<Vec<u8>> {
    unsafe {
        let mut bi = BITMAPINFO::default();
        bi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bi.bmiHeader.biWidth = w;
        bi.bmiHeader.biHeight = -h; // top-down
        bi.bmiHeader.biPlanes = 1;
        bi.bmiHeader.biBitCount = 32;
        bi.bmiHeader.biCompression = BI_RGB.0;
        let mut buf = vec![0u8; (w * h * 4) as usize];
        let n = GetDIBits(hdc, bmp, 0, h as u32, Some(buf.as_mut_ptr() as *mut _), &mut bi, DIB_RGB_COLORS);
        if n == 0 {
            return None;
        }
        Some(buf)
    }
}

/// GDI copy of a physical screen rect.
pub fn capture_screen_rect(r: &RECT) -> Result<Image, String> {
    let (w, h) = (r.right - r.left, r.bottom - r.top);
    if w <= 0 || h <= 0 {
        return Err("Failed to capture screenshot: empty area".into());
    }
    unsafe {
        let screen = GetDC(None);
        let mem = CreateCompatibleDC(Some(screen));
        let bmp = CreateCompatibleBitmap(screen, w, h);
        let old = SelectObject(mem, bmp.into());
        let ok = BitBlt(mem, 0, 0, w, h, Some(screen), r.left, r.top, SRCCOPY | CAPTUREBLT).is_ok();
        SelectObject(mem, old);
        let data = if ok { dib_read(mem, bmp, w, h) } else { None };
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        match data {
            Some(bgra) => Ok(Image { width: w as u32, height: h as u32, bgra }),
            None => Err("Failed to capture screenshot (BitBlt)".into()),
        }
    }
}

/// PrintWindow of the whole window rect, cropped to its visible frame.
pub fn capture_printwindow(h: HWND) -> Result<Image, String> {
    let wr = window_rect(h);
    let fr = window_frame(h);
    let (w, hh) = (wr.right - wr.left, wr.bottom - wr.top);
    if w <= 0 || hh <= 0 {
        return Err("the window has no size".into());
    }
    unsafe {
        let screen = GetDC(None);
        let mem = CreateCompatibleDC(Some(screen));
        let bmp = CreateCompatibleBitmap(screen, w, hh);
        let old = SelectObject(mem, bmp.into());
        let ok = PrintWindow(h, mem, PRINT_WINDOW_FLAGS(2)).as_bool(); // PW_RENDERFULLCONTENT
        SelectObject(mem, old);
        let data = if ok { dib_read(mem, bmp, w, hh) } else { None };
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        match data {
            Some(bgra) => {
                let img = Image { width: w as u32, height: hh as u32, bgra };
                Ok(img.crop(fr.left - wr.left, fr.top - wr.top, fr.right - fr.left, fr.bottom - fr.top))
            }
            None => Err("PrintWindow failed".into()),
        }
    }
}

fn d3d_device() -> Result<(ID3D11Device, ID3D11DeviceContext), String> {
    for dt in [D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP] {
        if let Ok(r) = create_device(dt) {
            return Ok(r);
        }
    }
    Err("no Direct3D 11 device".into())
}

fn create_device(dt: D3D_DRIVER_TYPE) -> windows::core::Result<(ID3D11Device, ID3D11DeviceContext)> {
    let mut device: Option<ID3D11Device> = None;
    let mut ctx: Option<ID3D11DeviceContext> = None;
    unsafe {
        D3D11CreateDevice(None, dt, HMODULE::default(), D3D11_CREATE_DEVICE_BGRA_SUPPORT, None, D3D11_SDK_VERSION, Some(&mut device), None, Some(&mut ctx))?;
    }
    Ok((device.unwrap(), ctx.unwrap()))
}

/// Windows.Graphics.Capture of one window (content only, even when covered).
pub fn capture_wgc(h: HWND, timeout_ms: u64) -> Result<Image, String> {
    let item: GraphicsCaptureItem = unsafe {
        let interop = windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>().map_err(|e| e.message())?;
        interop.CreateForWindow(h).map_err(|e| e.message())?
    };
    let (device, ctx) = d3d_device()?;
    let dxgi: IDXGIDevice = device.cast().map_err(|e| e.message())?;
    let d3d: IDirect3DDevice = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi) }.map_err(|e| e.message())?.cast().map_err(|e| e.message())?;
    let size = item.Size().map_err(|e| e.message())?;
    let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(&d3d, DirectXPixelFormat::B8G8R8A8UIntNormalized, 1, size).map_err(|e| e.message())?;
    let session = pool.CreateCaptureSession(&item).map_err(|e| e.message())?;
    let _ = session.SetIsBorderRequired(false);
    let _ = session.SetIsCursorCaptureEnabled(false);
    session.StartCapture().map_err(|e| e.message())?;
    let start = std::time::Instant::now();
    let frame = loop {
        if let Ok(f) = pool.TryGetNextFrame() {
            break f;
        }
        if start.elapsed().as_millis() as u64 > timeout_ms {
            let _ = session.Close();
            let _ = pool.Close();
            return Err("no frame from Windows.Graphics.Capture".into());
        }
        sleep_ms(15);
    };
    let result = (|| -> Result<Image, String> {
        let surface = frame.Surface().map_err(|e| e.message())?;
        let access: IDirect3DDxgiInterfaceAccess = surface.cast().map_err(|e| e.message())?;
        let tex: ID3D11Texture2D = unsafe { access.GetInterface() }.map_err(|e| e.message())?;
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        unsafe { tex.GetDesc(&mut desc) };
        desc.Usage = D3D11_USAGE_STAGING;
        desc.BindFlags = 0;
        desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
        desc.MiscFlags = 0;
        desc.SampleDesc = DXGI_SAMPLE_DESC { Count: 1, Quality: 0 };
        let mut staging: Option<ID3D11Texture2D> = None;
        unsafe { device.CreateTexture2D(&desc, None, Some(&mut staging)) }.map_err(|e| e.message())?;
        let staging = staging.ok_or("no staging texture")?;
        unsafe { ctx.CopyResource(&staging, &tex) };
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        unsafe { ctx.Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped)) }.map_err(|e| e.message())?;
        let (w, hh) = (desc.Width, desc.Height);
        let mut bgra = vec![0u8; (w * hh * 4) as usize];
        for row in 0..hh {
            let src = unsafe { std::slice::from_raw_parts((mapped.pData as *const u8).add((row * mapped.RowPitch) as usize), (w * 4) as usize) };
            bgra[(row * w * 4) as usize..((row + 1) * w * 4) as usize].copy_from_slice(src);
        }
        unsafe { ctx.Unmap(&staging, 0) };
        Ok(Image { width: w, height: hh, bgra })
    })();
    let _ = frame.Close();
    let _ = session.Close();
    let _ = pool.Close();
    let img = result?;
    // WGC returns the window including its invisible resize borders on some versions: crop to
    // the visible frame when the sizes say so.
    let wr = window_rect(h);
    let fr = window_frame(h);
    if img.width as i32 == wr.right - wr.left && img.height as i32 == wr.bottom - wr.top && (fr.right - fr.left) < img.width as i32 {
        return Ok(img.crop(fr.left - wr.left, fr.top - wr.top, fr.right - fr.left, fr.bottom - fr.top));
    }
    Ok(img)
}

/// Capture a window: WGC, then PrintWindow. Returns the image and the method used.
pub fn capture_window(h: HWND) -> Result<(Image, &'static str), String> {
    if unsafe { IsIconic(h).as_bool() } {
        return Err("the window is minimized".into());
    }
    let mut errs = Vec::new();
    if crate::common::env("AUX4_COMPUTER_WINDOWS_CAPTURE") != "printwindow" {
        match capture_wgc(h, 3000) {
            Ok(img) if img.width > 0 && img.distinct_colors(97) > 1 => return Ok((img, "wgc")),
            Ok(_) => errs.push("Windows.Graphics.Capture returned a blank frame".to_string()),
            Err(e) => errs.push(format!("Windows.Graphics.Capture: {}", e)),
        }
    }
    match capture_printwindow(h) {
        Ok(img) => Ok((img, "printwindow")),
        Err(e) => {
            errs.push(format!("PrintWindow: {}", e));
            Err(errs.join("; "))
        }
    }
}
