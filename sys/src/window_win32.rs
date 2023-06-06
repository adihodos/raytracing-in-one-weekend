#![cfg(windows)]
#![allow(dead_code)]

use std::cell::Cell;
use std::mem::{size_of, transmute, MaybeUninit};
use std::ptr::{null, null_mut};
use std::result::Result;
use std::string::String;

use winapi::shared::basetsd::LONG_PTR;
use winapi::shared::minwindef::{
    DWORD, FALSE, HIWORD, LOWORD, LPARAM, LRESULT, MAKELONG, TRUE, UINT, WORD, WPARAM,
};
use winapi::shared::windef::{HBRUSH, HDC, HGLRC, HWND, POINT, RECT};
use winapi::shared::windowsx::{GET_X_LPARAM, GET_Y_LPARAM};

use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::ChoosePixelFormat;
use winapi::um::wingdi::DescribePixelFormat;
use winapi::um::wingdi::SetPixelFormat;
use winapi::um::wingdi::PIXELFORMATDESCRIPTOR;
use winapi::um::wingdi::{
    wglCreateContext, wglDeleteContext, wglGetProcAddress, wglMakeCurrent, GetStockObject,
    SwapBuffers, PFD_DOUBLEBUFFER, PFD_DRAW_TO_WINDOW, PFD_MAIN_PLANE, PFD_SUPPORT_OPENGL,
    PFD_TYPE_RGBA, WHITE_BRUSH,
};
use winapi::um::winuser::CreateWindowExW;
use winapi::um::winuser::DefWindowProcW;
use winapi::um::winuser::DestroyWindow;
use winapi::um::winuser::GetDC;
use winapi::um::winuser::GetMonitorInfoW;
use winapi::um::winuser::LoadCursorW;
use winapi::um::winuser::LoadIconW;
use winapi::um::winuser::MonitorFromPoint;
use winapi::um::winuser::RegisterClassExW;
use winapi::um::winuser::ReleaseDC;
use winapi::um::winuser::WindowFromDC;
use winapi::um::winuser::CW_USEDEFAULT;
use winapi::um::winuser::IDC_ARROW;
use winapi::um::winuser::IDI_APPLICATION;
use winapi::um::winuser::MONITORINFO;
use winapi::um::winuser::MONITOR_DEFAULTTOPRIMARY;
use winapi::um::winuser::WNDCLASSEXW;
use winapi::um::winuser::{
    AdjustWindowRectEx, DispatchMessageW, GetClientRect, GetKeyNameTextW, GetMessageW,
    GetWindowLongPtrW, GetWindowRect, PeekMessageW, PostQuitMessage, SendMessageW,
    SetWindowLongPtrW, ShowWindow, TranslateMessage, UpdateWindow, CS_HREDRAW, CS_VREDRAW,
    GWLP_USERDATA, MK_CONTROL, MK_LBUTTON, MK_MBUTTON, MK_RBUTTON, MK_SHIFT, MK_XBUTTON1,
    MK_XBUTTON2, MSG, PM_NOREMOVE, SIZE_RESTORED, SW_SHOWNORMAL, WINDOWPOS, WM_CLOSE, WM_DESTROY,
    WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL,
    WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SIZE, WM_WINDOWPOSCHANGED, WS_OVERLAPPED, WS_POPUP,
};

use super::input::*;
use super::scope_guard::ScopeGuard;

#[allow(non_snake_case)]
fn MAKELPARAM(l: WORD, h: WORD) -> LPARAM {
    MAKELONG(l, h) as DWORD as LPARAM
}

#[allow(non_snake_case)]
mod wgl_ffi {
    pub const WGL_DRAW_TO_WINDOW_ARB: i32 = 0x2001;
    pub const WGL_ACCELERATION_ARB: i32 = 0x2003;
    pub const WGL_FULL_ACCELERATION_ARB: i32 = 0x2027;
    pub const WGL_SUPPORT_OPENGL_ARB: i32 = 0x2010;
    pub const WGL_DOUBLE_BUFFER_ARB: i32 = 0x2011;
    pub const WGL_PIXEL_TYPE_ARB: i32 = 0x2013;
    pub const WGL_TYPE_RGBA_ARB: i32 = 0x202B;
    pub const WGL_RED_BITS_ARB: i32 = 0x2015;
    pub const WGL_GREEN_BITS_ARB: i32 = 0x2017;
    pub const WGL_BLUE_BITS_ARB: i32 = 0x2019;
    pub const WGL_ALPHA_BITS_ARB: i32 = 0x201B;
    pub const WGL_DEPTH_BITS_ARB: i32 = 0x2022;
    pub const WGL_STENCIL_BITS_ARB: i32 = 0x2023;

