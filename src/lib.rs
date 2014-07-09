#![crate_name = "rust_hl_lua"]
#![crate_type = "lib"]
#![comment = "Lua bindings for Rust"]
#![license = "MIT"]
#![allow(visible_private_types)]
#![feature(macro_rules)]

extern crate libc;

use std::kinds::marker::ContravariantLifetime;

pub use lua_tables::LuaTable;

pub mod functions_read;
pub mod lua_tables;
pub mod userdata;

mod ffi;
mod functions_write;
mod rust_tables;
mod tuples;
mod values;


/**
 * Main object of the library
 * The lifetime parameter corresponds to the lifetime of the Lua object itself
 */
pub struct Lua<'lua> {
    lua: *mut ffi::lua_State,
    marker: ContravariantLifetime<'lua>,
    must_be_closed: bool,
    inside_callback: bool           // if true, we are inside a callback
}

/**
 * Object which allows access to a Lua variable
 */
pub struct LoadedVariable<'var, 'lua> {
    lua: &'var mut Lua<'lua>,
    size: uint       // number of elements at the top of the stack
}

/**
 * Should be implemented by whatever type is pushable on the Lua stack
 */
pub trait Pushable<'lua>: ::std::any::Any {
    /**
     * Pushes the value on the top of the stack
     * Must return the number of elements pushed
     */
    fn push_to_lua(self, lua: &mut Lua<'lua>) -> uint {
        self.push_as_userdata(lua)
    }

    /**
     * Pushes the value on the top of the stack as a user data
     * This means that the value will only be readable as itself and not convertible to anything
     * Also, objects pushed as userdata will not be cloned by Lua
     */
    fn push_as_userdata(self, lua: &mut Lua<'lua>) -> uint {
        userdata::push_userdata(self, lua)
    }

    /**
     * Pushes over another element
     */
    fn push_over<'var>(self, mut over: LoadedVariable<'var, 'lua>)
        -> (LoadedVariable<'var, 'lua>, uint)
    {
        let val = self.push_to_lua(over.lua);
        over.size += val;
        (over, val)
    }
}

/**
 * Should be implemented by types that can be read by consomming a LoadedVariable
 */
pub trait ConsumeReadable<'a, 'lua> {
    /**
     * Returns the LoadedVariable in case of failure
     */
    fn read_from_variable(var: LoadedVariable<'a, 'lua>) -> Result<Self, LoadedVariable<'a, 'lua>>;
}

/**
 * Should be implemented by whatever type can be read by copy from the Lua stack
 */
pub trait CopyReadable : Clone + ::std::any::Any {
    /**
     * # Arguments
     *  * `lua` - The Lua object to read from
     *  * `index` - The index on the stack to read from
     */
    fn read_from_lua<'lua>(lua: &mut Lua<'lua>, index: i32) -> Option<Self> {
        userdata::read_copy_userdata(lua, index)
    }
}

/**
 * Types that can be indices in Lua tables
 */
pub trait Index<'lua>: Pushable<'lua> + CopyReadable {
}

/**
 * Error that can happen when executing Lua code
 */
#[deriving(Show)]
pub enum LuaError {
    /**
     * There was a syntax error when parsing the Lua code
     */
    SyntaxError(String),

    /**
     * There was an error during execution of the Lua code (for example not enough parameters for a function call)
     */
    ExecutionError(String),

    /**
     * The call to `execute` has requested the wrong type of data
     */
    WrongType
}


// this alloc function is required to create a lua state
extern "C" fn alloc(_ud: *mut libc::c_void, ptr: *mut libc::c_void, _osize: libc::size_t, nsize: libc::size_t) -> *mut libc::c_void {
    unsafe {
        if nsize == 0 {
            libc::free(ptr as *mut libc::c_void);
            std::ptr::mut_null()
        } else {
            libc::realloc(ptr, nsize)
        }
    }
}

// called whenever lua encounters an unexpected error
extern "C" fn panic(lua: *mut ffi::lua_State) -> libc::c_int {
    let err = unsafe { ffi::lua_tostring(lua, -1) };
    fail!("PANIC: unprotected error in call to Lua API ({})\n", err);
}

