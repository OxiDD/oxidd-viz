import {ButtonConfig} from "../types/ButtonConfig";
import {ChoiceConfig} from "../types/ChoiceConfig";
import {CompositeConfig} from "../types/CompositeConfig";
import {ContainerConfig} from "../types/ContainerConfig";
import {FloatConfig} from "../types/FloatConfig";
import {IntConfig} from "../types/IntConfig";
import {LabelConfig} from "../types/LabelConfig";
import {LocationConfig} from "../types/LocationConfig";
import {PanelConfig} from "../types/PanelConfig";
import {TextConfig} from "../types/TextConfig";
import {TextOutputConfig} from "../types/TextOutputConfig";

export type IConfigObjectType =
    | IntConfig
    | FloatConfig
    | ChoiceConfig
    | LabelConfig
    | CompositeConfig
    | ButtonConfig
    | TextOutputConfig
    | TextConfig
    | PanelConfig
    | LocationConfig
    | ContainerConfig;
