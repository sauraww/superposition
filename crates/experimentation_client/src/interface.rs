use std::{
    ffi::{c_char, c_ulong, CStr},
    sync::Arc,
};

use crate::{Client, CLIENT_FACTORY};
use serde_json::Value;
use std::{
    cell::RefCell,
    ffi::{c_int, c_short, CString},
};
use tokio::{runtime::Runtime, task};

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

fn to_string<E>(e: E) -> String
where
    E: ToString,
{
    e.to_string()
}

fn error_block<E>(err: String) -> *mut E {
    update_last_error(err);
    std::ptr::null_mut()
}

fn cstring_to_rstring(s: *const c_char) -> Result<String, String> {
    let s = unsafe { CStr::from_ptr(s) };
    s.to_str().map(str::to_string).map_err(to_string)
}

fn rstring_to_cstring(s: String) -> CString {
    CString::new(s.as_str()).unwrap_or_default()
}

pub fn update_last_error(err: String) {
    println!("Setting LAST_ERROR: {}", err);

    LAST_ERROR.with(|prev| {
        *prev.borrow_mut() = Some(err);
    });
}

pub fn take_last_error() -> Option<String> {
    LAST_ERROR.with(|prev| prev.take())
}

#[no_mangle]
pub extern "C" fn last_error_length() -> c_int {
    LAST_ERROR.with(|prev| match *prev.borrow() {
        Some(ref err) => err.to_string().len() as c_int + 1,
        None => 0,
    })
}

#[no_mangle]
pub unsafe extern "C" fn last_error_message() -> *const c_char {
    let last_error = match take_last_error() {
        Some(err) => err,
        None => return std::ptr::null_mut(),
    };
    let error_message = last_error.to_string();
    // println!("Error in last_error_message {error_message}");
    let err = rstring_to_cstring(error_message);
    err.into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s);
    }
}

#[no_mangle]
pub extern "C" fn new_client(
    tenant: *const c_char,
    update_frequency: c_ulong,
    hostname: *const c_char,
) -> c_int {
    let tenant = match cstring_to_rstring(tenant) {
        Ok(value) => value,
        Err(err) => {
            update_last_error(err);
            return 1;
        }
    };
    let hostname = match cstring_to_rstring(hostname) {
        Ok(value) => value,
        Err(err) => {
            update_last_error(err);
            return 1;
        }
    };

    // println!("Creating cac client thread for tenant {tenant}");
    let local = task::LocalSet::new();
    local.block_on(&Runtime::new().unwrap(), async move {
        match CLIENT_FACTORY
            .create_client(tenant.clone(), update_frequency, hostname)
            .await
        {
            Ok(_) => 0,
            Err(err) => {
                update_last_error(err);
                1
            }
        }
    });
    0
}

#[no_mangle]
pub extern "C" fn start_polling_update(tenant: *const c_char) {
    if tenant.is_null() {
        return ();
    }
    unsafe {
        let client = get_client(tenant);
        let local = task::LocalSet::new();
        // println!("in FFI polling");
        local.block_on(
            &Runtime::new().unwrap(),
            (*client).clone().run_polling_updates(),
        );
    }
}

#[no_mangle]
pub extern "C" fn free_client(ptr: *mut Arc<Client>) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn get_client(tenant: *const c_char) -> *mut Arc<Client> {
    let ten = match cstring_to_rstring(tenant) {
        Ok(t) => t,
        Err(err) => {
            update_last_error(err);
            return std::ptr::null_mut();
        }
    };
    let local = task::LocalSet::new();
    local.block_on(
        &Runtime::new().unwrap(),
        // println!("fetching exp client thread for tenant {ten}");
        async move {
            match CLIENT_FACTORY.get_client(ten).await {
                Ok(client) => Box::into_raw(Box::new(client)),
                Err(err) => {
                    // println!("error occurred {err}");
                    update_last_error(err);
                    // println!("error set");
                    std::ptr::null_mut()
                }
            }
        },
    )
}

#[no_mangle]
pub extern "C" fn get_applicable_variant(
    client: *mut Arc<Client>,
    c_context: *const c_char,
    toss: c_short,
) -> *mut c_char {
    let context = match cstring_to_rstring(c_context) {
        Ok(c) => match serde_json::from_str::<Value>(c.as_str()) {
            Ok(con) => con,
            Err(err) => return error_block(err.to_string()),
        },
        Err(err) => return error_block(err),
    };
    // println!("Fetching variantIds");
    let local = task::LocalSet::new();
    let variants = local.block_on(&Runtime::new().unwrap(), unsafe {
        (*client).get_applicable_variant(&context, toss as i8)
    });
    // println!("variantIds: {:?}", variants);
    match serde_json::to_string::<Vec<String>>(&variants) {
        Ok(result) => rstring_to_cstring(result).into_raw(),
        Err(err) => error_block(err.to_string()),
    }
}

#[no_mangle]
pub extern "C" fn get_satisfied_experiments(
    client: *mut Arc<Client>,
    c_context: *const c_char,
) -> *mut c_char {
    let context = match cstring_to_rstring(c_context) {
        Ok(c) => match serde_json::from_str::<Value>(c.as_str()) {
            Ok(con) => con,
            Err(err) => return error_block(err.to_string()),
        },
        Err(err) => return error_block(err),
    };

    let local = task::LocalSet::new();
    let experiments = local.block_on(&Runtime::new().unwrap(), unsafe {
        (*client).get_satisfied_experiments(&context)
    });
    let experiments = match serde_json::to_value(experiments) {
        Ok(value) => value,
        Err(err) => return error_block(err.to_string()),
    };
    match serde_json::to_string(&experiments) {
        Ok(result) => rstring_to_cstring(result).into_raw(),
        Err(err) => error_block(err.to_string()),
    }
}

#[no_mangle]
pub extern "C" fn get_running_experiments(client: *mut Arc<Client>) -> *mut c_char {
    let local = task::LocalSet::new();
    let experiments = local.block_on(&Runtime::new().unwrap(), unsafe {
        (*client).get_running_experiments()
    });
    let experiments = match serde_json::to_value(experiments) {
        Ok(value) => value,
        Err(err) => return error_block(err.to_string()),
    };
    match serde_json::to_string(&experiments) {
        Ok(result) => rstring_to_cstring(result).into_raw(),
        Err(err) => error_block(err.to_string()),
    }
}
