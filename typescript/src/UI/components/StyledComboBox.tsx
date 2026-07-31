import React, {FC, forwardRef} from "react";
import {ComboBox, IComboBoxProps} from "@fluentui/react";
import {css} from "@emotion/css";

export const StyledComboBox: FC<IComboBoxProps> = forwardRef((props, ref) => (
    <ComboBox
        ref={ref}
        className={`${props.className} ${css({
            ".ms-ComboBox::after": {border: 0},
            minWidth: 120,
        })}`}
        {...props}
    />
));
