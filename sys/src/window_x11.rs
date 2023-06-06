#![cfg(unix)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use std::mem::MaybeUninit;
use std::option::Option;
use std::ptr::{null, null_mut};

use x11::glx::{
    arb::GLX_CONTEXT_CORE_PROFILE_BIT_ARB, arb::GLX_CONTEXT_DEBUG_BIT_ARB,
    arb::GLX_CONTEXT_FLAGS_ARB, arb::GLX_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB,
    arb::GLX_CONTEXT_MAJOR_VERSION_ARB, arb::GLX_CONTEXT_MINOR_VERSION_ARB,
    arb::GLX_CONTEXT_PROFILE_MASK_ARB, glXChooseFBConfig, glXDestroyContext,
    glXGetVisualFromFBConfig, glXMakeContextCurrent, glXQueryExtension, glXQueryExtensionsString,
    glXQueryVersion, glXSwapBuffers, GLXContext, GLXDrawable, GLXFBConfig, GLX_BUFFER_SIZE,
    GLX_CONFIG_CAVEAT, GLX_DEPTH_SIZE, GLX_DOUBLEBUFFER, GLX_DRAWABLE_TYPE, GLX_NONE,
    GLX_RENDER_TYPE, GLX_RGBA_BIT, GLX_RGBA_TYPE, GLX_STENCIL_SIZE, GLX_WINDOW_BIT,
};

use x11::xinerama::{XineramaQueryScreens, XineramaScreenInfo};

use x11::xlib::{
    AllocNone, AlreadyGrabbed, Atom, Bool, Button1, Button1Mask, Button2, Button2Mask, Button3,
    Button3Mask, Button4, Button4Mask, Button5, Button5Mask, ButtonPress, ButtonPressMask,
    ButtonRelease, ButtonReleaseMask, CWBackPixel, CWColormap, CWEventMask, CWOverrideRedirect,
    ClientMessage, ConfigureNotify, ControlMask, CurrentTime, Display, EnterWindowMask,
    ExposureMask, False, FocusChangeMask, FocusIn, FocusOut, GrabFrozen, GrabInvalidTime,
    GrabModeAsync, GrabNotViewable, InputHint, InputOutput, KeyPress, KeyPressMask, KeyRelease,
    KeyReleaseMask, KeySym, LeaveWindowMask, MotionNotify, NoSymbol, PBaseSize, PMinSize,
    PointerMotionMask, PropModeReplace, RevertToNone, Screen, ShiftMask, StateHint,
    StructureNotifyMask, SubstructureNotifyMask, SubstructureRedirectMask, True,
    VisibilityChangeMask, VisibilityNotify, VisibilityUnobscured, Window, XAllocSizeHints,
    XAllocWMHints, XButtonEvent, XChangeProperty, XClearWindow, XClientMessageEvent, XCloseDisplay,
    XConfigureEvent, XCreateColormap, XCreateWindow, XDefaultRootWindow, XDefaultScreen,
    XDestroyWindow, XEvent, XEventsQueued, XFlush, XFocusChangeEvent, XFree, XGetGeometry,
    XGrabKeyboard, XInternAtom, XKeyEvent, XLookupString, XMapRaised, XMotionEvent, XMoveWindow,
    XNextEvent, XOpenDisplay, XQueryPointer, XRootWindow, XSendEvent, XSetInputFocus,
    XSetWMProperties, XSetWMProtocols, XSetWindowAttributes, XSizeHints, XSync, XUngrabKeyboard,
    XVisibilityEvent, XVisualInfo, XWMHints, XWarpPointer, XWhitePixel, XA_ATOM,
};

use super::input::*;

use std::ffi::{CStr, CString};
use std::mem::transmute;
use std::os::raw::{c_int, c_long, c_uchar, c_uint, c_ulong, c_void};

type PFNGLXCREATECONTEXTATTRIBSARBPROC = unsafe extern "C" fn(
    dpy: *mut Display,
    config: GLXFBConfig,
    share_context: GLXContext,
    direct: Bool,
    attrib_list: *const c_int,
) -> GLXContext;

// type PFNGLXSWAPINTERVALEXT =
//     unsafe extern "C" fn(dpy: *mut Display, drawable: GLXDrawable, interval: c_int);

