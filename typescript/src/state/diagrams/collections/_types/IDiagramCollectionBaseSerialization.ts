import {IBaseViewSerialization} from "../../../_types/IBaseViewSerialization";
import {IDiagramSerialization} from "../../_types/IDiagramSerialization";
import {IDiagramTypeSerialization} from "../../_types/IDiagramTypeSerialization";
import {IDiagramCollectionConfig} from "./IDiagramCollectionType";

export type IDiagramCollectionBaseSerialization = {
    ID: string;
    diagrams: IDiagramTypeSerialization[];
};
