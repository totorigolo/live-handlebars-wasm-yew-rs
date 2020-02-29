#[macro_export]
macro_rules! impl_input_for {
    ($type:ty) => {
        impl crate::inputs::Input for $type {
            fn key(&self) -> &crate::Path {
                &self.info.key
            }

            fn name(&self) -> &str {
                &self.info.name
            }

            fn description(&self) -> Option<&str> {
                self.info.description.as_deref()
            }
        }
    };
}

#[macro_export]
macro_rules! for_all_inputtypes_variants {
    ($self:expr, $ident:ident => $expr:expr) => {
        match $self {
            crate::inputs::InputTypes::Text($ident) => $expr,
            crate::inputs::InputTypes::Boolean($ident) => $expr,
            crate::inputs::InputTypes::Number($ident) => $expr,
            crate::inputs::InputTypes::Group($ident) => $expr,
            crate::inputs::InputTypes::List($ident) => $expr,
        }
    };
}