#[link(name = "GL")]
extern "C" {
    pub fn glXGetProcAddress(_1: *const c_uchar) -> *mut c_void;
}

gen_unique_resource_type!(
    ScopedXineramaScreenInfo,
    XineramaScreenInfoDeleter,
    *mut XineramaScreenInfo,
    null_mut(),
    |xsi: *mut XineramaScreenInfo| unsafe {
        XFree(xsi as *mut libc::c_void);
    }
);

gen_unique_resource_type!(
    ScopedXVisualInfo,
    XVisualInfoDeleter,
    *mut XVisualInfo,
    null_mut(),
    |xvi: *mut XVisualInfo| unsafe {
        XFree(xvi as *mut libc::c_void);
    }
);

gen_unique_resource_type!(
    ScopedGLXFBConfig,
    GLXFBConfigDeleter,
    *mut GLXFBConfig,
    null_mut(),
    |fbc: *mut GLXFBConfig| unsafe {
        XFree(fbc as *mut libc::c_void);
    }
);

gen_unique_resource_type!(
    ScopedXSizeHints,
    XSizeHintsDeleter,
    *mut XSizeHints,
    null_mut(),
    |xsh: *mut XSizeHints| unsafe {
        XFree(xsh as *mut libc::c_void);
    }
);

gen_unique_resource_type!(
    ScopedXWMHints,
    XWMHintsDeleter,
    *mut XWMHints,
    null_mut(),
    |xwm: *mut XWMHints| unsafe {
        XFree(xwm as *mut libc::c_void);
    }
);

pub struct SimpleWindow {
    event_receiver: Option<Box<dyn Fn(&Event)>>,
    glcontext: GLXContext,
    window: c_ulong,
    delete_atom: c_ulong,
    dpy: *mut Display,
    size: std::cell::Cell<(i32, i32)>,
    win_size: std::cell::Cell<(i32, i32)>,
}

impl SimpleWindow {
    pub fn new() -> Result<SimpleWindow, String> {
        let dpy = unsafe { XOpenDisplay(null()) };
        if dpy.is_null() {
            return Err("Failed to open display!".into());
        }

        let primary_screen = platform_utils::get_primary_screen_info(dpy)?;
        println!("Primary screen {:?}", primary_screen);

        let default_screen = unsafe { XDefaultScreen(dpy) };
        let (xvisual, fbcfg) = platform_utils::get_suitable_xvisual(dpy)?;
        let root_window = unsafe { XRootWindow(dpy, default_screen) };

        let mut xswa = unsafe {
            let mut a = MaybeUninit::<XSetWindowAttributes>::zeroed().assume_init();

            a.background_pixel = XWhitePixel(dpy, default_screen);
            a.colormap = XCreateColormap(dpy, root_window, (**xvisual).visual, AllocNone);
            a.event_mask = KeyPressMask
                | FocusChangeMask
                | KeyReleaseMask
                | ButtonPressMask
                | ButtonReleaseMask
                | PointerMotionMask
                | ExposureMask
                | StructureNotifyMask
                | LeaveWindowMask
                | EnterWindowMask
                | SubstructureNotifyMask
                | VisibilityChangeMask;

            a
        };

        let window = unsafe {
            XCreateWindow(
                dpy,
                root_window,
                primary_screen.x_org as i32,
                primary_screen.y_org as i32,
                primary_screen.width as u32,
                primary_screen.height as u32,
                0,
                (**xvisual).depth,
                InputOutput as u32,
                (**xvisual).visual,
                CWEventMask | CWColormap | CWBackPixel | CWOverrideRedirect,
                &mut xswa as *mut XSetWindowAttributes,
            )
        };

        if window == 0 {
            return Err("XCreateWindow() failed!".to_string());
        }

        platform_utils::setup_size_hints(dpy, window, &primary_screen)?;

        //
        // Create and make modern OpenGL context as current
        let glcontext = platform_utils::create_opengl_context(dpy, &fbcfg, default_screen)?;
        unsafe {
            glXMakeContextCurrent(dpy, window, window, glcontext);
        }

        println!("OpenGL context created!");
        let delete_atom = platform_utils::setup_window(dpy, window, &primary_screen);

        //
        // we have an OpenGL active context so it's safe to load the function pointers now.
        gl_loader::init_gl();
        gl::load_with(|s| gl_loader::get_proc_address(s) as *const _);

        let size = platform_utils::get_window_client_rect(dpy, window);

        Ok(SimpleWindow {
            event_receiver: None,
            glcontext,
            window,
            delete_atom,
            dpy,
            size: std::cell::Cell::new(size),
            win_size: std::cell::Cell::new(size),
        })
    }

