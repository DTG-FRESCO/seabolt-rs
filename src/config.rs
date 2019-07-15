use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    ops::Deref,
    ptr::NonNull,
    slice,
};

make_enum!(Scheme,
    Direct => seabolt_sys::BOLT_SCHEME_DIRECT,
    Routing => seabolt_sys::BOLT_SCHEME_ROUTING,
    Neo4j => seabolt_sys::BOLT_SCHEME_NEO4J,
);

make_enum!(Transport,
    Plaintext => seabolt_sys::BOLT_TRANSPORT_PLAINTEXT,
    Encrypted => seabolt_sys::BOLT_TRANSPORT_ENCRYPTED,
);

pub trait NTTWrap
where
    Self: Sized,
    Self::ptr: Sized,
{
    type ptr;
}

pub struct NTTWrapper<'a, T: NTTWrap> {
    ptr: *mut T,
    a: PhantomData<&'a T>,
}

impl<'a, T: NTTWrap> NTTWrapper<'a, T>{
    pub fn new(ptr: NonNull<T::ptr>) -> Self {
        assert_eq!(std::mem::size_of::<T>(),
                   std::mem::size_of::<*mut T::ptr>());
        
        let ptr = Box::into_raw(Box::new(ptr.as_ptr())) as *mut T;
        NTTWrapper {
            ptr,
            a: PhantomData,
        }
    }
}

impl<'a, T: NTTWrap> Deref for NTTWrapper<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<'a, T: NTTWrap> Drop for NTTWrapper<'a, T> {
    fn drop(&mut self) {
        unsafe{ Box::from_raw(self.ptr as *mut *mut T::ptr) };
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Trust {
    ptr: *mut seabolt_sys::BoltTrust,
}

impl Trust {
    pub fn build() -> TrustBuilder {
        let ptr = unsafe { seabolt_sys::BoltTrust_create() };
        TrustBuilder {
            inner: Trust { ptr },
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut seabolt_sys::BoltTrust {
        self.ptr
    }

    pub fn certs(&self) -> Option<&[u8]> {
        let mut size = 0_u64;
        let ptr = unsafe { seabolt_sys::BoltTrust_get_certs(self.ptr, &mut size as *mut u64) };
        if size == 0 {
            None
        } else {
            Some(unsafe { slice::from_raw_parts(ptr as *const u8, size as usize) })
        }
    }

    pub fn verification(&self) -> bool {
        unsafe { seabolt_sys::BoltTrust_get_skip_verify(self.ptr) == 1 }
    }

    pub fn verify_hostname(&self) -> bool {
        unsafe { seabolt_sys::BoltTrust_get_skip_verify_hostname(self.ptr) == 1 }
    }
}

impl Drop for Trust {
    fn drop(&mut self) {
        unsafe { seabolt_sys::BoltTrust_destroy(self.ptr) }
    }
}

impl NTTWrap for Trust {
    type ptr = seabolt_sys::BoltTrust;
}

#[derive(Debug)]
pub struct TrustBuilder {
    inner: Trust,
}

impl TrustBuilder {
    pub fn finish(self) -> Trust {
        self.inner
    }

    pub fn with_certs(self, certs: &[u8]) -> Self {
        unsafe {
            seabolt_sys::BoltTrust_set_certs(
                self.inner.as_ptr(),
                certs.as_ptr() as *const i8,
                certs.len() as u64,
            );
        }
        self
    }

    pub fn verification(self, verify: bool) -> Self {
        unsafe {
            seabolt_sys::BoltTrust_set_skip_verify(self.inner.as_ptr(), if verify { 1 } else { 0 });
        }
        self
    }

    pub fn verify_hostname(self, verify: bool) -> Self {
        unsafe {
            seabolt_sys::BoltTrust_set_skip_verify_hostname(
                self.inner.as_ptr(),
                if verify { 1 } else { 0 },
            );
        }
        self
    }
}

#[derive(Debug)]
pub struct Config {
    ptr: *mut seabolt_sys::BoltConfig,
}

impl Config {
    pub fn build() -> ConfigBuilder {
        let ptr = unsafe { seabolt_sys::BoltConfig_create() };
        ConfigBuilder {
            inner: Config { ptr },
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut seabolt_sys::BoltConfig {
        self.ptr
    }

    pub fn get_scheme(&self) -> Scheme {
        Scheme::from_idx(unsafe { seabolt_sys::BoltConfig_get_scheme(self.ptr) as u32 })
    }

    pub fn get_transport(&self) -> Transport {
        Transport::from_idx(unsafe { seabolt_sys::BoltConfig_get_transport(self.ptr) as u32 })
    }

    pub fn get_trust(&self) -> Option<NTTWrapper<Trust>> {
        let ptr = unsafe { seabolt_sys::BoltConfig_get_trust(self.ptr) };
        NonNull::new(ptr).map(NTTWrapper::new)
    }

    pub fn get_user_agent(&self) -> Option<&str> {
        let ptr = unsafe { seabolt_sys::BoltConfig_get_user_agent(self.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) }.to_str().unwrap())
        }
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        unsafe { seabolt_sys::BoltConfig_destroy(self.ptr) }
    }
}

#[derive(Debug)]
pub struct ConfigBuilder {
    inner: Config,
}

impl ConfigBuilder {
    pub fn finish(self) -> Config {
        self.inner
    }

    pub fn with_scheme(self, scheme: Scheme) -> Self {
        unsafe {
            seabolt_sys::BoltConfig_set_scheme(self.inner.as_ptr(), scheme.as_idx() as i32);
        }
        self
    }

    pub fn with_transport(self, transport: Transport) -> Self {
        unsafe {
            seabolt_sys::BoltConfig_set_transport(self.inner.as_ptr(), transport.as_idx() as i32);
        }
        self
    }

    pub fn with_trust(self, trust: Trust) -> Self {
        unsafe {
            seabolt_sys::BoltConfig_set_trust(self.inner.as_ptr(), trust.as_ptr());
        }
        self
    }

    pub fn with_user_agent(self, user_agent: &str) -> Self {
        let user_agent = CString::new(user_agent).unwrap();
        unsafe {
            seabolt_sys::BoltConfig_set_user_agent(self.inner.as_ptr(), user_agent.as_ptr());
        }
        self
    }
}
