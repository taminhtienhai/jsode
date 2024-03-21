

#[test]
fn two_pointer() {
    struct Ptr(u8);
    let ptr = Ptr(10);

    let ptr_01 = &ptr as *const Ptr;
    let ptr_02 = &ptr as *const Ptr;


    println!("{:?}", ptr_01);
    println!("{:?}", ptr_02);
}