    pub fn size(&self) -> (i32, i32) {
        self.win_size.get()
    }

    fn handle_client_message_event(&self, cme: &XClientMessageEvent) -> bool {
        println!("{:?}", cme);
        cme.data.as_longs()[0] == self.delete_atom as c_long
    }

    fn handle_visibility_event(&self, vse: &XVisibilityEvent) {
        if vse.window == self.window && vse.state == VisibilityUnobscured {
            unsafe {
                XSetInputFocus(self.dpy, self.window, RevertToNone, CurrentTime);
            }
        }
    }

    fn handle_configure_event(&self, xce: &XConfigureEvent) -> Event {
        println!("XConfigureEvent {:?}", xce);
        self.size.replace((xce.width, xce.height));
        self.win_size.replace((xce.width, xce.height));

        Event::Configure(WindowConfigureEventData {
            width: xce.width,
            height: xce.height,
        })
    }

    fn handle_key_event(&self, xke: &mut XKeyEvent) -> Event {
        let mut tmp: [i8; 256] = [0i8; 256];
        let mut ks: KeySym = NoSymbol as KeySym;
        let buff_len = unsafe {
            XLookupString(
                xke as *mut XKeyEvent,
                tmp.as_mut_ptr(),
                tmp.len() as c_int,
                &mut ks as *mut KeySym,
                null_mut(),
            )
        };

        tmp[buff_len as usize] = 0;

        let mapped_key = platform_utils::map_x11_key_symbol(ks);

        let mut ke = KeyEventData::default();

        if buff_len != 0 {
            let slice = unsafe { CStr::from_ptr(tmp.as_ptr()) };
            if let Ok(key_desc) = slice.to_str() {
                unsafe {
                    ::std::ptr::copy_nonoverlapping(
                        key_desc.as_ptr(),
                        ke.name.as_mut_ptr(),
                        ke.name.len().min(key_desc.len()),
                    );
                }
            }
        }

        ke.pointer_x = xke.x;
        ke.pointer_y = xke.y;
        ke.keycode = mapped_key;
        ke.type_ = if xke.type_ == KeyPress {
            ActionType::Press
        } else {
            ActionType::Release
        };
        ke.button1 = (xke.state & Button1Mask) != 0;
        ke.button2 = (xke.state & Button2Mask) != 0;
        ke.button3 = (xke.state & Button3Mask) != 0;
        ke.button4 = (xke.state & Button4Mask) != 0;
        ke.button5 = (xke.state & Button5Mask) != 0;
        ke.shift = (xke.state & ShiftMask) != 0;
        ke.control = (xke.state & ControlMask) != 0;

        Event::Input(InputEventData::Key(ke))
    }

