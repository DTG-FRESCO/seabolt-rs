use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    slice,
};

make_enum!(ValueType,
    Null => seabolt_sys::BoltType::BOLT_NULL,
    Boolean => seabolt_sys::BoltType::BOLT_BOOLEAN,
    Integer => seabolt_sys::BoltType::BOLT_INTEGER,
    Float => seabolt_sys::BoltType::BOLT_FLOAT,
    String => seabolt_sys::BoltType::BOLT_STRING,
    Dictionary => seabolt_sys::BoltType::BOLT_DICTIONARY,
    List => seabolt_sys::BoltType::BOLT_LIST,
    Bytes => seabolt_sys::BoltType::BOLT_BYTES,
    Structure => seabolt_sys::BoltType::BOLT_STRUCTURE,
);

#[derive(Debug)]
pub struct Structure {
    pub code: i16,
    pub fields: Vec<Value>,
}

#[derive(Debug)]
pub struct Value {
    ptr: *mut seabolt_sys::BoltValue,
}

impl Value {
    pub(crate) fn new() -> Self {
        let ptr = unsafe { seabolt_sys::BoltValue_create() };
        Value { ptr }
    }

    pub(crate) unsafe fn from_ptr(p: *mut seabolt_sys::BoltValue) -> Self {
        Value { ptr: p }
    }

    pub(crate) fn as_ptr(&self) -> *mut seabolt_sys::BoltValue {
        self.ptr
    }

    pub fn get_type(&self) -> ValueType {
        ValueType::from_idx(unsafe { seabolt_sys::BoltValue_type(self.ptr) })
    }

    // Null
    pub fn null(&mut self) {
        unsafe {
            seabolt_sys::BoltValue_format_as_Null(self.ptr);
        }
    }

    pub fn from_null() -> Self {
        let mut tmp = Value::new();
        tmp.null();
        tmp
    }

    // Boolean
    pub fn boolean(&mut self, v: bool) {
        unsafe {
            seabolt_sys::BoltValue_format_as_Boolean(self.ptr, if v { 1 } else { 0 });
        }
    }

    pub fn as_boolean(&self) -> bool {
        assert_eq!(self.get_type(), ValueType::Boolean);
        unsafe { seabolt_sys::BoltBoolean_get(self.ptr) == 1 }
    }

    pub fn from_boolean(v: bool) -> Self {
        let mut tmp = Value::new();
        tmp.boolean(v);
        tmp
    }

    // Integer
    pub fn into_integer<T: Into<i64>>(self, v: T) -> Self {
        unsafe {
            seabolt_sys::BoltValue_format_as_Integer(self.ptr, v.into());
        }
        self
    }

    pub fn as_integer(&self) -> i64 {
        assert_eq!(self.get_type(), ValueType::Integer);
        unsafe { seabolt_sys::BoltInteger_get(self.ptr) }
    }

    pub fn from_integer<T: Into<i64>>(v: T) -> Self {
        Value::new().into_integer(v)
    }

    // Float
    pub fn into_float<T: Into<f64>>(self, v: T) -> Self {
        unsafe {
            seabolt_sys::BoltValue_format_as_Float(self.ptr, v.into());
        }
        self
    }

    pub fn as_float(&self) -> f64 {
        assert_eq!(self.get_type(), ValueType::Float);
        unsafe { seabolt_sys::BoltFloat_get(self.ptr) }
    }

    pub fn from_float<T: Into<f64>>(v: T) -> Self {
        Value::new().into_float(v)
    }

    // String
    pub fn into_string<T: ToString>(self, v: T) -> Self {
        let s = CString::new(v.to_string()).unwrap();
        unsafe {
            seabolt_sys::BoltValue_format_as_String(
                self.ptr,
                s.as_ptr(),
                s.to_bytes_with_nul().len() as i32,
            );
        }
        self
    }

    pub fn as_string(&self) -> &str {
        assert_eq!(self.get_type(), ValueType::String);
        unsafe {
            CStr::from_ptr(seabolt_sys::BoltString_get(self.ptr))
                .to_str()
                .unwrap()
        }
    }

    pub fn from_string<T: ToString>(v: T) -> Self {
        Value::new().into_string(v)
    }

