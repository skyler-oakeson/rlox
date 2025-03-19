#[macro_export]
macro_rules! map {
    ($({$k:expr, $v:expr}),*) => {
        {
            let mut m = HashMap::new();
            $(
                m.insert($k, $v);
            )*
            m
        }
    };
}

#[macro_export]
macro_rules! S {
    ($s:expr) => {
        $s.to_string()
    };
}