    fn handle_mouse_button_event(&self, xbe: &XButtonEvent) -> Event {
        let mapped_btn = match xbe.button {
            Button1 => Some(MouseButtonId::Button1),
            Button2 => Some(MouseButtonId::Button2),
            Button3 => Some(MouseButtonId::Button3),
            _ => None,
        };

        mapped_btn.map_or_else(
            || {
                let mut mwe = MouseWheelEventData::default();
                mwe.delta = if xbe.button == Button4 { 1 } else { -1 };
                mwe.pointer_x = xbe.x;
                mwe.pointer_y = xbe.y;
                mwe.button1 = (xbe.state & Button1Mask) != 0;
                mwe.button2 = (xbe.state & Button2Mask) != 0;
                mwe.button3 = (xbe.state & Button3Mask) != 0;
                mwe.button4 = (xbe.state & Button4Mask) != 0;
                mwe.button5 = (xbe.state & Button5Mask) != 0;
                mwe.shift = (xbe.state & ShiftMask) != 0;
                mwe.control = (xbe.state & ControlMask) != 0;

                Event::Input(InputEventData::MouseWheel(mwe))
            },
            |btn| {
                let mut mbtn_evt = MouseButtonEventData::default();
                mbtn_evt.pointer_x = xbe.x;
                mbtn_evt.pointer_y = xbe.y;
                mbtn_evt.type_ = if xbe.type_ == ButtonPress {
                    ActionType::Press
                } else {
                    ActionType::Release
                };

                mbtn_evt.button = btn;
                mbtn_evt.button1 = (xbe.state & Button1Mask) != 0;
                mbtn_evt.button2 = (xbe.state & Button2Mask) != 0;
                mbtn_evt.button3 = (xbe.state & Button3Mask) != 0;
                mbtn_evt.button4 = (xbe.state & Button4Mask) != 0;
                mbtn_evt.button5 = (xbe.state & Button5Mask) != 0;
                mbtn_evt.shift = (xbe.state & ShiftMask) != 0;
                mbtn_evt.control = (xbe.state & ControlMask) != 0;

                Event::Input(InputEventData::MouseButton(mbtn_evt))
            },
        )
    }

    fn handle_mouse_motion_event(&self, x11evt: &XMotionEvent) -> Event {
        let mut mme = MouseMotionEventData::default();

        mme.pointer_x = x11evt.x;
        mme.pointer_y = x11evt.y;
        mme.button1 = (x11evt.state & Button1Mask) != 0;
        mme.button2 = (x11evt.state & Button2Mask) != 0;
        mme.button3 = (x11evt.state & Button3Mask) != 0;
        mme.button4 = (x11evt.state & Button4Mask) != 0;
        mme.button5 = (x11evt.state & Button5Mask) != 0;
        mme.shift = (x11evt.state & ShiftMask) != 0;
        mme.control = (x11evt.state & ControlMask) != 0;

        Event::Input(InputEventData::MouseMotion(mme))
    }

    pub fn message_loop(&mut self, event_fn: Box<dyn Fn(&Event)>) {
        self.event_receiver = Some(event_fn);

        'main_loop: loop {
            //
            // send input begin event to receiver
            self.event_receiver
                .as_ref()
                .map(|evrec| (evrec)(&Event::InputBegin));

            while unsafe { XEventsQueued(self.dpy, 2) } != 0 {
                let mut window_event = unsafe { MaybeUninit::<XEvent>::zeroed().assume_init() };
                unsafe {
                    XNextEvent(self.dpy, &mut window_event as *mut _);
                }

                //
                // Handle messages in message queue
                unsafe {
                    let forward_event = match window_event.type_ {
                        VisibilityNotify => {
                            self.handle_visibility_event(&window_event.visibility);
                            None
                        }

                        ClientMessage => {
                            if self.handle_client_message_event(&window_event.client_message) {
                                break 'main_loop;
                            } else {
                                None
                            }
                        }

                        ConfigureNotify => {
                            Some(self.handle_configure_event(&window_event.configure))
                        }

                        KeyPress | KeyRelease => Some(self.handle_key_event(&mut window_event.key)),

                        ButtonPress | ButtonRelease => {
                            Some(self.handle_mouse_button_event(&window_event.button))
                        }

                        MotionNotify => Some(self.handle_mouse_motion_event(&window_event.motion)),

                        _ => {
                            println!("Event {:?}", window_event);
                        }
                    };

                    //
                    // forward the event if needed
                    forward_event.map(|e| {
                        self.event_receiver.as_ref().map(|evrecv| {
                            (evrecv)(&e);
                        })
                    });
                }
            }

            //
            // send input end event to receiver
            self.event_receiver
                .as_ref()
                .map(|evrec| (evrec)(&Event::InputEnd));

            //
            // send main loop event to receiver
            self.event_receiver.as_ref().map(|evrec| {
                let (w, h) = self.size.get();
                (evrec)(&Event::Loop(LoopEventData {
                    surface_width: w,
                    surface_height: h,
                    window_width: self.win_size.get().0,
                    window_height: self.win_size.get().1,
                }));
            });

            unsafe {
                glXSwapBuffers(self.dpy, self.window);
            }
        }
    }
}

