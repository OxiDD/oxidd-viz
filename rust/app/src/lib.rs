mod components;
mod configuration;
mod inputs;
mod new_wasm_interface;
mod traits;
mod types;
mod util;
mod wasm_interface;

use app_macros::{Component, Inheritable, InitDefault, Saveable};
use serde::{Deserialize, Serialize};
use util::panic_hook::set_panic_hook;
use wasm_bindgen::prelude::*;

use types::{mtbdd::mtbdd_drawer::MTBDDDiagram, qdd::qdd_drawer::QDDDiagram};

use crate::{
    components::{
        button_component::ButtonComp, Align, AlignMain, ComponentWithData, CompositeComp,
        CompositeItemComp, ContainerComp, FillComp, IntoComponentVec, LabelComp, LabelKind,
        OverlayComp, PanelButtonComp, PanelComp, PromptComp, TooltipComp,
    },
    inputs::{
        binary_input::{BinaryInput, BinaryInputComp},
        bool_input::{BoolInput, BoolInputComp},
        f32_input::{F32InputClamped, F32InputComp},
        string_input::{StringInput, StringInputComp},
        variant_input::{VariantInput, VariantInputComp, VariantOptions},
        CompWrapper, ComponentInput, DefaultInputComp, DynWrappedInput, F32Input,
        F32InputCompBuilder, Inheritable, InheritedInput, Saveable, U32Input, U32InputClamped,
        VariantComponentMapping, WrapBuilder,
    },
    new_wasm_interface::Component,
    util::watchables::{
        CloneableWatchableUtils, DynWatchableSetter, F32Field, Field, StringField, WatchableSetter,
        WatchableUtils,
    },
    wasm_interface::DiagramBox,
};

#[wasm_bindgen]
pub fn create_qdd_diagram() -> Option<DiagramBox> // And some DD type param
{
    set_panic_hook();
    Some(DiagramBox::new(Box::new(QDDDiagram::new())))
}

#[wasm_bindgen]
pub fn create_mtbdd_diagram() -> Option<DiagramBox> // And some DD type param
{
    set_panic_hook();
    Some(DiagramBox::new(Box::new(MTBDDDiagram::new())))
}