    pub const WGL_CONTEXT_DEBUG_BIT_ARB: i32 = 0x00000001;
    pub const WGL_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB: i32 = 0x00000002;
    pub const WGL_CONTEXT_MAJOR_VERSION_ARB: i32 = 0x2091;
    pub const WGL_CONTEXT_MINOR_VERSION_ARB: i32 = 0x2092;
    pub const WGL_CONTEXT_LAYER_PLANE_ARB: i32 = 0x2093;
    pub const WGL_CONTEXT_FLAGS_ARB: i32 = 0x2094;

    pub const WGL_CONTEXT_PROFILE_MASK_ARB: i32 = 0x9126;
    pub const WGL_CONTEXT_CORE_PROFILE_BIT_ARB: i32 = 0x00000001;
    pub const WGL_CONTEXT_COMPATIBILITY_PROFILE_BIT_ARB: i32 = 0x00000002;
    pub const WGL_CONTEXT_ROBUST_ACCESS_BIT_ARB: i32 = 0x00000004;
    pub const WGL_LOSE_CONTEXT_ON_RESET_ARB: i32 = 0x8252;
    pub const WGL_CONTEXT_RESET_NOTIFICATION_STRATEGY_ARB: i32 = 0x8256;
    pub const WGL_NO_RESET_NOTIFICATION_ARB: i32 = 0x8261;

    pub type PFNWGLCHOOSEPIXELFORMATARBPROC =
        unsafe extern "system" fn(
            hdc: winapi::shared::windef::HDC,
            piAttribList: *const std::os::raw::c_int,
            pfAttribList: *const std::os::raw::c_float,
            nMaxFormats: std::os::raw::c_uint,
            piFormats: *mut std::os::raw::c_int,
            nNumFormats: *mut std::os::raw::c_uint,
        ) -> winapi::shared::minwindef::BOOL;

    pub type PFNWGLCREATECONTEXTATTRIBSARBPROC =
        unsafe extern "system" fn(
            hDC: winapi::shared::windef::HDC,
            hShareContext: winapi::shared::windef::HGLRC,
            attribList: *const std::os::raw::c_int,
        ) -> winapi::shared::windef::HGLRC;
}

fn device_context_destructor(dc: HDC) {
    if !dc.is_null() {
        unsafe {
            let wnd = WindowFromDC(dc);
            ReleaseDC(wnd, dc);
        }
    }
}

fn wgl_context_destructor(wgl_ctx: HGLRC) {
    if !wgl_ctx.is_null() {
        unsafe {
            wglDeleteContext(wgl_ctx);
        }
    }
}

fn make_win_str(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    OsStr::new(s)
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>()
}

macro_rules! wgl_load_proc {
    ($proc_ty: ty, $pname: expr) => {
        unsafe {
            let proc_addr = std::ffi::CString::new($pname)
                .and_then(|proc_name| {
                    Ok(wglGetProcAddress(
                        proc_name.as_bytes_with_nul().as_ptr() as *const i8
                    ))
                })
                .unwrap_or(null_mut());

            if proc_addr.is_null() {
                let mut msg = String::from("Failed to load wgl function ");
                msg.push_str($pname);
                Err(msg)
            } else {
                Ok(std::mem::transmute::<_, $proc_ty>(proc_addr))
            }
        }
    };
}