impl std::ops::Drop for SimpleWindow {
    fn drop(&mut self) {
        if !self.glcontext.is_null() {
            unsafe {
                glXMakeContextCurrent(
                    self.dpy,
                    x11::glx::GLX_NONE as u64,
                    x11::glx::GLX_NONE as u64,
                    null_mut(),
                );
                glXDestroyContext(self.dpy, self.glcontext);
            }
        }

        if self.window != 0 {
            unsafe {
                XDestroyWindow(self.dpy, self.window);
            }
        }

        if !self.dpy.is_null() {
            unsafe {
                XCloseDisplay(self.dpy);
            }
        }
    }
}

mod platform_utils {
    use super::*;

    pub fn get_window_client_rect(dpy: *mut Display, win: Window) -> (i32, i32) {
        let mut rootwnd: c_ulong = 0;
        let mut xloc: c_int = 0;
        let mut yloc: c_int = 0;
        let mut width: c_uint = 0;
        let mut height: c_uint = 0;
        let mut bwidth: c_uint = 0;
        let mut depth: c_uint = 0;

        unsafe {
            XGetGeometry(
                dpy,
                win,
                &mut rootwnd as *mut c_ulong,
                &mut xloc as *mut c_int,
                &mut yloc as *mut c_int,
                &mut width as *mut c_uint,
                &mut height as *mut c_uint,
                &mut bwidth as *mut c_uint,
                &mut depth as *mut c_uint,
            );
        }

        (width as i32, height as i32)
    }

    pub fn get_primary_screen_info(dpy: *mut Display) -> Result<XineramaScreenInfo, String> {
        let mut num_screens: c_int = 0;
        let screens = ScopedXineramaScreenInfo::new(unsafe {
            XineramaQueryScreens(dpy, &mut num_screens as *mut c_int)
        })
        .ok_or("Failed to query xinerama screens!".to_string())?;

        if screens.is_null() || num_screens == 0 {
            return Err("Failed to get screen information".to_string());
        }

        let root_screen = unsafe { XDefaultScreen(dpy) };
        (0..num_screens)
            .find_map(|idx| unsafe {
                let curr_screen = screens.offset(idx as isize);
                if (*curr_screen).screen_number == root_screen {
                    Some(*curr_screen)
                } else {
                    None
                }
            })
            .ok_or("Failed to get root screen info!".to_string())
    }

    pub fn get_suitable_xvisual(
        dpy: *mut Display,
    ) -> Result<(ScopedXVisualInfo, GLXFBConfig), String> {
        let glx_extension_present =
            unsafe { glXQueryExtension(dpy, null_mut(), null_mut()) != False };

        if !glx_extension_present {
            return Err("GLX extension is not present!".to_string());
        }

        let mut glx_major: c_int = 0;
        let mut glx_minor: c_int = 0;

        unsafe {
            glXQueryVersion(
                dpy,
                &mut glx_major as *mut c_int,
                &mut glx_minor as *mut c_int,
            );
        }

        if glx_major < 1 || glx_minor != 4 {
            let s = format!("Wrong glx version : {} {}, need 1.4", glx_major, glx_minor);
            return Err(s);
        }

        let framebuffer_attributes = [
            GLX_BUFFER_SIZE,
            32,
            GLX_DOUBLEBUFFER,
            True,
            GLX_DEPTH_SIZE,
            24,
            GLX_STENCIL_SIZE,
            8,
            GLX_RENDER_TYPE,
            GLX_RGBA_BIT,
            GLX_DRAWABLE_TYPE,
            GLX_WINDOW_BIT,
            GLX_CONFIG_CAVEAT,
            GLX_NONE,
            0,
        ];

        let default_screen = unsafe { XDefaultScreen(dpy) };
        let mut supported_cfgs_count: i32 = 0;

        let supported_cfgs = ScopedGLXFBConfig::new(unsafe {
            glXChooseFBConfig(
                dpy,
                default_screen,
                framebuffer_attributes.as_ptr(),
                &mut supported_cfgs_count as *mut i32,
            )
        })
        .ok_or("Failed to get valid FB configurations".to_string())?;

        (0..supported_cfgs_count)
            .find_map(|idx| {
                let cfg = unsafe { *(*supported_cfgs).offset(idx as isize) };
                ScopedXVisualInfo::new(unsafe { glXGetVisualFromFBConfig(dpy, cfg) })
                    .map(|vi| (vi, cfg))
            })
            .ok_or("Failed to get XVisual!".to_string())
    }

