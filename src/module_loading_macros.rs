macro_rules! deserializable_module_data_no_visual {
    (
        $deser_vis:vis struct $deser_name:ident
        $($field_vis:vis $field:ident: $t:ty),*
    ) => {
        #[derive(Deserialize, Clone, Debug)]
        $deser_vis struct $deser_name {
            /// omit to not force deserialization
            $deser_vis _force_deser: Option<bool>,
            $($field_vis $field: Option<$t>),*
        }
    };
    (
        $deser_vis:vis struct $deser_name:ident
    ) => {
        #[derive(Deserialize, Clone, Debug)]
        $deser_vis struct $deser_name {
            /// omit to not force deserialization
            $deser_vis _force_deser: Option<bool>,
        }
    };
}

macro_rules! deserializable_module_data_yes_visual {
    (
        $deser_vis:vis struct $deser_name:ident
        $($field_vis:vis $field:ident: $t:ty),*
    ) => {
        #[derive(Deserialize, Clone, Debug)]
        $deser_vis struct $deser_name {
            /// omit to not force deserialization
            $deser_vis _force_deser: Option<bool>,
            $deser_vis visual_data: Option<VisualDeserData>,
            $($field_vis $field: Option<$t>),*
        }
    };
    (
        $deser_vis:vis struct $deser_name:ident
    ) => {
        #[derive(Deserialize, Clone, Debug)]
        $deser_vis struct $deser_name {
            /// omit to not force deserialization
            $deser_vis _force_deser: Option<bool>,
            $deser_vis visual_data: Option<VisualDeserData>
        }
    };
}

macro_rules! deserializable_module_data {
    (   
        [has_visual]

        $deser_vis:vis struct $deser_name:ident
        $($field_vis:vis $field:ident: $t:ty),*
    ) => {
        deserializable_module_data_yes_visual!{
            $deser_vis struct $deser_name
            $($field_vis $field: $t),*
        }
        impl_deserialization!(
            $deser_vis $deser_name
        );
    };
    (
        [has_visual]

        $deser_vis:vis struct $deser_name:ident
    ) => {
        deserializable_module_data_yes_visual!{
            $deser_vis struct $deser_name
        }
        impl_deserialization!(
            $deser_vis $deser_name
        );
    };
    (
        $deser_vis:vis struct $deser_name:ident
        $($field_vis:vis $field:ident: $t:ty),*
    ) => {
        deserializable_module_data_no_visual! {
            $deser_vis struct $deser_name
            $($field_vis $field: $t),*
        }
        impl_deserialization!(
            $deser_vis $deser_name
        );
    };
    (
        $deser_vis:vis struct $deser_name:ident
    ) => {
        deserializable_module_data_no_visual! {
            $deser_vis struct $deser_name
            $non_deser_vis struct $non_deser_name
        }
        impl_deserialization!(
            $deser_vis $deser_name
        );
        impl_priority!(
            $non_deser_vis $non_deser_name
        );
    }
}

macro_rules! impl_deserialization {
    (
        $vis:vis $name:ident
    ) => {
        impl AsAny for $name {
            $vis fn as_any(&self) -> &dyn Any {
                self
            }
        }
        impl Deserialization for $name {}
    };
}