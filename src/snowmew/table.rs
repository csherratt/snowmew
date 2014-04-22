
#![macro_escape]

#[macro_export]
macro_rules! database(
    ($name:ident {$($column:ident : $column_type:ty),+ }) => (
        struct $name {
            $(
                $column: BTreeMap<uint, $column_type>,
            )+
        }

        $(
            trait concat_idents!(kitten_, $column) {

            }
        )+
    )
)