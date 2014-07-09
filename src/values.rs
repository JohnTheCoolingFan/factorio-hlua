use super::ffi;
use super::Index;
use super::Lua;
use super::Pushable;
use super::CopyReadable;
use super::ConsumeReadable;
use super::LoadedVariable;

macro_rules! integer_impl(
    ($t:ident) => (
        impl<'lua> Pushable<'lua> for $t {
            fn push_to_lua(self, lua: &mut Lua) -> uint {
                unsafe { ffi::lua_pushinteger(lua.lua, self as ffi::lua_Integer) };
                1
            }
        }
        impl CopyReadable for $t {
            fn read_from_lua<'lua>(lua: &mut Lua<'lua>, index: i32) -> Option<$t> {
                let mut success: ::libc::c_int = unsafe { ::std::mem::uninitialized() };
                let val = unsafe { ffi::lua_tointegerx(lua.lua, index, &mut success) };
                match success {
                    0 => None,
                    _ => Some(val as $t)
                }
            }
        }
        impl<'a,'lua> ConsumeReadable<'a,'lua> for $t {
            fn read_from_variable(var: LoadedVariable<'a, 'lua>) -> Result<$t, LoadedVariable<'a, 'lua>> {
                match CopyReadable::read_from_lua(var.lua, -1) {
                    None => Err(var),
                    Some(a) => Ok(a)
                }
            }
        }
        impl<'lua> Index<'lua> for $t {
        }
    );
)

integer_impl!(int)
integer_impl!(i8)
integer_impl!(i16)
integer_impl!(i32)
//integer_impl!(i64)   // data loss

macro_rules! unsigned_impl(
    ($t:ident) => (
        impl<'lua> Pushable<'lua> for $t {
            fn push_to_lua(self, lua: &mut Lua) -> uint {
                unsafe { ffi::lua_pushunsigned(lua.lua, self as ffi::lua_Unsigned) };
                1
            }
        }
        impl CopyReadable for $t {
            fn read_from_lua<'lua>(lua: &mut Lua<'lua>, index: i32) -> Option<$t> {
                let mut success: ::libc::c_int = unsafe { ::std::mem::uninitialized() };
                let val = unsafe { ffi::lua_tounsignedx(lua.lua, index, &mut success) };
                match success {
                    0 => None,
                    _ => Some(val as $t)
                }
            }
        }
        impl<'a,'lua> ConsumeReadable<'a,'lua> for $t {
            fn read_from_variable(var: LoadedVariable<'a, 'lua>) -> Result<$t, LoadedVariable<'a, 'lua>> {
                match CopyReadable::read_from_lua(var.lua, -1) {
                    None => Err(var),
                    Some(a) => Ok(a)
                }
            }
        }
        impl<'lua> Index<'lua> for $t {
        }
    );
)

unsigned_impl!(uint)
unsigned_impl!(u8)
unsigned_impl!(u16)
unsigned_impl!(u32)
//unsigned_impl!(u64)   // data loss

macro_rules! numeric_impl(
    ($t:ident) => (
        impl<'lua> Pushable<'lua> for $t {
            fn push_to_lua(self, lua: &mut Lua) -> uint {
                unsafe { ffi::lua_pushnumber(lua.lua, self as f64) };
                1
            }
        }
        impl CopyReadable for $t {
            fn read_from_lua<'lua>(lua: &mut Lua<'lua>, index: i32) -> Option<$t> {
                let mut success: ::libc::c_int = unsafe { ::std::mem::uninitialized() };
                let val = unsafe { ffi::lua_tonumberx(lua.lua, index, &mut success) };
                match success {
                    0 => None,
                    _ => Some(val as $t)
                }
            }
        }
        impl<'a,'lua> ConsumeReadable<'a,'lua> for $t {
            fn read_from_variable(var: LoadedVariable<'a, 'lua>) -> Result<$t, LoadedVariable<'a, 'lua>> {
                match CopyReadable::read_from_lua(var.lua, -1) {
                    None => Err(var),
                    Some(a) => Ok(a)
                }
            }
        }
        impl<'lua> Index<'lua> for $t {
        }
    );
)

numeric_impl!(f32)
numeric_impl!(f64)

impl<'lua> Pushable<'lua> for String {
    fn push_to_lua(self, lua: &mut Lua) -> uint {
        unsafe { ffi::lua_pushstring(lua.lua, self.to_c_str().unwrap()) };
        1
    }
}

impl CopyReadable for String {
    fn read_from_lua<'lua>(lua: &mut Lua<'lua>, index: i32) -> Option<String> {
        let mut size: ::libc::size_t = unsafe { ::std::mem::uninitialized() };
        let cStrRaw = unsafe { ffi::lua_tolstring(lua.lua, index, &mut size) };
        if cStrRaw.is_null() {
            return None;
        }

        unsafe { ::std::c_str::CString::new(cStrRaw, false) }.as_str().map(|s| s.to_string())
    }
}

