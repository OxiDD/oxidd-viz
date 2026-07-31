import React, {FC, useCallback, useState} from "react";
import {ManualDiagramCollectionState} from "../../../../state/diagrams/collections/ManualDiagramCollectionState";
import {useWatch} from "../../../../watchables/react/useWatch";
import {DefaultButton, Stack, useTheme} from "@fluentui/react";
import {CenteredContainer} from "../../../components/layout/CenteredContainer";
import {DiagramSummary} from "../DiagramSummary";
import {DiagramCollection} from "../DiagramCollection";
import {DiagramCollectionContainer} from "./util/DiagramCollectionContainer";
import {DiagramCollectionHostModal} from "../modals/DiagramCollectionHostModal";

export const ManualDiagramCollection: FC<{
    collection: ManualDiagramCollectionState;
    onDelete?: () => void;
}> = ({collection, onDelete}) => {
    const watch = useWatch();
    const theme = useTheme();

    const [showHostModal, setShowHostModal] = useState(false);
    const onShowHostModal = useCallback(() => {
        setShowHostModal(true);
    }, []);
    const onHideHostModal = useCallback(() => {
        setShowHostModal(false);
    }, []);
    const onSelectHost = useCallback((host: string) => {
        collection
            .addCollection({
                type: "remote-http",
                url: host,
            })
            .commit();
        setShowHostModal(false);
    }, []);

    return (
        <DiagramCollectionContainer
            title="Manual"
            onDelete={onDelete}
            hideFrame={!onDelete}
            status={collection.status}>
            <Stack tokens={{childrenGap: theme.spacing.m}}>
                {watch(collection.diagrams).map(diagram => (
                    <Stack.Item align="stretch" key={diagram.ID}>
                        <DiagramSummary
                            diagram={diagram}
                            onDelete={() => collection.removeDiagram(diagram).commit()}
                        />
                    </Stack.Item>
                ))}
            </Stack>
            <Stack
                horizontal
                tokens={{childrenGap: theme.spacing.m}}
                style={{marginTop: theme.spacing.m}}>
                <AddDiagramButton onClick={() => collection.addDiagram("BDD").commit()}>
                    Add local BDD
                </AddDiagramButton>
                <AddDiagramButton onClick={() => collection.addDiagram("QDD").commit()}>
                    Add local advanced BDD
                </AddDiagramButton>
                <AddDiagramButton onClick={() => collection.addDiagram("MTBDD").commit()}>
                    Add local MTBDD
                </AddDiagramButton>
            </Stack>

            <Stack>
                {watch(collection.collections).map(subCollection => (
                    <Stack.Item
                        align="stretch"
                        key={subCollection.ID}
                        style={{marginTop: theme.spacing.m}}>
                        <DiagramCollection
                            collection={subCollection}
                            onDelete={() =>
                                collection.removeCollection(subCollection).commit()
                            }
                        />
                    </Stack.Item>
                ))}
            </Stack>
            <Stack
                horizontal
                tokens={{childrenGap: theme.spacing.m}}
                style={{marginTop: theme.spacing.m}}>
                {/* TODO: add modal for selecting the host to use + store this in settings as the default host */}
                <AddDiagramButton onClick={onShowHostModal}>
                    Add diagram source
                </AddDiagramButton>
            </Stack>
            <DiagramCollectionHostModal
                visible={showHostModal}
                onCancel={onHideHostModal}
                onSelect={onSelectHost}
            />
        </DiagramCollectionContainer>
    );
};

const AddDiagramButton: FC<{onClick: () => void}> = ({onClick, children}) => (
    <DefaultButton
        onClick={onClick}
        children={children}
        style={{
            flexGrow: 1,
            width: 200,
        }}
    />
);
