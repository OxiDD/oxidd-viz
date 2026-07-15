import React, {useEffect, useRef, useState} from "react";
import {LabelComp, LabelKind} from "oxidd-vis-rust";
import {ICompUI} from "../_types/ICompUI";
import {useWatch} from "../../../watchables/react/useWatch";
import {Label, Stack, useTheme} from "@fluentui/react";
import {css} from "@emotion/css";
import {usePersistentMemo} from "../../../utils/usePersistentMemo";
import {v4 as uuid} from "uuid";
import {NFC} from "../../../utils/_types/NFC";
import {IAriaRef} from "../_types/IAriaRef";
import {addAriaLabel} from "../ariaRef";

export const LabelCompUI: NFC<{
    data: LabelComp;
    ChildComp: ICompUI;
    className?: string;
    aria?: IAriaRef;
}> = ({data, ChildComp, className, aria}) => {
    const id = usePersistentMemo(() => uuid(), []);
    const watch = useWatch();
    const child = watch(data.input);
    const text = watch(data.label);
    const kind = watch(data.kind);

    const theme = useTheme();

    const [width, setWidth] = useState<number | undefined>();
    const labelRef = useRef<HTMLSpanElement | null>(null);
    useEffect(() => {
        if (!labelRef.current) return;

        const observer = new ResizeObserver(([entry]) => {
            const newWidth = entry.contentRect.width;
            if (newWidth > 0) {
                setWidth(newWidth);
            }
        });

        observer.observe(labelRef.current);
        return () => observer.disconnect();
    }, [text]);
    if (kind == LabelKind.Category) {
        return (
            <div className={className}>
                <Label className={`${css({marginBottom: 10, fontSize: 20})}`} id={id}>
                    {text}
                </Label>
                <ChildComp data={child} aria={addAriaLabel(id, aria)} />
            </div>
        );
    } else if (kind == LabelKind.Above) {
        return (
            <div className={className}>
                <Label className={className} id={id}>
                    {text}
                </Label>
                <ChildComp data={child} aria={addAriaLabel(id, aria)} />
            </div>
        );
    } else {
        return (
            <Stack
                horizontal
                tokens={{childrenGap: theme.spacing.s1}}
                className={`${css({">:nth-child(3)": {flex: "1 1"}, flexWrap: "wrap"})} ${className}`}>
                <Label
                    className={className}
                    style={{flex: "1 1", maxWidth: width}}
                    id={id}>
                    {text}
                </Label>
                <Label style={{position: "absolute", visibility: "hidden"}}>
                    <span style={{display: "inline-block"}} ref={labelRef}>
                        {text}
                    </span>
                </Label>
                <ChildComp data={child} aria={addAriaLabel(id, aria)} />
            </Stack>
        );
    }
};
