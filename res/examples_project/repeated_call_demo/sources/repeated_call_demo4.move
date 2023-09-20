module repeated_call_demo::repeated_call_demo4
{
    
    struct Test_Struct has drop { x: u8, y:u8 }
    // struct Coin has key, store{
    //     value: u64
    // }
    public fun demo4_f0(len:&u8) {
    }

    // pass borrow_local
    public fun demo4_f1() {
        let i = 0;
        demo4_f0(&i);
        let a = i+1;
        demo4_f0(&a);
    }

    // pass example borrow_field, read_ref, write_ref
    public fun demo4_f2() {
        let test_struct = Test_Struct{x:1,y:2};
        // borrow_field and read_ref
        let borrow_field_x = test_struct.x;
        let borrow_field_y = test_struct.y;
        demo4_f0(&borrow_field_x);
        demo4_f0(&borrow_field_y);
        // write_ref
        test_struct.x = 2;
    }

    // wrong
    public fun demo4_f3() {
        let test_struct = Test_Struct{x:1,y:2};
        demo4_f0(&test_struct.x);
        demo4_f0(&test_struct.x);
    }

    // pass
    public fun demo4_f4() {
        let test_struct = Test_Struct{x:1,y:2};
        demo4_f0(&test_struct.x);
        demo4_f0(&test_struct.y);
    }

    // wrong
    public fun demo4_f5() {
        let test_struct = Test_Struct{x:1,y:2};
        let a = test_struct.x;
        let b = test_struct.x;
        let c = a;
        demo4_f0(&c);
        demo4_f0(&b);
    }

    // pass freeze_ref
    public fun demo4_f6() {
        let i = 0;
        demo4_f0(&mut i);
        let a = i+1;
        demo4_f0(&mut a);
    }

    public fun demo4_f7(arg1:u8) {
        let a = arg1;
        let b =1;
        let c=b;
        demo4_f0(&a);
        demo4_f0(&c);
    }

}