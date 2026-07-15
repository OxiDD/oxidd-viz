import React from "react";
import {ModalComp} from "oxidd-vis-rust";
import {Modal, useTheme, mergeStyleSets} from "@fluentui/react";
import {NFC} from "../../../utils/_types/NFC";
import {ICompUI} from "../_types/ICompUI";
import {IAriaRef} from "../_types/IAriaRef";
import {useWatch} from "../../../watchables/react/useWatch";

export const ModalCompUI: NFC<{
    data: ModalComp;
    ChildComp: ICompUI;
    aria?: IAriaRef;
    className?: string;
}> = ({data, ChildComp, className, aria}) => {
    const watch = useWatch();
    const content = watch(data.content);
    const shown = watch(data.shown);
    const width = watch(data.width);
    const height = watch(data.height);
    const theme = useTheme();

    const contentStyles = React.useMemo(
        () =>
            mergeStyleSets({
                container: {
                    display: "flex",
                    flexFlow: "column nowrap",
                    alignItems: "stretch",
                    width: width ? `${width}px` : undefined,
                    height: height ? `${height}px` : undefined,
                    minHeight: 0,
                    minWidth: 0,
                },
                body: {
                    flex: "1 1 auto",
                    overflowY: "auto",
                },
            }),
        [width, height]
    );
    const clickOutside = () => {
        data.click_outside.set(data.click_outside.get() + 1).commit();
    };

    return (
        // Non-displayed div to get rid of the ghost element that messes up layout in flex-boxes
        // The actual modal will still render using a portal
        <div style={{display: "none"}}>
            <Modal
                isOpen={shown}
                className={className}
                onDismiss={clickOutside}
                containerClassName={contentStyles.container}>
                <div className={contentStyles.body}>
                    <ChildComp data={content} aria={aria} />
                </div>
            </Modal>
        </div>
    );
};
