module recursive_call_demo::recursive_call_demo1
{

    use recursive_call_demo::recursive_call_demo2;

    public fun demo1_f0() {
    }
    
    // case1: call self
    public fun demo1_f1(a: u8, b: u8) {
        demo1_f1(a,a+b);
    }
    
    // case2: call self or normal function
    public fun demo1_f2(a: u8, b: u8) {
        if (a > 10) {
            demo1_f2(a,a+b);
        } else {
            demo1_f0();
        }
    }

    // case3: call normal function or recursive call function
    public fun demo1_f3(a: u8, b: u8) {
        if (a > 10) {
            demo1_f1(a,a+b);
        } else {
            demo1_f0();
        }
    }

    // case4: call each other
    public fun demo1_f4(a: u8, b: u8){
        demo1_f5(a,a+b);
    }
    public fun demo1_f5(a: u8, b: u8) {
        demo1_f6(a,a+b);
    }
    public fun demo1_f6(a: u8, b: u8) {
        demo1_f5(a,a+b);
    }

    // case5: cross module call function which has recursive call 
    public fun demo1_f7(a: u8, b: u8) {
        recursive_call_demo2::demo2_f1(a,a+b);
    }

}