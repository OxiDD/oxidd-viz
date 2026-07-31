use wasm_bindgen::prelude::*;

#[derive(Clone)]
#[wasm_bindgen]
pub enum ConfigurationObjectType {
    Int,
    Float,
    Choice,
    Label,
    Composite,
    Button,
    Panel,
    Location,
    TextOutput,
    Text,
    Container,
}
