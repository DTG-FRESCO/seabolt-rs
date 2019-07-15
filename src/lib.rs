use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    ptr,
    sync::Mutex,
};

use lazy_static::lazy_static;
use seabolt_sys;

macro_rules! make_enum {
    ($name:ident,
     $($variant:ident => $constant:path),+ $(,)?) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq)]
        pub enum $name {
            $($variant,)+
            Unknown,
        }

        impl $name {
            fn from_idx(t: u32) -> Self {
                match t {
                    $($constant => $name::$variant,)+
                    _ => $name::Unknown,
                }
            }

            fn as_idx(&self) -> u32 {
                match self {
                    $($name::$variant => $constant,)+
                    $name::Unknown => unimplemented!(),
                }
            }
        }
     };
}

pub mod config;
mod value;
pub use config::Config;
pub use value::{Value, ValueType};

#[derive(Debug)]
pub struct Bolt;

impl Bolt {
    pub fn init() -> Option<Self> {
        lazy_static! {
            static ref SINGLE: Mutex<Option<Bolt>> = Mutex::new(Some(Bolt));
        };
        let init = SINGLE.lock().unwrap().take();
        if init.is_some() {
            unsafe {
                seabolt_sys::Bolt_startup();
            }
        }
        init
    }

    pub fn create_connector(&self, addr: &Address, auth: &Auth, config: &Config) -> Connector {
        Connector::new(addr, auth, config)
    }
}

impl Drop for Bolt {
    fn drop(&mut self) {
        unsafe {
            seabolt_sys::Bolt_shutdown();
        }
    }
}

#[derive(Debug)]
pub struct Address {
    ptr: *mut seabolt_sys::BoltAddress,
}

impl Address {
    pub fn new(addr: &str, port: &str) -> Self {
        let addr = CString::new(addr).unwrap();
        let port = CString::new(port).unwrap();

        let ptr = unsafe { seabolt_sys::BoltAddress_create(addr.as_ptr(), port.as_ptr()) };

        if ptr.is_null() {
            panic!()
        } else {
            Address { ptr }
        }
    }

    pub fn get_host(&self) -> &str {
        let s = unsafe { CStr::from_ptr(seabolt_sys::BoltAddress_host(self.ptr)) };
        s.to_str().unwrap()
    }

    pub fn get_port(&self) -> &str {
        let s = unsafe { CStr::from_ptr(seabolt_sys::BoltAddress_port(self.ptr)) };
        s.to_str().unwrap()
    }

    fn as_ptr(&self) -> *mut seabolt_sys::BoltAddress {
        self.ptr
    }
}

impl Drop for Address {
    fn drop(&mut self) {
        unsafe { seabolt_sys::BoltAddress_destroy(self.ptr) }
    }
}

#[derive(Debug)]
pub struct Connector<'a> {
    ptr: *mut seabolt_sys::BoltConnector,
    virt: PhantomData<&'a Bolt>,
}

impl<'a> Connector<'a> {
    fn new(addr: &Address, auth: &Auth, config: &Config) -> Self {
        let ptr = unsafe {
            seabolt_sys::BoltConnector_create(addr.as_ptr(), auth.as_ptr(), config.as_ptr())
        };
        Connector {
            ptr,
            virt: PhantomData,
        }
    }

    pub fn acquire() {}
}

impl<'a> Drop for Connector<'a> {
    fn drop(&mut self) {
        unsafe {
            seabolt_sys::BoltConnector_destroy(self.ptr);
        }
    }
}

pub struct Auth(Value);

impl Auth {
    pub(crate) fn as_ptr(&self) -> *mut seabolt_sys::BoltValue {
        self.0.as_ptr()
    }
}

pub fn basic_auth(username: &str, password: &str, realm: Option<&str>) -> Auth {
    let username = CString::new(username).unwrap();
    let password = CString::new(password).unwrap();
    let realm = realm.map(|v| CString::new(v).unwrap());
    let realm_ptr = if let Some(s) = realm {
        s.as_ptr()
    } else {
        ptr::null()
    };
    Auth(unsafe {
        Value::from_ptr(seabolt_sys::BoltAuth_basic(
            username.as_ptr(),
            password.as_ptr(),
            realm_ptr,
        ))
    })
}