#[wasm_bindgen]
pub fn test_panel() -> PanelComp {
    let text_ancestor = InheritedInput::default(StringInput::from("testing"));
    let text_ancestor_clone = text_ancestor.clone();
    // let inherited_text = InheritedInput::default(text_ancestor, "Default");
    let inherited_text = text_ancestor.inherit("Ancestor");
    let mut text_clone = inherited_text.clone();
    let text_clone2 = inherited_text.clone();

    let button = ButtonComp::builder()
        .icon("AlarmClock")
        .text(inherited_text.map(|v| Some(format!("Button: {v}"))))
        .build();
    let reset_button = ComponentWithData::new(
        button.clone(),
        button.on_click(move || {
            text_clone.set("reset".to_string());
        }),
    );

    let description = Field::new("hoi".to_string());
    let labeled_button = TooltipComp::builder()
        .tooltip(CompositeComp::builder().horizontal(true).build((
            "Something",
            ButtonComp::builder().icon("AlarmClock").build(),
        )))
        .build(
            LabelComp::builder()
                .kind(LabelKind::Inline)
                .label(description)
                .build(reset_button),
        );

    let text_ancestor_field: Component = LabelComp::wrapped("label", text_ancestor_clone).into();
    let text_field = StringInputComp::builder(LabelComp::wrapped("field", text_clone2))
        .multiline(true)
        .multiline_dynamic(true)
        .multiline_min(2);
    // .late_submit(true);

    let variant = VariantInput::new(Variants::V1);
    let variant_comp1 = variant.clone();
    let variant_comp2 = variant
        .clone()
        .comp_builder(|v| {
            ButtonComp::builder()
                .icon(match v {
                    Variants::V1 => "PageLink",
                    Variants::V2 => "CommentSolid",
                    Variants::V3 => "Installation",
                })
                .text("Some label")
                .build()
        })
        .horizontal(false);

    let v1_field = StringInputComp::builder(StringInput::new("V1".into())).build();
    let v2_field = StringInputComp::builder(StringInput::new("V2".into())).build();
    let v3_field = StringInputComp::builder(StringInput::new("V3".into())).build();
    let variant_comp3 = variant
        .clone()
        .comp_builder(move |v| {
            LabelComp::builder()
                .label(match v {
                    Variants::V1 => "V1",
                    Variants::V2 => "V2",
                    Variants::V3 => "V3",
                })
                .build(match v {
                    Variants::V1 => v1_field.clone(),
                    Variants::V2 => v2_field.clone(),
                    Variants::V3 => v3_field.clone(),
                })
        })
        .main_align(AlignMain::Stretch)
        .gap(1.0)
        .horizontal(true);

    let extra_panel_name = StringField::from("test panel");
    let extra_panel = PanelButtonComp::builder()
        .button(ButtonComp::builder().text("open panel").build())
        .panel(
            PanelComp::builder()
                .name(extra_panel_name)
                .id("extra-panel"),
        )
        .build(
            CompositeComp::builder()
                .perpendicular_align(Align::End)
                .build((
                    F32InputComp::builder(
                        F32InputClamped::builder(F32Input::new(0.0))
                            .min(-5.0)
                            .max(5.0)
                            .precision(0.5)
                            .build(),
                    )
                    .step_size(1.0)
                    .step_round(true)
                    .build(),
                    variant_comp1,
                    variant_comp2,
                    variant_comp3,
                )),
        );

    let binary_input = BinaryInputComp::builder(BinaryInput::new(None));

    let composite = (
        text_ancestor_field,
        text_field,
        labeled_button,
        extra_panel,
        binary_input,
    );

    let prompt_button = ButtonComp::builder().text("Some prompt").build();
    let prompt = (
        prompt_button.clone(),
        PromptComp::builder()
            .open_count(prompt_button.clicks())
            .build(("Hallo prompt", BoolInput::new(false))),
    );

    let (settings, inherited_settings, inherited_inherited_settings, settings_comp) =
        settings_comp();
    let with_overlay = FillComp::new(ContainerComp::builder().padding(1.0).build(FillComp::new((
        composite,
        prompt,
        CompositeComp::builder().horizontal(true).build((
            settings_comp,
            settings_field(&settings),
            settings_field(&inherited_settings),
            settings_field(&inherited_inherited_settings),
        )),
        OverlayComp::bottom_right(ButtonComp::builder().icon("AlarmClock").build()),
    ))));

    let panel_name = StringField::from("test panel");
    PanelComp::builder()
        .name(panel_name)
        .id("test-panel")
        .open_count(0)
        .build(with_overlay)
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
enum Variants {
    V1,
    V2,
    V3,
}
impl VariantOptions for Variants {
    fn get_variants() -> Vec<Self> {
        vec![Variants::V1, Variants::V2, Variants::V3]
    }
}
impl VariantComponentMapping for Variants {
    fn map(&self) -> Component {
        (match self {
            Variants::V1 => "V1",
            Variants::V2 => "V2",
            Variants::V3 => "V3",
        })
        .into()
    }
}

#[wasm_bindgen]
#[derive(Inheritable, InitDefault, Component, Saveable, Clone)]
#[label(spaced, map=|l, c| LabelComp::builder().label(l).kind(LabelKind::Inline).build(c))]
#[comp(build=builder.gap(2.0).perpendicular_align(Align::Stretch))]
struct MySettings {
    // Text input with custom label
    #[label(text = "bob")]
    #[comp(map=|s| StringInputComp::builder(s).multiline(true))]
    name: InheritedInput<StringInput>,

    // Number input with initialization that contains constraints
    #[init(U32InputClamped::builder(45u32).max(50).build())]
    value: InheritedInput<U32InputClamped>,

    // Number value with initialization
    #[init(32.5)]
    #[comp(build=builder.step_size(2.0))]
    other_value: InheritedInput<F32Input>,

    // Number value with initialization
    #[init(Variants::V2)]
    variant: InheritedInput<VariantInput<Variants>>,

    // Checkbox with custom label styling
    #[label(map=|l, c| LabelComp::builder().label(l).kind(LabelKind::Above).build(c))]
    check: InheritedInput<BoolInput>,
}

fn settings_comp() -> (MySettings, MySettings, MySettings, Component) {
    let settings = MySettings::default();
    let inherited_settings = settings.inherit("parent");
    let inherited_inherited_settings = inherited_settings.inherit("parent's parent");

    let res = PanelButtonComp::builder()
        .button(ButtonComp::builder().text("Open settings").build())
        .panel(
            PanelComp::builder()
                .name(StringField::from("settings"))
                .id("settings-panel"),
        )
        .build((
            settings.clone(),
            "Inherited:",
            inherited_settings.clone(),
            "Double inherited:",
            inherited_inherited_settings.clone(),
        ))
        .into();

    (
        settings,
        inherited_settings,
        inherited_inherited_settings,
        res,
    )
}

fn settings_field(settings: &MySettings) -> Component {
    let settings_text = StringInput::from("");
    let text_field =
        StringInputComp::builder(LabelComp::wrapped("settings", settings_text.clone()))
            .multiline(true)
            .multiline_dynamic(true)
            .multiline_min(2);

    let settings_read = settings.clone();
    let mut settings_text_read = settings_text.clone();
    let read_button = ButtonComp::builder().text("Read").build();
    let read_button = ComponentWithData::new(
        read_button.clone(),
        read_button.on_click(move || {
            let Some(val) = serde_json::to_string_pretty(&settings_read.save_value()).ok() else {
                return;
            };
            settings_text_read.set(val);
        }),
    );

    let mut settings_write = settings.clone();
    let settings_text_write = settings_text.clone();
    let write_button = ButtonComp::builder().text("Write").build();
    let write_button = ComponentWithData::new(
        write_button.clone(),
        write_button.on_click(move || {
            let text = settings_text_write.get();
            let Some(val) = serde_json::from_str(&text).ok() else {
                return;
            };
            settings_write.load_value(val);
        }),
    );

    CompositeComp::builder()
        .perpendicular_align(Align::Stretch)
        .build((
            text_field,
            CompositeComp::builder()
                .horizontal(true)
                .main_align(AlignMain::Stretch)
                .build((
                    CompositeItemComp::builder()
                        .grow_ratio(1.0)
                        .build(read_button),
                    CompositeItemComp::builder()
                        .grow_ratio(1.0)
                        .build(write_button),
                )),
        ))
        .into()
}
