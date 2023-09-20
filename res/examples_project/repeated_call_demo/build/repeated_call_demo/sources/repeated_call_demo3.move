module repeated_call_demo::repeated_call_demo3
{

    public fun demo3_f0(len:u8) {
    }

    // pass
    public fun demo3_f1() {
        let i = 0;
        demo3_f0(i);
        let a = i+1;
        demo3_f0(a);
    }
    
    // wrong
    public fun demo3_f2(a:u8) {
        demo3_f0(a);
        let b = a;
        demo3_f0(b);
        demo3_f0(a);
        let c = b;
        demo3_f0(c);
    }

    // wrong
    public fun demo3_f3(a:u8) {
        demo3_f0(a);
        let b = a;
        demo3_f0(b);
    }

    // pass
    public fun demo3_f4(a:u8) {
        demo3_f0(a);
        let b = a+1;
        demo3_f0(b);
    }


    // pass
    public fun demo3_f5(a:u8) {
        demo3_f0(a);
        let b = 1;
        let c = a+b;
        demo3_f0(c);
    }



}