fn get_suitable_pixel_format() -> Result<i32, String> {
    let temp_window_classname: &'static str = "__temporary_gl_ctx_window__";
    let w32_temp_window_classname = make_win_str(temp_window_classname);

    let mut wclass = unsafe { MaybeUninit::<WNDCLASSEXW>::zeroed().assume_init() };
    wclass.cbSize = std::mem::size_of_val(&wclass) as u32;
    wclass.lpfnWndProc = Some(DefWindowProcW);
    wclass.hInstance = unsafe { GetModuleHandleW(null()) };
    wclass.hCursor = unsafe { LoadCursorW(null_mut(), IDC_ARROW) };
    wclass.hIcon = unsafe { LoadIconW(null_mut(), IDI_APPLICATION) };
    wclass.lpszClassName = w32_temp_window_classname.as_ptr();

    unsafe {
        if RegisterClassExW(&wclass) == FALSE as u16 {
            return Err("Failed to register window class!".to_string());
        }
    }

    let guard_wnd = ScopeGuard::new(
        unsafe {
            let wnd_name = make_win_str("temp_window");

            CreateWindowExW(
                0,
                w32_temp_window_classname.as_ptr(),
                wnd_name.as_ptr(),
                WS_OVERLAPPED,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                null_mut(),
                null_mut(),
                wclass.hInstance,
                null_mut(),
            )
        },
        |wnd: HWND| {
            if !wnd.is_null() {
                unsafe {
                    DestroyWindow(wnd);
                }
            }
        },
    );

    if (*guard_wnd).is_null() {
        return Err("Failed to create temporary window".to_string());
    }

    let temp_dc = ScopeGuard::new(unsafe { GetDC(*guard_wnd) }, |dc| {
        if !dc.is_null() {
            let wnd = unsafe { WindowFromDC(dc) };
            unsafe {
                ReleaseDC(wnd, dc);
            }
        }
    });

    if (*temp_dc).is_null() {
        return Err("Failed to create temporary DC!".to_string());
    }

    let mut pfd = unsafe { MaybeUninit::<PIXELFORMATDESCRIPTOR>::zeroed().assume_init() };
    pfd.nSize = std::mem::size_of_val(&pfd) as u16;
    pfd.nVersion = 1;
    pfd.dwFlags = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER;
    pfd.iPixelType = PFD_TYPE_RGBA;
    pfd.cColorBits = 24;
    pfd.cDepthBits = 24;
    pfd.cStencilBits = 8;
    pfd.iLayerType = PFD_MAIN_PLANE;

    let pixel_format = unsafe { ChoosePixelFormat(*temp_dc, &pfd) };

    if pixel_format == 0 {
        return Err("Failed to found a suitable OpenGL pixel format!".to_string());
    }

    unsafe {
        if SetPixelFormat(*temp_dc, pixel_format, &pfd) == FALSE {
            return Err("Failed to set OpenGL pixel format for temporary window!".to_string());
        }
    }

    let wgl_ctx = ScopeGuard::new(unsafe { wglCreateContext(*temp_dc) }, |wgl_ctx| {
        if !wgl_ctx.is_null() {
            unsafe {
                wglDeleteContext(wgl_ctx);
            }
        }
    });

    if (*wgl_ctx).is_null() {
        return Err("Failed to create temporary OpenGL context!".to_string());
    }

    let ctx_restores_when_done =
        ScopeGuard::new(unsafe { wglMakeCurrent(*temp_dc, *wgl_ctx) }, |result| {
            if result != FALSE {
                unsafe {
                    wglMakeCurrent(null_mut(), null_mut());
                }
            }
        });

    if *ctx_restores_when_done == FALSE {
        return Err("Failed to make OpenGL context as current context".to_string());
    }

    #[allow(non_snake_case)]
    let wglChoosePixelFormatARB = unsafe {
        let proc_addr = std::ffi::CString::new("wglChoosePixelFormatARB")
            .and_then(|proc_name| {
                Ok(wglGetProcAddress(
                    proc_name.as_bytes_with_nul().as_ptr() as *const i8
                ))
            })
            .unwrap_or(null_mut());

        if proc_addr.is_null() {
            return Err("wglChoosePixelFormatARB procedure not found!".to_string());
        }

        std::mem::transmute::<_, PFNWGLCHOOSEPIXELFORMATARBPROC>(proc_addr)
    };

    use wgl_ffi::*;
    let pixel_format_attribs: [i32; 23] = [
        WGL_DRAW_TO_WINDOW_ARB,
        TRUE,
        WGL_ACCELERATION_ARB,
        WGL_FULL_ACCELERATION_ARB,
        WGL_SUPPORT_OPENGL_ARB,
        TRUE,
        WGL_DOUBLE_BUFFER_ARB,
        TRUE,
        WGL_PIXEL_TYPE_ARB,
        WGL_TYPE_RGBA_ARB,
        WGL_RED_BITS_ARB,
        8,
        WGL_GREEN_BITS_ARB,
        8,
        WGL_BLUE_BITS_ARB,
        8,
        WGL_ALPHA_BITS_ARB,
        8,
        WGL_DEPTH_BITS_ARB,
        24,
        WGL_STENCIL_BITS_ARB,
        8,
        0,
    ];

    let mut supported_formats: [i32; 16] = [0; 16];
    let mut format_count = 0u32;
    let query_result = unsafe {
        wglChoosePixelFormatARB(
            *temp_dc,
            pixel_format_attribs.as_ptr(),
            null(),
            supported_formats.len() as std::os::raw::c_uint,
            supported_formats.as_mut_ptr(),
            &mut format_count,
        )
    };

    if query_result != TRUE || format_count == 0 {
        return Err("wglChoosePixelFormatARB failed - no format found!".to_string());
    }

    Ok(supported_formats[0])
}

