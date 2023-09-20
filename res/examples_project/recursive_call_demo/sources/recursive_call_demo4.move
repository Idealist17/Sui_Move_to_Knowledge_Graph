module recursive_call_demo::recursive_call_demo4
{
    public fun demo2_f0(a: u8, b: u8) {
        demo2_f1(a, b);
    }

    public fun demo2_f1(a: u8, b: u8) {
        demo2_f2(a, b);
        demo2_f4(a, b);
    }

    public fun demo2_f2(a: u8, b: u8) {
        demo2_f3(a, b);
    }

       public fun demo2_f3(a: u8, b: u8) {
    }

    public fun demo2_f4(a: u8, b: u8) {
        demo2_f5(a, b);
    }
    public fun demo2_f5(a: u8, b: u8) {
        demo2_f1(a, b);
    }

    
}