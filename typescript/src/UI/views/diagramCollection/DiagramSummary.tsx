import React, {FC, ReactNode, useCallback, useState} from "react";
import {DiagramState} from "../../../state/diagrams/DiagramState";
import {
    DefaultButton,
    DirectionalHint,
    IconButton,
    Stack,
    useTheme,
} from "@fluentui/react";
import {useWatch} from "../../../watchables/react/useWatch";
import {DiagramSectionSummary} from "./DiagramSectionSummary";
import {css} from "@emotion/css";
import {StyledTooltipHost} from "../../components/StyledToolTipHost";
import {DDDMPSelectionModal} from "./modals/DDDMPSelectionModal";
import {usePersistentMemo} from "../../../utils/usePersistentMemo";
import {Derived} from "../../../watchables/Derived";
import {FileSource} from "../../../state/diagrams/sources/FileSource";
import {BuddySelectionModal} from "./modals/BuddySelectionModal";
import {mtbddDddmpSample} from "./samples/mtbddDddmpSample";
import {bddDddmpSample} from "./samples/bddDddmpSample";
import {bddBuddySample} from "./samples/bddBuddySample";

export const DiagramSummary: FC<{diagram: DiagramState; onDelete: () => void}> = ({
    diagram,
    onDelete,
}) => {
    const theme = useTheme();
    const watch = useWatch();

    const [showDDDMPInputModal, setShowDDDMPInputModal] = useState(false);
    const startCreatingDDDMPSection = useCallback(() => {
        setShowDDDMPInputModal(true);
    }, []);
    const stopCreatingDDDMPSection = useCallback(() => {
        setShowDDDMPInputModal(false);
    }, []);
    const createDDDMPSection = useCallback(
        (input: string, name?: string) => {
            setShowDDDMPInputModal(false);
            diagram.createSectionFromDDDMP(input, name).commit();
        },
        [diagram]
    );

    const [showBuddyInputModal, setShowBuddyInputModal] = useState(false);
    const startCreatingBuddySection = useCallback(() => {
        setShowBuddyInputModal(true);
    }, []);
    const stopCreatingBuddySection = useCallback(() => {
        setShowBuddyInputModal(false);
    }, []);
    const createBuddySection = useCallback(
        (input: string, vars?: string, name?: string) => {
            setShowBuddyInputModal(false);
            diagram.createSectionFromBuddy(input, vars, name).commit();
        },
        [diagram]
    );

    const watchableCanCreateFromFile = usePersistentMemo(
        () =>
            new Derived(
                watch =>
                    !watch(diagram.sections).some(
                        section => section instanceof FileSource
                    )
            ),
        [diagram]
    );
    const canCreateFromFile = watch(watchableCanCreateFromFile);

    const canCreateFromSelection = watch(diagram.selectedNodes).length > 0;
    const createSelectionSection = useCallback(() => {
        const nodes = diagram.selectedNodes.get();
        diagram.createSectionFromSelection(nodes).commit();
    }, [diagram]);

    return (
        <div
            className={css({
                backgroundColor: theme.palette.neutralLighterAlt,
            })}>
            <Stack
                horizontal
                className={css({
                    backgroundColor: theme.palette.neutralLighter,
                })}>
                <Stack.Item grow className={css({padding: theme.spacing.s1})}>
                    {watch(diagram.name)}
                </Stack.Item>
                <Stack.Item>
                    <IconButton
                        className={css({height: "100%"})}
                        iconProps={{iconName: "cancel"}}
                        // TODO: confirmation prompt
                        onClick={onDelete}
                    />
                </Stack.Item>
            </Stack>
            {/* Input management should be in rust eventually since it's diagram specific */}
            <Stack
                tokens={{childrenGap: theme.spacing.s1}}
                style={{padding: theme.spacing.s1}}>
                {watch(diagram.sections).map(section => (
                    <Stack.Item align="stretch" key={section.ID}>
                        <DiagramSectionSummary
                            section={section}
                            onDelete={() => diagram.removeSection(section).commit()}
                        />
                    </Stack.Item>
                ))}
                <Stack
                    horizontal
                    tokens={{childrenGap: theme.spacing.s1}}
                    style={{marginTop: theme.spacing.s1}}>
                    <AddSectionButton
                        onClick={startCreatingDDDMPSection}
                        hover={
                            <>
                                Create a diagram from a dddmp file
                                {!canCreateFromFile && (
                                    <>
                                        <br /> Only one file per diagram is supported
                                        right now
                                    </>
                                )}
                            </>
                        }
                        disabled={!canCreateFromFile}>
                        Load from DDDMP
                    </AddSectionButton>

                    {diagram.type == "MTBDD" ? undefined : (
                        <AddSectionButton
                            onClick={startCreatingBuddySection}
                            hover={
                                <>
                                    Create a diagram from a buddy file
                                    {!canCreateFromFile && (
                                        <>
                                            <br /> Only one file per diagram is supported
                                            right now
                                        </>
                                    )}
                                </>
                            }
                            disabled={!canCreateFromFile}>
                            Load from BuDDy
                        </AddSectionButton>
                    )}
                    <AddSectionButton
                        onClick={createSelectionSection}
                        hover={
                            <>
                                Create a diagram visualization for the selected nodes
                                {!canCreateFromSelection && (
                                    <>
                                        <br /> Select some node(s) in this diagram to
                                        enable
                                    </>
                                )}
                            </>
                        }
                        disabled={!canCreateFromSelection}>
                        Create from selection
                    </AddSectionButton>
                </Stack>
            </Stack>
            <DDDMPSelectionModal
                visible={showDDDMPInputModal}
                onCancel={stopCreatingDDDMPSection}
                onSelect={createDDDMPSection}
                example={diagram.type == "MTBDD" ? mtbddDddmpSample : bddDddmpSample}
            />
            <BuddySelectionModal
                visible={showBuddyInputModal}
                example={bddBuddySample}
                onCancel={stopCreatingBuddySection}
                onSelect={createBuddySection}
            />
        </div>
    );
};

const AddSectionButton: FC<{
    onClick: () => void;
    disabled?: boolean;
    hover: JSX.Element;
}> = ({onClick, disabled, hover, children}) => (
    <StyledTooltipHost
        styles={{root: {width: 180, flexGrow: 1}}}
        content={hover}
        directionalHint={DirectionalHint.bottomCenter}>
        <DefaultButton
            onClick={onClick}
            children={children}
            disabled={disabled}
            style={{
                width: "100%",
            }}
        />
    </StyledTooltipHost>
);
