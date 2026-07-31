import React, {FC} from "react";
import {IConfigObjectType} from "../../../state/configuration/_types/IConfigObjectType";
import {IntConfig} from "../../../state/configuration/types/IntConfig";
import {IntConfigComp} from "./IntConfigComp";
import {LabelConfig} from "../../../state/configuration/types/LabelConfig";
import {LabelConfigComp} from "./LabelConfigComp";
import {CompositeConfig} from "../../../state/configuration/types/CompositeConfig";
import {CompositeConfigComp} from "./CompositeConfigComp";
import {ChoiceConfig} from "../../../state/configuration/types/ChoiceConfig";
import {ChoiceConfigComp} from "./ChoiceConfigComp";
import {ButtonConfigComp} from "./ButtonConfigComp";
import {ButtonConfig} from "../../../state/configuration/types/ButtonConfig";
import {TextOutputConfig} from "../../../state/configuration/types/TextOutputConfig";
import {TextOutputConfigComp} from "./TextOutputConfigComp";
import {PanelConfig} from "../../../state/configuration/types/PanelConfig";
import {PanelConfigComp} from "./PanelConfigComp";
import {LocationConfig} from "../../../state/configuration/types/LocationConfig";
import {LocationConfigComp} from "./LocationConfigComp";
import {FloatConfigComp} from "./FloatConfigComp";
import {FloatConfig} from "../../../state/configuration/types/FloatConfig";
import {ContainerConfig} from "../../../state/configuration/types/ContainerConfig";
import {ContainerConfigComp} from "./ContainerConfigComp";
import {TextConfigComp} from "./TextConfigComp";
import {TextConfig} from "../../../state/configuration/types/TextConfig";

export const ConfigTypeComp: FC<{value: IConfigObjectType}> = ({value}) => {
    if (value instanceof IntConfig) return <IntConfigComp value={value} />;
    if (value instanceof FloatConfig) return <FloatConfigComp value={value} />;
    if (value instanceof LabelConfig)
        return <LabelConfigComp value={value} ChildComp={ConfigTypeComp} />;
    if (value instanceof CompositeConfig)
        return <CompositeConfigComp value={value} ChildComp={ConfigTypeComp} />;
    if (value instanceof ChoiceConfig) return <ChoiceConfigComp value={value} />;
    if (value instanceof ButtonConfig) return <ButtonConfigComp value={value} />;
    if (value instanceof TextOutputConfig) return <TextOutputConfigComp value={value} />;
    if (value instanceof TextConfig) return <TextConfigComp value={value} />;
    if (value instanceof PanelConfig) return <PanelConfigComp value={value} />;
    if (value instanceof LocationConfig)
        return <LocationConfigComp value={value} ChildComp={ConfigTypeComp} />;
    if (value instanceof ContainerConfig)
        return <ContainerConfigComp value={value} ChildComp={ConfigTypeComp} />;
    return <></>;
};
