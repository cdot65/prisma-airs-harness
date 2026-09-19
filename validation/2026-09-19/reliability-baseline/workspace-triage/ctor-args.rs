// Isolated reproducer: no repository code or dependencies modified.
#[used]
#[unsafe(link_section = ".init_array")]
static INITIALIZER: extern "C" fn() = before_main;
extern "C" fn before_main() {
    eprintln!("before_main argv={:?}", std::env::args().collect::<Vec<_>>());
}
fn main() {
    eprintln!("main argv={:?}", std::env::args().collect::<Vec<_>>());
}