    pub fn create_opengl_context(
        dpy: *mut Display,
        cfg: &GLXFBConfig,
        scr: c_int,
    ) -> Result<GLXContext, String> {
        let extensions_list = unsafe { glXQueryExtensionsString(dpy, scr) };
        if extensions_list.is_null() {
            return Err("Failed to get extensions list".to_string());
        }

        // println!(
        //     "Extensions list {}",
        //     CStr::from_ptr(extensions_list).to_str().unwrap_or_default()
        // );

        #[allow(non_snake_case)]
        let glXCreateContextAttribsARB = CString::new("glXCreateContextAttribsARB")
            .map_err(|_| "failed to translate proc name to C string".to_string())
            .and_then(|proc_name| unsafe {
                let func_addr = glXGetProcAddress(proc_name.as_bytes_with_nul().as_ptr());
                if func_addr.is_null() {
                    Err(format!(
                        "Failed to load {}",
                        proc_name.into_string().unwrap_or_default()
                    ))
                } else {
                    Ok(transmute::<_, PFNGLXCREATECONTEXTATTRIBSARBPROC>(func_addr))
                }
            })?;

        let opengl_context_attribs = [
            GLX_CONTEXT_MAJOR_VERSION_ARB,
            4,
            GLX_CONTEXT_MINOR_VERSION_ARB,
            5,
            GLX_CONTEXT_FLAGS_ARB,
            GLX_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB | GLX_CONTEXT_DEBUG_BIT_ARB,
            GLX_CONTEXT_PROFILE_MASK_ARB,
            GLX_CONTEXT_CORE_PROFILE_BIT_ARB,
            GLX_RENDER_TYPE,
            GLX_RGBA_TYPE,
            0,
        ];

        let context = unsafe {
            glXCreateContextAttribsARB(dpy, *cfg, null_mut(), True, opengl_context_attribs.as_ptr())
        };

        if context.is_null() {
            Err("Failed to create OpenGL context!".to_string())
        } else {
            Ok(context)
        }
    }

    pub fn setup_size_hints(
        dpy: *mut Display,
        win: Window,
        primary_screen: &XineramaScreenInfo,
    ) -> Result<(), String> {
        unsafe {
            let mut size_hints = ScopedXSizeHints::new(XAllocSizeHints())
                .ok_or("Failed to allocate size hints!".to_string())?;

            (**size_hints).flags = PMinSize | PBaseSize;
            (**size_hints).min_width = 1024;
            (**size_hints).max_width = 1024;
            (**size_hints).base_width = primary_screen.width as i32;
            (**size_hints).base_height = primary_screen.height as i32;

            let mut wm_hints = ScopedXWMHints::new(XAllocWMHints())
                .ok_or("Failed to allocate WM hints!".to_string())?;
            (**wm_hints).flags = StateHint | InputHint;
            (**wm_hints).initial_state = 0;
            (**wm_hints).input = True;

            XSetWMProperties(
                dpy,
                win,
                null_mut(),
                null_mut(),
                null_mut(),
                0,
                *size_hints,
                *wm_hints,
                null_mut(),
            );
        }

        Ok(())
    }

