import {AbstractConfigurationObject, ConfigurationObjectType} from "oxidd-vis-rust";
import {IConfigObjectType} from "./_types/IConfigObjectType";
import {IntConfig} from "./types/IntConfig";
import {LabelConfig} from "./types/LabelConfig";
import {CompositeConfig} from "./types/CompositeConfig";
import {ChoiceConfig} from "./types/ChoiceConfig";
import {ButtonConfig} from "./types/ButtonConfig";
import {TextOutputConfig} from "./types/TextOutputConfig";
import {PanelConfig} from "./types/PanelConfig";
import {IWatchable} from "../../watchables/_types/IWatchable";
import {IOwnedAbstractConfig} from "./ConfigurationObject";
import {LocationConfig} from "./types/LocationConfig";
import {FloatConfig} from "./types/FloatConfig";
import {ContainerConfig} from "./types/ContainerConfig";
import {TextConfig} from "./types/TextConfig";

/**
 * Creates the configuration object wrapper from the given abstract configuration object
 * @param ownedConfig The object for which to create the wrapper
 * @returns The object wrapper
 */
export function getConfigurationObjectWrapper(
    ownedConfig: IOwnedAbstractConfig
): IConfigObjectType {
    const type = ownedConfig.config.get_type();
    if (type == ConfigurationObjectType.Int) {
        return new IntConfig(ownedConfig);
    } else if (type == ConfigurationObjectType.Float) {
        return new FloatConfig(ownedConfig);
    } else if (type == ConfigurationObjectType.Label) {
        return new LabelConfig(ownedConfig);
    } else if (type == ConfigurationObjectType.Composite) {
        return new CompositeConfig(ownedConfig);
    } else if (type == ConfigurationObjectType.Choice) {
        return new ChoiceConfig(ownedConfig);
    } else if (type == ConfigurationObjectType.Button) {
        return new ButtonConfig(ownedConfig);
    } else if (type == ConfigurationObjectType.TextOutput) {
        return new TextOutputConfig(ownedConfig);
    } else if (type == ConfigurationObjectType.Text) {
        return new TextConfig(ownedConfig);
    } else if (type == ConfigurationObjectType.Panel) {
        return new PanelConfig(ownedConfig);
    } else if (type === ConfigurationObjectType.Location) {
        return new LocationConfig(ownedConfig);
    } else if (type === ConfigurationObjectType.Container) {
        return new ContainerConfig(ownedConfig);
    }

    return null as never;
}