impl<'lua> Lua<'lua> {
    /**
     * Builds a new Lua context
     * # Failure
     * The function fails if lua_newstate fails (which indicates lack of memory)
     */
    pub fn new() -> Lua {
        let lua = unsafe { ffi::lua_newstate(alloc, std::ptr::mut_null()) };
        if lua.is_null() {
            fail!("lua_newstate failed");
        }

        unsafe { ffi::lua_atpanic(lua, panic) };

        Lua {
            lua: lua,
            marker: ContravariantLifetime,
            must_be_closed: true,
            inside_callback: false
        }
    }

    /**
     * Takes an existing lua_State and build a Lua object from it
     * # Arguments
     *  * close_at_the_end: if true, lua_close will be called on the lua_State on the destructor
     */
    pub unsafe fn from_existing_state<T>(lua: *mut T, close_at_the_end: bool) -> Lua {
        Lua {
            lua: std::mem::transmute(lua),
            marker: ContravariantLifetime,
            must_be_closed: close_at_the_end,
            inside_callback: false
        }
    }

    /**
     * Opens all standard Lua libraries
     * This is done by calling `luaL_openlibs`
     */
    pub fn openlibs(&mut self) {
        unsafe { ffi::luaL_openlibs(self.lua) }
    }

    /**
     * Executes some Lua code on the context
     */
    pub fn execute<T: CopyReadable>(&mut self, code: &str) -> Result<T, LuaError> {
        let mut f = try!(functions_read::LuaFunction::load(self, code));
        f.call()
    }

    /**
     * Executes some Lua code on the context
     */
    pub fn execute_from_reader<T: CopyReadable, R: std::io::Reader + 'static>(&mut self, code: R) -> Result<T, LuaError> {
        let mut f = try!(functions_read::LuaFunction::load_from_reader(self, code));
        f.call()
    }

    /**
     * Reads the value of a global variable
     */
    pub fn get<'a, I: Str, V: ConsumeReadable<'a, 'lua>>(&'a mut self, index: I) -> Option<V> {
        unsafe { ffi::lua_getglobal(self.lua, index.as_slice().to_c_str().unwrap()); }
        ConsumeReadable::read_from_variable(LoadedVariable { lua: self, size: 1 }).ok()
    }

    /**
     * Modifies the value of a global variable
     */
    pub fn set<I: Str, V: Pushable<'lua>>(&mut self, index: I, value: V) {
        value.push_to_lua(self);
        unsafe { ffi::lua_setglobal(self.lua, index.as_slice().to_c_str().unwrap()); }
    }

    /**
     *
     */
    pub fn load_new_table<'var>(&'var mut self) -> LuaTable<'var, 'lua> {
        unsafe { ffi::lua_newtable(self.lua) };
        ConsumeReadable::read_from_variable(LoadedVariable { lua: self, size: 1 }).ok().unwrap()
    }
}

// TODO: these destructors crash the compiler
// https://github.com/mozilla/rust/issues/13853
// https://github.com/mozilla/rust/issues/14377
/*impl<'lua> Drop for Lua<'lua> {
    fn drop(&mut self) {
        if self.must_be_closed {
            unsafe { ffi::lua_close(self.lua) }
        }
    }
}

impl<'a> Drop for LoadedVariable<'a, 'lua> {
    fn drop(&mut self) {
        unsafe { ffi::lua_pop(self.lua.lua, self.size as libc::c_int) }
    }
}

impl<'a> LoadedVariable<'a, 'lua> {
    fn pop_nb(mut self, nb: uint) -> LoadedVariable<'a, 'lua> {
        assert!(nb <= self.size);
        unsafe { ffi::lua_pop(self.lua.lua, nb as libc::c_int); }
        self.size -= nb;
        self
    }
}*/

#[cfg(test)]
mod tests {
    #[test]
    fn globals_readwrite() {
        let mut lua = super::Lua::new();

        lua.set("a", 2i);
        let x: int = lua.get("a").unwrap();
        assert_eq!(x, 2)
    }

    #[test]
    fn execute() {
        let mut lua = super::Lua::new();

        let val: int = lua.execute("return 5").unwrap();
        assert_eq!(val, 5);
    }

    // TODO: doesn't compile, have absolutely NO IDEA why
    /*#[test]
    fn table_readwrite() {
        let mut lua = super::Lua::new();

        lua.execute("a = { foo = 5 }");

        assert_eq!(lua.access("a").get(&"foo").unwrap(), 2);

        {   let access = lua.access("a");
            access.set(5, 3);
            assert_eq!(access.get(5), 3);
        }
    }*/
}
