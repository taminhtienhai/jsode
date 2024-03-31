use json_parser_macro::reflection;



#[test]
fn two_pointer() {
    #[allow(dead_code)]
    struct Ptr(u8);
    let ptr = Ptr(10);

    let ptr_01 = &ptr as *const Ptr;
    let ptr_02 = &ptr as *const Ptr;


    println!("{:?}", ptr_01);
    println!("{:?}", ptr_02);
}

#[test]
fn reflection_macro() {
    #[reflection]
    fn sum(a: usize, b: usize) -> usize {
        println!("{}", __fn_ident);
        println!("{}", __arg_0);
        println!("{}", __arg_1);
        a + b
    }

    let _ = sum(1,2);
}