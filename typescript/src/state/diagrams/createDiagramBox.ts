import {
    create_bdd_min_diagram,
    create_mtbdd_diagram,
    create_qdd_diagram,
    DiagramBox,
} from "oxidd-vis-rust";
import {IDiagramType} from "./_types/IDiagramTypeSerialization";

/**
 * Creates a new diagram box depending on the passed diagram type
 * @param type The diagram type to create
 * @returns The created diagram box
 */
export function createDiagramBox(type: IDiagramType): DiagramBox {
    console.log(type);
    // TODO: create different types here
    if (type == "MTBDD") {
        const diagramBox = create_mtbdd_diagram();
        if (!diagramBox) throw Error("Could not create a new DD");
        return diagramBox;
    } else if (type == "BDD") {
        const diagramBox = create_bdd_min_diagram();
        if (!diagramBox) throw Error("Could not create a new DD");
        return diagramBox;
    } else {
        const diagramBox = create_qdd_diagram();
        // const diagramBox = create_bdd_min_diagram();
        if (!diagramBox) throw Error("Could not create a new DD");
        return diagramBox;
    }
}