fn get_primary_monitor_dimensions() -> Result<(i32, i32), String> {
    let origin = POINT { x: 0, y: 0 };
    let primary_monitor = unsafe { MonitorFromPoint(origin, MONITOR_DEFAULTTOPRIMARY) };

    if primary_monitor.is_null() {
        return Err("Failed to get handle to primary monitor!".to_string());
    }

    let mut monitor_info = unsafe {
        MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            rcMonitor: MaybeUninit::<RECT>::zeroed().assume_init(),
            rcWork: MaybeUninit::<RECT>::zeroed().assume_init(),
            dwFlags: 0,
        }
    };

    unsafe {
        if GetMonitorInfoW(primary_monitor, &mut monitor_info) != TRUE {
            return Err("Failed to query primary monitor dimensions!".to_string());
        }
    }

    Ok((
        (monitor_info.rcWork.right - monitor_info.rcWork.left).abs(),
        (monitor_info.rcWork.bottom - monitor_info.rcWork.top).abs(),
    ))
}

gen_unique_resource_type!(
    UniqueOpenGLContext,
    OpenGLContextDeleter,
    HGLRC,
    null_mut(),
    |ctx| {
        unsafe {
            wglDeleteContext(ctx);
        }
    }
);

gen_unique_resource_type!(
    UniqueDeviceContext,
    WinDeviceContextDeleter,
    HDC,
    null_mut(),
    |dc| {
        unsafe {
            let wnd = WindowFromDC(dc);
            ReleaseDC(wnd, dc);
        }
    }
);

#[derive(Copy, Clone, Debug)]
pub struct FrameContext {
    pub screen_width: i32,
    pub screen_height: i32,
}

pub struct SimpleWindow {
    event_receiver: Option<Box<dyn Fn(&Event)>>,
    opengl_context: UniqueOpenGLContext,
    window_dc: UniqueDeviceContext,
    window: HWND,
    win_size: Cell<(i32, i32)>,
    framebuffer_size: Cell<(i32, i32)>,
}

impl std::ops::Drop for SimpleWindow {
    fn drop(&mut self) {
        unsafe {
            wglMakeCurrent(null_mut(), null_mut());
        }
    }
}

