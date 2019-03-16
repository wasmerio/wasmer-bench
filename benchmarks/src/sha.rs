extern crate sha1;

#[no_mangle]
pub unsafe extern "C" fn sha1(iters: i32) {
    let mut m = sha1::Sha1::new();
    for i in 0..iters {
        m.update(&format!("hello sha {}", i).into_bytes()[..]);
        m.digest().to_string();
    }
}
