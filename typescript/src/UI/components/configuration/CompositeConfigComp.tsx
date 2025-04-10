import React, {FC} from "react";
import {CompositeConfig} from "../../../state/configuration/types/CompositeConfig";
import {IConfigObjectType} from "../../../state/configuration/_types/IConfigObjectType";
import {useWatch} from "../../../watchables/react/useWatch";
import {Stack, useTheme} from "@fluentui/react";
import {css} from "@emotion/css";

export const CompositeConfigComp: FC<{
    value: CompositeConfig;
    ChildComp: FC<{value: IConfigObjectType}>;
}> = ({value, ChildComp}) => {
    const watch = useWatch();
    const isHorizontal = watch(value.isHorizontal);
    const theme = useTheme();
    return (
        <Stack
            tokens={{childrenGap: theme.spacing.s1}}
            className={isHorizontal ? css({">*": {flex: 1}}) : undefined}
            horizontal={isHorizontal}>
            {watch(value.children).map((child, i) => (
                <ChildComp key={i} value={child} />
            ))}
        </Stack>
    );
};
