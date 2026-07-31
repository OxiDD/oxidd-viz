use std::{cell::RefCell, rc::Rc};

use itertools::Itertools;
use js_sys::Array;
use wasm_bindgen::JsValue;

use crate::configuration::{
    configuration_object::{
        AbstractConfigurationObject, Abstractable, ConfigObjectGetter, Configurable,
        ConfigurationObject, ValueMapping,
    },
    configuration_object_types::ConfigurationObjectType,
    mutator::Mutator,
    util::js_object::JsObject,
};

/// A configuration that allows the user from choosing between some predefined options
#[derive(Clone)]
pub struct TextConfig {
    data: ConfigurationObject<TextConfig, TextValue>,
}

#[derive(Clone)]
struct TextValue {
    options: Vec<String>,
    current: String,
}

impl TextConfig {
    pub fn new(choices: Vec<String>, init: String) -> TextConfig {
        TextConfig {
            data: ConfigurationObject::new(TextValue {
                options: choices,
                current: init,
            }),
        }
    }

    pub fn set_text(&mut self, text: String) -> Mutator<(), ()> {
        self.data.set_value(move |cur| {
            Some(TextValue {
                options: cur.options.clone(),
                current: text,
            })
        })
    }

    pub fn get(&self) -> String {
        self.data.with_value(|v| v.current.clone())
    }

    pub fn set_options(&mut self, choices: Vec<String>) -> Mutator<(), ()> {
        self.data.set_value(|cur| {
            Some(TextValue {
                options: choices,
                current: cur.current.clone(),
            })
        })
    }

    pub fn get_options(&self) -> Vec<String> {
        self.data.with_value(|v| v.options.clone())
    }
}

impl TextConfig {
    pub fn set(&mut self, val: String) -> Mutator<(), ()> {
        self.data.set_value(move |cur| {
            Some(TextValue {
                options: cur.options.clone(),
                current: val,
            })
        })
    }
}

impl Abstractable for TextConfig {
    fn get_abstract(&self) -> AbstractConfigurationObject {
        AbstractConfigurationObject::new(ConfigurationObjectType::Text, self.data.clone())
    }
}
impl ConfigObjectGetter<TextConfig, TextValue> for TextConfig {
    fn with_config_object<O, U: FnOnce(&mut ConfigurationObject<TextConfig, TextValue>) -> O>(
        &mut self,
        e: U,
    ) -> O {
        e(&mut self.data)
    }
}

impl ValueMapping<TextValue> for TextConfig {
    fn to_js_value(val: &TextValue) -> JsValue {
        JsObject::new()
            .set(
                "options",
                JsValue::from(
                    val.options
                        .iter()
                        .map(|v| JsValue::from(v))
                        .collect::<Array>(),
                ),
            )
            .set("value", val.current.clone())
            .into()
    }
    fn from_js_value(js_val: JsValue, cur: &TextValue) -> Option<TextValue> {
        let obj = JsObject::load(js_val);
        let current = obj
            .get("value")
            .and_then(|v| v.as_string())
            .unwrap_or_default();
        let options = obj
            .get("options")
            .and_then(|v| {
                if !Array::is_array(&v) {
                    return None;
                }

                Some(
                    Array::from(&v)
                        .iter()
                        .filter_map(|v| v.as_string())
                        .collect::<Vec<_>>(),
                )
            })
            .unwrap_or_else(|| cur.options.clone());

        // let options =
        Some(TextValue { current, options })
    }

    fn get_children(_val: &TextValue) -> Option<Vec<AbstractConfigurationObject>> {
        None
    }
}