impl SimpleWindow {
    pub fn new() -> Result<SimpleWindow, String> {
        let screen_size = get_primary_monitor_dimensions()?;

        let window_class_name = make_win_str("__rusted_opengl_window__");
        let mut wclass = unsafe { MaybeUninit::<WNDCLASSEXW>::zeroed().assume_init() };
        wclass.cbSize = std::mem::size_of_val(&wclass) as u32;
        wclass.style = CS_HREDRAW | CS_VREDRAW;
        wclass.lpfnWndProc = Some(SimpleWindow::window_proc_stub);
        wclass.hInstance = unsafe { GetModuleHandleW(null()) };
        wclass.hIcon = unsafe { LoadIconW(null_mut(), IDI_APPLICATION) };
        wclass.hCursor = unsafe { LoadCursorW(null_mut(), IDC_ARROW) };
        wclass.hbrBackground =
            unsafe { std::mem::transmute::<_, HBRUSH>(GetStockObject(WHITE_BRUSH as i32)) };
        wclass.lpszClassName = window_class_name.as_ptr();

        unsafe {
            if RegisterClassExW(&wclass) == FALSE as u16 {
                return Err("Failed to register main window class".to_string());
            }
        }

        let client_rect = unsafe {
            let mut client_rect = RECT {
                left: 0,
                top: 0,
                right: screen_size.0,
                bottom: screen_size.1,
            };

            if AdjustWindowRectEx(&mut client_rect, WS_POPUP, FALSE, 0) != TRUE {
                return Err("Failed to calc window client size!".to_string());
            }

            client_rect
        };

        println!(
            "Adjusted client rect = {}:{} x {}:{}",
            client_rect.left, client_rect.top, client_rect.right, client_rect.bottom
        );

        let window = unsafe {
            CreateWindowExW(
                0,
                window_class_name.as_ptr(),
                make_win_str("Rusted OpenGL").as_ptr(),
                WS_POPUP,
                client_rect.left,
                client_rect.top,
                (client_rect.right - client_rect.left).abs(),
                (client_rect.bottom - client_rect.top).abs(),
                null_mut(),
                null_mut(),
                GetModuleHandleW(null()),
                null_mut(),
            )
        };

        if window.is_null() {
            return Err("Failed to create main window!".to_string());
        }

        //
        // get the device context
        let window_dc = UniqueDeviceContext::new(unsafe { GetDC(window) })
            .ok_or("Failed to get window DC!".to_string())?;

        //
        // set pixel a format suitable for OpenGL
        let pixel_format = get_suitable_pixel_format()?;
        let pfd = unsafe {
            let mut pfd = MaybeUninit::<PIXELFORMATDESCRIPTOR>::zeroed().assume_init();
            if DescribePixelFormat(
                *window_dc,
                pixel_format,
                size_of::<PIXELFORMATDESCRIPTOR>() as u32,
                &mut pfd,
            ) == FALSE
            {
                Err("Failed to describe pixel format".to_string())
            } else {
                Ok(pfd)
            }
        }?;

        unsafe {
            if SetPixelFormat(*window_dc, pixel_format, &pfd) != TRUE {
                return Err("Failed to set pixel format".to_string());
            }
        }

        #[allow(non_snake_case)]
        let wglCreateContextAttribsARB = unsafe {
            let wgl_ctx = ScopeGuard::new(wglCreateContext(*window_dc), wgl_context_destructor);
            if wgl_ctx.is_null() {
                return Err("Failed to create temporary OpenGL context".to_string());
            }

            let _context_needs_restored =
                ScopeGuard::new(wglMakeCurrent(*window_dc, *wgl_ctx), |res| {
                    if res == TRUE {
                        wglMakeCurrent(*window_dc, *wgl_ctx);
                    }
                });

            if *_context_needs_restored != TRUE {
                return Err("Failed to make temporary OpenGL context current".to_string());
            }

            let func_ptr = wgl_load_proc!(
                wgl_ffi::PFNWGLCREATECONTEXTATTRIBSARBPROC,
                "wglCreateContextAttribsARB"
            );

            func_ptr
        }?;

        let opengl_context_attributes: [i32; 9] = [
            wgl_ffi::WGL_CONTEXT_MAJOR_VERSION_ARB,
            4,
            wgl_ffi::WGL_CONTEXT_MINOR_VERSION_ARB,
            5,
            wgl_ffi::WGL_CONTEXT_FLAGS_ARB,
            wgl_ffi::WGL_CONTEXT_DEBUG_BIT_ARB | wgl_ffi::WGL_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB,
            wgl_ffi::WGL_CONTEXT_PROFILE_MASK_ARB,
            wgl_ffi::WGL_CONTEXT_CORE_PROFILE_BIT_ARB,
            0,
        ];

        let opengl_context = UniqueOpenGLContext::new(unsafe {
            wglCreateContextAttribsARB(*window_dc, null_mut(), opengl_context_attributes.as_ptr())
        })
        .ok_or("Failed to create OpenGL context!".to_string())?;

        unsafe {
            if wglMakeCurrent(*window_dc, *opengl_context) != TRUE {
                return Err("Failed to make OpenGL context current!".to_string());
            }
        }

        //
        // we have an OpenGL active context so it's safe to load the function pointers now.
        gl_loader::init_gl();
        gl::load_with(|s| gl_loader::get_proc_address(s) as *const _);

        let win_size = unsafe {
            let mut rc = std::mem::MaybeUninit::<RECT>::zeroed().assume_init();
            GetClientRect(window, &mut rc);
            ((rc.right - rc.left) as i32, (rc.bottom - rc.top) as i32)
        };

        Ok(SimpleWindow {
            event_receiver: None,
            window,
            window_dc,
            opengl_context,
            win_size: Cell::new(win_size),
            framebuffer_size: Cell::new((0, 0)),
        })
    }

    pub fn size(&self) -> (i32, i32) {
        self.win_size.get()
    }