impl<'a,'lua> ConsumeReadable<'a,'lua> for String {
    fn read_from_variable(var: LoadedVariable<'a, 'lua>) -> Result<String, LoadedVariable<'a, 'lua>> {
        match CopyReadable::read_from_lua(var.lua, -1) {
            None => Err(var),
            Some(a) => Ok(a)
        }
    }
}

impl<'lua> Index<'lua> for String {
}

impl<'str, 'lua> Pushable<'lua> for &'str str {
    fn push_to_lua(self, lua: &mut Lua<'lua>) -> uint {
        unsafe { ffi::lua_pushstring(lua.lua, self.to_c_str().unwrap()) }
        1
    }
}

impl<'lua> Pushable<'lua> for bool {
    fn push_to_lua(self, lua: &mut Lua<'lua>) -> uint {
        unsafe { ffi::lua_pushboolean(lua.lua, self.clone() as ::libc::c_int) };
        1
    }
}

impl CopyReadable for bool {
    fn read_from_lua<'lua>(lua: &mut Lua<'lua>, index: i32) -> Option<bool> {
        if unsafe { ffi::lua_isboolean(lua.lua, index) } != true {
            return None;
        }

        Some(unsafe { ffi::lua_toboolean(lua.lua, index) != 0 })
    }
}

impl<'a,'lua> ConsumeReadable<'a,'lua> for bool {
    fn read_from_variable(var: LoadedVariable<'a, 'lua>) -> Result<bool, LoadedVariable<'a, 'lua>> {
        match CopyReadable::read_from_lua(var.lua, -1) {
            None => Err(var),
            Some(a) => Ok(a)
        }
    }
}

impl<'lua> Index<'lua> for bool {
}

impl<'lua> Pushable<'lua> for () {
    fn push_to_lua(self, _: &mut Lua) -> uint {
        0
    }
}

impl CopyReadable for () {
    fn read_from_lua(_: &mut Lua, _: i32) -> Option<()> {
        Some(())
    }
}

impl<'a,'lua> ConsumeReadable<'a,'lua> for () {
    fn read_from_variable(_: LoadedVariable<'a, 'lua>) -> Result<(), LoadedVariable<'a, 'lua>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn read_ints() {
        let mut lua = super::super::Lua::new();

        lua.set("a", 2i);

        let x: int = lua.get("a").unwrap();
        assert_eq!(x, 2);

        let y: i8 = lua.get("a").unwrap();
        assert_eq!(y, 2);

        let z: i16 = lua.get("a").unwrap();
        assert_eq!(z, 2);

        let w: i32 = lua.get("a").unwrap();
        assert_eq!(w, 2);

        let a: uint = lua.get("a").unwrap();
        assert_eq!(a, 2);

        let b: u8 = lua.get("a").unwrap();
        assert_eq!(b, 2);

        let c: u16 = lua.get("a").unwrap();
        assert_eq!(c, 2);

        let d: u32 = lua.get("a").unwrap();
        assert_eq!(d, 2);
    }

    #[test]
    fn write_ints() {
        // TODO: 

        let mut lua = super::super::Lua::new();

        lua.set("a", 2i);
        let x: int = lua.get("a").unwrap();
        assert_eq!(x, 2);
    }

    #[test]
    fn readwrite_floats() {
        let mut lua = super::super::Lua::new();

        lua.set("a", 2.51234 as f32);
        lua.set("b", 3.4123456789 as f64);

        let x: f32 = lua.get("a").unwrap();
        assert!(x - 2.51234 < 0.000001);

        let y: f64 = lua.get("a").unwrap();
        assert!(y - 2.51234 < 0.000001);

        let z: f32 = lua.get("b").unwrap();
        assert!(z - 3.4123456789 < 0.000001);

        let w: f64 = lua.get("b").unwrap();
        assert!(w - 3.4123456789 < 0.000001);
    }

    #[test]
    fn readwrite_bools() {
        let mut lua = super::super::Lua::new();

        lua.set("a", true);
        lua.set("b", false);

        let x: bool = lua.get("a").unwrap();
        assert_eq!(x, true);

        let y: bool = lua.get("b").unwrap();
        assert_eq!(y, false);
    }

    #[test]
    fn readwrite_strings() {
        let mut lua = super::super::Lua::new();

        lua.set("a", "hello");
        lua.set("b", "hello".to_string());

        let x: String = lua.get("a").unwrap();
        assert_eq!(x.as_slice(), "hello");

        let y: String = lua.get("b").unwrap();
        assert_eq!(y.as_slice(), "hello");
    }

    #[test]
    fn int_to_string() {
        let mut lua = super::super::Lua::new();

        lua.set("a", 2i);

        let x: String = lua.get("a").unwrap();
        assert_eq!(x.as_slice(), "2");
    }

    #[test]
    fn string_to_int() {
        let mut lua = super::super::Lua::new();

        lua.set("a", "2");
        lua.set("b", "aaa");

        let x: int = lua.get("a").unwrap();
        assert_eq!(x, 2);

        let y: Option<int> = lua.get("b");
        assert!(y.is_none());
    }
}
