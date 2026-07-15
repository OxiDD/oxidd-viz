import {Align, AlignMain, CompositeComp} from "oxidd-vis-rust";
import React from "react";
import {ICompUI} from "../_types/ICompUI";
import {useWatch} from "../../../watchables/react/useWatch";
import {Stack, useTheme} from "@fluentui/react";
import {css} from "@emotion/css";
import {NFC} from "../../../utils/_types/NFC";
import {multiplySize} from "../../../utils/multiplySize";
import {IAriaRef} from "../_types/IAriaRef";

export const CompositeCompUI: NFC<{
    data: CompositeComp;
    ChildComp: ICompUI;
    className?: string;
    aria?: IAriaRef;
}> = ({data, ChildComp, className, aria}) => {
    const watch = useWatch();
    const children = watch(data.children);
    const isHorizontal = watch(data.horizontal);
    const childGap = watch(data.gap);

    const align = watch(data.main_align);
    const perpendicularAlign = watch(data.perpendicular_align);
    const horizontalAlign = isHorizontal ? align : perpendicularAlign;
    const verticalAlign = isHorizontal ? perpendicularAlign : align;

    const theme = useTheme();
    if (children.length == 0) {
        return <></>;
    }

    return (
        <Stack
            tokens={{childrenGap: multiplySize(childGap, theme.spacing.s1)}}
            className={`${css({position: "relative"})} ${className}`}
            aria-describedby={aria?.descriptionID}
            aria-labelledby={aria?.labelID}
            horizontal={isHorizontal}
            verticalAlign={alignToText(verticalAlign)}
            horizontalAlign={alignToText(horizontalAlign)}>
            {children.map((child, i) => (
                <ChildComp key={i} data={child} />
            ))}
        </Stack>
    );
};

export function alignToText(align: Align | AlignMain) {
    switch (align) {
        case AlignMain.Start:
        case Align.Start:
            return "start";
        case AlignMain.Center:
        case Align.Center:
            return "center";
        case AlignMain.End:
        case Align.End:
            return "end";
        case AlignMain.Stretch:
        case Align.Stretch:
            return "stretch";
        case AlignMain.SpaceAround:
            return "space-around";
        case AlignMain.SpaceBetween:
            return "space-between";
    }
}
