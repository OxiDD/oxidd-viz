import React from "react";
import {CompositeItemComp} from "oxidd-vis-rust";
import {NFC} from "../../../utils/_types/NFC";
import {IAriaRef} from "../_types/IAriaRef";
import {StackItem} from "@fluentui/react";
import {ICompUI} from "../_types/ICompUI";
import {useWatch} from "../../../watchables/react/useWatch";
import {alignToText} from "./CompositeCompUI";
import {css} from "@emotion/css";

export const CompositeItemCompUI: NFC<{
    data: CompositeItemComp;
    ChildComp: ICompUI;
    className?: string;
    aria?: IAriaRef;
}> = ({data, ChildComp, className, aria}) => {
    const watch = useWatch();
    const child = watch(data.child);
    const align = watch(data.perpendicular_align);
    const grow = watch(data.grow_ratio);
    const shrink = watch(data.shrink_ratio);
    return (
        // <StackItem align={alignToText(align) as any} grow={grow} shrink={shrink}>
        //     <ChildComp data={child} className={className} aria={aria} />
        // </StackItem>
        <ChildComp
            data={child}
            className={`${className} ${css({flexGrow: grow, flexShrink: shrink, alignSelf: alignToText(align)})}`}
            aria={aria}
        />
    );
};
