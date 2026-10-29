include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

unsafe extern "C" {
    fn mindrove_scope_shim_anchor();
}

pub fn ensure_scope_shim_linked() {
    unsafe {
        mindrove_scope_shim_anchor();
    }
}
