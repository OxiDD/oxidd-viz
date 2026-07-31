import React, {ChangeEvent, FC, useCallback} from "react";
import {useWatch} from "../../../watchables/react/useWatch";
import {
    ActionButton,
    ComboBox,
    IComboBoxStyles,
    IDropdownOption,
    IStackTokens,
    Stack,
    useTheme,
} from "@fluentui/react";
import {StyledDropdown} from "../StyledDropdown";
import {TextConfig} from "../../../state/configuration/types/TextConfig";
import {StyledComboBox} from "../StyledComboBox";

// Optional styling to make the example look nicer
export const TextConfigComp: FC<{value: TextConfig}> = ({value}) => {
    const watch = useWatch();
    const onChange = useCallback(
        (event: unknown, option?: IDropdownOption, index?: number, val?: string) => {
            if (val != undefined) {
                value.set(val).commit();
            }
        },
        []
    );
    const options = watch(value.options);
    const theme = useTheme();
    const onFileChange = useCallback(async (event: ChangeEvent<HTMLInputElement>) => {
        const file = event.target.files?.[0];
        if (!file) return;

        const reader = new FileReader();
        reader.readAsText(file);
        reader.onload = () => {
            const result = reader.result;
            if (result) {
                let options = (result as string)
                    .split(/\r?\n|;/)
                    .map(v => v.trim())
                    .filter(Boolean);
                value.setOptions(options).commit();
            }
        };
    }, []);

    return (
        <Stack horizontal>
            <StyledComboBox
                styles={{container: {flexGrow: 1}}}
                allowFreeform
                autoComplete="on"
                text={watch(value.selected)}
                options={options.map((text, i) => ({key: i, text}))}
                onChange={onChange}
                useComboBoxAsMenuWidth
                comboBoxOptionStyles={{
                    flexContainer: {
                        height: "auto",
                    },
                }}
            />
            <div style={{position: "relative"}}>
                <input
                    type="file"
                    style={{
                        cursor: "pointer",
                        position: "absolute",
                        zIndex: 1,
                        opacity: 0,
                        left: 0,
                        right: 0,
                        top: 0,
                        bottom: 0,
                    }}
                    onChange={onFileChange}
                />
                <ActionButton
                    iconProps={{iconName: "Edit"}}
                    styles={{icon: {color: theme.palette.neutralPrimary}}}></ActionButton>
            </div>
        </Stack>
    );
};
