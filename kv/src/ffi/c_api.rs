use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;
use std::slice;

use crate::db::{Db, DbIterator};

pub struct KeyliteDb {
    inner: Db,
}

pub struct KeyliteIterator {
    inner: DbIterator,
}

#[repr(C)]
pub enum KeyliteResult {
    Ok = 0,
    ErrNull = 1,
    ErrIo = 2,
    ErrUtf8 = 3,
    ErrOther = 4,
}

// TODO: handle the multi thread stuff
#[no_mangle]
pub unsafe extern "C" fn keylite_open(
    path: *const c_char,
    db_out: *mut *mut KeyliteDb,
) -> KeyliteResult {
    if path.is_null() || db_out.is_null() {
        return KeyliteResult::ErrNull;
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return KeyliteResult::ErrUtf8,
    };

    match Db::open(path_str) {
        Ok(db) => {
            let boxed = Box::new(KeyliteDb { inner: db });
            *db_out = Box::into_raw(boxed);
            KeyliteResult::Ok
        }
        Err(e) => {
            eprintln!("keylite_open error: {}", e);
            KeyliteResult::ErrIo
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn keylite_close(db: *mut KeyliteDb) {
    if !db.is_null() {
        let _ = Box::from_raw(db);
    }
}

#[no_mangle]
pub unsafe extern "C" fn keylite_put(
    db: *mut KeyliteDb,
    key: *const u8,
    key_len: usize,
    val: *const u8,
    val_len: usize,
) -> KeyliteResult {
    if db.is_null() || key.is_null() || val.is_null() {
        return KeyliteResult::ErrNull;
    }

    let db = &(*db).inner;
    let key_slice = slice::from_raw_parts(key, key_len);
    let val_slice = slice::from_raw_parts(val, val_len);

    match db.put(key_slice, val_slice) {
        Ok(_) => KeyliteResult::Ok,
        Err(e) => {
            eprintln!("keylite_put error: {}", e);
            KeyliteResult::ErrOther
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn keylite_get(
    db: *mut KeyliteDb,
    key: *const u8,
    key_len: usize,
    val_out: *mut *mut u8,
    val_len_out: *mut usize,
) -> KeyliteResult {
    if db.is_null() || key.is_null() || val_out.is_null() || val_len_out.is_null() {
        return KeyliteResult::ErrNull;
    }

    let db = &(*db).inner;
    let key_slice = slice::from_raw_parts(key, key_len);

    match db.get(key_slice) {
        Some(val) => {
            let len = val.len();
            let mut boxed = val.into_boxed_slice();
            *val_out = boxed.as_mut_ptr();
            *val_len_out = len;
            std::mem::forget(boxed);
            KeyliteResult::Ok
        }
        None => {
            *val_out = ptr::null_mut();
            *val_len_out = 0;
            KeyliteResult::Ok
        } // Err(e) => {
          //     eprintln!("keylite_get error: {}", e);
          //     KeyliteResult::ErrOther
          // }
    }
}

// TODO: should be an internal method and not to be revealed in the api
#[no_mangle]
pub unsafe extern "C" fn keylite_free_value(val: *mut u8, len: usize) {
    if !val.is_null() && len > 0 {
        let _ = Box::from_raw(slice::from_raw_parts_mut(val, len));
    }
}

#[no_mangle]
pub unsafe extern "C" fn keylite_del(
    db: *mut KeyliteDb,
    key: *const u8,
    key_len: usize,
) -> KeyliteResult {
    if db.is_null() || key.is_null() {
        return KeyliteResult::ErrNull;
    }

    let db = &(*db).inner;
    let key_slice = slice::from_raw_parts(key, key_len);

    match db.del(key_slice) {
        Ok(_) => KeyliteResult::Ok,
        Err(e) => {
            eprintln!("keylite_del error: {}", e);
            KeyliteResult::ErrOther
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn keylite_put_str(
    db: *mut KeyliteDb,
    key: *const c_char,
    val: *const c_char,
) -> KeyliteResult {
    if db.is_null() || key.is_null() || val.is_null() {
        return KeyliteResult::ErrNull;
    }

    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => return KeyliteResult::ErrUtf8,
    };

    let val_str = match CStr::from_ptr(val).to_str() {
        Ok(s) => s,
        Err(_) => return KeyliteResult::ErrUtf8,
    };

    let db = &(*db).inner;

    match db.put(key_str.as_bytes(), val_str.as_bytes()) {
        Ok(_) => KeyliteResult::Ok,
        Err(e) => {
            eprintln!("keylite_put_str error: {}", e);
            KeyliteResult::ErrOther
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn keylite_get_str(
    db: *mut KeyliteDb,
    key: *const c_char,
    val_out: *mut *mut c_char,
) -> KeyliteResult {
    if db.is_null() || key.is_null() || val_out.is_null() {
        return KeyliteResult::ErrNull;
    }

    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => return KeyliteResult::ErrUtf8,
    };

    let db = &(*db).inner;

    match db.get(key_str.as_bytes()) {
        Some(val) => match std::str::from_utf8(&val) {
            Ok(s) => {
                let c_string = match std::ffi::CString::new(s) {
                    Ok(cs) => cs,
                    Err(_) => {
                        *val_out = ptr::null_mut();
                        return KeyliteResult::ErrUtf8;
                    }
                };
                *val_out = c_string.into_raw();
                KeyliteResult::Ok
            }
            Err(_) => {
                *val_out = ptr::null_mut();
                KeyliteResult::ErrUtf8
            }
        },
        None => {
            *val_out = ptr::null_mut();
            KeyliteResult::Ok
        } // Err(e) => {
          //     eprintln!("keylite_get_str error: {}", e);
          //     KeyliteResult::ErrOther
          // }
    }
}

#[no_mangle]
pub unsafe extern "C" fn keylite_free_str(val: *mut c_char) {
    if !val.is_null() {
        let _ = std::ffi::CString::from_raw(val);
    }
}

#[no_mangle]
pub unsafe extern "C" fn keylite_del_str(db: *mut KeyliteDb, key: *const c_char) -> KeyliteResult {
    if db.is_null() || key.is_null() {
        return KeyliteResult::ErrNull;
    }

    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => return KeyliteResult::ErrUtf8,
    };

    let db = &(*db).inner;

    match db.del(key_str.as_bytes()) {
        Ok(_) => KeyliteResult::Ok,
        Err(e) => {
            eprintln!("keylite_del_str error: {}", e);
            KeyliteResult::ErrOther
        }
    }
}

// Scan operations
#[no_mangle]
pub unsafe extern "C" fn keylite_scan(
    db: *mut KeyliteDb,
    start: *const u8,
    start_len: usize,
    end: *const u8,
    end_len: usize,
    iter_out: *mut *mut KeyliteIterator,
) -> KeyliteResult {
    if db.is_null() || iter_out.is_null() {
        return KeyliteResult::ErrNull;
    }

    let db = &(*db).inner;

    let start_slice = if start.is_null() {
        None
    } else {
        Some(slice::from_raw_parts(start, start_len))
    };

    let end_slice = if end.is_null() {
        None
    } else {
        Some(slice::from_raw_parts(end, end_len))
    };

    let iter = db.scan(start_slice, end_slice);
    let boxed = Box::new(KeyliteIterator { inner: iter });
    *iter_out = Box::into_raw(boxed);
    KeyliteResult::Ok
}

#[no_mangle]
pub unsafe extern "C" fn keylite_scan_str(
    db: *mut KeyliteDb,
    start: *const c_char,
    end: *const c_char,
    iter_out: *mut *mut KeyliteIterator,
) -> KeyliteResult {
    if db.is_null() || iter_out.is_null() {
        return KeyliteResult::ErrNull;
    }

    let db = &(*db).inner;

    let start_slice = if start.is_null() {
        None
    } else {
        match CStr::from_ptr(start).to_str() {
            Ok(s) => Some(s.as_bytes()),
            Err(_) => return KeyliteResult::ErrUtf8,
        }
    };

    let end_slice = if end.is_null() {
        None
    } else {
        match CStr::from_ptr(end).to_str() {
            Ok(s) => Some(s.as_bytes()),
            Err(_) => return KeyliteResult::ErrUtf8,
        }
    };

    let iter = db.scan(start_slice, end_slice);
    let boxed = Box::new(KeyliteIterator { inner: iter });
    *iter_out = Box::into_raw(boxed);
    KeyliteResult::Ok
}

#[no_mangle]
pub unsafe extern "C" fn keylite_iter_next(
    iter: *mut KeyliteIterator,
    key_out: *mut *mut u8,
    key_len_out: *mut usize,
    val_out: *mut *mut u8,
    val_len_out: *mut usize,
) -> KeyliteResult {
    if iter.is_null() || key_out.is_null() || key_len_out.is_null() 
        || val_out.is_null() || val_len_out.is_null() {
        return KeyliteResult::ErrNull;
    }

    let iter = &mut (*iter).inner;

    match iter.next() {
        Some((key, val)) => {
            let key_len = key.len();
            let val_len = val.len();
            
            let mut key_boxed = key.into_boxed_slice();
            let mut val_boxed = val.into_boxed_slice();
            
            *key_out = key_boxed.as_mut_ptr();
            *key_len_out = key_len;
            *val_out = val_boxed.as_mut_ptr();
            *val_len_out = val_len;
            
            std::mem::forget(key_boxed);
            std::mem::forget(val_boxed);
            
            KeyliteResult::Ok
        }
        None => {
            *key_out = ptr::null_mut();
            *key_len_out = 0;
            *val_out = ptr::null_mut();
            *val_len_out = 0;
            KeyliteResult::Ok
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn keylite_iter_free(iter: *mut KeyliteIterator) {
    if !iter.is_null() {
        let _ = Box::from_raw(iter);
    }
}