    fn window_proc(&self, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let message_processing_result = match msg {
            WM_CLOSE => unsafe {
                DestroyWindow(self.window);
                Some(0)
            },

            WM_DESTROY => unsafe {
                PostQuitMessage(0);
                Some(0)
            },

            WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP => {
                let event_mouse_button = self.event_mouse_button(msg, wparam, lparam);
                self.event_receiver
                    .as_ref()
                    .map(|ev_recv_ptr| ev_recv_ptr(&event_mouse_button));
                Some(0)
            }

            WM_KEYDOWN | WM_KEYUP => {
                let key_event = self.event_key(msg, wparam, lparam);
                self.event_receiver
                    .as_ref()
                    .map(|ev_ptr| ev_ptr(&key_event));
                Some(0)
            }

            WM_MOUSEMOVE => {
                let event_mouse_move = self.event_mouse_move(wparam, lparam);
                self.event_receiver
                    .as_ref()
                    .map(|ev_recv_ptr| ev_recv_ptr(&event_mouse_move));
                Some(0)
            }

            WM_MOUSEWHEEL => {
                let event_wheel = self.event_mouse_wheel(wparam, lparam);
                self.event_receiver
                    .as_ref()
                    .map(|eventfun| eventfun(&event_wheel));
                Some(0)
            }

            WM_SIZE => {
                let width = LOWORD(lparam as u32) as i32;
                let height = HIWORD(lparam as u32) as i32;

                let event_resize = self.event_wmsize(width, height);

                self.event_receiver
                    .as_ref()
                    .map(|event_fn| event_fn(&event_resize));
                Some(0)
            }

            WM_WINDOWPOSCHANGED => {
                let ptr = unsafe { transmute::<_, *const WINDOWPOS>(lparam) };
                self.event_windowposchanged(&unsafe { *ptr });
                Some(0)
            }

            _ => None,
        };

        message_processing_result
            .unwrap_or(unsafe { DefWindowProcW(self.window, msg, wparam, lparam) })
    }

