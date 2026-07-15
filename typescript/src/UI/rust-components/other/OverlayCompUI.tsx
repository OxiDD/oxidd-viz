import React from "react";
import {Component, OverlayComp} from "oxidd-vis-rust";
import {NFC} from "../../../utils/_types/NFC";
import {ICompUI} from "../_types/ICompUI";
import {IAriaRef} from "../_types/IAriaRef";
import {useWatch} from "../../../watchables/react/useWatch";
import {css} from "@emotion/css";

export const OverlayCompUI: NFC<{
    data: OverlayComp;
    ChildComp: ICompUI;
    aria: IAriaRef;
    className?: string;
}> = ({data, ChildComp, className, aria}) => {
    const watch = useWatch();
    const content = watch(data.content);
    const x = watch(data.x);
    const y = watch(data.y);

    return (
        // <div
        //     className={className}
        //     style={{position: "relative", width: "100%", height: "100%"}}>
        <ChildComp
            data={content}
            aria={aria}
            className={`${css({
                position: "absolute",
                left: `${x * 100}%`,
                top: `${y * 100}%`,
                transform: `translate(${-x * 100}%, ${-y * 100}%)`,
            })} ${className}`}
        />
        // </div>
    );
};

// const AbsoluteChildComp: NFC<{
//     child: Component
//     ChildComp: ICompUI;
//     className: string;
// }> = ({child, ChildComp}) => {
//     const watch = useWatch();
//     const childFill = child.as_fill();
//     if (childFill) {
//         const comp = watch(childFill.content);
//         const fillWidth = watch(childFill.full_width);
//         const fillHeight = watch(childFill.full_height);
//         return <AbsoluteChildComp child={comp} />
//     }
// }