    pub fn setup_window(dpy: *mut Display, window: Window, screen: &XineramaScreenInfo) -> Atom {
        unsafe {
            XClearWindow(dpy, window);
            XMapRaised(dpy, window);
            XMoveWindow(dpy, window, screen.x_org as i32, screen.y_org as i32);

            let mut window_delete_atom = XInternAtom(
                dpy,
                CStr::from_bytes_with_nul(b"WM_DELETE_ATOM\0")
                    .unwrap_or_default()
                    .as_ptr(),
                False,
            );

            XSetWMProtocols(dpy, window, &mut window_delete_atom as *mut c_ulong, 1);

            let _NET_WM_STATE = XInternAtom(
                dpy,
                CStr::from_bytes_with_nul(b"_NET_WM_STATE\0")
                    .unwrap_or_default()
                    .as_ptr(),
                False,
            );

            let _NET_WM_STATE_FULLSCREEN = XInternAtom(
                dpy,
                CStr::from_bytes_with_nul(b"_NET_WM_STATE_FULLSCREEN\0")
                    .unwrap()
                    .as_ptr(),
                False,
            );

            let mut e = MaybeUninit::<XEvent>::zeroed().assume_init();
            e.type_ = ClientMessage;
            e.client_message.window = window;
            e.client_message.message_type = _NET_WM_STATE;
            e.client_message.format = 32;
            e.client_message.data.set_long(0, 1);
            e.client_message
                .data
                .set_long(1, _NET_WM_STATE_FULLSCREEN as c_long);
            e.client_message.data.set_long(2, 0);

            XSendEvent(
                dpy,
                XDefaultRootWindow(dpy),
                False,
                SubstructureNotifyMask | SubstructureRedirectMask,
                &mut e,
            );

            XFlush(dpy);
            window_delete_atom
        }
    }

    pub fn map_x11_key_symbol(x11_key: KeySym) -> KeySymbol {
        //
        // special keys have byte2 set to 0xFF, regular keys to 0x00
        let byte2 = (x11_key as u16 & 0xFF00) >> 8;
        //
        //  byte 0 is the table lookup index
        let sym_idx = (x11_key as u32 & 0xFFu32) as usize;

        if byte2 == 0x00 {
            assert!(sym_idx < X11_LATIN1_KEYS_MAPPING_TABLE.len());
            X11_LATIN1_KEYS_MAPPING_TABLE[sym_idx]
        } else if byte2 == 0xFF {
            assert!(sym_idx < X11_MISC_FUNCTION_KEYS_MAPPING_TABLE.len());
            X11_MISC_FUNCTION_KEYS_MAPPING_TABLE[sym_idx]
        } else {
            KeySymbol::Unknown
        }
    }

    const X11_LATIN1_KEYS_MAPPING_TABLE: [KeySymbol; 256] = [
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
        KeySymbol::Space,
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

    const X11_MISC_FUNCTION_KEYS_MAPPING_TABLE: [KeySymbol; 256] = [
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
        KeySymbol::Clear,
        KeySymbol::Unknown,
        KeySymbol::Enter,
        KeySymbol::Unknown,
        KeySymbol::Unknown,
        KeySymbol::Unknown,
        KeySymbol::Unknown,
        KeySymbol::Unknown,
        KeySymbol::Pause,
        KeySymbol::ScrolLock,
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
        KeySymbol::Home,
        KeySymbol::Left,
        KeySymbol::Up,
        KeySymbol::Right,
        KeySymbol::Down,
        KeySymbol::PageUp,
        KeySymbol::PageDown,
        KeySymbol::End,
        KeySymbol::Unknown,
        KeySymbol::Unknown,
        KeySymbol::Unknown,
        KeySymbol::Unknown,
        KeySymbol::Unknown,
        KeySymbol::Unknown,
        KeySymbol::Unknown,
        KeySymbol::Unknown,
        KeySymbol::Select,
        KeySymbol::PrintScreen,
        KeySymbol::Unknown,
        KeySymbol::Insert,
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
        KeySymbol::KpMultiply,
        KeySymbol::KpAdd,
        KeySymbol::Unknown,
        KeySymbol::KpMinus,
        KeySymbol::Unknown,
        KeySymbol::KpDivide,
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
        KeySymbol::Unknown,
        KeySymbol::Unknown,
        KeySymbol::Unknown,
        KeySymbol::Unknown,
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
        KeySymbol::Unknown,
        KeySymbol::Unknown,
        KeySymbol::LeftShift,
        KeySymbol::RightShift,
        KeySymbol::LeftControl,
        KeySymbol::RightControl,
        KeySymbol::Unknown,
        KeySymbol::Unknown,
        KeySymbol::LeftMenu,
        KeySymbol::RightMenu,
        KeySymbol::LeftAlt,
        KeySymbol::RightAlt,
        KeySymbol::LeftWin,
        KeySymbol::RightWin,
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
        KeySymbol::Del,
    ];
}