    extern "system" fn window_proc_stub(
        w: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            let window_obj =
                transmute::<_, *const SimpleWindow>(GetWindowLongPtrW(w, GWLP_USERDATA));

            if window_obj.is_null() {
                DefWindowProcW(w, msg, wparam, lparam)
            } else {
                (*window_obj).window_proc(msg, wparam, lparam)
            }
        }
    }

    fn event_mouse_button(&self, ty: u32, wp: WPARAM, lp: LPARAM) -> Event {
        let mut mbe = MouseButtonEventData::default();
        mbe.type_ = if ty == WM_LBUTTONDOWN || ty == WM_RBUTTONDOWN {
            ActionType::Press
        } else {
            ActionType::Release
        };
        mbe.pointer_x = GET_X_LPARAM(lp);
        mbe.pointer_y = GET_Y_LPARAM(lp);
        mbe.button = if ty == WM_LBUTTONDOWN || ty == WM_LBUTTONUP {
            MouseButtonId::Button1
        } else {
            MouseButtonId::Button3
        };

        mbe.button1 = (wp & MK_LBUTTON) != 0;
        mbe.button2 = (wp & MK_MBUTTON) != 0;
        mbe.button3 = (wp & MK_RBUTTON) != 0;
        mbe.button4 = (wp & MK_XBUTTON1) != 0;
        mbe.button5 = (wp & MK_XBUTTON2) != 0;
        mbe.shift = (wp & MK_SHIFT) != 0;
        mbe.control = (wp & MK_CONTROL) != 0;

        Event::Input(InputEventData::MouseButton(mbe))
    }

    fn event_mouse_move(&self, wp: WPARAM, lp: LPARAM) -> Event {
        Event::Input(InputEventData::MouseMotion(MouseMotionEventData {
            pointer_x: GET_X_LPARAM(lp),
            pointer_y: GET_Y_LPARAM(lp),
            button1: (wp & MK_LBUTTON) != 0,
            button2: (wp & MK_MBUTTON) != 0,
            button3: (wp & MK_RBUTTON) != 0,
            button4: (wp & MK_XBUTTON1) != 0,
            button5: (wp & MK_XBUTTON2) != 0,
            shift: (wp & MK_SHIFT) != 0,
            control: (wp & MK_CONTROL) != 0,
        }))
    }

    fn event_mouse_wheel(&self, wp: WPARAM, lp: LPARAM) -> Event {
        use winapi::um::winuser::{GET_KEYSTATE_WPARAM, GET_WHEEL_DELTA_WPARAM};

        let keys = GET_KEYSTATE_WPARAM(wp) as usize;

        Event::Input(InputEventData::MouseWheel(MouseWheelEventData {
            pointer_x: GET_X_LPARAM(lp),
            pointer_y: GET_Y_LPARAM(lp),
            delta: GET_WHEEL_DELTA_WPARAM(wp) as i32 / 120,
            button1: (keys & MK_LBUTTON) != 0,
            button2: (keys & MK_MBUTTON) != 0,
            button3: (keys & MK_RBUTTON) != 0,
            button4: (keys & MK_XBUTTON1) != 0,
            button5: (keys & MK_XBUTTON2) != 0,
            shift: (keys & MK_SHIFT) != 0,
            control: (keys & MK_CONTROL) != 0,
        }))
    }

    fn map_key_symbol(key_sym: WPARAM) -> KeySymbol {
        if key_sym as usize >= WIN32_KEYS_MAPPING_TABLE.len() {
            println!(
                "Warning : no mapping exists for native key symbol {}",
                key_sym
            );
            return KeySymbol::Unknown;
        }

        WIN32_KEYS_MAPPING_TABLE[key_sym as usize]
    }

    fn event_key(&self, msg: u32, wp: WPARAM, lp: LPARAM) -> Event {
        let action_type = if msg == WM_KEYDOWN {
            ActionType::Press
        } else {
            ActionType::Release
        };

        let mut ke = KeyEventData {
            pointer_x: GET_X_LPARAM(lp),
            pointer_y: GET_Y_LPARAM(lp),
            keycode: Self::map_key_symbol(wp),
            type_: action_type,
            button1: (wp & MK_LBUTTON) != 0,
            button2: (wp & MK_MBUTTON) != 0,
            button3: (wp & MK_RBUTTON) != 0,
            button4: (wp & MK_XBUTTON1) != 0,
            button5: (wp & MK_XBUTTON2) != 0,
            shift: (wp & MK_SHIFT) != 0,
            control: (wp & MK_CONTROL) != 0,
            name: unsafe { ::std::mem::zeroed() },
        };

        unsafe {
            let mut key_name_buff = [0u16; 256];

            let res = GetKeyNameTextW(
                lp as i32,
                key_name_buff.as_mut_ptr(),
                key_name_buff.len() as i32,
            );

            if res != 0 {
                if let Ok(str) = String::from_utf16(&key_name_buff[0..res as usize]) {
                    let bytes = str.as_bytes();

                    if bytes.len() <= ke.name.len() {
                        ::std::ptr::copy_nonoverlapping(
                            bytes.as_ptr(),
                            ke.name.as_mut_ptr(),
                            ke.name.len().min(bytes.len()),
                        );
                    }
                }
            }
        }

        Event::Input(InputEventData::Key(ke))
    }

    fn event_windowposchanged(&self, w: &WINDOWPOS) {
        println!("WINDOWPOSCHANGED {} {}", w.cx, w.cy);
        self.win_size.set((w.cx, w.cy));
    }

    fn event_wmsize(&self, width: i32, height: i32) -> Event {
        self.framebuffer_size.set((width, height));

        let win_size = unsafe {
            let mut wr = MaybeUninit::<RECT>::zeroed().assume_init();
            GetWindowRect(self.window, &mut wr);
            (wr.right - wr.left, wr.bottom - wr.top)
        };

        println!(
            "WM_SIZE event! W: {}, H: {}, FBX: {}, FBY: {}",
            width, height, win_size.0, win_size.1
        );

        self.win_size.set(win_size);
        Event::Configure(WindowConfigureEventData { width, height })
    }

    pub fn message_loop(&mut self, event_fn: Box<dyn Fn(&Event)>) {
        self.event_receiver = Some(event_fn);
        unsafe {
            SetWindowLongPtrW(
                self.window,
                GWLP_USERDATA,
                transmute::<_, LONG_PTR>(self as *const _),
            );

            let mut rc: RECT = MaybeUninit::zeroed().assume_init();
            GetClientRect(self.window, &mut rc);
            SendMessageW(
                self.window,
                WM_SIZE,
                SIZE_RESTORED,
                MAKELPARAM(
                    (rc.right - rc.left).abs() as WORD,
                    (rc.bottom - rc.top).abs() as WORD,
                ),
            );

            ShowWindow(self.window, SW_SHOWNORMAL);
            UpdateWindow(self.window);

            let mut msg = MaybeUninit::<MSG>::zeroed().assume_init();

            'main_loop: loop {
                self.event_receiver
                    .as_ref()
                    .map(|evrecv| (evrecv)(&Event::InputBegin));

                'message_loop: loop {
                    if PeekMessageW(&mut msg, null_mut(), 0, 0, PM_NOREMOVE) == TRUE {
                        let res = GetMessageW(&mut msg, null_mut(), 0, 0);
                        if res <= 0 {
                            break 'main_loop;
                        }

                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    } else {
                        break 'message_loop;
                    }
                }

                self.event_receiver
                    .as_ref()
                    .map(|evrecv| (evrecv)(&Event::InputEnd));

                //
                // update + draw here
                let client_rect = {
                    let mut r = MaybeUninit::<RECT>::zeroed().assume_init();
                    GetClientRect(self.window, &mut r);
                    r
                };

                self.event_receiver.as_ref().map(|event_receiver| {
                    event_receiver(&Event::Loop(LoopEventData {
                        surface_width: client_rect.right - client_rect.left,
                        surface_height: client_rect.bottom - client_rect.top,
                        window_width: self.win_size.get().0,
                        window_height: self.win_size.get().1,
                    }))
                });

                SwapBuffers(*self.window_dc);
                // std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }
}

