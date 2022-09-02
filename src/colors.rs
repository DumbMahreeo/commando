#[macro_export]
macro_rules! color {
    ($color:tt) => {
        if ::std::env::var_os("NO_COLOR").is_none() {
            color!(__ $color)
        } else {
            ""
        }
    };

    (__ reset) => {
        "\x1B[0m"
    };

    (__ white) => {
        concat!(color!(__ reset), "\x1B[37;1m")
    };

    (__ red) => {
        concat!(color!(__ reset), "\x1B[31;1m")
    };

    (__ green) => {
        concat!(color!(__ reset), "\x1B[32;1m")
    };

    (__ yellow) => {
        concat!(color!(__ reset), "\x1B[33;1m")
    };
}
