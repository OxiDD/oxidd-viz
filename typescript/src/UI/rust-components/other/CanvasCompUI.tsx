import {CanvasComp} from "oxidd-vis-rust";
import React, {useCallback, useEffect, useRef} from "react";
import {useWatch} from "../../../watchables/react/useWatch";
import {DefaultButton, IconButton, PrimaryButton} from "@fluentui/react";
import {NFC} from "../../../utils/_types/NFC";
import {IAriaRef} from "../_types/IAriaRef";

export const CanvasCompUI: NFC<{
    data: CanvasComp;
    className?: string;
    aria?: IAriaRef;
}> = ({data, className, aria}) => {
    // const watch = useWatch();
    const ref = useRef<HTMLCanvasElement>(null);
    useEffect(() => {
        if (ref.current) data.addInstance(ref.current).commit();

        return () => {
            if (ref.current) data.removeInstance(ref.current).commit();
        };
    }, []);

    return (
        <canvas
            ref={ref}
            className={className}
            aria-describedby={aria?.descriptionID}
            aria-labelledby={aria?.labelID}
        />
    );
};