    // Dict
    pub fn into_dict<T: IntoIterator<Item = (String, Value)>>(self, v: T) -> Self {
        let dict = v.into_iter().collect::<HashMap<_, _>>();
        unsafe {
            seabolt_sys::BoltValue_format_as_Dictionary(self.ptr, dict.len() as i32);
        }
        for (i, (k, v)) in dict.into_iter().enumerate() {
            let s = CString::new(k).unwrap();
            unsafe {
                seabolt_sys::BoltDictionary_set_key(
                    self.ptr,
                    i as i32,
                    s.as_ptr(),
                    s.as_bytes_with_nul().len() as i32,
                );
            }
            let p = unsafe { seabolt_sys::BoltDictionary_value(self.ptr, i as i32) };
            unsafe { seabolt_sys::BoltValue_copy(v.ptr, p) };
        }
        self
    }

    pub fn as_dict(&self) -> HashMap<String, Value> {
        assert_eq!(self.get_type(), ValueType::Dictionary);
        let size = unsafe { seabolt_sys::BoltValue_size(self.ptr) };
        let mut dict: HashMap<String, Value> = HashMap::with_capacity(size as usize);
        for i in 0..size {
            let k = unsafe {
                CStr::from_ptr(seabolt_sys::BoltDictionary_get_key(self.ptr, i))
                    .to_str()
                    .unwrap()
            };
            let v = unsafe { Value::from_ptr(seabolt_sys::BoltDictionary_value(self.ptr, i)) };
            dict.insert(k.to_string(), v);
        }
        dict
    }

    pub fn from_dict<T: IntoIterator<Item = (String, Value)>>(v: T) -> Self {
        Value::new().into_dict(v)
    }

    // List
    pub fn into_list<T: IntoIterator<Item = Value>>(self, v: T) -> Self {
        let vec = v.into_iter().collect::<Vec<_>>();
        unsafe {
            seabolt_sys::BoltValue_format_as_List(self.ptr, vec.len() as i32);
        }
        for (i, v) in vec.into_iter().enumerate() {
            let p = unsafe { seabolt_sys::BoltList_value(self.ptr, i as i32) };
            unsafe { seabolt_sys::BoltValue_copy(v.ptr, p) };
        }
        self
    }

    pub fn as_list(&self) -> Vec<Value> {
        assert_eq!(self.get_type(), ValueType::List);
        let size = unsafe { seabolt_sys::BoltValue_size(self.ptr) };
        let mut vec: Vec<Value> = Vec::with_capacity(size as usize);
        for i in 0..size {
            let v = unsafe { Value::from_ptr(seabolt_sys::BoltList_value(self.ptr, i)) };
            vec.push(v);
        }
        vec
    }

    pub fn from_list<T: IntoIterator<Item = Value>>(v: T) -> Self {
        Value::new().into_list(v)
    }

    // Bytes
    pub fn into_bytes(self, v: &mut [u8]) -> Self {
        unsafe {
            seabolt_sys::BoltValue_format_as_Bytes(
                self.ptr,
                v.as_mut_ptr() as *mut i8,
                v.len() as i32,
            );
        }
        self
    }

    pub fn as_bytes(&self) -> &[u8] {
        assert_eq!(self.get_type(), ValueType::Bytes);
        let size = unsafe { seabolt_sys::BoltValue_size(self.ptr) as usize };

        unsafe { slice::from_raw_parts(seabolt_sys::BoltBytes_get_all(self.ptr) as *mut u8, size) }
    }

    pub fn from_bytes(v: &mut [u8]) -> Self {
        Value::new().into_bytes(v)
    }

    // Structure
    pub fn into_structure(self, code: i16, fields: Vec<Value>) -> Self {
        unsafe {
            seabolt_sys::BoltValue_format_as_Structure(self.ptr, code, fields.len() as i32);
        }

        for (i, v) in fields.into_iter().enumerate() {
            let p = unsafe { seabolt_sys::BoltStructure_value(self.ptr, i as i32) };
            unsafe { seabolt_sys::BoltValue_copy(v.ptr, p) };
        }

        self
    }

    pub fn as_structure(&self) -> Structure {
        assert_eq!(self.get_type(), ValueType::Structure);
        let size = unsafe { seabolt_sys::BoltValue_size(self.ptr) };

        let code = unsafe { seabolt_sys::BoltStructure_code(self.ptr) };
        let mut fields = Vec::with_capacity(size as usize);

        for i in 0..size {
            let v = unsafe { Value::from_ptr(seabolt_sys::BoltStructure_value(self.ptr, i)) };
            fields.push(v);
        }

        Structure { code, fields }
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        unsafe {
            seabolt_sys::BoltValue_destroy(self.ptr);
        }
    }
}
