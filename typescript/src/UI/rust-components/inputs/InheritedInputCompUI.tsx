import {InheritedInputComp} from "oxidd-vis-rust";
import React, {useCallback} from "react";
import {useWatch} from "../../../watchables/react/useWatch";
import {Checkbox, IconButton, Stack, TooltipDelay} from "@fluentui/react";
import {NFC} from "../../../utils/_types/NFC";
import {IAriaRef} from "../_types/IAriaRef";
import {ICompUI} from "../_types/ICompUI";
import {StyledTooltipHost} from "../../components/StyledToolTipHost";
import {css} from "@emotion/css";

export const InheritedInputCompUI: NFC<{
    data: InheritedInputComp;
    ChildComp: ICompUI;
    className?: string;
    aria?: IAriaRef;
}> = ({data, ChildComp, className, aria}) => {
    const watch = useWatch();
    const inheriting = watch(data.inheriting);
    const label = watch(data.inherited_from);
    const child = watch(data.input);

    const onClick = useCallback(() => {
        data.set_inherit(!inheriting).commit();
    }, [data, inheriting]);

    return (
        <Stack horizontal>
            <StyledTooltipHost
                delay={TooltipDelay.long}
                content={inheriting ? label.inherited_label : label.local_label}>
                <IconButton
                    // disabled={inheriting}
                    onClick={onClick}
                    style={{color: "inherit"}}
                    iconProps={{iconName: inheriting ? "Link" : "LocationDot"}}
                />
            </StyledTooltipHost>

            <ChildComp data={child} className={css({flex: 1})} />
        </Stack>
    );
};