const WIN32_KEYS_MAPPING_TABLE: [KeySymbol; 256] = [
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Backspace,
    KeySymbol::Tab,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Clear,
    KeySymbol::Enter,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Pause,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Escape,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Space,
    KeySymbol::PageUp,
    KeySymbol::PageDown,
    KeySymbol::End,
    KeySymbol::Home,
    KeySymbol::Left,
    KeySymbol::Up,
    KeySymbol::Right,
    KeySymbol::Down,
    KeySymbol::Select,
    KeySymbol::PrintScreen,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Insert,
    KeySymbol::Del,
    KeySymbol::Unknown,
    KeySymbol::Key0,
    KeySymbol::Key1,
    KeySymbol::Key2,
    KeySymbol::Key3,
    KeySymbol::Key4,
    KeySymbol::Key5,
    KeySymbol::Key6,
    KeySymbol::Key7,
    KeySymbol::Key8,
    KeySymbol::Key9,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::KeyA,
    KeySymbol::KeyB,
    KeySymbol::KeyC,
    KeySymbol::KeyD,
    KeySymbol::KeyE,
    KeySymbol::KeyF,
    KeySymbol::KeyG,
    KeySymbol::KeyH,
    KeySymbol::KeyI,
    KeySymbol::KeyJ,
    KeySymbol::KeyK,
    KeySymbol::KeyL,
    KeySymbol::KeyM,
    KeySymbol::KeyN,
    KeySymbol::KeyO,
    KeySymbol::KeyP,
    KeySymbol::KeyQ,
    KeySymbol::KeyR,
    KeySymbol::KeyS,
    KeySymbol::KeyT,
    KeySymbol::KeyU,
    KeySymbol::KeyV,
    KeySymbol::KeyW,
    KeySymbol::KeyX,
    KeySymbol::KeyY,
    KeySymbol::KeyZ,
    KeySymbol::LeftWin,
    KeySymbol::RightWin,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Kp0,
    KeySymbol::Kp1,
    KeySymbol::Kp2,
    KeySymbol::Kp3,
    KeySymbol::Kp4,
    KeySymbol::Kp5,
    KeySymbol::Kp6,
    KeySymbol::Kp7,
    KeySymbol::Kp8,
    KeySymbol::Kp9,
    KeySymbol::KpMultiply,
    KeySymbol::KpAdd,
    KeySymbol::Unknown,
    KeySymbol::KpMinus,
    KeySymbol::Unknown,
    KeySymbol::KpDivide,
    KeySymbol::F1,
    KeySymbol::F2,
    KeySymbol::F3,
    KeySymbol::F4,
    KeySymbol::F5,
    KeySymbol::F6,
    KeySymbol::F7,
    KeySymbol::F8,
    KeySymbol::F9,
    KeySymbol::F10,
    KeySymbol::F11,
    KeySymbol::F12,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::ScrolLock,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::LeftShift,
    KeySymbol::RightShift,
    KeySymbol::LeftControl,
    KeySymbol::RightControl,
    KeySymbol::LeftMenu,
    KeySymbol::RightMenu,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
    KeySymbol::Unknown,
];
